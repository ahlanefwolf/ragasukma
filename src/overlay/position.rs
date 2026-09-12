use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

use super::state::OverlayState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionConfig {
    pub corner: String,
    pub margin_x: i32,
    pub margin_y: i32,
    pub size: i32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            corner: "top-right".to_string(),
            margin_x: 24,
            margin_y: 24,
            size: 320,
        }
    }
}

pub fn config_path() -> PathBuf {
    crate::config::get_app_config_dir().join("overlay_pos.json")
}

pub fn load_position_config() -> PositionConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<PositionConfig>(&content) {
                return cfg;
            }
        }
    }
    PositionConfig::default()
}

pub fn save_position_config(cfg: &PositionConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, json);
    }
}

/// Ambil geometri output aktif dari swaymsg (logical pixels, bukan physical)
/// Return: (width, height, x, y) dalam logical px
pub fn get_sway_output_geometry() -> Option<(i32, i32, i32, i32)> {
    let out = std::process::Command::new("swaymsg")
        .args(["-t", "get_outputs"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let arr = v.as_array()?;
    for o in arr {
        if o.get("active").and_then(|a| a.as_bool()).unwrap_or(false) {
            if let Some(rect) = o.get("rect") {
                let x = rect.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = rect.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let w = rect.get("width").and_then(|v| v.as_i64()).unwrap_or(1920) as i32;
                let h = rect.get("height").and_then(|v| v.as_i64()).unwrap_or(1080) as i32;
                return Some((w, h, x, y));
            }
        }
    }
    if let Some(o) = arr.first() {
        if let Some(rect) = o.get("rect") {
            let x = rect.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = rect.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let w = rect.get("width").and_then(|v| v.as_i64()).unwrap_or(1920) as i32;
            let h = rect.get("height").and_then(|v| v.as_i64()).unwrap_or(1080) as i32;
            return Some((w, h, x, y));
        }
    }
    None
}

pub fn apply_position_to_window(window: &tauri::Window, cfg: &PositionConfig, avatar_type: &str) {
    let w = if avatar_type.eq_ignore_ascii_case("vrm") {
        (cfg.size * 9) / 16
    } else {
        cfg.size
    };
    let h = cfg.size;

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

    let w = w.min(screen_w);
    let h = h.min(screen_h);

    let (x, y) = match cfg.corner.as_str() {
        "top-left" => (
            screen_x + cfg.margin_x,
            screen_y + cfg.margin_y,
        ),
        "top-right" => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + cfg.margin_y,
        ),
        "bottom-left" => (
            screen_x + cfg.margin_x,
            screen_y + (screen_h - h - cfg.margin_y),
        ),
        "bottom-right" => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + (screen_h - h - cfg.margin_y),
        ),
        _ => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + cfg.margin_y,
        ),
    };

    let _ = window.set_size(tauri::Size::Physical(PhysicalSize {
        width: w as u32,
        height: h as u32,
    }));
    let _ = window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }));
    let _ = window.set_always_on_top(true);

    if std::env::var("SWAYSOCK").is_ok() {
        let sway_cmd = format!(
            r#"[title="(?i).*ragasukma.*|.*overlay.*"] floating enable, sticky enable, border none, resize set {} px {} px, move position {} px {} px"#,
            w, h, x, y
        );
        thread::spawn(move || {
            let _ = std::process::Command::new("swaymsg")
                .args([&sway_cmd])
                .output();
        });
    }
}

#[tauri::command]
pub fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
pub fn drag_window_delta(
    app: tauri::AppHandle,
    dx: i32,
    dy: i32,
    state: tauri::State<'_, OverlayState>,
) {
    if let Some(main_win) = app.get_webview_window("main") {
        let window = main_win.as_ref().window();
        if let Ok(pos) = window.outer_position() {
            let new_x = pos.x + dx;
            let new_y = pos.y + dy;
            let _ = window.set_position(tauri::Position::Physical(PhysicalPosition {
                x: new_x,
                y: new_y,
            }));

            if let Ok(Some(monitor)) = window.current_monitor() {
                let m_size = monitor.size();
                let avatar_type = crate::config::load_user_config().avatar_type;
                let mut cfg = match state.position_config.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let corner = cfg.corner.clone();

                match corner.as_str() {
                    "top-right" => {
                        let w = if avatar_type.eq_ignore_ascii_case("vrm") { (cfg.size * 9) / 16 } else { cfg.size };
                        cfg.margin_x = (m_size.width as i32 - (new_x + w)).max(-200);
                        cfg.margin_y = new_y.max(-200);
                    }
                    "top-left" => {
                        cfg.margin_x = new_x.max(-200);
                        cfg.margin_y = new_y.max(-200);
                    }
                    "bottom-right" => {
                        let w = if avatar_type.eq_ignore_ascii_case("vrm") { (cfg.size * 9) / 16 } else { cfg.size };
                        let h = cfg.size;
                        cfg.margin_x = (m_size.width as i32 - (new_x + w)).max(-200);
                        cfg.margin_y = (m_size.height as i32 - (new_y + h)).max(-200);
                    }
                    "bottom-left" => {
                        let h = cfg.size;
                        cfg.margin_x = new_x.max(-200);
                        cfg.margin_y = (m_size.height as i32 - (new_y + h)).max(-200);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tauri::command]
pub fn save_current_position(state: tauri::State<'_, OverlayState>) {
    let cfg = match state.position_config.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    save_position_config(&cfg);
}

#[tauri::command]
pub fn change_size_delta(
    app: tauri::AppHandle,
    delta: i32,
    state: tauri::State<'_, OverlayState>,
) -> i32 {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let old_size = cfg.size;
    let new_size = (old_size + delta).clamp(120, 1800);

    if new_size != old_size {
        cfg.size = new_size;
        let avatar_type = crate::config::load_user_config().avatar_type;
        if let Some(main_win) = app.get_webview_window("main") {
            apply_position_to_window(&main_win.as_ref().window(), &cfg, &avatar_type);
        }
        save_position_config(&cfg);
    }
    cfg.size
}

#[tauri::command]
pub fn reset_size(app: tauri::AppHandle, state: tauri::State<'_, OverlayState>) {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    cfg.size = 320;
    let avatar_type = crate::config::load_user_config().avatar_type;
    if let Some(main_win) = app.get_webview_window("main") {
        apply_position_to_window(&main_win.as_ref().window(), &cfg, &avatar_type);
    }
    save_position_config(&cfg);
}

#[tauri::command]
pub fn set_corner(app: tauri::AppHandle, corner: String, state: tauri::State<'_, OverlayState>) {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    cfg.corner = corner.clone();
    cfg.margin_x = 24;
    cfg.margin_y = 24;
    let avatar_type = crate::config::load_user_config().avatar_type;
    if let Some(main_win) = app.get_webview_window("main") {
        apply_position_to_window(&main_win.as_ref().window(), &cfg, &avatar_type);
    }
    save_position_config(&cfg);
}
