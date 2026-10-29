#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesktopElement {
    pub id: usize,
    pub label: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[i32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowItem {
    pub id: usize,
    pub app_name: String,
    pub title: String,
    pub active: bool,
    pub window_id: String,
}

fn default_source() -> String {
    "desktop_window".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowSnapshot {
    pub app_name: String,
    pub window_title: String,
    pub window_id: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub timestamp: u64,
    pub windows: Vec<WindowItem>,
    pub buttons: Vec<DesktopElement>,
    pub inputs: Vec<DesktopElement>,
    pub texts: Vec<String>,
    pub text_summary: String,
}

impl Default for ActiveWindowSnapshot {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            window_title: String::new(),
            window_id: String::new(),
            source: default_source(),
            timestamp: current_epoch_ms(),
            windows: Vec::new(),
            buttons: Vec::new(),
            inputs: Vec::new(),
            texts: Vec::new(),
            text_summary: String::new(),
        }
    }
}

impl ActiveWindowSnapshot {
    pub fn save_to_file(&self) -> Result<(), String> {
        let mut to_save = self.clone();

        //  Vision Protection: If previous snapshot is from vision_perception on jendela/aplikasi which sama,
        // preserve vision data (buttons with center/bbox coords, inputs, and OCR text) so it is not overwritten
        // oleh scan dangkal OS / accessibility tree!
        if to_save.source != "vision_perception" {
            if let Ok(prev) = Self::load_from_file() {
                let is_same_app = !to_save.app_name.is_empty()
                    && prev.app_name.eq_ignore_ascii_case(&to_save.app_name);
                let is_same_win_id =
                    !to_save.window_id.is_empty() && prev.window_id == to_save.window_id;

                if prev.source == "vision_perception" && (is_same_app || is_same_win_id) {
                    if !prev.buttons.is_empty() {
                        to_save.buttons = prev.buttons;
                    }
                    if !prev.inputs.is_empty() {
                        to_save.inputs = prev.inputs;
                    }
                    if !prev.texts.is_empty() {
                        to_save.texts = prev.texts;
                        to_save.text_summary = prev.text_summary;
                    }
                    to_save.source = "vision_perception".to_string();
                }
            }
        }

        let path = get_snapshot_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(&to_save)
            .map_err(|e| format!("Failed to serialize snapshot: {e}"))?;

        fs::write(&path, &json).map_err(|e| format!("Failed to write to {path:?}: {e}"))?;

        // In dev mode only: mirror to local assets/ for quick inspection
        if PathBuf::from("Cargo.toml").exists() {
            let local = PathBuf::from("assets/active_window_app.json");
            if local != path {
                if let Some(parent) = local.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&local, &json);
            }
        }

        Ok(())
    }

    pub fn load_from_file() -> Result<Self, String> {
        let path = get_snapshot_path();
        if !path.exists() {
            return Err("File active_window_app.json does not exist yet".to_string());
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {e}", path))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {e}"))
    }
}

pub fn get_snapshot_path() -> PathBuf {
    if PathBuf::from("Cargo.toml").exists() {
        let local = PathBuf::from("assets/active_window_app.json");
        if let Some(parent) = local.parent() {
            let _ = fs::create_dir_all(parent);
        }
        return local;
    }

    let cache_dir = crate::config::get_app_cache_dir();
    let _ = fs::create_dir_all(&cache_dir);
    cache_dir.join("active_window_app.json")
}

pub fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn scan_active_window_sync() -> Result<ActiveWindowSnapshot, String> {
    #[cfg(target_os = "windows")]
    {
        windows::scan_active_window_windows()
    }

    #[cfg(target_os = "linux")]
    {
        pollster::block_on(linux::scan_active_window_linux())
    }

    #[cfg(target_os = "macos")]
    {
        pollster::block_on(macos::scan_active_window_macos())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

pub fn get_focused_window_quick_info() -> Option<(String, String)> {
    #[cfg(target_os = "windows")]
    {
        windows::get_focused_window_quick_info_windows()
    }

    #[cfg(target_os = "linux")]
    {
        linux::get_focused_window_quick_info_linux()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_focused_window_quick_info_macos()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum WindowScannerIpcCmd {
    #[serde(rename = "scan")]
    Scan,
    #[serde(rename = "quick_info")]
    QuickInfo,
    #[serde(rename = "quit")]
    Quit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowScannerIpcResp {
    pub success: bool,
    pub snapshot: Option<ActiveWindowSnapshot>,
    pub quick_info: Option<(String, String)>,
    pub error: Option<String>,
}

pub static WINDOW_SCANNER_SUPERVISOR: crate::services::ManagedWorkerProcess =
    crate::services::ManagedWorkerProcess::new(
        "--worker-window-scanner",
        std::time::Duration::from_secs(10),
    );

/// Dedicated Child Process Entry Point for Window Accessibility Scanner.
/// Isolates native OS accessibility calls (UIAutomation / ATSPI) so freezes or deadlocks do not hang the main app.
/// Automatically terminates after 10 seconds of idle inactivity.
pub fn run_window_scanner_worker_process() -> anyhow::Result<()> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let (tx_line, rx_line) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("scanner-ipc-stdin".to_string())
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
        // Idle timeout: 10 seconds of no scan requests -> exit process!
        let line = match rx_line.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(l) => l,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                crate::term_log!(
                    "[Window Scanner Process] 10s idle timeout reached. Exiting process to free memory."
                );
                std::process::exit(0);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => std::process::exit(0),
        };

        if line.is_empty() {
            continue;
        }

        let cmd: WindowScannerIpcCmd = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let err_resp = WindowScannerIpcResp {
                    success: false,
                    snapshot: None,
                    quick_info: None,
                    error: Some(format!("Invalid command JSON: {e}")),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp)?);
                let _ = stdout.flush();
                continue;
            }
        };

        match cmd {
            WindowScannerIpcCmd::Scan => {
                match scan_active_window_sync() {
                    Ok(snap) => {
                        let resp = WindowScannerIpcResp {
                            success: true,
                            snapshot: Some(snap),
                            quick_info: None,
                            error: None,
                        };
                        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                    }
                    Err(e) => {
                        let resp = WindowScannerIpcResp {
                            success: false,
                            snapshot: None,
                            quick_info: None,
                            error: Some(e),
                        };
                        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                    }
                }
                let _ = stdout.flush();
            }
            WindowScannerIpcCmd::QuickInfo => {
                let info = get_focused_window_quick_info();
                let resp = WindowScannerIpcResp {
                    success: true,
                    snapshot: None,
                    quick_info: info,
                    error: None,
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                let _ = stdout.flush();
            }
            WindowScannerIpcCmd::Quit => {
                break;
            }
        }
    }

    Ok(())
}

/// Request active window accessibility scan via isolated child process with 3.5s watchdog timeout.
pub fn scan_active_window_ipc() -> Result<ActiveWindowSnapshot, String> {
    let cmd = WindowScannerIpcCmd::Scan;
    let json_cmd = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    match WINDOW_SCANNER_SUPERVISOR.send_command(&json_cmd, std::time::Duration::from_millis(3500)) {
        Ok(resp_str) => {
            if let Ok(resp) = serde_json::from_str::<WindowScannerIpcResp>(&resp_str) {
                if resp.success {
                    if let Some(snap) = resp.snapshot {
                        let _ = snap.save_to_file();
                        return Ok(snap);
                    }
                }
                return Err(resp.error.unwrap_or_else(|| "Unknown scan error".to_string()));
            }
            Err("Failed to parse scan response".to_string())
        }
        Err(e) => {
            crate::term_err!("[Window Scanner Supervisor] Watchdog / process error: {e}");
            Err(e)
        }
    }
}

/// Request active window quick info via isolated child process with 1.5s watchdog timeout.
/// Request active window accessibility scan with automatic fallback
pub fn scan_active_window_auto() -> Result<ActiveWindowSnapshot, String> {
    scan_active_window_ipc().or_else(|_| scan_active_window_sync())
}

#[allow(dead_code)]
pub fn get_focused_window_quick_info_ipc() -> Option<(String, String)> {
    let cmd = WindowScannerIpcCmd::QuickInfo;
    let json_cmd = serde_json::to_string(&cmd).ok()?;

    match WINDOW_SCANNER_SUPERVISOR.send_command(&json_cmd, std::time::Duration::from_millis(1500)) {
        Ok(resp_str) => {
            if let Ok(resp) = serde_json::from_str::<WindowScannerIpcResp>(&resp_str) {
                if resp.success {
                    return resp.quick_info;
                }
            }
            None
        }
        Err(_) => None,
    }
}
