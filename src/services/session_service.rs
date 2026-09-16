use serde::Deserialize;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::app_state::{ACTIVE_WEB_SESSION, ActiveWebSession, ServerSharedState};
use crate::server::send_json_response;

/// Klaim kepemilikan sesi dashboard web tunggal (hanya dipanggil 1x saat tab dibuka / di-refresh)
pub fn handle_session_claim(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    #[derive(Deserialize)]
    struct ClaimPayload {
        token: String,
        active_tab: Option<String>,
    }
    if let Ok(payload) = serde_json::from_str::<ClaimPayload>(body_str) {
        let now = std::time::Instant::now();
        let current_tab = payload.active_tab.unwrap_or_else(|| "menu".to_string());
        if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
            *lock = Some(ActiveWebSession {
                token: payload.token.clone(),
                active_tab: current_tab,
                last_seen: now,
            });
        }
        if let Ok(mut s) = shared.lock() {
            s.state_version = s.state_version.wrapping_add(1);
        }
        // Kirim event session claim ke antrean agar semua SSE client terupdate instan
        crate::app_state::push_client_action("session:claimed", Some(&payload.token));

        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid payload, expected {token}\"}");
    }
}

/// Update active tab saat berpindah tab di dashboard
pub fn handle_session_tab(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct TabPayload {
        active_tab: String,
    }
    if let Ok(payload) = serde_json::from_str::<TabPayload>(body_str) {
        if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
            if let Some(session) = lock.as_mut() {
                session.active_tab = payload.active_tab;
                session.last_seen = std::time::Instant::now();
            }
        }
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid tab payload\"}");
    }
}
