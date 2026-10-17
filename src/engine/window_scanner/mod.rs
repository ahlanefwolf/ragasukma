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

        let local = PathBuf::from("assets/active_window_app.json");
        if local != path {
            if let Some(parent) = local.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&local, &json);
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
    let local = PathBuf::from("assets/active_window_app.json");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        if let Some(parent) = local.parent() {
            let _ = fs::create_dir_all(parent);
        }
        return local;
    }

    let base_dir = crate::config::get_app_config_dir();
    let assets_dir = base_dir.join("assets");
    if !assets_dir.exists() {
        let _ = fs::create_dir_all(&assets_dir);
    }
    assets_dir.join("active_window_app.json")
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

pub async fn scan_active_window() -> Result<ActiveWindowSnapshot, String> {
    #[cfg(target_os = "windows")]
    {
        windows::scan_active_window_windows()
    }

    #[cfg(target_os = "linux")]
    {
        linux::scan_active_window_linux().await
    }

    #[cfg(target_os = "macos")]
    {
        macos::scan_active_window_macos().await
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

pub fn start_window_watcher() {
    let _ = std::thread::Builder::new()
        .name("ragasukma-window-watcher".to_string())
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut last_quick_info: Option<(String, String)> = None;

            loop {
                std::thread::sleep(std::time::Duration::from_millis(1500));

                #[cfg(target_os = "linux")]
                linux::cleanup_idle_atspi_connection();

                let current_info = get_focused_window_quick_info();

                let window_changed = match (&last_quick_info, &current_info) {
                    (Some(last), Some(curr)) => last != curr,
                    (None, Some(_)) => true,
                    _ => false,
                };

                // Only scan if jendela/aplikasi active correct-correct berpindah.
                // No more force_periodic 30s overwriting visual vision_perception results.
                if window_changed {
                    #[cfg(target_os = "linux")]
                    {
                        // Lightweight window list scan: updates active_window_app.json
                        // without waking up AT-SPI or D-Bus!
                        if let Ok(snap) = linux::scan_window_list_quick_linux() {
                            let _ = snap.save_to_file();
                        }
                    }

                    #[cfg(target_os = "windows")]
                    {
                        let _ = windows::scan_active_window_windows();
                    }

                    #[cfg(target_os = "macos")]
                    {
                        if let Ok(snap) = pollster::block_on(macos::scan_active_window_macos()) {
                            let _ = snap.save_to_file();
                        }
                    }

                    last_quick_info = current_info;
                }
            }
        })
        .expect("Failed to spawn window watcher thread");
}
