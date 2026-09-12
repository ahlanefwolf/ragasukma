use std::sync::Mutex;
use super::position::PositionConfig;

pub static APP_HANDLE: std::sync::LazyLock<Mutex<Option<tauri::AppHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

pub struct OverlayState {
    pub position_config: Mutex<PositionConfig>,
}

#[tauri::command]
pub fn get_app_config(state: tauri::State<'_, OverlayState>) -> serde_json::Value {
    let u = crate::config::load_user_config();
    let cfg = match state.position_config.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    serde_json::json!({
        "assistant_name": u.assistant_name,
        "avatar_type": u.avatar_type,
        "tts_engine": u.tts_engine,
        "size": cfg.size,
        "corner": cfg.corner,
    })
}

#[tauri::command]
pub fn get_app_state() -> Option<serde_json::Value> {
    crate::tray::TrayState::get_state()
}

#[tauri::command]
pub fn send_command(cmd: String) {
    if cmd == "test_voice" {
        std::thread::spawn(|| {
            let u = crate::config::load_user_config();
            let user_callsign = u.user_callsign;

            let tts = crate::tts::TextToSpeech::new();
            let is_interrupted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let text = format!("Halo {}! Ini adalah pengujian suara asisten. Suara saya sudah terdengar dengan jelas kan?", user_callsign);
            let _ = tts.speak_interruptible(&text, is_interrupted);
        });
        return;
    }
    crate::tray::TrayState::push_command(cmd);
}

#[tauri::command]
pub fn get_full_user_config() -> crate::config::UserConfig {
    crate::config::load_user_config()
}

#[tauri::command]
pub fn save_full_user_config(config: crate::config::UserConfig) -> Result<(), String> {
    crate::config::save_user_config(&config)?;
    crate::tray::TrayState::emit_event("assistant:config-updated", serde_json::to_value(&config).unwrap_or_default());
    Ok(())
}
