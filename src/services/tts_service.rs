use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::net::TcpStream;

use crate::audio::supertonic::{get_engine, get_last_audio_wav, init_engine, set_last_audio_wav};
use crate::server::send_json_response;

#[derive(Deserialize)]
struct SynthesizeRequest {
    text: String,
    #[serde(default)]
    voice: String,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    speed: Option<f32>,
}

pub fn handle_tts_synthesize(body: &str, stream: &mut TcpStream) {
    let req: SynthesizeRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            let resp = json!({ "error": format!("Invalid JSON request: {e}") }).to_string();
            send_json_response(stream, 400, &resp);
            return;
        }
    };

    if req.text.trim().is_empty() {
        let resp = json!({ "error": "Text cannot be empty" }).to_string();
        send_json_response(stream, 400, &resp);
        return;
    }

    let engine = match get_engine() {
        Some(e) => e,
        None => match init_engine() {
            Ok(e) => e,
            Err(e) => {
                let resp =
                    json!({ "error": format!("Failed to initialize Supertonic engine: {e}") })
                        .to_string();
                send_json_response(stream, 500, &resp);
                return;
            }
        },
    };

    let cfg = crate::assets::load_config();
    let voice = if !req.voice.is_empty() {
        req.voice
    } else if !cfg.tts_voice_name.is_empty() {
        cfg.tts_voice_name
    } else {
        "F4".to_string()
    };

    let lang = if !req.lang.is_empty() {
        req.lang
    } else if !cfg.lang.is_empty() {
        cfg.lang
    } else {
        "id".to_string()
    };

    let speed = req.speed.unwrap_or(if cfg.tts_speed > 0.0 {
        cfg.tts_speed
    } else {
        1.05
    });

    match engine.synthesize(&req.text, &voice, &lang, speed) {
        Ok(wav_bytes) => {
            let id = format!("{:x}", md5_hash(&req.text));
            set_last_audio_wav(id.clone(), wav_bytes.clone());

            let resp_header = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: audio/wav\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 Access-Control-Allow-Origin: *\r\n\
                 X-Audio-Id: {}\r\n\
                 Cache-Control: no-cache\r\n\r\n",
                wav_bytes.len(),
                id
            );
            let _ = stream.write_all(resp_header.as_bytes());
            let _ = stream.write_all(&wav_bytes);
        }
        Err(e) => {
            let resp = json!({ "error": format!("Synthesis failed: {e}") }).to_string();
            send_json_response(stream, 500, &resp);
        }
    }
}

pub fn handle_get_tts_audio(full_url: &str, _header_str: &str, stream: &mut TcpStream) {
    let req_id = full_url.split('?').nth(1).and_then(|query| {
        for part in query.split('&') {
            let mut kv = part.splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                if k == "id" {
                    return Some(v);
                }
            }
        }
        None
    });

    if let Some(wav_bytes) = get_last_audio_wav(req_id) {
        let resp_header = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: audio/wav\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Cache-Control: no-cache\r\n\r\n",
            wav_bytes.len()
        );
        let _ = stream.write_all(resp_header.as_bytes());
        let _ = stream.write_all(&wav_bytes);
    } else {
        send_json_response(stream, 404, "{\"error\":\"No audio available\"}");
    }
}

fn md5_hash(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
