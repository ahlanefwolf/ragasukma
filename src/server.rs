use rust_embed::RustEmbed;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app_state::*;
use crate::services::*;

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct EmbeddedWebDist;

// In-Memory Bidirectional Channel Stream Bridge
// Zero OS socket overhead, purely in-memory buffers


pub struct VirtualHttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct VirtualHttpStream {
    pub response: VirtualHttpResponse,
    buffer: Vec<u8>,
}

impl Default for VirtualHttpStream {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualHttpStream {
    pub fn new() -> Self {
        Self {
            response: VirtualHttpResponse {
                status: 200,
                headers: Vec::new(),
                body: Vec::new(),
            },
            buffer: Vec::new(),
        }
    }

    pub fn parse_buffer_if_needed(&mut self) {
        if !self.buffer.is_empty() {
            if let Some(pos) = self.buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                let header_part = String::from_utf8_lossy(&self.buffer[..pos]);
                let body_part = self.buffer[pos + 4..].to_vec();

                let mut lines = header_part.lines();
                if let Some(status_line) = lines.next() {
                    let parts: Vec<&str> = status_line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(code) = parts[1].parse::<u16>() {
                            self.response.status = code;
                        }
                    }
                }

                for line in lines {
                    if let Some((k, v)) = line.split_once(':') {
                        self.response.headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }

                self.response.body = body_part;
                self.buffer.clear();
            }
        }
    }
}

impl Write for VirtualHttpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.parse_buffer_if_needed();
        Ok(())
    }
}

pub fn send_json_response<W: Write + ?Sized>(stream: &mut W, status: u16, json: &str) {
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
         Connection: close\r\n\
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

pub fn send_disk_file_stream_with_range<W: Write + ?Sized>(
    stream: &mut W,
    mime_type: &str,
    file_path: &std::path::Path,
    header_str: &str,
) -> std::io::Result<()> {
    use std::io::{Seek, SeekFrom};

    let mut file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            send_json_response(stream, 404, "{\"error\":\"File not found\"}");
            return Err(e);
        }
    };

    let metadata = file.metadata()?;
    let total_len = metadata.len();

    if total_len == 0 {
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: 0\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccept-Ranges: bytes\r\n\r\n",
            mime_type
        );
        let _ = stream.write_all(resp.as_bytes());
        return Ok(());
    }

    // Check Range header: bytes=start-end (Required for HTML5 <audio> m4a/mp4 container seeking)
    let range_header = header_str.lines().find_map(|line| {
        let lower = line.to_lowercase();
        if lower.starts_with("range:") {
            Some(line.split(':').nth(1)?.trim())
        } else {
            None
        }
    });

    let (start, end, is_range) = if let Some(range_val) = range_header {
        if let Some(bytes_range) = range_val.strip_prefix("bytes=") {
            let parts: Vec<&str> = bytes_range.split('-').collect();
            let start = parts
                .first()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let end = parts
                .get(1)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(total_len - 1)
                .min(total_len - 1);

            if start <= end && start < total_len {
                (start, end, true)
            } else {
                (0, total_len - 1, false)
            }
        } else {
            (0, total_len - 1, false)
        }
    } else {
        (0, total_len - 1, false)
    };

    let chunk_len = (end - start) + 1;

    let resp_header = if is_range {
        format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccept-Ranges: bytes\r\nCache-Control: no-cache\r\n\r\n",
            mime_type, start, end, total_len, chunk_len
        )
    } else {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccept-Ranges: bytes\r\nCache-Control: no-cache\r\n\r\n",
            mime_type, total_len
        )
    };

    stream.write_all(resp_header.as_bytes())?;

    // Seek to start position
    file.seek(SeekFrom::Start(start))?;

    // Streams are in 64KB chunks
    let mut remaining = chunk_len;
    let mut buffer = [0u8; 65536];

    while remaining > 0 {
        let to_read = (remaining as usize).min(buffer.len());
        let n = file.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        if stream.write_all(&buffer[..n]).is_err() {
            break;
        }
        remaining -= n as u64;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_api_route<W: Write>(
    method: &str,
    url: &str,
    full_url: &str,
    header_str: &str,
    body_str: &str,
    body_bytes: &[u8],
    mut stream: W,
    shared: Arc<Mutex<ServerSharedState>>,
) {
    match (method, url) {
        // Ping & Handshake
        ("GET", "/api/ping") => {
            send_json_response(
                &mut stream,
                200,
                "{\"app\":\"ragasukma\",\"status\":\"running\"}",
            );
        }

        // OpenAI-Compatible /v1 API Endpoints
        ("OPTIONS", path) if path.starts_with("/v1") || path.starts_with("/api/v1") => {
            handle_v1_options(&mut stream);
        }
        ("GET", "/v1") | ("GET", "/v1/") | ("GET", "/api/v1") | ("GET", "/api/v1/") => {
            send_json_response(
                &mut stream,
                200,
                "{\"status\":\"ok\",\"service\":\"ragasukma-openai-v1\",\"version\":\"0.2.0\"}",
            );
        }
        ("GET", "/v1/models") | ("GET", "/api/v1/models") => {
            handle_v1_models(&mut stream);
        }
        ("POST", "/v1/chat/completions") | ("POST", "/api/v1/chat/completions") => {
            handle_v1_chat_completions(body_str, &mut stream);
        }
        ("POST", "/v1/completions") | ("POST", "/api/v1/completions") => {
            handle_v1_chat_completions(body_str, &mut stream);
        }

        // Assets Service
        ("GET", "/api/assets/info") => handle_assets_info(&mut stream),
        ("POST", "/api/upload/vrm") => handle_upload_vrm(body_bytes, &mut stream, &shared),
        ("POST", "/api/upload/avatar") => {
            handle_upload_avatar(full_url, body_bytes, &mut stream, &shared)
        }
        ("POST", path) if path.starts_with("/api/upload/refvoice") => {
            handle_upload_refvoice(full_url, body_bytes, &mut stream)
        }
        ("POST", "/api/delete/refvoice") => handle_delete_refvoice(&mut stream),

        // Browser Sessions & DOM Snapshot Service
        ("POST", "/api/browser/snapshot") => {
            let (resp_body, _) = handle_post_browser_snapshot(body_str);
            send_json_response(&mut stream, 200, &resp_body);
        }
        ("POST", "/api/browser/copied_text") => {
            let (resp_body, _) = handle_post_browser_copied_text(body_str);
            send_json_response(&mut stream, 200, &resp_body);
        }
        ("POST", "/api/browser/tabs") => {
            let (resp_body, _) = handle_post_browser_tabs(body_str);
            send_json_response(&mut stream, 200, &resp_body);
        }
        ("GET", "/api/browser/tabs") => {
            let (resp_body, _) = handle_get_browser_tabs();
            send_json_response(&mut stream, 200, &resp_body);
        }
        ("GET", "/api/browser/command/next") => {
            let (resp_body, _) = handle_get_next_browser_command();
            send_json_response(&mut stream, 200, &resp_body);
        }

        // Config Service
        ("GET", "/api/config") => handle_get_config(&mut stream),
        ("POST", "/api/config") => handle_post_config(body_str, &mut stream, &shared),

        // Poses & Tuner Service
        ("GET", "/api/poses") => handle_get_poses(&mut stream),
        ("POST", "/api/poses") => handle_post_poses(body_str, &mut stream, &shared),
        ("POST", "/api/tuner/preview") => handle_tuner_preview(body_str, &mut stream, &shared),
        ("POST", "/api/tuner/live") => handle_tuner_live(body_str, &mut stream, &shared),

        // Skills & Applications Service
        ("GET", "/api/system/apps") => handle_get_apps(&mut stream),
        ("GET", "/api/system/extension-path") => handle_get_extension_path(&mut stream),
        ("GET", "/api/system/model-status") => {
            let status = crate::services::model_manager_service::get_model_status();
            let json = serde_json::to_string(&status).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/system/model-download") => {
            crate::services::model_manager_service::ensure_model_ready_background();
            let status = crate::services::model_manager_service::get_model_status();
            let json = serde_json::to_string(&status).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("GET", "/api/skills") => handle_get_skills(&mut stream),
        ("POST", "/api/skills") => handle_post_skills(body_str, &mut stream),
        ("POST", "/api/skills/delete") => handle_delete_skill(body_str, &mut stream),
        ("POST", "/api/skills/run") => handle_run_skill(body_str, &mut stream),

        // Automation Recipes Service
        ("GET", "/api/recipes") => handle_get_recipes(&mut stream),
        ("POST", "/api/recipes/save") => handle_save_recipe(body_str, &mut stream),
        ("POST", "/api/recipes/delete") => handle_delete_recipe(body_str, &mut stream),
        ("POST", "/api/recipes/run") => handle_run_recipe(body_str, &mut stream),

        // Conversation History Service
        ("GET", "/api/history/dates") => handle_get_history_dates(&mut stream),
        ("GET", path) if path.starts_with("/api/history/view") => {
            handle_get_history_by_date(full_url, &mut stream);
        }
        ("POST", "/api/history/delete") => handle_delete_history(body_str, &mut stream),

        // Cloudflared Remote Tunnel & Mobile Voice
        ("GET", "/api/remote/tunnel/status") => handle_tunnel_status(&mut stream),
        ("GET", "/api/remote/tunnel/check") => handle_tunnel_check(&mut stream),
        ("POST", "/api/remote/tunnel/install") => handle_tunnel_install(&mut stream),
        ("POST", "/api/remote/tunnel/start") => handle_tunnel_start(&mut stream),
        ("POST", "/api/remote/tunnel/stop") => handle_tunnel_stop(&mut stream),
        ("POST", "/api/stt/transcript") => handle_stt_transcript(body_str, &mut stream),
        ("POST", "/api/tts/state") => handle_tts_state(body_str, &mut stream, &shared),
        ("POST", "/api/tts/synthesize") => handle_tts_synthesize(body_str, &mut stream),
        ("POST", "/api/tts/ack") => crate::services::tts_service::handle_tts_ack(body_str, &mut stream),
        ("GET", path) if path.starts_with("/api/tts/audio") => {
            handle_get_tts_audio(full_url, header_str, &mut stream);
        }
        ("POST", path) if path.starts_with("/api/remote/text") => {
            handle_remote_text(full_url, body_str, &mut stream);
        }

        // Session & Single-Tab Control
        // Session Claim Service (1x when tab is opened)
        ("POST", "/api/session/claim") => handle_session_claim(body_str, &mut stream, &shared),
        ("POST", "/api/session/tab") => handle_session_tab(body_str, &mut stream),

        // Logs Service
        ("GET", path) if path.starts_with("/api/logs") => handle_get_logs(full_url, &mut stream),
        ("POST", "/api/logs/clear") => handle_clear_logs(&mut stream),

        // LLM Providers - Antigravity
        ("GET", "/api/llm/antigravity/models") => handle_get_antigravity_models(&mut stream),
        ("GET", "/api/llm/antigravity/accounts") => handle_get_antigravity_accounts(&mut stream),
        ("POST", "/api/llm/antigravity/accounts") => {
            handle_post_antigravity_account(body_str, &mut stream)
        }
        ("POST", "/api/llm/antigravity/accounts/delete") => {
            handle_delete_antigravity_account(body_str, &mut stream)
        }
        ("POST", "/api/llm/antigravity/accounts/refresh") => {
            handle_refresh_antigravity_account(body_str, &mut stream)
        }
        ("POST", "/api/llm/antigravity/accounts/toggle") => {
            handle_toggle_antigravity_account(body_str, &mut stream)
        }
        ("POST", "/api/llm/antigravity/accounts/quota") => {
            handle_get_antigravity_account_quota(body_str, full_url, &mut stream)
        }
        ("GET", path) if path.starts_with("/api/llm/antigravity/accounts/quota") => {
            handle_get_antigravity_account_quota(body_str, full_url, &mut stream);
        }
        ("GET", path) if path.starts_with("/api/llm/antigravity/oauth/url") => {
            handle_get_antigravity_auth_url(full_url, &mut stream);
        }
        ("GET", path) if path.starts_with("/api/llm/antigravity/oauth/callback") => {
            handle_antigravity_oauth_callback(full_url, &mut stream);
        }
        ("POST", "/api/llm/antigravity/oauth/exchange") => {
            handle_post_antigravity_oauth_exchange(body_str, &mut stream);
        }

        // Menu Service
        ("POST", "/api/cmd") => handle_menu_cmd(body_str, &mut stream, &shared),
        ("GET", "/api/state") => handle_get_state(&mut stream, &shared),
        ("POST", "/api/state") => handle_post_state(body_str, &mut stream, &shared),

        // Events Service (Actions Polling)
        ("GET", path) if path.starts_with("/api/actions/since") => {
            handle_get_actions_since(full_url, &mut stream);
        }

        // Music Storage & Downloader Service (Local Static Audio)
        ("GET", "/api/music/check") => handle_music_check(&mut stream),
        ("POST", "/api/music/install-ytdlp") => handle_music_install_ytdlp(&mut stream),
        ("GET", "/api/music/list") => handle_music_list(&mut stream),
        ("GET", path) if path.starts_with("/music/") || path.starts_with("/api/music/stream") => {
            handle_music_stream(full_url, header_str, &mut stream);
        }
        ("POST", "/api/music/control") => handle_music_control(body_str, &mut stream),
        ("POST", "/api/music/download") => handle_music_download(body_str, &mut stream),
        ("POST", "/api/music/delete") => handle_music_delete(body_str, &mut stream),

        // Assets & Avatar Image Static Serving (Dedicated API prefix to avoid collision with web/dist/assets)
        ("GET", path) if path.starts_with("/api/assets/file/") => {
            let filename = path.strip_prefix("/api/assets/file/").unwrap_or("");
            let clean_filename = filename
                .trim_start_matches('/')
                .split('?')
                .next()
                .unwrap_or("");
            let file_path = crate::assets::find_file_in_candidates(clean_filename);

            if file_path.exists() && file_path.is_file() {
                if let Ok(data) = std::fs::read(&file_path) {
                    let mime_type = match clean_filename
                        .split('.')
                        .next_back()
                        .unwrap_or("")
                        .to_lowercase()
                        .as_str()
                    {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "webp" => "image/webp",
                        "vrm" | "glb" => "model/gltf-binary",
                        "json" => "application/json",
                        "wav" => "audio/wav",
                        "mp3" => "audio/mpeg",
                        "ogg" => "audio/ogg",
                        "flac" => "audio/flac",
                        "m4a" => "audio/mp4",
                        "aac" => "audio/aac",
                        _ => "application/octet-stream",
                    };
                    let resp_header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=3600\r\n\r\n",
                        mime_type,
                        data.len()
                    );
                    let _ = stream.write_all(resp_header.as_bytes());
                    let _ = stream.write_all(&data);
                } else {
                    let body = "500 Internal Server Error";
                    let resp = format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
            } else {
                let body = "404 Asset Not Found";
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        }

        // Static Web Dashboard Files (Prioritas: file fisik di web/dist/ jika ada, fallback: EmbeddedWebDist)
        ("GET", path) => {
            let clean_path = path.trim_start_matches('/');
            let target_path = if clean_path.is_empty() {
                "index.html"
            } else {
                clean_path
            };

            let mime_for = |fname: &str| match fname.split('.').next_back().unwrap_or("") {
                "html" => "text/html; charset=utf-8",
                "js" => "application/javascript; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "svg" => "image/svg+xml",
                "png" => "image/png",
                "json" => "application/json",
                _ => "application/octet-stream",
            };

            // 1. Cek file fisik langsung di disk (Development / Live dist update)
            let disk_file = std::path::Path::new("web/dist").join(target_path);
            let disk_index = std::path::Path::new("web/dist/index.html");

            if disk_file.is_file() {
                if let Ok(bytes) = std::fs::read(&disk_file) {
                    let mime = mime_for(target_path);
                    let resp_header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nPragma: no-cache\r\nExpires: 0\r\n\r\n",
                        mime,
                        bytes.len()
                    );
                    let _ = stream.write_all(resp_header.as_bytes());
                    let _ = stream.write_all(&bytes);
                    return;
                }
            } else if disk_index.is_file() {
                // SPA fallback dari disk
                if let Ok(bytes) = std::fs::read(disk_index) {
                    let resp_header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nPragma: no-cache\r\nExpires: 0\r\n\r\n",
                        bytes.len()
                    );
                    let _ = stream.write_all(resp_header.as_bytes());
                    let _ = stream.write_all(&bytes);
                    return;
                }
            }

            // 2. Fallback jika tidak ada folder web/dist di disk (Binary standalone embedded)
            if let Some(file) = EmbeddedWebDist::get(target_path) {
                let mime_type = mime_for(target_path);
                let resp_header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nPragma: no-cache\r\nExpires: 0\r\n\r\n",
                    mime_type,
                    file.data.len()
                );
                let _ = stream.write_all(resp_header.as_bytes());
                let _ = stream.write_all(&file.data);
            } else if let Some(index) = EmbeddedWebDist::get("index.html") {
                let resp_header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nPragma: no-cache\r\nExpires: 0\r\n\r\n",
                    index.data.len()
                );
                let _ = stream.write_all(resp_header.as_bytes());
                let _ = stream.write_all(&index.data);
            } else {
                let body = "404 Not Found";
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        }
        _ => {
            let body = "405 Method Not Allowed";
            let resp = format!(
                "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    }
}



async fn axum_tts_audio_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let req_id = params.get("id").map(|s| s.as_str());

    if let Some(wav_bytes) = crate::audio::supertonic::get_last_audio_wav(req_id) {
        return serve_audio_bytes_with_range(wav_bytes, "audio/wav", &headers);
    }

    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({ "error": "TTS Audio not found or not synthesized yet" })),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
struct EvictAudioPayload {
    id: Option<String>,
}

async fn axum_tts_evict_handler(
    axum::Json(payload): axum::Json<EvictAudioPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(id) = payload.id {
        if !id.is_empty() {
            crate::audio::supertonic::evict_audio(&id);
        }
    }
    axum::Json(serde_json::json!({ "success": true })).into_response()
}

async fn axum_music_stream_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let filename = params
        .get("filename")
        .or_else(|| params.get("file"))
        .cloned()
        .unwrap_or_default();

    if filename.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Missing filename param" })),
        )
            .into_response();
    }

    serve_music_file(&filename, &headers)
}

async fn axum_music_stream_path_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let clean_filename = crate::services::music_service::decode_url_component(&path);
    serve_music_file(&clean_filename, &headers)
}

fn serve_music_file(filename: &str, headers: &axum::http::HeaderMap) -> axum::response::Response {
    use axum::response::IntoResponse;

    let music_dir = crate::config::get_music_dir();
    let file_path = music_dir.join(filename);

    if !file_path.exists() || !file_path.is_file() {
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "File not found" })),
        )
            .into_response();
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime_type = match ext.as_str() {
        "mp3" => "audio/mpeg",
        "webm" => "audio/webm",
        "opus" => "audio/opus",
        "ogg" => "audio/ogg",
        "m4a" | "mp4" | "aac" => "audio/mp4",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    };

    match std::fs::read(&file_path) {
        Ok(bytes) => serve_audio_bytes_with_range(bytes, mime_type, headers),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Failed to read audio file" })),
        )
            .into_response(),
    }
}

fn serve_audio_bytes_with_range(
    bytes: Vec<u8>,
    mime_type: &str,
    headers: &axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let total_len = bytes.len() as u64;
    let range_header = headers
        .get("range")
        .and_then(|v| v.to_str().ok());

    let (start, end, is_range) = if let Some(range_val) = range_header {
        if let Some(bytes_range) = range_val.strip_prefix("bytes=") {
            let parts: Vec<&str> = bytes_range.split('-').collect();
            let start = parts
                .first()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let end = parts
                .get(1)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(total_len.saturating_sub(1))
                .min(total_len.saturating_sub(1));

            if start <= end && start < total_len {
                (start, end, true)
            } else {
                (0, total_len.saturating_sub(1), false)
            }
        } else {
            (0, total_len.saturating_sub(1), false)
        }
    } else {
        (0, total_len.saturating_sub(1), false)
    };

    let chunk_len = (end - start) + 1;
    let start_idx = (start as usize).min(bytes.len());
    let end_idx = ((end + 1) as usize).min(bytes.len());
    let slice = bytes[start_idx..end_idx].to_vec();

    if is_range {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::PARTIAL_CONTENT)
            .header("Content-Type", mime_type)
            .header("Content-Range", format!("bytes {}-{}/{}", start, end, total_len))
            .header("Content-Length", chunk_len.to_string())
            .header("Accept-Ranges", "bytes")
            .header("Cache-Control", "no-cache, no-store")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::from(slice))
            .unwrap_or_else(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response())
    } else {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("Content-Type", mime_type)
            .header("Content-Length", total_len.to_string())
            .header("Accept-Ranges", "bytes")
            .header("Cache-Control", "no-cache, no-store")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::from(slice))
            .unwrap_or_else(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response())
    }
}

async fn handle_axum_fallback(
    req: axum::extract::Request,
    shared: Arc<Mutex<ServerSharedState>>,
) -> axum::response::Response {
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    let (parts, body) = req.into_parts();
    let method = parts.method.as_str().to_string();
    let full_url = parts.uri.to_string();
    let url = parts.uri.path().to_string();

    let mut header_str = format!("{} {} HTTP/1.1\r\n", method, full_url);
    for (k, v) in parts.headers.iter() {
        if let Ok(v_str) = v.to_str() {
            header_str.push_str(&format!("{}: {}\r\n", k.as_str(), v_str));
        }
    }
    header_str.push_str("\r\n");

    let body_bytes = match to_bytes(body, 100 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let response_result = tokio::task::spawn_blocking(move || {
        let mut virtual_stream = VirtualHttpStream::new();
        handle_api_route(
            &method,
            &url,
            &full_url,
            &header_str,
            &body_str,
            &body_bytes,
            &mut virtual_stream,
            shared,
        );
        virtual_stream.parse_buffer_if_needed();
        virtual_stream.response
    })
    .await;

    let res = match response_result {
        Ok(r) => r,
        Err(_) => {
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let mut builder = axum::response::Response::builder().status(res.status);
    for (k, v) in res.headers {
        let k_trim = k.trim();
        let v_trim = v.trim();
        if !k_trim.eq_ignore_ascii_case("connection")
            && !k_trim.eq_ignore_ascii_case("content-length")
        {
            builder = builder.header(k_trim, v_trim);
        }
    }

    builder
        .body(axum::body::Body::from(res.body))
        .unwrap_or_else(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response())
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    thread::spawn(move || {
        let addr_str = "127.0.0.1:42128";

        let rt = match tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[Web Server] Failed to initialize Tokio runtime: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let tokio_listener = match tokio::net::TcpListener::bind(addr_str).await {
                Ok(l) => {
                    println!(" [Web Dashboard & API Server] Running at http://{}", addr_str);
                    l
                }
                Err(e) => {
                    eprintln!("[Web Server] Failed to bind Tokio TcpListener to {}: {}", addr_str, e);
                    return;
                }
            };

            let cors = tower_http::cors::CorsLayer::permissive();

            let app = axum::Router::new()
                .route("/api/ws", axum::routing::get(crate::services::events_service::axum_ws_handler))
                .route("/api/tts/audio", axum::routing::get(axum_tts_audio_handler))
                .route("/api/tts/evict", axum::routing::post(axum_tts_evict_handler))
                .route("/api/tts/ack", axum::routing::post(axum_tts_evict_handler))
                .route("/api/music/stream", axum::routing::get(axum_music_stream_handler))
                .route("/music/*path", axum::routing::get(axum_music_stream_path_handler))
                .layer(cors)
                .fallback({
                    let shared_for_fallback = Arc::clone(&shared);
                    move |req: axum::extract::Request| {
                        let shared = Arc::clone(&shared_for_fallback);
                        async move {
                            handle_axum_fallback(req, shared).await
                        }
                    }
                })
                .with_state(shared);

            if let Err(e) = axum::serve(tokio_listener, app).await {
                eprintln!("[Web Server] Axum server error: {}", e);
            }
        });
    });
}
