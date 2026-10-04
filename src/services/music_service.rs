use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::net::TcpStream;

use crate::engine::ytdlp::{check_ytdlp, download_audio, list_downloaded};
use crate::server::{send_disk_file_stream_with_range, send_json_response};

pub fn handle_music_control(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct ControlPayload {
        action: String,
        payload: Option<String>,
    }

    if let Ok(data) = serde_json::from_str::<ControlPayload>(body_str) {
        let action_str = data.action.trim().to_lowercase();
        let payload_str = data.payload.unwrap_or_default();
        let res = crate::engine::actions::media::execute_media_action(&action_str, &payload_str);
        if let Some((title, desc)) = res {
            let json = serde_json::json!({
                "success": true,
                "title": title,
                "description": desc
            })
            .to_string();
            send_json_response(stream, 200, &json);
            return;
        }
    }

    send_json_response(
        stream,
        400,
        "{\"success\":false,\"error\":\"Invalid control action or payload\"}",
    );
}

pub fn handle_music_check(stream: &mut TcpStream) {
    let info = check_ytdlp();
    let json = serde_json::to_string(&info).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_music_install_ytdlp(stream: &mut TcpStream) {
    match crate::engine::ytdlp::install_ytdlp() {
        Ok(msg) => {
            let info = check_ytdlp();
            let json = serde_json::json!({
                "success": true,
                "message": msg,
                "info": info
            })
            .to_string();
            send_json_response(stream, 200, &json);
        }
        Err(e) => {
            let json = serde_json::json!({
                "success": false,
                "message": e
            })
            .to_string();
            send_json_response(stream, 500, &json);
        }
    }
}

pub fn handle_music_list(stream: &mut TcpStream) {
    let music_dir = crate::config::get_music_dir();
    let tracks = list_downloaded(&music_dir);
    let resp = serde_json::json!({
        "music_dir": music_dir.to_string_lossy(),
        "tracks": tracks
    });
    let json = serde_json::to_string(&resp).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

fn decode_url_component(input: &str) -> String {
    let mut bytes = Vec::new();
    let input_bytes = input.as_bytes();
    let mut i = 0;
    while i < input_bytes.len() {
        if input_bytes[i] == b'%' && i + 2 < input_bytes.len() {
            if let Ok(h) = u8::from_str_radix(
                std::str::from_utf8(&input_bytes[i + 1..=i + 2]).unwrap_or(""),
                16,
            ) {
                bytes.push(h);
                i += 3;
                continue;
            }
        }
        bytes.push(input_bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).to_string()
}

pub fn handle_music_stream(full_url: &str, header_str: &str, stream: &mut TcpStream) {
    let url_without_query = full_url.split_once('?').map(|(p, _)| p).unwrap_or(full_url);

    // 1. Coba take name file from path direct /music/<filename>
    let filename = if let Some(stripped) = url_without_query.strip_prefix("/music/") {
        decode_url_component(stripped)
    } else {
        // 2. Fallback query param ?filename=... or ?file=...
        let query_str = full_url.split_once('?').map(|(_, q)| q).unwrap_or("");
        form_urlencoded::parse(query_str.as_bytes())
            .find(|(k, _)| k == "filename" || k == "file")
            .map(|(_, v)| v.into_owned())
            .unwrap_or_default()
    };

    if filename.is_empty() {
        send_json_response(stream, 400, "{\"error\":\"Missing filename param\"}");
        return;
    }

    let music_dir = crate::config::get_music_dir();
    let file_path = music_dir.join(&filename);

    if file_path.exists() {
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

        let _ = send_disk_file_stream_with_range(stream, mime_type, &file_path, header_str);
    } else {
        let body = "{\"error\":\"File not found\"}";
        let resp = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes());
    }
}

pub fn handle_music_download(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct DownloadPayload {
        query: String,
    }
    if let Ok(payload) = serde_json::from_str::<DownloadPayload>(body_str) {
        let music_dir = crate::config::get_music_dir();
        match download_audio(&payload.query, &music_dir) {
            Ok(track) => {
                #[derive(serde::Serialize)]
                struct DownloadResponse {
                    success: bool,
                    track: crate::engine::ytdlp::YtTrackInfo,
                }
                let resp = DownloadResponse {
                    success: true,
                    track,
                };
                let json = serde_json::to_string(&resp).unwrap_or_default();
                send_json_response(stream, 200, &json);
            }
            Err(e) => {
                let msg = format!("{{\"success\":false,\"error\":\"{}\"}}", e);
                send_json_response(stream, 500, &msg);
            }
        }
    } else {
        send_json_response(
            stream,
            400,
            "{\"error\":\"Invalid payload, expected {query}\"}",
        );
    }
}

pub fn handle_music_delete(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct DeletePayload {
        filename: String,
    }
    if let Ok(payload) = serde_json::from_str::<DeletePayload>(body_str) {
        let music_dir = crate::config::get_music_dir();
        let file_path = music_dir.join(&payload.filename);
        let _ = fs::remove_file(&file_path);

        let info_json = file_path.with_extension("info.json");
        let info_json_alt = music_dir.join(format!("{}.info.json", payload.filename));
        let _ = fs::remove_file(info_json);
        let _ = fs::remove_file(info_json_alt);

        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(
            stream,
            400,
            "{\"error\":\"Invalid payload, expected {filename}\"}",
        );
    }
}
