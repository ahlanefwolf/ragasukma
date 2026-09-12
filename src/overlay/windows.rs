use tauri::Manager;
use super::position::apply_position_to_window;
use super::state::OverlayState;

pub fn open_browser_url_native(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
}

#[tauri::command]
pub fn open_browser_url(url: String) -> Result<(), String> {
    open_browser_url_native(&url);
    Ok(())
}

#[tauri::command]
pub fn open_tuner_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pose-tuner") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "pose-tuner", tauri::WebviewUrl::App("tuner.html".into()))
        .title("VRM Pose & Motion Tuner")
        .inner_size(880.0, 680.0)
        .resizable(true)
        .decorations(true)
        .always_on_top(false)
        .focused(true)
        .devtools(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_avatar_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    let window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
        .title("ragasukma-overlay")
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
pub fn toggle_avatar_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        let is_vis = window.is_visible().unwrap_or(false);
        if is_vis {
            let _ = window.hide();
            Ok(false)
        } else {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
            Ok(true)
        }
    } else {
        create_avatar_window(&app)?;
        Ok(true)
    }
}

#[tauri::command]
pub fn is_avatar_visible(app: tauri::AppHandle) -> bool {
    if let Some(window) = app.get_webview_window("main") {
        window.is_visible().unwrap_or(false)
    } else {
        false
    }
}

#[tauri::command]
pub fn close_current_window(window: tauri::Window) {
    if window.label() == "assistant-menu" {
        std::process::exit(0);
    } else {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn open_menu_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("assistant-menu") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "assistant-menu", tauri::WebviewUrl::App("menu.html".into()))
        .title("Assistant Menu & Settings")
        .inner_size(380.0, 590.0)
        .resizable(true)
        .decorations(true)
        .always_on_top(true)
        .focused(true)
        .devtools(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_skills_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("skills") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "skills", tauri::WebviewUrl::App("skills.html".into()))
        .title("Skill Commands Manager")
        .inner_size(680.0, 560.0)
        .resizable(true)
        .decorations(true)
        .always_on_top(true)
        .focused(true)
        .devtools(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "settings", tauri::WebviewUrl::App("settings.html".into()))
        .title("Assistant Settings")
        .inner_size(520.0, 640.0)
        .resizable(true)
        .decorations(true)
        .always_on_top(true)
        .devtools(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
