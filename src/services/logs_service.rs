use std::net::TcpStream;

use crate::server::send_json_response;

pub fn handle_get_logs(stream: &mut TcpStream) {
    let (logs, _) = crate::config::get_logs_since(0);
    let json = serde_json::to_string(&logs).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_clear_logs(stream: &mut TcpStream) {
    crate::config::clear_logs();
    send_json_response(stream, 200, "{\"success\":true}");
}
