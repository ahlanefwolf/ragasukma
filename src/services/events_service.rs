use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app_state::ServerSharedState;
use crate::ui::pose_system::AssistantVisualState;
use serde::Serialize;

fn write_sse_header(stream: &mut TcpStream) -> bool {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(None);
    let _ = stream.set_write_timeout(None);

    let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache, no-transform\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\nX-Accel-Buffering: no\r\n\r\n";
    if stream.write_all(header.as_bytes()).is_err() {
        return false;
    }
    stream.flush().is_ok()
}

// ---------------------------------------------------------------------------
// 1. GLOBAL APP STATE & ACTION SSE STREAM (/api/events/state)
// Murni status visual AI dan trigger skill perintah (Event-driven)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct CoreStatePayload {
    pub state: String,
    pub emotion: String,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub is_running: bool,
}

pub fn handle_sse_state(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    if !write_sse_header(&mut stream) {
        return;
    }

    let mut last_state = String::new();
    let mut last_emotion = String::new();
    let mut last_voice_muted = false;
    let mut last_mic_muted = false;
    let mut last_is_running = false;
    let mut initial_sent = false;
    let mut ping_ticks: u32 = 0;
    let mut last_action_id = crate::app_state::get_latest_action_id();

    loop {
        let mut had_activity = false;

        // 1. Cek perubahan Status Visual AI (event: state)
        let (should_send_state, payload_json) = if let Ok(lock) = shared.lock() {
            let st_str = if lock.is_shutdown {
                "shutdown"
            } else {
                match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                }
            };

            let v_muted = lock.voice_muted;
            let m_muted = lock.mic_muted;
            let emo = lock.current_emotion.clone();
            let running = lock.is_running;

            let changed = !initial_sent
                || st_str != last_state
                || emo != last_emotion
                || v_muted != last_voice_muted
                || m_muted != last_mic_muted
                || running != last_is_running;

            if changed {
                initial_sent = true;
                last_state = st_str.to_string();
                last_emotion = emo.clone();
                last_voice_muted = v_muted;
                last_mic_muted = m_muted;
                last_is_running = running;

                let payload = CoreStatePayload {
                    state: st_str.to_string(),
                    emotion: emo,
                    voice_muted: v_muted,
                    mic_muted: m_muted,
                    is_running: running,
                };
                (true, serde_json::to_string(&payload).unwrap_or_default())
            } else {
                (false, String::new())
            }
        } else {
            (false, String::new())
        };

        if should_send_state {
            let msg = format!("event: state\ndata: {}\n\n", payload_json);
            if stream.write_all(msg.as_bytes()).is_err() || stream.flush().is_err() {
                break;
            }
            had_activity = true;
        }

        // 2. Cek apakah ada Aksi Perintah Baru dari AI (event: action) - Multi-Client Pub-Sub
        let (pending_actions, new_last_id) = crate::app_state::get_client_actions_since(last_action_id);
        last_action_id = new_last_id;
        for act in pending_actions {
            if let Ok(act_json) = serde_json::to_string(&act) {
                let msg = format!("event: action\ndata: {}\n\n", act_json);
                if stream.write_all(msg.as_bytes()).is_err() || stream.flush().is_err() {
                    return;
                }
                had_activity = true;
            }
        }

        if had_activity {
            ping_ticks = 0;
        } else {
            ping_ticks += 1;
            // Ping heartbeat setiap ~3 detik (30 * 100ms)
            if ping_ticks >= 30 {
                ping_ticks = 0;
                let ping = ": ping\n\n";
                if stream.write_all(ping.as_bytes()).is_err() || stream.flush().is_err() {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

// ---------------------------------------------------------------------------
// 2. HTTP ACTIONS POLLING (/api/actions/since?last_id=...)
// Fallback resilient untuk Mobile Tunnel / Cloudflare yang membuffer SSE
// ---------------------------------------------------------------------------
pub fn handle_get_actions_since(full_url: &str, stream: &mut TcpStream) {
    let last_id = full_url
        .split('?')
        .nth(1)
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let mut kv = pair.split('=');
                if kv.next()? == "last_id" {
                    kv.next()?.parse::<u64>().ok()
                } else {
                    None
                }
            })
        })
        .unwrap_or(0);

    let (actions, new_last_id) = crate::app_state::get_client_actions_since(last_id);
    let resp = serde_json::json!({
        "actions": actions,
        "last_id": new_last_id
    })
    .to_string();

    crate::server::send_json_response(stream, 200, &resp);
}
