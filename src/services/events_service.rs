use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app_state::{ACTIVE_WEB_SESSION, ServerSharedState};
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
}

pub fn handle_sse_state(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    if !write_sse_header(&mut stream) {
        return;
    }

    let mut last_state = String::new();
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

            let changed = !initial_sent || st_str != last_state;
            if changed {
                initial_sent = true;
                last_state = st_str.to_string();

                let payload = CoreStatePayload {
                    state: st_str.to_string(),
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
// 2. MENU SSE STREAM (/api/events/menu)
// Terpisah murni untuk Menu Tab:
// - event: 'controls' -> { is_running, is_paused, voice_muted, mic_muted } (Event-driven)
// - event: 'mic'      -> { level } (Realtime 20 FPS / 50ms)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct MenuControlsPayload {
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
}

pub fn handle_sse_menu(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    if !write_sse_header(&mut stream) {
        return;
    }

    let mut last_mic: Option<u8> = None;
    let mut ping_ticks: u32 = 0;

    loop {
        // Cek apakah tab yang aktif di server saat ini adalah "menu"
        let is_menu_active = ACTIVE_WEB_SESSION
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|x| x.active_tab == "menu"))
            .unwrap_or(true);

        if !is_menu_active {
            // Jika user sedang di tab lain, matikan stream SSE ini dari server
            break;
        }

        let mut had_activity = false;

        let (mic_changed, mic_val) = if let Ok(lock) = shared.lock() {
            let current_mic = if lock.mic_muted { 0 } else { lock.mic_level };
            let m_changed = last_mic != Some(current_mic);
            if m_changed {
                last_mic = Some(current_mic);
            }
            (m_changed, current_mic)
        } else {
            (false, 0)
        };

        // Kirim event 'mic' HANYA jika nilai mic level berubah atau ada suara
        if mic_changed && (mic_val > 0 || last_mic == Some(0)) {
            let mic_msg = format!("event: mic\ndata: {{\"level\":{}}}\n\n", mic_val);
            if stream.write_all(mic_msg.as_bytes()).is_err() || stream.flush().is_err() {
                break;
            }
            had_activity = true;
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

        // ~16 FPS (60 ms per tick) - backend efisien, frontend lerp ke 60 FPS
        thread::sleep(Duration::from_millis(60));
    }
}
