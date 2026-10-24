use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BunInfo {
    pub installed: bool,
    pub version: String,
    pub path: Option<String>,
    pub is_installing: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct BunState {
    pub is_installing: bool,
    pub last_error: Option<String>,
}

pub static BUN_STATE: Mutex<Option<BunState>> = Mutex::new(None);

/// Locate Bun executable path in candidate locations or system PATH.
pub fn find_bun_bin() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let p1 = PathBuf::from(&user_profile)
                .join(".bun")
                .join("bin")
                .join("bun.exe");
            if p1.is_file() {
                return Some(p1);
            }
        }
        if let Ok(app_data) = std::env::var("APPDATA") {
            let p2 = PathBuf::from(&app_data)
                .join("npm")
                .join("bun.cmd");
            if p2.is_file() {
                return Some(p2);
            }
        }
        if let Ok(local_data) = std::env::var("LOCALAPPDATA") {
            let p3 = PathBuf::from(&local_data)
                .join("Programs")
                .join("bun")
                .join("bun.exe");
            if p3.is_file() {
                return Some(p3);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let p1 = PathBuf::from(&home).join(".bun").join("bin").join("bun");
            if p1.is_file() {
                return Some(p1);
            }
        }
        let candidates = ["/usr/local/bin/bun", "/opt/homebrew/bin/bun", "/usr/bin/bun"];
        for c in &candidates {
            let p = PathBuf::from(c);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // Try finding via `where` (Windows) or `which` (Unix)
    #[cfg(target_os = "windows")]
    let check_cmd = Command::new("where.exe").arg("bun").output();
    #[cfg(not(target_os = "windows"))]
    let check_cmd = Command::new("which").arg("bun").output();

    if let Ok(output) = check_cmd {
        if output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = out_str.lines().next() {
                let trimmed = first_line.trim();
                if !trimmed.is_empty() {
                    let p = PathBuf::from(trimmed);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Query Bun status and installed version if available.
pub fn get_bun_info() -> BunInfo {
    let state_lock = BUN_STATE.lock().unwrap_or_else(|e| e.into_inner());
    let (is_installing, last_error) = if let Some(ref st) = *state_lock {
        (st.is_installing, st.last_error.clone())
    } else {
        (false, None)
    };

    let bun_path = find_bun_bin();
    if let Some(ref bin) = bun_path {
        let output = Command::new(bin).arg("--version").output();
        if let Ok(out) = output {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return BunInfo {
                    installed: true,
                    version: if v.starts_with('v') { v } else { format!("v{v}") },
                    path: Some(bin.to_string_lossy().to_string()),
                    is_installing,
                    error: last_error,
                };
            }
        }
    }

    BunInfo {
        installed: false,
        version: String::new(),
        path: None,
        is_installing,
        error: last_error,
    }
}

/// Trigger non-blocking installer if Bun is not yet installed.
pub fn ensure_bun_installed_background() {
    let current = get_bun_info();
    if current.installed {
        crate::term_log!("[Bun] Bun runtime is already installed: {} ({:?})", current.version, current.path);
        return;
    }

    {
        let mut lock = BUN_STATE.lock().unwrap_or_else(|e| e.into_inner());
        let st = lock.get_or_insert_with(BunState::default);
        if st.is_installing {
            crate::term_log!("[Bun] Installation is already running in background...");
            return;
        }
        st.is_installing = true;
        st.last_error = None;
    }

    crate::term_log!("[Bun] Bun runtime not detected. Spawning automated installer...");

    thread::spawn(move || {
        let install_result = run_bun_installer();

        let mut lock = BUN_STATE.lock().unwrap_or_else(|e| e.into_inner());
        let st = lock.get_or_insert_with(BunState::default);
        st.is_installing = false;

        match install_result {
            Ok(ver) => {
                st.last_error = None;
                crate::term_log!("[Bun] Bun successfully installed! Detected version: {}", ver);
            }
            Err(e) => {
                crate::term_err!("[Bun] Installation failed: {}", e);
                st.last_error = Some(e);
            }
        }
    });
}

/// Execute official Bun installation script per operating system.
pub fn run_bun_installer() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        crate::term_log!("[Bun] Running PowerShell installer: irm bun.sh/install.ps1 | iex");
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "irm bun.sh/install.ps1 | iex"])
            .output()
            .map_err(|e| format!("Failed to spawn PowerShell: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Bun installer failed:\nStdout: {stdout}\nStderr: {stderr}"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        crate::term_log!("[Bun] Running curl installer: curl -fsSL https://bun.com/install | bash");
        let output = Command::new("bash")
            .args(["-c", "curl -fsSL https://bun.com/install | bash"])
            .output()
            .map_err(|e| format!("Failed to spawn bash: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Bun installer failed:\nStdout: {stdout}\nStderr: {stderr}"));
        }
    }

    let info = get_bun_info();
    if info.installed {
        Ok(info.version)
    } else {
        Err("Bun installation script finished but bun binary was not detected in PATH or standard directory.".to_string())
    }
}
