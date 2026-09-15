use rust_embed::RustEmbed;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::assets::*;
use super::state::*;
use crate::native::pose_system::AssistantVisualState;
use crate::engine::{
    execute_command_by_os, get_installed_applications, parse_skills_markdown,
    save_skills_markdown, SkillItem,
};

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct EmbeddedWebDist;

pub fn handle_api_route(
    method: &str,
    url: &str,
    full_url: &str,
    body_str: &str,
    body_bytes: &[u8],
    mut stream: TcpStream,
    shared: Arc<Mutex<ServerSharedState>>,
) {
    match (method, url) {
        ("GET", "/api/assets/info") => {
            let assets_dir = get_assets_dir();
            let vrm_path = find_file_in_candidates("char.vrm");
            let vrm_exists = vrm_path.exists();
            let vrm_size = if vrm_exists {
                fs::metadata(&vrm_path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            let mut images = Vec::new();
            if let Ok(entries) = fs::read_dir(&assets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ["svg", "png", "jpg", "jpeg", "webp"].contains(&ext_lower.as_str()) {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                images.push(name.to_string());
                            }
                        }
                    }
                }
            }

            #[derive(serde::Serialize)]
            struct AssetInfoResponse {
                vrm_exists: bool,
                vrm_size_bytes: u64,
                vrm_size_mb: f32,
                vrm_path: String,
                images: Vec<String>,
            }

            let resp = AssetInfoResponse {
                vrm_exists,
                vrm_size_bytes: vrm_size,
                vrm_size_mb: (vrm_size as f32) / (1024.0 * 1024.0),
                vrm_path: vrm_path.to_string_lossy().to_string(),
                images,
            };

            let json = serde_json::to_string(&resp).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/upload/vrm") => {
            if body_bytes.is_empty() {
                send_json_response(&mut stream, 400, "{\"error\":\"Empty file payload\"}");
                return;
            }

            let assets_dir = get_assets_dir();
            let _ = fs::create_dir_all(&assets_dir);
            let target_path = assets_dir.join("char.vrm");

            match fs::write(&target_path, body_bytes) {
                Ok(_) => {
                    crate::term_log!(
                        "✨ [Asset Manager] Model VRM berhasil diunggah ({:.2} MB) ke {:?}",
                        (body_bytes.len() as f32) / (1024.0 * 1024.0),
                        target_path
                    );
                    if let Ok(mut lock) = shared.lock() {
                        lock.vrm_updated = true;
                    }
                    send_json_response(
                        &mut stream,
                        200,
                        "{\"success\":true,\"message\":\"VRM model uploaded and reloaded successfully\"}",
                    );
                }
                Err(e) => {
                    crate::term_err!("⚠️ [Asset Manager] Gagal menyimpan file VRM: {}", e);
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                }
            }
        }
        ("POST", "/api/upload/avatar") => {
            if body_bytes.is_empty() {
                send_json_response(&mut stream, 400, "{\"error\":\"Empty image payload\"}");
                return;
            }

            let slot = full_url.split('?').nth(1).and_then(|q| {
                for pair in q.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "slot" {
                            return Some(v.to_string());
                        }
                    }
                }
                None
            }).unwrap_or_else(|| "idle".to_string());

            let filename = match slot.to_lowercase().as_str() {
                "listening" => "listening.png",
                "talking" | "speaking" => "talking.png",
                "thinking" => "thinking.png",
                _ => "idle.png",
            };

            let assets_dir = get_assets_dir();
            let _ = fs::create_dir_all(&assets_dir);
            let target_path = assets_dir.join(filename);

            match fs::write(&target_path, body_bytes) {
                Ok(_) => {
                    crate::term_log!("✨ [Asset Manager] Gambar avatar slot '{}' tersimpan ({:?})", slot, target_path);
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
                Err(e) => {
                    crate::term_err!("⚠️ [Asset Manager] Gagal menyimpan avatar: {}", e);
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                }
            }
        }
        ("GET", "/api/config") => {
            let cfg = load_config();
            let json = serde_json::to_string(&cfg).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/config") => {
            if let Ok(cfg) = serde_json::from_str::<crate::config::UserConfig>(body_str) {
                if let Err(e) = save_config(&cfg) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.config_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid config format\"}");
            }
        }
        ("GET", "/api/poses") => {
            let poses = load_poses();
            let json = serde_json::to_string(&poses).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/poses") => {
            if let Ok(poses) = serde_json::from_str::<crate::native::pose_system::PosesConfig>(body_str) {
                if let Err(e) = save_poses(&poses) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.poses_updated = true;
                        lock.preview_poses = None;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid poses format\"}");
            }
        }
        ("POST", "/api/tuner/preview") => {
            #[derive(Deserialize)]
            struct TunerPreviewPayload {
                poses: crate::native::pose_system::PosesConfig,
                emotion: Option<String>,
                state: Option<String>,
            }
            if let Ok(payload) = serde_json::from_str::<TunerPreviewPayload>(body_str) {
                if let Ok(mut lock) = shared.lock() {
                    lock.tuning_mode = false;
                    lock.live_tuning_bones.clear();
                    lock.preview_poses = Some(payload.poses);
                    if let Some(st_str) = payload.state {
                        lock.current_state = match st_str.to_lowercase().as_str() {
                            "listening" => AssistantVisualState::Listening,
                            "speaking" => AssistantVisualState::Speaking,
                            "thinking" => AssistantVisualState::Thinking,
                            _ => AssistantVisualState::Idle,
                        };
                    }
                    if let Some(emo) = payload.emotion {
                        lock.current_emotion = emo;
                    }
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid preview payload\"}");
            }
        }
        ("POST", "/api/tuner/live") => {
            #[derive(Deserialize)]
            struct TunerLivePayload {
                active: bool,
                #[serde(default)]
                bones: std::collections::HashMap<String, [f32; 3]>,
                emotion: Option<String>,
                state: Option<String>,
            }
            if let Ok(payload) = serde_json::from_str::<TunerLivePayload>(body_str) {
                if let Ok(mut lock) = shared.lock() {
                    if payload.active {
                        lock.tuning_mode = true;
                        if let Some(st_str) = payload.state {
                            lock.current_state = match st_str.to_lowercase().as_str() {
                                "listening" => AssistantVisualState::Listening,
                                "speaking" => AssistantVisualState::Speaking,
                                "thinking" => AssistantVisualState::Thinking,
                                _ => AssistantVisualState::Idle,
                            };
                        }
                        lock.live_tuning_bones = payload.bones;
                        if let Some(emo) = payload.emotion {
                            lock.current_emotion = emo;
                        }
                    } else {
                        lock.tuning_mode = false;
                        lock.live_tuning_bones.clear();
                        lock.preview_poses = None;
                        lock.poses_updated = true;

                        let (real_state, real_emotion) = crate::tray::TrayState::get_current_runtime_state();
                        lock.current_state = real_state;
                        lock.current_emotion = real_emotion;
                    }
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid live tuner payload\"}");
            }
        }
        ("GET", "/api/system/apps") => {
            let apps = get_installed_applications();
            let json = serde_json::to_string(&apps).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("GET", "/api/skills") => {
            let skills = parse_skills_markdown();
            let json = serde_json::to_string(&skills).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/skills") => {
            #[derive(Deserialize)]
            struct AddSkillPayload {
                name: String,
                description: String,
                command: String,
            }
            if let Ok(item) = serde_json::from_str::<AddSkillPayload>(body_str) {
                let mut skills = parse_skills_markdown();
                if let Some(pos) = skills.iter().position(|s| s.name.eq_ignore_ascii_case(&item.name)) {
                    skills[pos] = SkillItem {
                        name: item.name,
                        description: item.description,
                        command: item.command,
                    };
                } else {
                    skills.push(SkillItem {
                        name: item.name,
                        description: item.description,
                        command: item.command,
                    });
                }
                if let Err(e) = save_skills_markdown(&skills) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid skill format\"}");
            }
        }
        ("POST", "/api/skills/delete") => {
            #[derive(Deserialize)]
            struct DeleteSkillPayload {
                name: String,
            }
            if let Ok(item) = serde_json::from_str::<DeleteSkillPayload>(body_str) {
                let mut skills = parse_skills_markdown();
                skills.retain(|s| !s.name.eq_ignore_ascii_case(&item.name));
                if let Err(e) = save_skills_markdown(&skills) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid delete payload\"}");
            }
        }
        ("POST", "/api/skills/run") => {
            #[derive(Deserialize)]
            struct RunSkillPayload {
                command: String,
            }
            if let Ok(payload) = serde_json::from_str::<RunSkillPayload>(body_str) {
                if let Err(e) = execute_command_by_os(&payload.command) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid run payload\"}");
            }
        }
        ("POST", "/api/session/heartbeat") => {
            #[derive(Deserialize)]
            struct HeartbeatPayload {
                token: String,
            }
            if let Ok(payload) = serde_json::from_str::<HeartbeatPayload>(body_str) {
                let now = std::time::Instant::now();
                let mut is_active = false;
                if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
                    if let Some(sess) = lock.as_mut() {
                        if sess.token == payload.token {
                            sess.last_seen = now;
                            is_active = true;
                        } else if now.duration_since(sess.last_seen) > Duration::from_secs(30) {
                            *sess = ActiveWebSession {
                                token: payload.token,
                                last_seen: now,
                            };
                            is_active = true;
                        }
                    } else {
                        *lock = Some(ActiveWebSession {
                            token: payload.token,
                            last_seen: now,
                        });
                        is_active = true;
                    }
                }
                send_json_response(&mut stream, 200, &format!("{{\"is_active\":{}}}", is_active));
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid payload\"}");
            }
        }
        ("POST", "/api/session/takeover") => {
            #[derive(Deserialize)]
            struct TakeoverPayload {
                token: String,
            }
            if let Ok(payload) = serde_json::from_str::<TakeoverPayload>(body_str) {
                if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
                    *lock = Some(ActiveWebSession {
                        token: payload.token,
                        last_seen: std::time::Instant::now(),
                    });
                }
                if let Ok(mut lock) = shared.lock() {
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid payload\"}");
            }
        }
        ("GET", "/api/logs") => {
            let logs = crate::config::get_all_logs();
            let json = serde_json::to_string(&logs).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/logs/clear") => {
            crate::config::clear_logs();
            send_json_response(&mut stream, 200, "{\"success\":true}");
        }
        ("POST", "/api/cmd") => {
            #[derive(Deserialize)]
            struct CmdPayload {
                cmd: String,
            }
            if let Ok(payload) = serde_json::from_str::<CmdPayload>(body_str) {
                if payload.cmd == "start_chat" || payload.cmd == "toggle_pause" {
                    if let Ok(mut lock) = shared.lock() {
                        lock.tuning_mode = false;
                        lock.live_tuning_bones.clear();
                        lock.preview_poses = None;
                        lock.poses_updated = true;
                    }
                }
                crate::tray::TrayState::push_command(payload.cmd);
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid cmd payload\"}");
            }
        }
        ("GET", "/api/events") => {
            handle_sse_client(stream, shared);
        }
        ("GET", "/api/state") => {
            if let Ok(lock) = shared.lock() {
                let st_str = match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                };
                let active_token = ACTIVE_WEB_SESSION.lock().ok().and_then(|s| s.as_ref().map(|x| x.token.clone()));
                let payload = AppStatePayload {
                    state: st_str.to_string(),
                    is_running: lock.is_running,
                    is_paused: lock.is_paused,
                    voice_muted: lock.voice_muted,
                    mic_muted: lock.mic_muted,
                    active_session_token: active_token,
                };
                let json = serde_json::to_string(&payload).unwrap_or_default();
                send_json_response(&mut stream, 200, &json);
            }
        }
        ("POST", "/api/state") => {
            if let Ok(payload) = serde_json::from_str::<AppStatePayload>(body_str) {
                let mut cmd_to_send = None;
                if let Ok(mut lock) = shared.lock() {
                    let prev_running = lock.is_running;
                    let prev_paused = lock.is_paused;

                    lock.is_running = payload.is_running;
                    lock.is_paused = payload.is_paused;
                    lock.voice_muted = payload.voice_muted;
                    lock.mic_muted = payload.mic_muted;
                    lock.state_version = lock.state_version.wrapping_add(1);

                    if payload.is_running && !prev_running {
                        lock.tuning_mode = false;
                        lock.live_tuning_bones.clear();
                        lock.preview_poses = None;
                        lock.poses_updated = true;
                        cmd_to_send = Some("start_chat".to_string());
                    } else if !payload.is_running && prev_running {
                        cmd_to_send = Some("stop_chat".to_string());
                    } else if payload.is_paused != prev_paused {
                        cmd_to_send = Some("toggle_pause".to_string());
                    }
                }
                if let Some(cmd) = cmd_to_send {
                    crate::tray::TrayState::push_command(cmd);
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid state payload\"}");
            }
        }
        ("GET", path) => {
            let clean_path = path.trim_start_matches('/');
            let target_path = if clean_path.is_empty() {
                "index.html"
            } else {
                clean_path
            };

            if let Some(file) = EmbeddedWebDist::get(target_path) {
                let mime_type = match target_path.split('.').last().unwrap_or("") {
                    "html" => "text/html; charset=utf-8",
                    "js" => "application/javascript; charset=utf-8",
                    "css" => "text/css; charset=utf-8",
                    "svg" => "image/svg+xml",
                    "png" => "image/png",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                };

                let resp_header = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: {}\r\n\
                     Content-Length: {}\r\n\
                     Access-Control-Allow-Origin: *\r\n\
                     Cache-Control: max-age=3600\r\n\r\n",
                    mime_type,
                    file.data.len()
                );
                let _ = stream.write_all(resp_header.as_bytes());
                let _ = stream.write_all(&file.data);
            } else if let Some(index) = EmbeddedWebDist::get("index.html") {
                let resp_header = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: text/html; charset=utf-8\r\n\
                     Content-Length: {}\r\n\
                     Access-Control-Allow-Origin: *\r\n\r\n",
                    index.data.len()
                );
                let _ = stream.write_all(resp_header.as_bytes());
                let _ = stream.write_all(&index.data);
            } else {
                let body = "404 Not Found";
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: {}\r\n\
                     Access-Control-Allow-Origin: *\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        }
        _ => {
            let body = "405 Method Not Allowed";
            let resp = format!(
                "HTTP/1.1 405 Method Not Allowed\r\n\
                 Content-Length: {}\r\n\
                 Access-Control-Allow-Origin: *\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    }
}

pub fn handle_sse_client(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    let header = "HTTP/1.1 200 OK\r\n\
                  Content-Type: text/event-stream\r\n\
                  Cache-Control: no-cache\r\n\
                  Connection: keep-alive\r\n\
                  Access-Control-Allow-Origin: *\r\n\r\n";
    if stream.write_all(header.as_bytes()).is_err() {
        return;
    }

    let mut last_seen_version: u64 = 0;
    let mut initial_sent = false;

    loop {
        let (should_send, payload_json) = if let Ok(lock) = shared.lock() {
            let ver = lock.state_version;
            if !initial_sent || ver != last_seen_version {
                last_seen_version = ver;
                initial_sent = true;

                let st_str = match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                };
                let active_token = ACTIVE_WEB_SESSION.lock().ok().and_then(|s| s.as_ref().map(|x| x.token.clone()));
                let payload = AppStatePayload {
                    state: st_str.to_string(),
                    is_running: lock.is_running,
                    is_paused: lock.is_paused,
                    voice_muted: lock.voice_muted,
                    mic_muted: lock.mic_muted,
                    active_session_token: active_token,
                };
                (true, serde_json::to_string(&payload).unwrap_or_default())
            } else {
                (false, String::new())
            }
        } else {
            (false, String::new())
        };

        if should_send {
            let event_msg = format!("data: {}\n\n", payload_json);
            if stream.write_all(event_msg.as_bytes()).is_err() {
                break;
            }
            let _ = stream.flush();
        }

        thread::sleep(Duration::from_millis(50));
    }
}

pub fn send_json_response(stream: &mut TcpStream, status: u16, json: &str) {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let resp = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Content-Length\r\n\r\n{}",
        status,
        status_text,
        json.len(),
        json
    );
    let _ = stream.write_all(resp.as_bytes());
}
