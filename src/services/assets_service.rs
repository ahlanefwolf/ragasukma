use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::app_state::ServerSharedState;
use crate::assets::{find_file_in_candidates, get_assets_dir};
use crate::server::send_json_response;

pub fn handle_assets_info(stream: &mut dyn Write) {
    let assets_dir = get_assets_dir();
    let vrm_path = find_file_in_candidates("char.vrm");
    let vrm_exists = vrm_path.exists();
    let vrm_size = if vrm_exists {
        fs::metadata(&vrm_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let refvoice_path = crate::audio::openvoice::find_ref_voice_path();
    let refvoice_exists = refvoice_path.is_some();
    let (refvoice_filename, refvoice_size_bytes, refvoice_size_mb) = if let Some(ref p) = refvoice_path {
        let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        (name, size, (size as f32) / (1024.0 * 1024.0))
    } else {
        (String::new(), 0, 0.0)
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
        refvoice_exists: bool,
        refvoice_filename: String,
        refvoice_size_bytes: u64,
        refvoice_size_mb: f32,
        images: Vec<String>,
    }

    let resp = AssetInfoResponse {
        vrm_exists,
        vrm_size_bytes: vrm_size,
        vrm_size_mb: (vrm_size as f32) / (1024.0 * 1024.0),
        vrm_path: vrm_path.to_string_lossy().to_string(),
        refvoice_exists,
        refvoice_filename,
        refvoice_size_bytes,
        refvoice_size_mb,
        images,
    };

    let json = serde_json::to_string(&resp).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_upload_vrm(
    body_bytes: &[u8],
    stream: &mut dyn Write,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    if body_bytes.is_empty() {
        send_json_response(stream, 400, "{\"error\":\"Empty file payload\"}");
        return;
    }

    let assets_dir = get_assets_dir();
    let _ = fs::create_dir_all(&assets_dir);
    let target_path = assets_dir.join("char.vrm");

    match fs::write(&target_path, body_bytes) {
        Ok(_) => {
            crate::term_log!(
                "[Asset Manager] VRM model successfully uploaded ({:.2} MB) to {:?}",
                (body_bytes.len() as f32) / (1024.0 * 1024.0),
                target_path
            );
            if let Ok(mut lock) = shared.lock() {
                lock.vrm_updated = true;
            }
            send_json_response(
                stream,
                200,
                "{\"success\":true,\"message\":\"VRM model uploaded and reloaded successfully\"}",
            );
        }
        Err(e) => {
            crate::term_err!("[Asset Manager] Failed to save VRM file: {}", e);
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        }
    }
}

pub fn handle_upload_avatar(
    full_url: &str,
    body_bytes: &[u8],
    stream: &mut dyn Write,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    if body_bytes.is_empty() {
        send_json_response(stream, 400, "{\"error\":\"Empty image payload\"}");
        return;
    }

    let slot = full_url
        .split('?')
        .nth(1)
        .and_then(|q| {
            for pair in q.split('&') {
                let mut kv = pair.split('=');
                if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                    if k == "slot" {
                        return Some(v.to_string());
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| "idle".to_string());

    let filename = match slot.to_lowercase().as_str() {
        "listening" => "char-avatar-listening.png",
        "speaking" => "char-avatar-speaking.png",
        "thinking" => "char-avatar-thinking.png",
        _ => "char-avatar-idle.png",
    };

    let assets_dir = get_assets_dir();
    let _ = fs::create_dir_all(&assets_dir);
    let target_path = assets_dir.join(filename);

    match fs::write(&target_path, body_bytes) {
        Ok(_) => {
            crate::term_log!(
                "[Asset Manager] Avatar image for slot '{}' saved to {:?}",
                slot,
                target_path
            );
            if let Ok(mut lock) = shared.lock() {
                lock.vrm_updated = true;
            }
            send_json_response(stream, 200, "{\"success\":true}");
        }
        Err(e) => {
            crate::term_err!("[Asset Manager] Failed to save avatar: {}", e);
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        }
    }
}

pub fn handle_upload_refvoice(
    full_url: &str,
    body_bytes: &[u8],
    stream: &mut dyn Write,
) {
    if body_bytes.is_empty() {
        send_json_response(stream, 400, "{\"error\":\"Empty audio file payload\"}");
        return;
    }

    // Determine extension from URL query params (e.g. ?ext=mp3 or ?filename=myref.wav)
    let ext = full_url
        .split('?')
        .nth(1)
        .and_then(|q| {
            for pair in q.split('&') {
                let mut kv = pair.split('=');
                if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                    if k == "ext" {
                        return Some(v.to_string());
                    }
                    if k == "filename" {
                        if let Some(pos) = v.rfind('.') {
                            return Some(v[pos + 1..].to_string());
                        }
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| "wav".to_string())
        .to_lowercase();

    let clean_ext = match ext.as_str() {
        "mp3" | "ogg" | "flac" | "m4a" | "aac" | "wav" => ext,
        _ => "wav".to_string(),
    };

    let assets_dir = get_assets_dir();
    let _ = fs::create_dir_all(&assets_dir);

    // Remove any existing refvoice.* files to keep the naming clean and unambiguous
    let supported_exts = ["wav", "mp3", "ogg", "flac", "m4a", "aac"];
    for old_ext in &supported_exts {
        let old_path = assets_dir.join(format!("refvoice.{old_ext}"));
        if old_path.exists() {
            let _ = fs::remove_file(old_path);
        }
    }

    let target_filename = format!("refvoice.{clean_ext}");
    let target_path = assets_dir.join(&target_filename);

    match fs::write(&target_path, body_bytes) {
        Ok(_) => {
            crate::term_log!(
                "[Asset Manager] Reference voice successfully saved as {:?} ({:.2} MB)",
                target_path,
                (body_bytes.len() as f32) / (1024.0 * 1024.0)
            );
            crate::audio::openvoice::invalidate_ref_cache();
            send_json_response(
                stream,
                200,
                &format!("{{\"success\":true,\"filename\":\"{}\"}}", target_filename),
            );
        }
        Err(e) => {
            crate::term_err!("[Asset Manager] Failed to save reference voice file: {}", e);
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        }
    }
}

pub fn handle_delete_refvoice(stream: &mut dyn Write) {
    let assets_dir = get_assets_dir();
    let supported_exts = ["wav", "mp3", "ogg", "flac", "m4a", "aac"];
    let mut deleted = false;
    for ext in &supported_exts {
        let path = assets_dir.join(format!("refvoice.{ext}"));
        if path.exists() && fs::remove_file(&path).is_ok() {
            deleted = true;
        }
    }

    crate::audio::openvoice::invalidate_ref_cache();
    crate::term_log!("[Asset Manager] Reference voice deleted, fallback to native Supertonic TTS");
    send_json_response(
        stream,
        200,
        &format!("{{\"success\":true,\"deleted\":{deleted}}}"),
    );
}
