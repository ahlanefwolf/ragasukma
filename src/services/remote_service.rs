use serde_json::json;
use std::net::TcpStream;

use crate::engine::cloudflared::{
    check_cloudflared, get_tunnel_status, install_cloudflared, is_auth_key_valid,
    start_cloudflared_tunnel, stop_cloudflared_tunnel,
};
use crate::server::send_json_response;

pub fn handle_tunnel_status(stream: &mut TcpStream) {
    let (status, url, auth_key) = get_tunnel_status();
    let cf_info = check_cloudflared();
    let resp = json!({
        "status": status,
        "url": url,
        "auth_key": auth_key,
        "cloudflared": cf_info
    })
    .to_string();
    send_json_response(stream, 200, &resp);
}

pub fn handle_tunnel_check(stream: &mut TcpStream) {
    let cf_info = check_cloudflared();
    let resp = json!(cf_info).to_string();
    send_json_response(stream, 200, &resp);
}

pub fn handle_tunnel_install(stream: &mut TcpStream) {
    match install_cloudflared() {
        Ok(msg) => {
            let info = check_cloudflared();
            let resp = json!({
                "success": true,
                "message": msg,
                "info": info
            })
            .to_string();
            send_json_response(stream, 200, &resp);
        }
        Err(err) => {
            let resp = json!({
                "success": false,
                "message": err
            })
            .to_string();
            send_json_response(stream, 500, &resp);
        }
    }
}

pub fn handle_tunnel_start(stream: &mut TcpStream) {
    match start_cloudflared_tunnel(42128) {
        Ok((url, auth_key)) => {
            let status = if url.is_empty() {
                "starting"
            } else {
                "running"
            };
            let resp = json!({
                "status": status,
                "url": if url.is_empty() { None } else { Some(url) },
                "auth_key": auth_key
            })
            .to_string();
            send_json_response(stream, 200, &resp);
        }
        Err(err) => {
            let resp = json!({
                "error": err
            })
            .to_string();
            send_json_response(stream, 500, &resp);
        }
    }
}

pub fn handle_tunnel_stop(stream: &mut TcpStream) {
    let _ = stop_cloudflared_tunnel();
    let resp = json!({
        "status": "stopped"
    })
    .to_string();
    send_json_response(stream, 200, &resp);
}

fn extract_query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == key {
                return Some(v);
            }
        }
    }
    None
}

pub fn handle_remote_text(full_url: &str, body_str: &str, stream: &mut TcpStream) {
    let query = full_url.split('?').nth(1).unwrap_or("");
    let key = extract_query_param(query, "key").unwrap_or("");

    if !is_auth_key_valid(key) {
        send_json_response(
            stream,
            401,
            "{\"error\":\"Unauthorized: invalid remote key\"}",
        );
        return;
    }

    #[derive(serde::Deserialize)]
    struct TextPayload {
        text: String,
    }

    if let Ok(payload) = serde_json::from_str::<TextPayload>(body_str) {
        let clean = payload.text.trim();
        if clean.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"Empty text string\"}");
            return;
        }

        let injected = crate::engine::remote_text::inject_remote_text(clean.to_string());
        if injected {
            crate::term_log!("📱 [Mobile Remote] Received text from phone: \"{}\"", clean);
            send_json_response(
                stream,
                200,
                "{\"success\":true,\"message\":\"Text sent to LLM\"}",
            );
        } else {
            crate::term_log!("⚠️ [Mobile Web STT] Chat loop is not active!");
            send_json_response(
                stream,
                503,
                "{\"error\":\"Voice chat loop is not currently active\"}",
            );
        }
    } else {
        send_json_response(
            stream,
            400,
            "{\"error\":\"Invalid payload, expected {text: string}\"}",
        );
    }
}

pub fn handle_stt_transcript(body_str: &str, stream: &mut TcpStream) {
    #[derive(serde::Deserialize)]
    struct SttPayload {
        #[serde(alias = "transcript")]
        text: String,
    }

    if let Ok(payload) = serde_json::from_str::<SttPayload>(body_str) {
        let clean = payload.text.trim();
        if clean.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"Empty transcript string\"}");
            return;
        }

        // 1. Anti-Echolalia Protection: Tolak transkrip jika merupakan pantulan suara speaker asisten sendiri
        if crate::app_state::is_echo_of_last_spoken(clean) {
            crate::term_log!(
                "🤫 [Anti-Echolalia] Echo transcript from assistant speaker ignored: \"{}\"",
                clean
            );
            send_json_response(
                stream,
                200,
                "{\"success\":false,\"ignored\":true,\"message\":\"Echo of assistant speech ignored\"}",
            );
            return;
        }

        let cfg = crate::config::load_user_config();
        let ai_speaking = crate::app_state::is_ai_speaking()
            || crate::audio::player::is_native_voice_playing();

        // Jika AI sedang berbicara dan interupsi suara dinonaktifkan oleh pengguna,
        // tolak mentah-mentah transkrip agar tidak terjadi tumpang-tindih ucapan.
        if ai_speaking && !cfg.voice_interruption {
            crate::term_log!(
                "🤫 [Chrome Web STT] Transcript ignored because assistant is speaking (interruption disabled): \"{}\"",
                clean
            );
            send_json_response(
                stream,
                200,
                "{\"success\":false,\"ignored\":true,\"message\":\"AI is currently speaking and voice interruption is disabled\"}",
            );
            return;
        }

        let injected = crate::engine::remote_text::inject_remote_text(clean.to_string());
        if injected {
            if ai_speaking && cfg.voice_interruption {
                crate::term_log!(
                    "⚡ [Chrome Web STT] Voice interruption cuts assistant speech: \"{}\"",
                    clean
                );
                crate::app_state::trigger_interruption();
            } else {
                crate::term_log!("🎙️ [Chrome Web STT] Transcript received: \"{}\"", clean);
            }
            let asst_name = cfg.assistant_name;
            crate::tray::TrayState::update(
                crate::services::tray_service::AssistantState::Thinking,
                &format!("{asst_name} Thinking..."),
            );
            send_json_response(
                stream,
                200,
                "{\"success\":true,\"message\":\"Transcript injected to LLM\"}",
            );
        } else {
            crate::term_log!("⚠️ [Chrome Web STT] Chat loop is not active!");
            send_json_response(
                stream,
                503,
                "{\"error\":\"Voice chat loop is not currently active\"}",
            );
        }
    } else {
        send_json_response(
            stream,
            400,
            "{\"error\":\"Invalid payload, expected {text: string} or {transcript: string}\"}",
        );
    }
}

pub fn handle_tts_state(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &std::sync::Arc<std::sync::Mutex<crate::app_state::ServerSharedState>>,
) {
    #[derive(serde::Deserialize)]
    struct TtsStatePayload {
        #[serde(default)]
        speaking: bool,
        #[serde(default)]
        listening: bool,
        #[serde(default)]
        state: Option<String>,
        emotion: Option<String>,
    }

    if let Ok(payload) = serde_json::from_str::<TtsStatePayload>(body_str) {
        let emotion = payload.emotion.unwrap_or_else(|| "neutral".to_string());
        let target_state = if let Some(ref st) = payload.state {
            st.to_lowercase()
        } else if payload.speaking {
            "speaking".to_string()
        } else if payload.listening {
            "listening".to_string()
        } else {
            "idle".to_string()
        };

        if target_state == "speaking" {
            crate::app_state::set_ai_speaking(true);
            crate::app_state::touch_speech_activity();
            let asst_name = crate::config::load_user_config().assistant_name;
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Speaking,
                &format!("{asst_name} Speaking"),
                &emotion,
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = crate::ui::pose_system::AssistantVisualState::Speaking;
                lock.current_emotion = emotion;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if target_state == "listening" {
            // Guard: Jangan izinkan masuk mode listening jika voice assistant belum dinyalakan (standby)
            if !crate::app_state::is_session_running() {
                send_json_response(stream, 200, "{\"success\":true,\"ignored\":true}");
                return;
            }
            crate::audio::player::stop_native_voice();
            crate::app_state::set_ai_speaking(false);
            let asst_name = crate::config::load_user_config().assistant_name;
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Listening,
                &format!("{asst_name} Listening..."),
                &emotion,
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = crate::ui::pose_system::AssistantVisualState::Listening;
                lock.current_emotion = emotion;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if target_state == "thinking" {
            crate::app_state::set_ai_speaking(false);
            let asst_name = crate::config::load_user_config().assistant_name;
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Thinking,
                &format!("{asst_name} Thinking..."),
                &emotion,
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = crate::ui::pose_system::AssistantVisualState::Thinking;
                lock.current_emotion = emotion;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else {
            crate::audio::player::stop_native_voice();
            crate::app_state::set_ai_speaking(false);
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Idle,
                "Standby",
                "neutral",
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = crate::ui::pose_system::AssistantVisualState::Idle;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        }
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid TTS state payload\"}");
    }
}
