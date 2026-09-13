use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn poses_config_path() -> PathBuf {
    let local = PathBuf::from("poses.json");
    if local.exists() {
        return local;
    }
    crate::config::get_app_config_dir().join("poses.json")
}

#[tauri::command]
pub fn get_poses_config() -> serde_json::Value {
    let path = poses_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            return v;
        }
    }
    serde_json::json!({})
}

#[tauri::command]
pub fn save_poses_config(app: tauri::AppHandle, config_json: String) -> Result<(), String> {
    let path = poses_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&path, &config_json).map_err(|e| e.to_string())?;

    let backup = crate::config::get_app_config_dir().join("poses.json");
    if let Some(parent) = backup.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(backup, &config_json);
    crate::term_log!("💾 [Pose Tuner] Berhasil menyimpan poses.json ke {:?}", path);

    crate::tray::TrayState::emit_event("assistant:poses-updated", serde_json::json!({
        "status": "success",
        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()
    }));

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.eval("if (window.loadExternalPoses) { window.loadExternalPoses(); }");
    }

    Ok(())
}

pub fn get_asset_file_path(filename: &str) -> Option<PathBuf> {
    let check_variants = |base_name: &str| -> Vec<String> {
        if base_name.ends_with(".vrm") || base_name.ends_with(".glb") {
            vec![base_name.to_string()]
        } else {
            let stem = base_name.trim_end_matches(".svg")
                .trim_end_matches(".png")
                .trim_end_matches(".webp")
                .trim_end_matches(".jpg");
            vec![
                format!("{}.svg", stem),
                format!("{}.png", stem),
                format!("{}.webp", stem),
                format!("{}.jpg", stem),
            ]
        }
    };

    let candidates = check_variants(filename);

    for cand in candidates {
        let local = PathBuf::from("assets").join(&cand);
        if local.exists() {
            return Some(local);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let p1 = parent.join("assets").join(&cand);
                if p1.exists() { return Some(p1); }
                if let Some(parent2) = parent.parent() {
                    let p2 = parent2.join("assets").join(&cand);
                    if p2.exists() { return Some(p2); }
                    if let Some(parent3) = parent2.parent() {
                        let p3 = parent3.join("assets").join(&cand);
                        if p3.exists() { return Some(p3); }
                    }
                }
            }
        }
        let home_asset = crate::config::get_app_config_dir().join("assets").join(&cand);
        if home_asset.exists() {
            return Some(home_asset);
        }
        let ui_asset = PathBuf::from("ui/assets").join(&cand);
        if ui_asset.exists() {
            return Some(ui_asset);
        }
    }
    None
}

#[tauri::command]
pub fn save_avatar_file(app: tauri::AppHandle, asset_type: String, file_bytes: Vec<u8>) -> Result<String, String> {
    if file_bytes.is_empty() {
        return Err("File kosong / tidak valid".to_string());
    }

    let base_state = match asset_type.to_lowercase().as_str() {
        "vrm" => "char.vrm",
        "idle" | "image_idle" => "char-avatar-idle",
        "listening" | "image_listening" => "char-avatar-listening",
        "thinking" | "image_thinking" => "char-avatar-thinking",
        "talking" | "speaking" | "image_talking" => "char-avatar-talking",
        _ => return Err(format!("Tipe aset '{}' tidak dikenal", asset_type)),
    };

    let target_filename = if base_state == "char.vrm" {
        "char.vrm".to_string()
    } else {
        let ext = if file_bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            "png"
        } else if file_bytes.starts_with(b"RIFF") && file_bytes.len() > 12 && &file_bytes[8..12] == b"WEBP" {
            "webp"
        } else if file_bytes.starts_with(b"\xff\xd8\xff") {
            "jpg"
        } else {
            "svg"
        };
        format!("{}.{}", base_state, ext)
    };

    let local_dir = PathBuf::from("assets");
    if local_dir.exists() {
        if base_state != "char.vrm" {
            for old_ext in &["svg", "png", "webp", "jpg"] {
                let _ = fs::remove_file(local_dir.join(format!("{}.{}", base_state, old_ext)));
            }
        }
        let local_path = local_dir.join(&target_filename);
        let _ = fs::write(&local_path, &file_bytes);
    }

    let home_assets = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&home_assets);
    if base_state != "char.vrm" {
        for old_ext in &["svg", "png", "webp", "jpg"] {
            let _ = fs::remove_file(home_assets.join(format!("{}.{}", base_state, old_ext)));
        }
    }
    let home_path = home_assets.join(&target_filename);
    let _ = fs::write(&home_path, &file_bytes);

    crate::term_log!("🎨 [Avatar Custom] File avatar berhasil disimpan sebagai '{}' ({} bytes)", target_filename, file_bytes.len());

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.eval("if (window.reloadAvatar) { window.reloadAvatar(); }");
    }

    Ok(format!("Aset {} berhasil diperbarui!", target_filename))
}

#[tauri::command]
pub fn get_external_asset_bytes(filename: String) -> Result<Vec<u8>, String> {
    let path = get_asset_file_path(&filename);
    crate::term_log!("🔍 [Overlay Asset Request] filename: '{}', resolved path: {:?}", filename, path);
    let path = path.ok_or_else(|| {
        crate::term_err!("❌ [Overlay Asset Error] File aset '{}' tidak ditemukan di folder assets eksternal!", filename);
        format!("File aset '{}' tidak ditemukan di folder assets eksternal", filename)
    })?;
    let bytes = fs::read(&path).map_err(|e| {
        crate::term_err!("❌ [Overlay Asset Read Error] Gagal baca {:?}: {}", path, e);
        e.to_string()
    })?;
    crate::term_log!("✅ [Overlay Asset Success] Berhasil membaca {:?} ({} bytes)", path, bytes.len());
    Ok(bytes)
}

#[tauri::command]
pub fn get_external_asset(filename: String) -> Result<String, String> {
    use base64::Engine;
    let path = get_asset_file_path(&filename)
        .ok_or_else(|| format!("File aset '{}' tidak ditemukan di folder assets eksternal", filename))?;

    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let mime = if filename.ends_with(".vrm") || filename.ends_with(".glb") {
        "model/gltf-binary"
    } else if filename.ends_with(".svg") {
        "image/svg+xml"
    } else if filename.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    };

    Ok(format!("data:{};base64,{}", mime, b64))
}
