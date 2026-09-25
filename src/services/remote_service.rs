use std::net::TcpStream;
use serde_json::json;

use crate::engine::cloudflared::{
    get_tunnel_status, is_auth_key_valid, start_cloudflared_tunnel, stop_cloudflared_tunnel,
};
use crate::server::send_json_response;

pub fn handle_tunnel_status(stream: &mut TcpStream) {
    let (status, url, auth_key) = get_tunnel_status();
    let resp = json!({
        "status": status,
        "url": url,
        "auth_key": auth_key
    })
    .to_string();
    send_json_response(stream, 200, &resp);
}

pub fn handle_tunnel_start(stream: &mut TcpStream) {
    match start_cloudflared_tunnel(42128) {
        Ok((url, auth_key)) => {
            let resp = json!({
                "status": "running",
                "url": url,
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
        send_json_response(stream, 401, "{\"error\":\"Unauthorized: invalid remote key\"}");
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
            crate::term_log!("📱 [Mobile Remote] Menerima teks dari HP: \"{}\"", clean);
            send_json_response(stream, 200, "{\"success\":true,\"message\":\"Text sent to LLM\"}");
        } else {
            crate::term_log!("⚠️ [Mobile Web STT] Chat loop belum berjalan aktif!");
            send_json_response(stream, 503, "{\"error\":\"Voice chat loop is not currently active\"}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid payload, expected {text: string}\"}");
    }
}
