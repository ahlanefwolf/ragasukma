use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use std::thread;

use crate::app_state::push_client_action;
use crate::engine::ytdlp::download_audio;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum BackgroundTask {
    DownloadAndPlayMusic { query: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskState {
    pub is_busy: bool,
    pub task_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum TaskIpcCmd {
    #[serde(rename = "download_audio")]
    DownloadAudio { query: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "quit")]
    Quit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskIpcResp {
    pub success: bool,
    pub title: Option<String>,
    pub filename: Option<String>,
    pub error: Option<String>,
}

pub static TASK_SUPERVISOR: crate::services::ManagedWorkerProcess =
    crate::services::ManagedWorkerProcess::new(
        "--worker-task",
        std::time::Duration::from_secs(10),
    );

/// Dedicated Child Process Entry Point for Background Task & Action Runner.
/// Runs heavy or cancellation-prone workloads in an isolated process.
/// Automatically terminates after 10 seconds of idle inactivity.
pub fn run_task_worker_process() -> anyhow::Result<()> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let (tx_line, rx_line) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("task-ipc-stdin".to_string())
        .spawn(move || {
            let mut line = String::new();
            let mut handle = stdin.lock();
            while let Ok(n) = handle.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                if tx_line.send(line.trim().to_string()).is_err() {
                    break;
                }
                line.clear();
            }
        })?;

    loop {
        // Idle timeout: 10 seconds after task completion -> exit process!
        let line = match rx_line.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(l) => l,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                crate::term_log!(
                    "[Task Worker Process] 10s idle timeout reached. Exiting process to free memory."
                );
                std::process::exit(0);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => std::process::exit(0),
        };

        if line.is_empty() {
            continue;
        }

        let cmd: TaskIpcCmd = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let err_resp = TaskIpcResp {
                    success: false,
                    title: None,
                    filename: None,
                    error: Some(format!("Invalid task command: {e}")),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp)?);
                let _ = stdout.flush();
                continue;
            }
        };

        match cmd {
            TaskIpcCmd::DownloadAudio { query } => {
                let music_dir = crate::config::get_music_dir();
                match download_audio(&query, &music_dir) {
                    Ok(track) => {
                        let resp = TaskIpcResp {
                            success: true,
                            title: Some(track.title),
                            filename: Some(track.filename),
                            error: None,
                        };
                        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                    }
                    Err(e) => {
                        let resp = TaskIpcResp {
                            success: false,
                            title: None,
                            filename: None,
                            error: Some(e.to_string()),
                        };
                        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                    }
                }
                let _ = stdout.flush();
            }
            TaskIpcCmd::Ping => {
                let resp = TaskIpcResp {
                    success: true,
                    title: None,
                    filename: None,
                    error: None,
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                let _ = stdout.flush();
            }
            TaskIpcCmd::Quit => break,
        }
    }
    Ok(())
}

static TASK_SENDER: Mutex<Option<Sender<BackgroundTask>>> = Mutex::new(None);
static CURRENT_TASK_INFO: Mutex<Option<BackgroundTaskState>> = Mutex::new(None);
static ACTIVE_DOWNLOADS: std::sync::LazyLock<Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

struct DownloadGuard(String);

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
            active.remove(&self.0);
        }
        set_current_task(None);
    }
}

/// Cancel / kill currently running background task process immediately
pub fn cancel_current_task() {
    TASK_SUPERVISOR.kill_now();
    set_current_task(None);
}

/// Initialize dedicated background worker thread for yt-dlp & other heavy asynchronous tasks
pub fn init_background_task_worker() {
    let mut sender_lock = match TASK_SENDER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if sender_lock.is_some() {
        return; // Already initialized
    }

    let (tx, rx) = channel::<BackgroundTask>();
    *sender_lock = Some(tx);

    let builder = thread::Builder::new()
        .name("ragasukma-task-worker".to_string())
        .stack_size(2 * 1024 * 1024);

    if let Err(e) = builder.spawn(move || {
        crate::term_log!("[Task Worker] Dedicated Background Task Engine ready.");

        while let Ok(task) = rx.recv() {
            match task {
                BackgroundTask::DownloadAndPlayMusic { query } => {
                    let _guard = DownloadGuard(query.trim().to_lowercase());
                    set_current_task(Some(BackgroundTaskState {
                        is_busy: true,
                        task_type: "ytdlp_download".to_string(),
                        description: format!("Downloading song: {query}"),
                    }));

                    crate::term_log!(" [Task Worker: yt-dlp] Downloading via isolated process: '{}'...", query);

                    let cmd = TaskIpcCmd::DownloadAudio {
                        query: query.clone(),
                    };
                    let cmd_json = match serde_json::to_string(&cmd) {
                        Ok(j) => j,
                        Err(e) => {
                            crate::term_err!("Failed to serialize task cmd: {e}");
                            continue;
                        }
                    };

                    // Execute download via isolated child process (up to 3 min timeout)
                    match TASK_SUPERVISOR.send_command(&cmd_json, std::time::Duration::from_secs(180)) {
                        Ok(resp_str) => {
                            if let Ok(resp) = serde_json::from_str::<TaskIpcResp>(&resp_str) {
                                if resp.success {
                                    let title = resp.title.unwrap_or_else(|| query.clone());
                                    let filename = resp.filename.unwrap_or_default();
                                    crate::term_log!(
                                        " [Task Worker: yt-dlp] Done: '{}' -> Playing directly to speaker!",
                                        title
                                    );

                                    let music_dir = crate::config::get_music_dir();
                                    let file_path = music_dir.join(&filename);
                                    crate::audio::player::play_file(&file_path, &title);

                                    push_client_action("music:play", Some(&title));

                                    let notify_msg = format!(
                                        "[SYSTEM STATUS: Song '{}' has been downloaded and is now playing automatically. Inform the user briefly and warmly. DO NOT include any [CMD:media:play] or other commands — music is already playing.]",
                                        title
                                    );
                                    crate::engine::remote_text::inject_remote_text(notify_msg);

                                    // Immediately drop task child process to release RAM to OS
                                    TASK_SUPERVISOR.kill_now();
                                } else {
                                    let err = resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                    crate::term_err!(" [Task Worker: yt-dlp] Failed: {err}");
                                    let err_msg = format!(
                                        "[SYSTEM STATUS: Failed to download song '{}'. Detail: {}. Inform the user briefly and warmly.]",
                                        query, err
                                    );
                                    crate::engine::remote_text::inject_remote_text(err_msg);

                                    TASK_SUPERVISOR.kill_now();
                                }
                            }
                        }
                        Err(e) => {
                            crate::term_err!(" [Task Worker: yt-dlp] Process error: {e}");
                            let err_msg = format!(
                                "[SYSTEM STATUS: Song download timed out or failed: {}. Inform user briefly.]",
                                e
                            );
                            crate::engine::remote_text::inject_remote_text(err_msg);
                            TASK_SUPERVISOR.kill_now();
                        }
                    }
                }
            }
        }
    }) {
        crate::term_err!("[Task Worker] Failed to spawn background task worker thread: {e}");
    }
}

fn set_current_task(state: Option<BackgroundTaskState>) {
    if let Ok(mut lock) = CURRENT_TASK_INFO.lock() {
        *lock = state;
    }
}

/// Send download & play music tasks to Background Worker
pub fn queue_download_and_play(query: &str) -> bool {
    let q_clean = query.trim();
    if q_clean.is_empty() {
        return false;
    }
    let q_key = q_clean.to_lowercase();
    if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
        if active.contains(&q_key) {
            crate::term_log!(
                "[Task Worker: yt-dlp] Song '{}' is already currently downloading or queued. Skipping duplicate queue.",
                q_clean
            );
            return true;
        }
        active.insert(q_key.clone());
    }

    init_background_task_worker();
    if let Ok(lock) = TASK_SENDER.lock() {
        if let Some(tx) = lock.as_ref() {
            let sent = tx
                .send(BackgroundTask::DownloadAndPlayMusic {
                    query: q_clean.to_string(),
                })
                .is_ok();
            if !sent {
                if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
                    active.remove(&q_key);
                }
            }
            return sent;
        }
    }

    if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
        active.remove(&q_key);
    }
    false
}

/// Find out whether a background worker is currently busy processing media downloads
pub fn is_ytdlp_busy() -> bool {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.as_ref()
            .map(|t| t.is_busy && t.task_type == "ytdlp_download")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Gets current active task info (for AI context or telemetry)
pub fn get_active_task_info() -> Option<BackgroundTaskState> {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.clone()
    } else {
        None
    }
}
