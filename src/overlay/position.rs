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
    #[serde(default)]
    pub custom_x: Option<i32>,
    #[serde(default)]
    pub custom_y: Option<i32>,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            corner: "top-right".to_string(),
            margin_x: 24,
            margin_y: 24,
            size: 320,
            custom_x: None,
            custom_y: None,
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

/// Cari rect window overlay di dalam tree Sway
fn find_window_rect_in_sway_tree(v: &serde_json::Value) -> Option<(i32, i32)> {
    let app_id = v.get("app_id").and_then(|a| a.as_str()).unwrap_or("");
    let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let is_match = app_id.to_lowercase().contains("ragasukma")
        || name.to_lowercase().contains("ragasukma")
        || name.to_lowercase().contains("overlay");

    if is_match {
        if let Some(rect) = v.get("rect") {
            let x = rect.get("x")?.as_i64()? as i32;
            let y = rect.get("y")?.as_i64()? as i32;
            return Some((x, y));
        }
    }

    if let Some(nodes) = v.get("floating_nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Some(p) = find_window_rect_in_sway_tree(node) {
                return Some(p);
            }
        }
    }

    if let Some(nodes) = v.get("nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Some(p) = find_window_rect_in_sway_tree(node) {
                return Some(p);
            }
        }
    }
    None
}

/// Dapatkan koordinat fisik aktual window di semua OS (Sway IPC / Windows / X11)
pub fn get_actual_window_position(window: &tauri::Window) -> Option<(i32, i32)> {
    if std::env::var("SWAYSOCK").is_ok() {
        if let Ok(out) = std::process::Command::new("swaymsg")
            .args(["-t", "get_tree"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Ok(tree) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(pos) = find_window_rect_in_sway_tree(&tree) {
                    return Some(pos);
                }
            }
        }
    }

    if let Ok(pos) = window.outer_position() {
        if pos.x > 0 || pos.y > 0 {
            return Some((pos.x, pos.y));
        }
    }
    None
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

/// Dapatkan batas layar (width, height, offset_x, offset_y)
pub fn get_screen_bounds(window: &tauri::Window) -> (i32, i32, i32, i32) {
    if std::env::var("SWAYSOCK").is_ok() {
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
    }
}

pub fn get_current_overlay_coords(
    window: &tauri::Window,
    cfg: &PositionConfig,
    avatar_type: &str,
) -> (i32, i32) {
    if let (Some(cx), Some(cy)) = (cfg.custom_x, cfg.custom_y) {
        return (cx, cy);
    }

    let (screen_w, screen_h, screen_x, screen_y) = get_screen_bounds(window);
    let w = if avatar_type.eq_ignore_ascii_case("vrm") {
        (cfg.size * 9) / 16
    } else {
        cfg.size
    };
    let h = cfg.size;

    match cfg.corner.as_str() {
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
    }
}

pub fn apply_position_to_window(window: &tauri::Window, cfg: &PositionConfig, avatar_type: &str) {
    let w = if avatar_type.eq_ignore_ascii_case("vrm") {
        (cfg.size * 9) / 16
    } else {
        cfg.size
    };
    let h = cfg.size;

    let (x, y) = get_current_overlay_coords(window, cfg, avatar_type);

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
        let _ = std::process::Command::new("swaymsg")
            .args([&sway_cmd])
            .output();
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
        let mut cfg = match state.position_config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let avatar_type = crate::config::load_user_config().avatar_type;

        let (cur_x, cur_y) = get_current_overlay_coords(&window, &cfg, &avatar_type);
        let new_x = cur_x + dx;
        let new_y = cur_y + dy;

        cfg.custom_x = Some(new_x);
        cfg.custom_y = Some(new_y);

        let _ = window.set_position(tauri::Position::Physical(PhysicalPosition {
            x: new_x,
            y: new_y,
        }));

        if std::env::var("SWAYSOCK").is_ok() {
            let sway_cmd = format!(
                r#"[title="(?i).*ragasukma.*|.*overlay.*"] move position {} px {} px"#,
                new_x, new_y
            );
            thread::spawn(move || {
                let _ = std::process::Command::new("swaymsg")
                    .args([&sway_cmd])
                    .output();
            });
        }
    }
}

pub fn update_position_config_from_window(
    window: &tauri::Window,
    cfg: &mut PositionConfig,
    _avatar_type: &str,
) {
    if let Some((x, y)) = get_actual_window_position(window) {
        cfg.custom_x = Some(x);
        cfg.custom_y = Some(y);
    }
}

#[tauri::command]
pub fn save_current_position(app: tauri::AppHandle, state: tauri::State<'_, OverlayState>) {
    let cfg = {
        let mut cfg_guard = match state.position_config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(main_win) = app.get_webview_window("main") {
            let avatar_type = crate::config::load_user_config().avatar_type;
            update_position_config_from_window(&main_win.as_ref().window(), &mut cfg_guard, &avatar_type);
        }
        cfg_guard.clone()
    };
    save_position_config(&cfg);
}

#[tauri::command]
pub fn set_overlay_size(
    app: tauri::AppHandle,
    size: i32,
    state: tauri::State<'_, OverlayState>,
) -> i32 {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let avatar_type = crate::config::load_user_config().avatar_type;
    let target_size = size.clamp(160, 1400);

    if let Some(main_win) = app.get_webview_window("main") {
        let window = main_win.as_ref().window();
        let (screen_w, screen_h, screen_x, screen_y) = get_screen_bounds(&window);

        let old_w = if avatar_type.eq_ignore_ascii_case("vrm") { (cfg.size * 9) / 16 } else { cfg.size };
        let old_h = cfg.size;
        let new_w = if avatar_type.eq_ignore_ascii_case("vrm") { (target_size * 9) / 16 } else { target_size };
        let new_h = target_size;

        let (cur_x, cur_y) = if let Some((act_x, act_y)) = get_actual_window_position(&window) {
            (act_x, act_y)
        } else {
            get_current_overlay_coords(&window, &cfg, &avatar_type)
        };

        // Jika avatar di paruh bawah layar, tahan posisi dasar/bawah (tumbuh ke atas)
        let new_y = if cur_y + old_h > screen_y + (screen_h / 2) {
            cur_y - (new_h - old_h)
        } else {
            cur_y
        };

        // Jika avatar di paruh kanan layar, tahan posisi kanan (tumbuh ke kiri)
        let new_x = if cur_x + old_w > screen_x + (screen_w / 2) {
            cur_x - (new_w - old_w)
        } else {
            cur_x
        };

        let max_x = (screen_x + screen_w - new_w).max(screen_x);
        let max_y = (screen_y + screen_h - new_h).max(screen_y);
        let clamped_x = new_x.clamp(screen_x, max_x);
        let clamped_y = new_y.clamp(screen_y, max_y);

        cfg.custom_x = Some(clamped_x);
        cfg.custom_y = Some(clamped_y);
        cfg.size = target_size;

        apply_position_to_window(&window, &cfg, &avatar_type);
    } else {
        cfg.size = target_size;
    }
    save_position_config(&cfg);
    cfg.size
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
    let avatar_type = crate::config::load_user_config().avatar_type;

    let old_size = cfg.size;
    let new_size = (old_size + delta).clamp(120, 1800);

    if new_size != old_size {
        if let Some(main_win) = app.get_webview_window("main") {
            let window = main_win.as_ref().window();
            let (screen_w, screen_h, screen_x, screen_y) = get_screen_bounds(&window);

            let old_w = if avatar_type.eq_ignore_ascii_case("vrm") { (old_size * 9) / 16 } else { old_size };
            let old_h = old_size;
            let new_w = if avatar_type.eq_ignore_ascii_case("vrm") { (new_size * 9) / 16 } else { new_size };
            let new_h = new_size;

            let (cur_x, cur_y) = if let Some((act_x, act_y)) = get_actual_window_position(&window) {
                (act_x, act_y)
            } else {
                get_current_overlay_coords(&window, &cfg, &avatar_type)
            };

            // Jika avatar di paruh bawah layar, tahan posisi dasar/bawah (tumbuh ke atas)
            let new_y = if cur_y + old_h > screen_y + (screen_h / 2) {
                cur_y - (new_h - old_h)
            } else {
                cur_y
            };

            // Jika avatar di paruh kanan layar, tahan posisi kanan (tumbuh ke kiri)
            let new_x = if cur_x + old_w > screen_x + (screen_w / 2) {
                cur_x - (new_w - old_w)
            } else {
                cur_x
            };

            let max_x = (screen_x + screen_w - new_w).max(screen_x);
            let max_y = (screen_y + screen_h - new_h).max(screen_y);
            let clamped_x = new_x.clamp(screen_x, max_x);
            let clamped_y = new_y.clamp(screen_y, max_y);

            cfg.custom_x = Some(clamped_x);
            cfg.custom_y = Some(clamped_y);
            cfg.size = new_size;

            apply_position_to_window(&window, &cfg, &avatar_type);
        } else {
            cfg.size = new_size;
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
    cfg.custom_x = None;
    cfg.custom_y = None;
    cfg.margin_x = 24;
    cfg.margin_y = 24;
    let avatar_type = crate::config::load_user_config().avatar_type;
    if let Some(main_win) = app.get_webview_window("main") {
        let window = main_win.as_ref().window();
        let (x, y) = get_current_overlay_coords(&window, &cfg, &avatar_type);
        cfg.custom_x = Some(x);
        cfg.custom_y = Some(y);
        apply_position_to_window(&window, &cfg, &avatar_type);
    }
    save_position_config(&cfg);
}
