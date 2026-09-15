pub mod assets;
pub mod handlers;
pub mod state;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub use assets::*;
pub use handlers::*;
pub use state::*;

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
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete HTTP headers")),
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

    // WebSocket Upgrade Route untuk Browser Companion
    if url == "/ws/browser" || header_str.to_lowercase().contains("upgrade: websocket") {
        crate::engine_browser::handle_websocket_upgrade(stream, &header_str);
        return;
    }

    // CORS preflight OPTIONS
    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type, Content-Length\r\n\
                    Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => "",
    };

    handle_api_route(method, url, full_url, body_str, &body_bytes, stream, shared);
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    thread::spawn(move || {
        let addr = "127.0.0.1:42128";
        let listener = match TcpListener::bind(addr) {
            Ok(l) => {
                println!("🌐 [Web Dashboard & API Server] Berjalan di http://{}", addr);
                open_browser(&format!("http://{}", addr));
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
