use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::pose_system::{AssistantVisualState, PosesConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub assistant_name: String,
    pub user_callsign: String,
    pub personality_role: String,
    pub stt_engine: String,
    pub groq_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    pub voice_mode: String,
    pub mic_sensitivity: String,
    pub vad_silence_limit: String,
    pub noise_reduction_level: u8,
    pub tts_engine: String,
    #[serde(default = "default_tts_lang")]
    pub tts_lang: String,
    pub tts_speed: f32,
    pub tts_eleven_labs_key: String,
    pub elevenlabs_voice_id: String,
    pub avatar_type: String,
    #[serde(default = "default_always_on_top")]
    pub always_on_top: bool,
    #[serde(default)]
    pub enable_terminal_log: bool,
}

fn default_always_on_top() -> bool {
    true
}

fn default_tts_lang() -> String {
    "id".to_string()
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            assistant_name: "Laras".to_string(),
            user_callsign: "Mas".to_string(),
            personality_role: "partner".to_string(),
            stt_engine: "google".to_string(),
            groq_api_key: "".to_string(),
            llm_base_url: "http://localhost:20128/v1".to_string(),
            llm_model: "ag/gemini-3.8-flash-low".to_string(),
            llm_api_key: "".to_string(),
            voice_mode: "auto".to_string(),
            mic_sensitivity: "2".to_string(),
            vad_silence_limit: "1".to_string(),
            noise_reduction_level: 0,
            tts_engine: "google".to_string(),
            tts_lang: "id".to_string(),
            tts_speed: 1.15,
            tts_eleven_labs_key: "".to_string(),
            elevenlabs_voice_id: "".to_string(),
            avatar_type: "vrm".to_string(),
            always_on_top: true,
            enable_terminal_log: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStatePayload {
    pub state: String,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
}

pub struct ServerSharedState {
    pub current_state: AssistantVisualState,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub poses_updated: bool,
    pub config_updated: bool,
    pub tuning_mode: bool,
    pub live_tuning_bones: HashMap<String, [f32; 3]>,
}

pub fn find_file_in_candidates(filename: &str) -> PathBuf {
    let candidates = [
        PathBuf::from(filename),
        PathBuf::from(format!("../{}", filename)),
        PathBuf::from(format!("C:/Project/ragasukma/{}", filename)),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    // Fallback AppData
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("ragasukma").join(filename);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(filename)
}

pub fn load_config() -> UserConfig {
    let path = find_file_in_candidates("config-ai.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<UserConfig>(&content) {
            return cfg;
        }
    }
    UserConfig::default()
}

pub fn save_config(cfg: &UserConfig) -> Result<(), String> {
    let path = find_file_in_candidates("config-ai.json");
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_poses() -> PosesConfig {
    let path = find_file_in_candidates("poses.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(p) = serde_json::from_str::<PosesConfig>(&content) {
            return p;
        }
    }
    PosesConfig::default()
}

pub fn save_poses(poses: &PosesConfig) -> Result<(), String> {
    let path = find_file_in_candidates("poses.json");
    let json = serde_json::to_string_pretty(poses).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn parse_skills_markdown() -> Vec<SkillItem> {
    let path = find_file_in_candidates("skill-command.md");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut skills = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains(":---") || trimmed.contains("Nama Skill") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            let name = parts[1].to_string();
            let description = parts[2].to_string();
            let command = parts[3].replace('`', "");

            if !name.is_empty() && !command.is_empty() {
                skills.push(SkillItem {
                    name,
                    description,
                    command,
                });
            }
        }
    }
    skills
}

pub fn save_skills_markdown(skills: &[SkillItem]) -> Result<(), String> {
    let mut out = String::from(
        "# ⚡ Skill Command Registry\n\n\
Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu, atau otomatisasi AI.\n\
File ini dapat diedit langsung atau dikelola melalui UI Skill Commands.\n\n\
| Nama Skill | Deskripsi | Command / Script |\n\
| :--- | :--- | :--- |\n",
    );

    for s in skills {
        out.push_str(&format!(
            "| {} | {} | `{}` |\n",
            s.name.trim(),
            s.description.trim(),
            s.command.trim()
        ));
    }

    let path = find_file_in_candidates("skill-command.md");
    fs::write(path, out).map_err(|e| e.to_string())
}

pub fn execute_command_by_os(cmd: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "cmd", "/C", cmd])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sh")
            .args(["-c", cmd])
            .spawn()?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new("sh")
            .args(["-c", cmd])
            .spawn()?;
    }
    Ok(())
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    thread::spawn(move || {
        let addr = "127.0.0.1:4242";
        let listener = match TcpListener::bind(addr) {
            Ok(l) => {
                println!("🌐 [Web Dashboard & API Server] Berjalan di http://{}", addr);
                l
            }
            Err(e) => {
                eprintln!("⚠️ [Web Server] Gagal bind ke {}: {}", addr, e);
                return;
            }
        };

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let shared_clone = Arc::clone(&shared);
                thread::spawn(move || {
                    handle_client(stream, shared_clone);
                });
            }
        }
    });
}

fn handle_client(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    let mut buffer = [0u8; 65536];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = req_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let url = parts[1];

    // CORS preflight OPTIONS
    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type\r\n\
                    Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    // Body parsing
    let body = if let Some(pos) = req_str.find("\r\n\r\n") {
        &req_str[pos + 4..]
    } else {
        ""
    };

    match (method, url) {
        ("GET", "/api/config") => {
            let cfg = load_config();
            let json = serde_json::to_string(&cfg).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/config") => {
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(body) {
                if let Err(e) = save_config(&cfg) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.config_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid JSON config\"}");
            }
        }
        ("POST", "/api/tuner/live") => {
            #[derive(Deserialize)]
            struct TunerLivePayload {
                active: bool,
                #[serde(default)]
                bones: HashMap<String, [f32; 3]>,
            }
            if let Ok(payload) = serde_json::from_str::<TunerLivePayload>(body) {
                if let Ok(mut lock) = shared.lock() {
                    lock.tuning_mode = payload.active;
                    if payload.active {
                        lock.live_tuning_bones = payload.bones;
                    }
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid tuner payload\"}");
            }
        }
        ("GET", "/api/poses") => {
            let poses = load_poses();
            let json = serde_json::to_string(&poses).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/poses") => {
            if let Ok(poses) = serde_json::from_str::<PosesConfig>(body) {
                if let Err(e) = save_poses(&poses) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.poses_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid JSON poses\"}");
            }
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
            if let Ok(item) = serde_json::from_str::<AddSkillPayload>(body) {
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
            if let Ok(item) = serde_json::from_str::<DeleteSkillPayload>(body) {
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
            if let Ok(item) = serde_json::from_str::<RunSkillPayload>(body) {
                if let Err(e) = execute_command_by_os(&item.command) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid run payload\"}");
            }
        }
        ("GET", "/api/state") => {
            if let Ok(lock) = shared.lock() {
                let st_str = match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                };
                let payload = AppStatePayload {
                    state: st_str.to_string(),
                    is_paused: lock.is_paused,
                    voice_muted: lock.voice_muted,
                    mic_muted: lock.mic_muted,
                };
                let json = serde_json::to_string(&payload).unwrap_or_default();
                send_json_response(&mut stream, 200, &json);
            }
        }
        ("POST", "/api/state") => {
            if let Ok(payload) = serde_json::from_str::<AppStatePayload>(body) {
                if let Ok(mut lock) = shared.lock() {
                    match payload.state.as_str() {
                        "listening" => lock.current_state = AssistantVisualState::Listening,
                        "speaking" => lock.current_state = AssistantVisualState::Speaking,
                        "thinking" => lock.current_state = AssistantVisualState::Thinking,
                        _ => lock.current_state = AssistantVisualState::Idle,
                    }
                    lock.is_paused = payload.is_paused;
                    lock.voice_muted = payload.voice_muted;
                    lock.mic_muted = payload.mic_muted;
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid state payload\"}");
            }
        }
        ("GET", path) => {
            serve_static_file(&mut stream, path);
        }
        _ => {
            send_json_response(&mut stream, 404, "{\"error\":\"Not Found\"}");
        }
    }
}

fn send_json_response(stream: &mut TcpStream, status: u16, json: &str) {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: application/json; charset=utf-8\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
        Access-Control-Allow-Headers: Content-Type\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\r\n\
        {}",
        status,
        status_text,
        json.as_bytes().len(),
        json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_static_file(stream: &mut TcpStream, req_path: &str) {
    let clean_path = if req_path == "/" { "/index.html" } else { req_path };
    let clean_path = clean_path.trim_start_matches('/');

    let dist_dirs = [
        PathBuf::from("web/dist"),
        PathBuf::from("ui-native/web/dist"),
        PathBuf::from("C:/Project/ragasukma/ui-native/web/dist"),
    ];

    for base in &dist_dirs {
        let file_path = base.join(clean_path);
        if file_path.exists() && file_path.is_file() {
            if let Ok(bytes) = fs::read(&file_path) {
                let content_type = if clean_path.ends_with(".html") {
                    "text/html; charset=utf-8"
                } else if clean_path.ends_with(".js") {
                    "application/javascript; charset=utf-8"
                } else if clean_path.ends_with(".css") {
                    "text/css; charset=utf-8"
                } else if clean_path.ends_with(".svg") {
                    "image/svg+xml"
                } else if clean_path.ends_with(".png") {
                    "image/png"
                } else if clean_path.ends_with(".ico") {
                    "image/x-icon"
                } else {
                    "application/octet-stream"
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                    Content-Type: {}\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\r\n",
                    content_type,
                    bytes.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&bytes);
                return;
            }
        }
    }

    send_json_response(stream, 404, "{\"error\":\"Static file not found\"}");
}
