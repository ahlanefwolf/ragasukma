use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::config::UserConfig;
use crate::native::pose_system::PosesConfig;

pub fn get_assets_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let beside = exe_dir.join("assets");
            if beside.exists() {
                return beside;
            }
            let mac_resources = exe_dir.join("../Resources/assets");
            if mac_resources.exists() {
                return mac_resources;
            }
        }
    }

    let local = PathBuf::from("assets");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        let _ = fs::create_dir_all(&local);
        return local;
    }
    let parent = PathBuf::from("../assets");
    if parent.exists() {
        return parent;
    }
    let cfg_dir = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&cfg_dir);
    cfg_dir
}

pub fn find_file_in_candidates(filename: &str) -> PathBuf {
    let mut candidates = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("assets").join(filename));
            candidates.push(exe_dir.join(filename));
            candidates.push(exe_dir.join("../Resources/assets").join(filename));
            candidates.push(exe_dir.join("../Resources").join(filename));
        }
    }

    candidates.push(PathBuf::from(filename));
    candidates.push(PathBuf::from(format!("assets/{}", filename)));
    candidates.push(PathBuf::from(format!("../assets/{}", filename)));
    candidates.push(PathBuf::from(format!("../{}", filename)));

    candidates.push(crate::config::get_app_config_dir().join("assets").join(filename));
    candidates.push(crate::config::get_app_config_dir().join(filename));

    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let appdata_path = PathBuf::from(appdata);
        let p = appdata_path.join("ragasukma").join("assets").join(filename);
        if p.exists() {
            return p;
        }
        let p_root = appdata_path.join("ragasukma").join(filename);
        if p_root.exists() {
            return p_root;
        }
    }

    PathBuf::from(filename)
}

pub fn load_config() -> UserConfig {
    crate::config::load_user_config()
}

pub fn save_config(cfg: &UserConfig) -> Result<(), String> {
    crate::config::save_user_config(cfg)
}

pub fn load_poses() -> PosesConfig {
    let path = find_file_in_candidates("poses.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(p) = serde_json::from_str::<PosesConfig>(&content) {
            return p;
        }
    }
    PosesConfig::default()
}

pub fn save_poses(poses: &PosesConfig) -> Result<(), String> {
    let path = find_file_in_candidates("poses.json");
    let json = serde_json::to_string_pretty(poses).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Memastikan semua file aset default (char.vrm, poses.json, avatar PNG, skill-command.md)
/// otomatis tersalin ke direktori config pengguna (~/.config/ragasukma/assets) saat pertama kali dijalankan.
pub fn ensure_default_user_assets() {
    let target_dir = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&target_dir);

    let default_files = [
        "char.vrm",
        "poses.json",
        "skill-command.md",
        "config-ai.json",
        "char-avatar-idle.png",
        "char-avatar-listening.png",
        "char-avatar-talking.png",
        "char-avatar-thinking.png",
    ];

    for file in &default_files {
        let target_file = target_dir.join(file);
        if !target_file.exists() {
            let source = find_file_in_candidates(file);
            if source.exists() && source != target_file {
                let _ = fs::copy(&source, &target_file);
            }
        }
    }
}

pub fn open_browser(url: &str) {
    let url = url.to_string();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(150));

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &url])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&url)
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&url)
                .spawn();
        }
    });
}
