use std::io::Write;

use crate::server::send_json_response;

pub fn handle_get_logs(full_url: &str, stream: &mut dyn Write) {
    let last_id = full_url
        .split('?')
        .nth(1)
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let mut kv = pair.split('=');
                if kv.next()? == "last_id" {
                    kv.next()?.parse::<usize>().ok()
                } else {
                    None
                }
            })
        })
        .unwrap_or(0);

    let (logs, _) = crate::config::get_logs_since(last_id);
    let json = serde_json::to_string(&logs).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_clear_logs(stream: &mut dyn Write) {
    crate::config::clear_logs();
    send_json_response(stream, 200, "{\"success\":true}");
}
