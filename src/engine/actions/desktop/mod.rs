pub mod click;
pub mod input;
pub mod window;

use crate::engine::window_scanner::ActiveWindowSnapshot;
use std::fs;

pub use click::{click_at_coordinates, click_desktop_button, find_target_button};
pub use input::{press_key, type_text};
pub use window::{close_window, focus_window, maximize_window, minimize_window};

pub fn execute_desktop_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "scan_visual" | "ocr" => {
            crate::engine::perception::trigger_vision_scan_async();
            Some((
                "Scan Screen".to_string(),
                "Visual screen scan running in background.".to_string(),
            ))
        }
        "focus" => {
            let target = payload.trim();
            if focus_window(target) {
                Some((
                    "Focus Window".to_string(),
                    format!("Window '{target}' focused"),
                ))
            } else {
                Some((
                    "Focus Window".to_string(),
                    format!("Failed to find or focus window '{target}'"),
                ))
            }
        }
        "minimize" => {
            let target = payload.trim();
            if minimize_window(target) {
                Some((
                    "Minimize Window".to_string(),
                    "Window minimized successfully.".to_string(),
                ))
            } else {
                Some((
                    "Minimize Window".to_string(),
                    "Failed to minimize window".to_string(),
                ))
            }
        }
        "maximize" => {
            let target = payload.trim();
            if maximize_window(target) {
                Some((
                    "Maximize Window".to_string(),
                    "Window maximized/restored.".to_string(),
                ))
            } else {
                Some((
                    "Maximize Window".to_string(),
                    "Failed to maximize window".to_string(),
                ))
            }
        }
        "close" => {
            let target = payload.trim();
            if close_window(target) {
                Some((
                    "Close Window".to_string(),
                    format!("Window '{target}' closed"),
                ))
            } else {
                Some((
                    "Close Window".to_string(),
                    format!("Failed to close window '{target}'"),
                ))
            }
        }
        "click_button" | "click_text" => {
            let target = payload.trim();
            if click_desktop_button(target) {
                Some((
                    "Click Desktop Element".to_string(),
                    format!("Element/text '{target}' clicked"),
                ))
            } else {
                Some((
                    "Click Desktop Element".to_string(),
                    format!("Element/text '{target}' not found on screen"),
                ))
            }
        }
        "click_at" => {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() >= 2 {
                let x = parts[0].trim().parse::<i32>().unwrap_or(0);
                let y = parts[1].trim().parse::<i32>().unwrap_or(0);
                if click_at_coordinates(x, y) {
                    Some((
                        "Click Coordinates".to_string(),
                        format!("Clicked at ({x}, {y})"),
                    ))
                } else {
                    Some((
                        "Click Coordinates".to_string(),
                        format!("Failed to click at point ({x}, {y})"),
                    ))
                }
            } else {
                Some((
                    "Click Coordinates".to_string(),
                    "Invalid format, use desktop:click_at|x|y".to_string(),
                ))
            }
        }
        "type" => {
            let text = payload;
            if type_text(text) {
                Some(("Type Text".to_string(), format!("Typed text: {text}")))
            } else {
                Some(("Type Text".to_string(), "Failed to type text".to_string()))
            }
        }
        "key" => {
            let key_name = payload.trim();
            if press_key(key_name) {
                Some(("Press Key".to_string(), format!("Key '{key_name}' pressed")))
            } else {
                Some((
                    "Press Key".to_string(),
                    format!("Key '{key_name}' not supported"),
                ))
            }
        }
        _ => None,
    }
}

pub(crate) fn load_snapshot() -> Option<ActiveWindowSnapshot> {
    let path = crate::assets::find_file_in_candidates("active_window_app.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(snap) = serde_json::from_str::<ActiveWindowSnapshot>(&content) {
            return Some(snap);
        }
    }
    None
}
