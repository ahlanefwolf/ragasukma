use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::app_state::ServerSharedState;
use crate::assets::{load_config, save_config};
use crate::server::send_json_response;

pub fn handle_get_config(stream: &mut TcpStream) {
    let cfg = load_config();
    let json = serde_json::to_string(&cfg).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_post_config(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    if let Ok(cfg) = serde_json::from_str::<crate::config::UserConfig>(body_str) {
        if let Err(e) = save_config(&cfg) {
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        } else {
            if let Ok(mut lock) = shared.lock() {
                lock.config_updated = true;
            }
            send_json_response(stream, 200, "{\"success\":true}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid config format\"}");
    }
}
