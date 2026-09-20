use rust_embed::RustEmbed;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app_state::*;
use crate::services::*;

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct EmbeddedWebDist;

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

pub fn send_disk_file_stream_with_range(
    stream: &mut TcpStream,
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

    // Cek header Range: bytes=start-end (Wajib untuk HTML5 <audio> m4a/mp4 container seeking)
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

    // Seek ke posisi start
    file.seek(SeekFrom::Start(start))?;

    // Stream dalam chunk 64KB
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
pub fn handle_api_route(
    method: &str,
    url: &str,
    full_url: &str,
    header_str: &str,
    body_str: &str,
    body_bytes: &[u8],
    mut stream: TcpStream,
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
        ("POST", "/api/upload/avatar") => handle_upload_avatar(full_url, body_bytes, &mut stream),

        // Browser Sessions & DOM Snapshot Service
        ("POST", "/api/browser/snapshot") => {
            let (resp_body, _) = handle_post_browser_snapshot(body_str);
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
        ("GET", "/api/skills") => handle_get_skills(&mut stream),
        ("POST", "/api/skills") => handle_post_skills(body_str, &mut stream),
        ("POST", "/api/skills/delete") => handle_delete_skill(body_str, &mut stream),
        ("POST", "/api/skills/run") => handle_run_skill(body_str, &mut stream),

        // Session & Single-Tab Control
        // Session Claim Service (1x saat tab dibuka)
        ("POST", "/api/session/claim") => handle_session_claim(body_str, &mut stream, &shared),
        ("POST", "/api/session/tab") => handle_session_tab(body_str, &mut stream),

        // Logs Service
        ("GET", "/api/logs") => handle_get_logs(&mut stream),
        ("POST", "/api/logs/clear") => handle_clear_logs(&mut stream),

        // LLM Providers - Antigravity
        ("GET", "/api/llm/antigravity/accounts") => handle_get_antigravity_accounts(&mut stream),
        ("POST", "/api/llm/antigravity/accounts") => handle_post_antigravity_account(body_str, &mut stream),
        ("POST", "/api/llm/antigravity/accounts/delete") => handle_delete_antigravity_account(body_str, &mut stream),
        ("POST", "/api/llm/antigravity/accounts/refresh") => handle_refresh_antigravity_account(body_str, &mut stream),
        ("POST", "/api/llm/antigravity/accounts/toggle") => handle_toggle_antigravity_account(body_str, &mut stream),
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

        // Events Service (Separated SSE Streams)
        ("GET", "/api/events") | ("GET", "/api/events/state") => handle_sse_state(stream, shared),
        ("GET", "/api/events/menu") => handle_sse_menu(stream, shared),

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

        // Static Web Dashboard Files
        ("GET", path) => {
            let clean_path = path.trim_start_matches('/');
            let target_path = if clean_path.is_empty() {
                "index.html"
            } else {
                clean_path
            };

            if let Some(file) = EmbeddedWebDist::get(target_path) {
                let mime_type = match target_path.split('.').next_back().unwrap_or("") {
                    "html" => "text/html; charset=utf-8",
                    "js" => "application/javascript; charset=utf-8",
                    "css" => "text/css; charset=utf-8",
                    "svg" => "image/svg+xml",
                    "png" => "image/png",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                };

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

pub fn read_http_request(stream: &mut TcpStream) -> std::io::Result<(String, Vec<u8>)> {
    let mut header_bytes = Vec::new();
    let mut buf = [0u8; 4096];
    let mut header_end_pos = None;

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        header_bytes.extend_from_slice(&buf[..n]);
        if let Some(pos) = header_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end_pos = Some(pos);
            break;
        }
        if header_bytes.len() > 1024 * 1024 {
            break;
        }
    }

    let end_pos = match header_end_pos {
        Some(pos) => pos,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Incomplete HTTP headers",
            ));
        }
    };

    let header_str = String::from_utf8_lossy(&header_bytes[..end_pos]).to_string();
    let mut content_length = 0usize;
    for line in header_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length > 100 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "HTTP Content-Length exceeds 100MB limit",
        ));
    }

    let initial_body = header_bytes[end_pos + 4..].to_vec();
    let mut body = initial_body;
    while body.len() < content_length {
        let to_read = (content_length - body.len()).min(65536);
        let mut chunk = vec![0u8; to_read];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }

    Ok((header_str, body))
}

fn handle_client(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    // Hindari socket menggantung tak berujung saat browser me-refresh cepat
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

    let (header_str, body_bytes) = match read_http_request(&mut stream) {
        Ok(res) => res,
        Err(_) => return,
    };

    let mut lines = header_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let full_url = parts[1];
    let url = full_url.split('?').next().unwrap_or(full_url);

    // Jika ini SSE endpoint atau OpenAI API /v1, hilangkan read/write timeout agar stream long-lived tidak terputus timeout
    if url.starts_with("/api/events") || url.starts_with("/v1") || url.starts_with("/api/v1") {
        let _ = stream.set_read_timeout(None);
        let _ = stream.set_write_timeout(None);
    }

    // CORS preflight OPTIONS
    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
                    Access-Control-Allow-Headers: *\r\n\
                    Connection: close\r\n\
                    Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let body_str = std::str::from_utf8(&body_bytes).unwrap_or_default();

    handle_api_route(
        method,
        url,
        full_url,
        &header_str,
        body_str,
        &body_bytes,
        stream,
        shared,
    );
}

static PREBOUND_TCP_LISTENER: Mutex<Option<TcpListener>> = Mutex::new(None);

pub fn register_prebound_listener(listener: TcpListener) {
    if let Ok(mut lock) = PREBOUND_TCP_LISTENER.lock() {
        *lock = Some(listener);
    }
}

pub fn take_prebound_listener() -> Option<TcpListener> {
    if let Ok(mut lock) = PREBOUND_TCP_LISTENER.lock() {
        lock.take()
    } else {
        None
    }
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    thread::spawn(move || {
        let addr = "127.0.0.1:42128";
        let listener = if let Some(prebound) = take_prebound_listener() {
            println!(
                "🌐 [Web Dashboard & API Server] Berjalan di http://{}",
                addr
            );
            prebound
        } else {
            match TcpListener::bind(addr) {
                Ok(l) => {
                    println!(
                        "🌐 [Web Dashboard & API Server] Berjalan di http://{}",
                        addr
                    );
                    l
                }
                Err(e) => {
                    eprintln!("⚠️ [Web Server] Gagal bind ke {}: {}", addr, e);
                    return;
                }
            }
        };

        for stream in listener.incoming().flatten() {
            let shared_clone = Arc::clone(&shared);
            thread::spawn(move || {
                handle_client(stream, shared_clone);
            });
        }
    });
}
