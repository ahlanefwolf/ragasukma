use serde::Deserialize;
use serde_json::json;
use std::io::Write;

use crate::audio::supertonic::{
    get_engine, get_last_audio_opus, get_last_audio_wav, init_engine, set_last_audio_opus,
    set_last_audio_wav,
};
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

pub fn handle_tts_synthesize(body: &str, stream: &mut dyn Write) {
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

            let (audio_bytes, content_type) =
                if let Ok(opus_bytes) = crate::audio::opus::encode_wav_to_opus(&wav_bytes) {
                    set_last_audio_opus(id.clone(), opus_bytes.clone());
                    (opus_bytes, "audio/ogg; codecs=opus")
                } else {
                    (wav_bytes, "audio/wav")
                };

            let resp_header = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: {}\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 Access-Control-Allow-Origin: *\r\n\
                 X-Audio-Id: {}\r\n\
                 Cache-Control: no-cache\r\n\r\n",
                content_type,
                audio_bytes.len(),
                id
            );
            let _ = stream.write_all(resp_header.as_bytes());
            let _ = stream.write_all(&audio_bytes);
        }
        Err(e) => {
            let resp = json!({ "error": format!("Synthesis failed: {e}") }).to_string();
            send_json_response(stream, 500, &resp);
        }
    }
}

pub fn handle_get_tts_audio(full_url: &str, header_str: &str, stream: &mut dyn Write) {
    let mut req_id = None;
    let mut req_format = None;

    if let Some(query) = full_url.split('?').nth(1) {
        for part in query.split('&') {
            let mut kv = part.splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                if k == "id" {
                    req_id = Some(v);
                } else if k == "format" {
                    req_format = Some(v);
                }
            }
        }
    }

    let (audio_bytes, content_type) = if req_format == Some("wav") {
        if let Some(bytes) = get_last_audio_wav(req_id) {
            (bytes, "audio/wav")
        } else {
            send_json_response(stream, 404, "{\"error\":\"Audio not found\"}");
            return;
        }
    } else if let Some(opus) = get_last_audio_opus(req_id) {
        (opus, "audio/ogg; codecs=opus")
    } else if let Some(wav) = get_last_audio_wav(req_id) {
        if let Ok(opus) = crate::audio::opus::encode_wav_to_opus(&wav) {
            if let Some(id) = req_id {
                set_last_audio_opus(id.to_string(), opus.clone());
            }
            (opus, "audio/ogg; codecs=opus")
        } else {
            (wav, "audio/wav")
        }
    } else {
        send_json_response(stream, 404, "{\"error\":\"Audio not found\"}");
        return;
    };

    let total_len = audio_bytes.len() as u64;
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
    let resp_header = if is_range {
        format!(
            "HTTP/1.1 206 Partial Content\r\n\
             Content-Type: {}\r\n\
             Content-Range: bytes {}-{}/{}\r\n\
             Content-Length: {}\r\n\
             Accept-Ranges: bytes\r\n\
             Connection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, OPTIONS\r\n\
             Access-Control-Allow-Headers: *\r\n\
             Cache-Control: no-cache, no-store\r\n\r\n",
            content_type, start, end, total_len, chunk_len
        )
    } else {
        format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Accept-Ranges: bytes\r\n\
             Connection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, OPTIONS\r\n\
             Access-Control-Allow-Headers: *\r\n\
             Cache-Control: no-cache, no-store\r\n\r\n",
            content_type, total_len
        )
    };

    let _ = stream.write_all(resp_header.as_bytes());
    if start < total_len && end < total_len {
        let _ = stream.write_all(&audio_bytes[start as usize..=end as usize]);
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
