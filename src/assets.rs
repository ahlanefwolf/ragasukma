use rust_embed::RustEmbed;
use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::config::UserConfig;
use crate::ui::pose_system::PosesConfig;

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
    candidates.push(get_assets_dir().join(filename));

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

    candidates.push(
        crate::config::get_app_config_dir()
            .join("assets")
            .join(filename),
    );
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

pub fn get_extension_dir() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(cur) = std::env::current_dir() {
        candidates.push(cur.join("extension"));
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("extension"));
            candidates.push(exe_dir.join("../Resources/extension"));
        }
    }
    candidates.push(PathBuf::from("extension"));

    for candidate in candidates {
        if candidate.exists() {
            return std::fs::canonicalize(&candidate).unwrap_or(candidate);
        }
    }
    PathBuf::from("extension")
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

/// Memastikan seluruh file aset default (char.vrm, poses.json, avatar PNG, skill-command.md, config-ai.json, todo.md, dll)
/// otomatis diekstrak ke direktori config pengguna (~/.config/ragasukma/assets) saat pertama kali diinstal/dijalankan.
pub fn ensure_default_user_assets() {
    let cfg_assets_dir = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&cfg_assets_dir);

    // Ekstrak semua file aset yang ter-embed dalam binary secara dinamis
    for file_path in EmbeddedAssets::iter() {
        let file_name = file_path.as_ref();
        let target_path = cfg_assets_dir.join(file_name);

        if !target_path.exists() {
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Some(embedded_file) = EmbeddedAssets::get(file_name) {
                if let Err(e) = fs::write(&target_path, embedded_file.data) {
                    eprintln!(
                        "⚠️ Failed to extract default asset '{}': {:?}",
                        file_name, e
                    );
                } else {
                    println!(
                        "📦 [Assets] Successfully extracted default asset: {:?}",
                        target_path
                    );
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let apps_dir = PathBuf::from(format!("{}/.local/share/applications", home));
        let _ = fs::create_dir_all(&apps_dir);
        let desktop_file = apps_dir.join("ragasukma.desktop");

        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ragasukma"));
        let exec_str = exe_path.to_string_lossy();
        let icon_path = find_file_in_candidates("avatar.png");

        let content = format!(
            "[Desktop Entry]\n\
Version=1.0\n\
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
            println!(
                "🚀 [Desktop Integration] Application shortcut created at ~/.local/share/applications/ragasukma.desktop"
            );
        }
    }
}

pub fn open_browser(url: &str) {
    let url = url.to_string();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(150));

        #[cfg(target_os = "windows")]
        {
            let mut opened = false;
            let chrome_candidates = ["chrome.exe", "msedge.exe", "brave.exe"];
            for browser in chrome_candidates {
                let mut cmd = std::process::Command::new(browser);
                cmd.arg(&url);

                if cmd.spawn().is_ok() {
                    opened = true;
                    break;
                }
            }
            if !opened {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &url])
                    .spawn();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let chromium_candidates = [
                "google-chrome-stable",
                "google-chrome",
                "brave-browser",
                "chromium-browser",
                "chromium",
                "microsoft-edge-stable",
            ];

            let mut opened = false;

            // Jika URL adalah internal chrome://, buka langsung di binary Chrome/Brave sebagai tab baru
            for bin in chromium_candidates {
                let mut cmd = std::process::Command::new(bin);
                cmd.arg(&url);
                cmd.env_remove("LD_LIBRARY_PATH");
                cmd.env_remove("LD_PRELOAD");

                if cmd.spawn().is_ok() {
                    println!("🌐 [Browser Tab] Opening new tab via {}", bin);
                    opened = true;
                    break;
                }
            }

            if !opened {
                let mut cmd = std::process::Command::new("xdg-open");
                cmd.arg(&url);
                cmd.env_remove("LD_LIBRARY_PATH");
                cmd.env_remove("LD_PRELOAD");
                let _ = cmd.spawn();
            }
        }
    });
}
