use rust_embed::RustEmbed;
use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::config::UserConfig;
use crate::native::pose_system::PosesConfig;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct EmbeddedAssets;

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

/// Memastikan semua file aset default (char.vrm, poses.json, avatar PNG, skill-command.md, config-ai.json)
/// otomatis diekstrak ke direktori config pengguna (~/.config/ragasukma/assets) saat pertama kali dijalankan.
pub fn ensure_default_user_assets() {
    let target_dir = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&target_dir);

    // 1. Ekstrak dari embedded binary assets
    for file in EmbeddedAssets::iter() {
        let file_str = file.as_ref();
        let target_path = target_dir.join(file_str);
        if !target_path.exists() {
            if let Some(embedded_file) = EmbeddedAssets::get(file_str) {
                if let Some(parent) = target_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&target_path, embedded_file.data.as_ref());
                println!("✨ [Asset Extractor] Mengekstrak aset default: {:?}", file_str);
            }
        }
    }

    #[cfg(target_os = "linux")]
    ensure_linux_desktop_entry();
}

#[cfg(target_os = "linux")]
pub fn ensure_linux_desktop_entry() {
    let exec_path = if let Ok(appimage_path) = std::env::var("APPIMAGE") {
        PathBuf::from(appimage_path)
    } else if let Ok(current) = std::env::current_exe() {
        current
    } else {
        return;
    };

    let exec_str = exec_path.to_string_lossy();
    // Jangan overwrite jika sedang dijalankan dari target build development
    if exec_str.contains("/target/debug/") || exec_str.contains("/target/release/") {
        return;
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        let apps_dir = home_path.join(".local/share/applications");
        let _ = fs::create_dir_all(&apps_dir);
        let desktop_file = apps_dir.join("ragasukma.desktop");

        let icon_path = crate::config::get_app_config_dir().join("assets").join("icon.png");

        let content = format!(
            "[Desktop Entry]\n\
Name=Ragasukma\n\
Comment=AI Voice Assistant & 3D/2D Avatar Assistant\n\
Exec=\"{}\"\n\
Icon={}\n\
Terminal=false\n\
Type=Application\n\
Categories=Utility;AudioVideo;\n\
StartupWMClass=ragasukma\n",
            exec_str,
            icon_path.to_string_lossy()
        );

        let need_write = match fs::read_to_string(&desktop_file) {
            Ok(existing) => existing != content,
            Err(_) => true,
        };

        if need_write {
            let _ = fs::write(&desktop_file, content);
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&desktop_file) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&desktop_file, perms);
            }
            println!("🚀 [Desktop Integration] Menu shortcut aplikasi terpasang di ~/.local/share/applications/ragasukma.desktop");
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
