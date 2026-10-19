use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A managed worker process that communicates over stdin/stdout via newline-delimited JSON.
pub struct ManagedWorkerProcess {
    pub flag: &'static str,
    pub idle_timeout: Duration,
    state: Mutex<Option<ActiveWorkerState>>,
}

struct ActiveWorkerState {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    last_active: Instant,
}

impl ManagedWorkerProcess {
    pub const fn new(flag: &'static str, idle_timeout: Duration) -> Self {
        Self {
            flag,
            idle_timeout,
            state: Mutex::new(None),
        }
    }

    /// Spawns a fresh child process if not currently running or if previous child terminated.
    fn ensure_spawned(&self) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;

        // Check if existing worker is still alive
        if let Some(ref mut active) = *guard {
            match active.child.try_wait() {
                Ok(None) => {
                    // Still running
                    return Ok(());
                }
                _ => {
                    // Exited or error; clean up
                    let _ = active.child.kill();
                    *guard = None;
                }
            }
        }

        let exe_path = std::env::current_exe().map_err(|e| format!("current_exe error: {e}"))?;

        let mut child = Command::new(&exe_path)
            .arg(self.flag)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn worker '{}': {e}", self.flag))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture stdin for worker".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout for worker".to_string())?;

        let (tx, rx) = mpsc::channel();
        let flag_name = self.flag.to_string();

        std::thread::Builder::new()
            .name(format!("ipc-reader-{}", self.flag.trim_start_matches('-')))
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line) {
                    if n == 0 {
                        break; // EOF
                    }
                    let trimmed = line.trim();
                    // Filter out non-JSON log lines to prevent stdout contamination
                    if trimmed.starts_with('{') && trimmed.ends_with('}') {
                        if tx.send(trimmed.to_string()).is_err() {
                            break;
                        }
                    } else if !trimmed.is_empty() {
                        eprintln!("[Worker {} Log]: {}", flag_name, trimmed);
                    }
                    line.clear();
                }
            })
            .map_err(|e| format!("Failed to spawn line reader thread: {e}"))?;

        *guard = Some(ActiveWorkerState {
            child,
            stdin,
            rx,
            last_active: Instant::now(),
        });

        Ok(())
    }

    /// Sends a JSON command to the worker and waits up to `timeout` for the response.
    /// If the worker deadlocks / hangs, it is automatically terminated via `kill()`.
    pub fn send_command(&self, json_cmd: &str, timeout: Duration) -> Result<String, String> {
        self.ensure_spawned()?;

        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let active = guard
            .as_mut()
            .ok_or_else(|| "Active worker not found".to_string())?;

        active.last_active = Instant::now();

        // Write command line
        if let Err(e) = writeln!(active.stdin, "{}", json_cmd) {
            let _ = active.child.kill();
            *guard = None;
            return Err(format!("Failed to write to worker stdin: {e}"));
        }
        let _ = active.stdin.flush();

        // Await response with timeout
        match active.rx.recv_timeout(timeout) {
            Ok(response) => {
                active.last_active = Instant::now();
                Ok(response)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                eprintln!(
                    "[Supervisor Watchdog] Worker '{}' timed out after {:?}. Force killing process!",
                    self.flag, timeout
                );
                let _ = active.child.kill();
                let _ = active.child.wait();
                *guard = None;
                Err(format!(
                    "Worker '{}' watchdog timeout ({:?})",
                    self.flag, timeout
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!(
                    "[Supervisor Watchdog] Worker '{}' process disconnected unexpectedly.",
                    self.flag
                );
                let _ = active.child.kill();
                let _ = active.child.wait();
                *guard = None;
                Err(format!("Worker '{}' disconnected unexpectedly", self.flag))
            }
        }
    }

    /// Checks if the worker has been idle longer than its configured timeout, and terminates it if so.
    /// Also reaps any terminated child processes to prevent zombie (<defunct>) processes.
    pub fn drop_if_idle(&self) -> bool {
        if let Ok(mut guard) = self.state.lock() {
            if let Some(ref mut active) = *guard {
                // 1. Check if child already exited (reap zombie)
                match active.child.try_wait() {
                    Ok(Some(_status)) => {
                        *guard = None;
                        return true;
                    }
                    Err(_) => {
                        let _ = active.child.kill();
                        let _ = active.child.wait();
                        *guard = None;
                        return true;
                    }
                    Ok(None) => {} // Still running
                }

                // 2. Terminate if idle timeout exceeded
                if active.last_active.elapsed() > self.idle_timeout {
                    crate::term_log!(
                        "[Process Supervisor] Worker '{}' idle for {:?}. Terminating process to reclaim RAM.",
                        self.flag,
                        self.idle_timeout
                    );
                    // Attempt graceful quit, then kill and wait to reap zombie
                    let _ = writeln!(active.stdin, "{{\"cmd\":\"quit\"}}");
                    let _ = active.stdin.flush();
                    let _ = active.child.kill();
                    let _ = active.child.wait();
                    *guard = None;
                    return true;
                }
            }
        }
        false
    }

    /// Forcefully kills the worker process immediately.
    pub fn kill_now(&self) {
        if let Ok(mut guard) = self.state.lock() {
            if let Some(ref mut active) = *guard {
                let _ = active.child.kill();
                let _ = active.child.wait();
                *guard = None;
            }
        }
    }

    /// Checks if worker is currently running.
    pub fn is_running(&self) -> bool {
        if let Ok(mut guard) = self.state.lock() {
            if let Some(ref mut active) = *guard {
                return matches!(active.child.try_wait(), Ok(None));
            }
        }
        false
    }
}

/// Starts background reaper thread that checks idle workers every 5 seconds,
/// terminating idle processes and reclaiming OS memory immediately.
pub fn start_global_reaper_worker() {
    let _ = std::thread::Builder::new()
        .name("ragasukma-worker-reaper".to_string())
        .stack_size(256 * 1024)
        .spawn(|| loop {
            std::thread::sleep(Duration::from_secs(5));
            crate::audio::supertonic::TTS_SUPERVISOR.drop_if_idle();
            crate::engine::tasks::TASK_SUPERVISOR.drop_if_idle();
            crate::engine::perception::VISION_SUPERVISOR.drop_if_idle();
            crate::engine::window_scanner::WINDOW_SCANNER_SUPERVISOR.drop_if_idle();
        });
}
