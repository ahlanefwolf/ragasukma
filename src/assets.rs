use rust_embed::RustEmbed;
use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::config::UserConfig;
use crate::ui::pose_system::PosesConfig;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "i18n/*"]
#[include = "char.vrm"]
#[include = "char-avatar-*.png"]
#[include = "config-ai.json"]
#[include = "icon.png"]
#[include = "poses.json"]
#[include = "ragasukma-companion.zip"]
pub struct EmbeddedAssets;

pub use ragasukma_core::assets::{
    find_all_file_candidates, find_file_in_candidates, get_assets_dir,
};

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
    let mut path = find_file_in_candidates("poses.json");
    if !path.exists() {
        path = crate::config::get_app_config_dir().join("poses.json");
    }
    let json = serde_json::to_string_pretty(poses).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Ensures default config files (config-ai.json, poses.json) go to user config directory (~/.config/ragasukma)
/// and large media assets (char.vrm, avatars, companion zip, i18n) go to user data directory (~/.local/share/ragasukma/assets).
pub fn ensure_default_user_assets() {
    let assets_dir = get_assets_dir();
    let _ = fs::create_dir_all(&assets_dir);

    let config_dir = crate::config::get_app_config_dir();
    let _ = fs::create_dir_all(&config_dir);

    let recipes_dir = crate::engine::actions::recipe::types::get_recipes_dir();
    let _ = fs::create_dir_all(&recipes_dir);

    // Dynamically extract all asset files embedded in the binary
    for file_path in EmbeddedAssets::iter() {
        let file_name = file_path.as_ref();
        let is_config_file = file_name == "config-ai.json" || file_name == "poses.json";
        let target_path = if is_config_file {
            config_dir.join(file_name)
        } else {
            assets_dir.join(file_name)
        };

        let existing = find_file_in_candidates(file_name);
        if !target_path.exists() && !existing.exists() {
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Some(embedded_file) = EmbeddedAssets::get(file_name) {
                if let Err(e) = fs::write(&target_path, embedded_file.data) {
                    eprintln!(
                        " Failed to extract default asset '{}': {:?}",
                        file_name, e
                    );
                } else {
                    println!(
                        "[Assets] Successfully extracted default asset: {:?}",
                        target_path
                    );
                }
            }
        }
    }

    // Ensure initial account registries exist in config directory
    let config_initial_files: &[(&str, &str)] = &[
        ("antigravity_accounts.json", "[]\n"),
        ("codex_accounts.json", "[]\n"),
        ("claude_accounts.json", "[]\n"),
    ];

    for (filename, default_content) in config_initial_files {
        let existing = find_file_in_candidates(filename);
        if !existing.exists() {
            let target_path = config_dir.join(filename);
            let _ = fs::write(&target_path, default_content);
        }
        let local_path = PathBuf::from("assets").join(filename);
        if PathBuf::from("Cargo.toml").exists() && !local_path.exists() {
            let _ = fs::write(&local_path, default_content);
        }
    }

    // Ensure initial runtime snapshots exist in cache directory (or local dev assets)
    let cache_dir = crate::config::get_app_cache_dir();
    let _ = fs::create_dir_all(&cache_dir);

    let cache_initial_files: &[(&str, &str)] = &[
        (
            "active_window_app.json",
            "{\n  \"app_name\": \"\",\n  \"window_title\": \"\",\n  \"window_id\": \"\",\n  \"source\": \"desktop_window\",\n  \"timestamp\": 0,\n  \"windows\": [],\n  \"buttons\": [],\n  \"inputs\": [],\n  \"texts\": [],\n  \"text_summary\": \"\"\n}\n",
        ),
        (
            "active_tab_browser.json",
            "{\n  \"url\": \"\",\n  \"domain\": \"\",\n  \"title\": \"\",\n  \"timestamp\": 0,\n  \"tabs\": [],\n  \"buttons\": [],\n  \"inputs\": [],\n  \"texts\": [],\n  \"text_summary\": \"\",\n  \"is_loading\": false,\n  \"pending_requests\": 0,\n  \"is_network_idle\": true\n}\n",
        ),
    ];

    for (filename, default_content) in cache_initial_files {
        let local_path = PathBuf::from("assets").join(filename);
        if PathBuf::from("Cargo.toml").exists() {
            if !local_path.exists() {
                let _ = fs::write(&local_path, default_content);
            }
        } else {
            let target_path = cache_dir.join(filename);
            if !target_path.exists() {
                let _ = fs::write(&target_path, default_content);
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
        let icon_cand = find_file_in_candidates("icon.png");
        let icon_path = if icon_cand.exists() {
            icon_cand
        } else {
            find_file_in_candidates("avatar.png")
        };

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
                "[Desktop Integration] Application shortcut created at ~/.local/share/applications/ragasukma.desktop"
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
            use windows::core::HSTRING;
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

            let h_op = HSTRING::from("open");
            let h_url = HSTRING::from(&url);

            unsafe {
                let _ = ShellExecuteW(
                    None,
                    windows::core::PCWSTR(h_op.as_wide().as_ptr()),
                    windows::core::PCWSTR(h_url.as_wide().as_ptr()),
                    None,
                    None,
                    SW_SHOWNORMAL,
                );
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

            // If the URL is internal chrome://, open it directly in the Chrome/Brave binary as a new tab
            for bin in chromium_candidates {
                let mut cmd = std::process::Command::new(bin);
                cmd.arg(&url);
                cmd.env_remove("LD_LIBRARY_PATH");
                cmd.env_remove("LD_PRELOAD");

                if cmd.spawn().is_ok() {
                    println!("[Browser Tab] Opening new tab via {}", bin);
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
