use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};
use super::position::{apply_position_to_window, get_sway_output_geometry};
use super::state::OverlayState;
use std::thread;

pub fn open_browser_url_native(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn open_browser_url(url: String) -> Result<(), String> {
    open_browser_url_native(&url);
    Ok(())
}

#[tauri::command]
pub fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.as_ref().window().minimize();
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_maximize_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let window = main_win.as_ref().window();
        if let Ok(is_max) = window.is_maximized() {
            if is_max {
                let _ = window.unmaximize();
                return Ok(false);
            } else {
                let _ = window.maximize();
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[tauri::command]
pub fn set_window_mode(
    app: tauri::AppHandle,
    mode: String,
    page: Option<String>,
    state: tauri::State<'_, OverlayState>,
) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let window = main_win.as_ref().window();
        if mode == "panel" {
            // 1. Simpan posisi fisik aktual avatar sebelum bertransformasi ke panel
            if !state.is_panel_mode.load(std::sync::atomic::Ordering::SeqCst) {
                let mut cfg = match state.position_config.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let avatar_type = crate::config::load_user_config().avatar_type;
                super::position::update_position_config_from_window(&window, &mut cfg, &avatar_type);
                super::position::save_position_config(&cfg);
            }
            state.is_panel_mode.store(true, std::sync::atomic::Ordering::SeqCst);

            let _ = window.set_skip_taskbar(false);
            let _ = window.set_decorations(true);
            let _ = window.set_shadow(true);

            let page_str = page.as_deref().unwrap_or("");
            let is_tuner = page_str.contains("tuner");

            let title = match page_str {
                p if p.contains("menu") => "Ragasukma - Menu Utama",
                p if p.contains("settings") => "Ragasukma - Pengaturan",
                p if p.contains("skills") => "Ragasukma - Skill Commands",
                p if p.contains("tuner") => "Ragasukma - Pose Tuner",
                _ => "Ragasukma - Panel",
            };
            let _ = window.set_title(title);

            let (screen_w, screen_h, screen_x, screen_y) = if std::env::var("SWAYSOCK").is_ok() {
                get_sway_output_geometry().unwrap_or_else(|| {
                    if let Ok(Some(m)) = window.primary_monitor() {
                        (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
                    } else {
                        (1920, 1080, 0, 0)
                    }
                })
            } else if let Ok(Some(m)) = window.current_monitor() {
                (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
            } else if let Ok(Some(m)) = window.primary_monitor() {
                (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
            } else {
                (1920, 1080, 0, 0)
            };

            if is_tuner {
                let _ = window.maximize();
                let _ = window.set_always_on_top(true);
                let _ = window.set_focus();

                if std::env::var("SWAYSOCK").is_ok() {
                    let sway_cmd = format!(
                        r#"[title="(?i).*ragasukma.*|.*overlay.*"] floating enable, sticky enable, border normal, resize set {} px {} px, move position {} px {} px"#,
                        screen_w, screen_h, screen_x, screen_y
                    );
                    thread::spawn(move || {
                        let _ = std::process::Command::new("swaymsg")
                            .args([&sway_cmd])
                            .output();
                    });
                }
            } else {
                let _ = window.unmaximize();

                // Default ukuran ideal sesuai konten halaman
                let (target_w, target_h) = match page_str {
                    p if p.contains("menu") => (480, 660),
                    p if p.contains("settings") => (640, 720),
                    p if p.contains("skills") => (780, 650),
                    _ => (820, 680),
                };

                let panel_w = target_w.min(screen_w.saturating_sub(40));
                let panel_h = target_h.min(screen_h.saturating_sub(60));
                let panel_x = screen_x + (screen_w - panel_w) / 2;
                let panel_y = screen_y + (screen_h - panel_h) / 2;

                let _ = window.set_size(tauri::Size::Physical(PhysicalSize {
                    width: panel_w as u32,
                    height: panel_h as u32,
                }));
                let _ = window.set_position(tauri::Position::Physical(PhysicalPosition {
                    x: panel_x,
                    y: panel_y,
                }));
                let _ = window.set_always_on_top(true);
                let _ = window.set_focus();

                if std::env::var("SWAYSOCK").is_ok() {
                    let sway_cmd = format!(
                        r#"[title="(?i).*ragasukma.*|.*overlay.*"] floating enable, sticky enable, border normal, resize set {} px {} px, move position {} px {} px"#,
                        panel_w, panel_h, panel_x, panel_y
                    );
                    thread::spawn(move || {
                        let _ = std::process::Command::new("swaymsg")
                            .args([&sway_cmd])
                            .output();
                    });
                }
            }
        } else {
            state.is_panel_mode.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = window.set_skip_taskbar(true);
            let _ = window.unmaximize();
            let _ = window.set_decorations(false);
            let _ = window.set_shadow(false);
            let _ = window.set_title("Ragasukma Overlay");

            let avatar_type = crate::config::load_user_config().avatar_type;
            let cfg = match state.position_config.lock() {
                Ok(guard) => guard.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            };
            apply_position_to_window(&window, &cfg, &avatar_type);
        }
    }
    Ok(())
}

pub fn create_avatar_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        return Ok(window);
    }

    let window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
        .title("Ragasukma Overlay")
        .inner_size(180.0, 320.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .shadow(false)
        .resizable(true)
        .skip_taskbar(true)
        .focused(false)
        .devtools(true)
        .build()
        .map_err(|e| e.to_string())?;

    let u = crate::config::load_user_config();
    let state = app.state::<OverlayState>();
    let cfg = match state.position_config.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    apply_position_to_window(&window.as_ref().window(), &cfg, &u.avatar_type);

    Ok(window)
}

#[tauri::command]
pub fn close_current_window(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
pub fn open_menu_window(app: tauri::AppHandle) -> Result<(), String> {
    let _ = app.emit("open_view", serde_json::json!({ "url": "menu.html", "title": "Menu Utama" }));
    Ok(())
}

#[tauri::command]
pub fn open_skills_window(app: tauri::AppHandle) -> Result<(), String> {
    let _ = app.emit("open_view", serde_json::json!({ "url": "skills.html", "title": "Skill Commands" }));
    Ok(())
}

#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    let _ = app.emit("open_view", serde_json::json!({ "url": "settings.html", "title": "Pengaturan" }));
    Ok(())
}

#[tauri::command]
pub fn open_tuner_window(app: tauri::AppHandle) -> Result<(), String> {
    let _ = app.emit("open_view", serde_json::json!({ "url": "tuner.html", "title": "Pose Tuner" }));
    Ok(())
}
