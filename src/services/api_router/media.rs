use std::sync::{Arc, Mutex};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::ServerSharedState;
use crate::ui::pose_system::AssistantVisualState;

pub fn register_media_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        // Poses & Tuner
        .route("/api/poses", get(axum_get_poses).post(axum_post_poses))
        .route("/api/tuner/preview", post(axum_tuner_preview))
        .route("/api/tuner/live", post(axum_tuner_live))
        // Music Management
        .route("/api/music/check", get(axum_music_check))
        .route("/api/music/install-ytdlp", post(axum_music_install_ytdlp))
        .route("/api/music/list", get(axum_music_list))
        .route("/api/music/control", post(axum_music_control))
        .route("/api/music/download", post(axum_music_download))
        .route("/api/music/delete", post(axum_music_delete))
        // STT & TTS
        .route("/api/stt/transcript", post(axum_stt_transcript))
        .route("/api/tts/state", post(axum_tts_state))
        .route("/api/tts/synthesize", post(axum_tts_synthesize))
        // Assets & Static File Serving
        .route("/api/assets/info", get(axum_assets_info))
        .route("/api/assets/file/*path", get(axum_assets_file))
        .route("/api/upload/vrm", post(axum_upload_vrm))
        .route("/api/upload/avatar", post(axum_upload_avatar))
        .route("/api/upload/refvoice", post(axum_upload_refvoice))
        .route("/api/delete/refvoice", post(axum_delete_refvoice))
}

// -----------------------------------------------------------------------------
// 1. Poses & Tuner Service
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_poses() -> impl IntoResponse {
    Json(crate::assets::load_poses())
}

pub(crate) async fn axum_post_poses(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(poses): Json<crate::ui::pose_system::PosesConfig>,
) -> impl IntoResponse {
    if let Err(e) = crate::assets::save_poses(&poses) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response()
    } else {
        if let Ok(mut lock) = shared.lock() {
            lock.poses_updated = true;
            lock.preview_poses = None;
        }
        Json(json!({ "success": true })).into_response()
    }
}

#[derive(Deserialize)]
pub(crate) struct TunerPreviewPayload {
    pub(crate) poses: crate::ui::pose_system::PosesConfig,
    pub(crate) emotion: Option<String>,
    pub(crate) state: Option<String>,
}

pub(crate) async fn axum_tuner_preview(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<TunerPreviewPayload>,
) -> impl IntoResponse {
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
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub(crate) struct TunerLivePayload {
    pub(crate) active: bool,
    #[serde(default)]
    pub(crate) bones: std::collections::HashMap<String, [f32; 3]>,
    pub(crate) emotion: Option<String>,
    pub(crate) state: Option<String>,
}

pub(crate) async fn axum_tuner_live(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<TunerLivePayload>,
) -> impl IntoResponse {
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
        }
    }
    Json(json!({ "success": true }))
}

// -----------------------------------------------------------------------------
// 2. Music Management
// -----------------------------------------------------------------------------
pub(crate) async fn axum_music_check() -> impl IntoResponse {
    Json(crate::engine::ytdlp::check_ytdlp())
}

pub(crate) async fn axum_music_install_ytdlp() -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(crate::engine::ytdlp::install_ytdlp).await;
    match res {
        Ok(Ok(msg)) => {
            let info = crate::engine::ytdlp::check_ytdlp();
            Json(json!({
                "success": true,
                "message": msg,
                "info": info
            }))
            .into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": e })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_music_list() -> impl IntoResponse {
    let music_dir = crate::config::get_music_dir();
    let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
    Json(json!({
        "music_dir": music_dir.to_string_lossy(),
        "tracks": tracks,
        "volume": crate::audio::player::get_music_volume()
    }))
}

#[derive(Deserialize)]
pub(crate) struct MediaControlPayload {
    pub(crate) action: String,
    pub(crate) payload: Option<String>,
}

pub(crate) async fn axum_music_control(Json(data): Json<MediaControlPayload>) -> impl IntoResponse {
    let action_str = data.action.trim().to_lowercase();
    let payload_str = data.payload.unwrap_or_default();
    let res = crate::engine::actions::media::execute_media_action(&action_str, &payload_str);
    if let Some((title, desc)) = res {
        Json(json!({
            "success": true,
            "title": title,
            "description": desc
        }))
        .into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Invalid control action or payload" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub(crate) struct MediaDownloadPayload {
    pub(crate) query: String,
}

pub(crate) async fn axum_music_download(Json(payload): Json<MediaDownloadPayload>) -> impl IntoResponse {
    let music_dir = crate::config::get_music_dir();
    let query = payload.query;
    let res = tokio::task::spawn_blocking(move || {
        crate::engine::ytdlp::download_audio(&query, &music_dir)
    })
    .await;

    match res {
        Ok(Ok(track)) => Json(json!({
            "success": true,
            "track": track
        }))
        .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct MediaDeletePayload {
    pub(crate) filename: String,
}

pub(crate) async fn axum_music_delete(Json(payload): Json<MediaDeletePayload>) -> impl IntoResponse {
    let music_dir = crate::config::get_music_dir();
    let file_path = music_dir.join(&payload.filename);
    let _ = std::fs::remove_file(&file_path);

    let info_json = file_path.with_extension("info.json");
    let info_json_alt = music_dir.join(format!("{}.info.json", payload.filename));
    let _ = std::fs::remove_file(info_json);
    let _ = std::fs::remove_file(info_json_alt);

    Json(json!({ "success": true }))
}

// -----------------------------------------------------------------------------
// 3. STT & TTS
// -----------------------------------------------------------------------------
pub(crate) async fn axum_stt_transcript(
    Json(payload): Json<super::automation::RemoteTextPayload>,
) -> impl IntoResponse {
    let clean = payload.text.trim();
    if !clean.is_empty() {
        println!("[Mobile STT -> Remote Text] Transcript received: {}", clean);
        crate::audio::supertonic::prewarm_tts_worker();
        crate::engine::remote_text::inject_remote_text(clean.to_string());
    }
    Json(json!({ "success": true }))
}

#[derive(Deserialize, Default)]
pub(crate) struct TtsStatePayload {
    #[serde(default)]
    pub(crate) speaking: Option<bool>,
    #[serde(default)]
    pub(crate) listening: Option<bool>,
    #[serde(default)]
    pub(crate) state: Option<String>,
    #[serde(default)]
    pub(crate) emotion: Option<String>,
    #[serde(default)]
    pub(crate) event: Option<String>,
    #[serde(default)]
    pub(crate) id: Option<String>,
}

pub(crate) async fn axum_tts_state(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<TtsStatePayload>,
) -> impl IntoResponse {
    // 1. Check audio eviction / event ended
    if let Some(ref ev) = payload.event {
        if ev == "ended" || ev == "error" {
            if let Ok(mut lock) = shared.lock() {
                if lock.current_state == AssistantVisualState::Speaking {
                    lock.current_state = AssistantVisualState::Idle;
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
            }
        }
    }
    if let Some(ref id) = payload.id {
        if !id.is_empty() {
            crate::audio::supertonic::evict_audio(id);
        }
    }

    // 2. Check speaking / listening / state / emotion transitions from mobile/desktop
    let emotion = payload.emotion.unwrap_or_else(|| "neutral".to_string());
    let target_state = if let Some(ref st) = payload.state {
        st.to_lowercase()
    } else if payload.speaking == Some(true) {
        "speaking".to_string()
    } else if payload.listening == Some(true) {
        "listening".to_string()
    } else if payload.speaking == Some(false) || payload.listening == Some(false) {
        "idle".to_string()
    } else {
        String::new()
    };

    if !target_state.is_empty() {
        if target_state == "listening" {
            crate::audio::supertonic::prewarm_tts_worker();
        }
        if target_state == "speaking" {
            crate::app_state::set_ai_speaking(true);
            crate::app_state::touch_speech_activity();
            let asst_name = crate::config::load_user_config().assistant_name;
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Speaking,
                &format!("{asst_name} Speaking"),
                &emotion,
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = AssistantVisualState::Speaking;
                lock.current_emotion = emotion;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if target_state == "listening" {
            if crate::app_state::is_session_running() {
                let is_speaking = crate::app_state::is_ai_speaking() || crate::audio::player::is_native_voice_playing();
                let cfg = crate::assets::load_config();
                if is_speaking && cfg.voice_interruption {
                    crate::audio::player::stop_native_voice();
                    crate::app_state::trigger_interruption();
                }
                crate::app_state::set_ai_speaking(false);
                let asst_name = cfg.assistant_name;
                crate::tray::TrayState::update_with_emotion(
                    crate::services::tray_service::AssistantState::Listening,
                    &format!("{asst_name} Listening..."),
                    &emotion,
                );
                if let Ok(mut lock) = shared.lock() {
                    lock.current_state = AssistantVisualState::Listening;
                    lock.current_emotion = emotion;
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
            }
        } else if target_state == "thinking" {
            if !crate::audio::player::is_native_voice_playing() {
                crate::app_state::set_ai_speaking(false);
            }
            let asst_name = crate::config::load_user_config().assistant_name;
            crate::tray::TrayState::update_with_emotion(
                crate::services::tray_service::AssistantState::Thinking,
                &format!("{asst_name} Thinking..."),
                &emotion,
            );
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = AssistantVisualState::Thinking;
                lock.current_emotion = emotion;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if target_state == "idle" {
            if !crate::audio::player::is_native_voice_playing() {
                crate::app_state::set_ai_speaking(false);
                crate::tray::TrayState::update_with_emotion(
                    crate::services::tray_service::AssistantState::Idle,
                    "Standby",
                    "neutral",
                );
            }
            if let Ok(mut lock) = shared.lock() {
                lock.current_state = AssistantVisualState::Idle;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        }
    }

    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub(crate) struct SynthesizePayload {
    pub(crate) text: String,
    #[serde(default, alias = "tts_voice_name", alias = "voice_name")]
    pub(crate) voice: String,
    #[serde(default)]
    pub(crate) lang: String,
    #[serde(default, alias = "tts_speed")]
    pub(crate) speed: Option<f32>,
}

pub(crate) async fn axum_tts_synthesize(Json(req): Json<SynthesizePayload>) -> impl IntoResponse {
    if req.text.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Text cannot be empty" })),
        )
            .into_response();
    }

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

    let text_clone = req.text.clone();
    let res = tokio::task::spawn_blocking(move || {
        crate::engine::os_efficiency::notify_active();
        crate::audio::supertonic::synthesize_auto(&text_clone, &voice, &lang, speed)
            .map_err(|e| format!("Synthesis failed: {e}"))
    })
    .await;

    match res {
        Ok(Ok(wav_bytes)) => {
            crate::engine::os_efficiency::notify_active();
            let mut h = 0xcbf29ce484222325u64;
            for b in req.text.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            let id = format!("{:x}", h);
            crate::audio::supertonic::set_last_audio_wav(id.clone(), wav_bytes.clone());

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "audio/wav")
                .header("Content-Length", wav_bytes.len().to_string())
                .header("Access-Control-Allow-Origin", "*")
                .header("X-Audio-Id", id)
                .header("Cache-Control", "no-cache")
                .body(axum::body::Body::from(wav_bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Ok(Err(err_msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err_msg })),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": join_err.to_string() })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// 4. Assets & Static File Serving
// -----------------------------------------------------------------------------
pub(crate) async fn axum_assets_info() -> impl IntoResponse {
    let assets_dir = crate::assets::get_assets_dir();
    let vrm_path = crate::assets::find_file_in_candidates("char.vrm");
    let vrm_exists = vrm_path.exists();
    let vrm_size = if vrm_exists {
        std::fs::metadata(&vrm_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let refvoice_path = crate::audio::openvoice::find_ref_voice_path();
    let refvoice_exists = refvoice_path.is_some();
    let (refvoice_filename, refvoice_size_bytes, refvoice_size_mb) = if let Some(ref p) = refvoice_path {
        let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        (name, size, (size as f32) / (1024.0 * 1024.0))
    } else {
        (String::new(), 0, 0.0)
    };

    let mut images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&assets_dir) {
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

    Json(json!({
        "vrm_exists": vrm_exists,
        "vrm_size_bytes": vrm_size,
        "vrm_size_mb": (vrm_size as f32) / (1024.0 * 1024.0),
        "vrm_path": vrm_path.to_string_lossy(),
        "refvoice_exists": refvoice_exists,
        "refvoice_filename": refvoice_filename,
        "refvoice_size_bytes": refvoice_size_bytes,
        "refvoice_size_mb": refvoice_size_mb,
        "images": images
    }))
}

pub(crate) async fn axum_assets_file(
    Path(path): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let clean_path = path.trim_start_matches('/');
    let clean_filename = clean_path.split('?').next().unwrap_or("");
    let file_path = crate::assets::find_file_in_candidates(clean_filename);

    if file_path.exists() && file_path.is_file() {
        let ext = clean_filename.split('.').next_back().unwrap_or("").to_lowercase();
        if let Ok(data) = std::fs::read(&file_path) {
            // Audio files support HTTP 206 Partial Content (Range header) for seamless HTML5 <audio> seeking & mobile
            if ["wav", "mp3", "ogg", "flac", "m4a", "aac", "webm", "opus"].contains(&ext.as_str()) {
                let mime = match ext.as_str() {
                    "mp3" => "audio/mpeg",
                    "ogg" => "audio/ogg",
                    "flac" => "audio/flac",
                    "m4a" | "aac" => "audio/mp4",
                    "webm" => "audio/webm",
                    "opus" => "audio/opus",
                    _ => "audio/wav",
                };
                return crate::server::serve_audio_bytes_with_range(data, mime, &headers);
            }

            let mime_type = match ext.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "vrm" => "model/gltf-binary",
                "json" => "application/json",
                _ => "application/octet-stream",
            };
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .header("Content-Length", data.len().to_string())
                .header("Cache-Control", "public, max-age=3600")
                .header("Access-Control-Allow-Origin", "*")
                .body(axum::body::Body::from(data))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    }
    (StatusCode::NOT_FOUND, "404 Asset Not Found").into_response()
}

#[derive(Deserialize)]
pub(crate) struct VrmUploadQuery {
    #[serde(default)]
    pub(crate) filename: Option<String>,
}

pub(crate) async fn axum_upload_vrm(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Query(q): Query<VrmUploadQuery>,
    body: Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty file payload" })),
        )
            .into_response();
    }

    if let Some(ref fname) = q.filename {
        static RE_VRM_EXT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"(?i)\.vrm$").expect("valid vrm regex")
        });
        if !RE_VRM_EXT.is_match(fname) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid format: 3D model strictly requires a .vrm file" })),
            )
                .into_response();
        }
    }

    let assets_dir = crate::assets::get_assets_dir();
    let _ = std::fs::create_dir_all(&assets_dir);
    let target_path = assets_dir.join("char.vrm");

    match std::fs::write(&target_path, &body) {
        Ok(_) => {
            crate::term_log!(
                "[Asset Manager] VRM model successfully uploaded ({:.2} MB) to {:?}",
                (body.len() as f32) / (1024.0 * 1024.0),
                target_path
            );
            if let Ok(mut lock) = shared.lock() {
                lock.vrm_updated = true;
            }
            Json(json!({
                "success": true,
                "message": "VRM model uploaded and reloaded successfully"
            }))
            .into_response()
        }
        Err(e) => {
            crate::term_err!("[Asset Manager] Failed to save VRM file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct SlotQuery {
    #[serde(default)]
    pub(crate) slot: Option<String>,
}

pub(crate) async fn axum_upload_avatar(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Query(q): Query<SlotQuery>,
    body: Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty image payload" })),
        )
            .into_response();
    }

    let slot = q.slot.unwrap_or_else(|| "idle".to_string());
    let filename = match slot.to_lowercase().as_str() {
        "listening" => "char-avatar-listening.png",
        "speaking" => "char-avatar-speaking.png",
        "thinking" => "char-avatar-thinking.png",
        _ => "char-avatar-idle.png",
    };

    let assets_dir = crate::assets::get_assets_dir();
    let _ = std::fs::create_dir_all(&assets_dir);
    let target_path = assets_dir.join(filename);

    match std::fs::write(&target_path, &body) {
        Ok(_) => {
            crate::term_log!(
                "[Asset Manager] Avatar image for slot '{}' saved to {:?}",
                slot,
                target_path
            );
            if let Ok(mut lock) = shared.lock() {
                lock.vrm_updated = true;
            }
            Json(json!({ "success": true })).into_response()
        }
        Err(e) => {
            crate::term_err!("[Asset Manager] Failed to save avatar: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct RefvoiceQuery {
    #[serde(default)]
    pub(crate) ext: Option<String>,
    #[serde(default)]
    pub(crate) filename: Option<String>,
}

pub(crate) async fn axum_upload_refvoice(
    Query(q): Query<RefvoiceQuery>,
    body: Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty audio file payload" })),
        )
            .into_response();
    }

    let ext = q.ext.or_else(|| {
        q.filename.as_ref().and_then(|f| f.split('.').next_back().map(|s| s.to_string()))
    }).unwrap_or_else(|| "wav".to_string()).to_lowercase();

    let clean_ext = match ext.as_str() {
        "mp3" => "mp3",
        "ogg" => "ogg",
        "flac" => "flac",
        "m4a" => "m4a",
        _ => "wav",
    };

    let assets_dir = crate::assets::get_assets_dir();
    let _ = std::fs::create_dir_all(&assets_dir);

    // Remove old refvoice files
    for old_ext in &["wav", "mp3", "ogg", "flac", "m4a"] {
        let old = assets_dir.join(format!("refvoice.{}", old_ext));
        let _ = std::fs::remove_file(old);
    }

    let target_path = assets_dir.join(format!("refvoice.{}", clean_ext));
    match std::fs::write(&target_path, &body) {
        Ok(_) => {
            crate::term_log!(
                "[OpenVoice V2] Custom reference voice uploaded ({:.2} MB) to {:?}",
                (body.len() as f32) / (1024.0 * 1024.0),
                target_path
            );
            crate::audio::openvoice::invalidate_ref_cache();
            crate::audio::supertonic::clear_all_audio_cache();
            crate::audio::supertonic::TTS_SUPERVISOR.kill_now();
            Json(json!({
                "success": true,
                "message": format!("Reference voice uploaded as refvoice.{}", clean_ext)
            }))
            .into_response()
        }
        Err(e) => {
            crate::term_err!("[OpenVoice V2] Failed to save refvoice: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn axum_delete_refvoice() -> impl IntoResponse {
    let assets_dir = crate::assets::get_assets_dir();
    let mut deleted = false;
    for ext in &["wav", "mp3", "ogg", "flac", "m4a"] {
        let file = assets_dir.join(format!("refvoice.{}", ext));
        if file.exists() {
            let _ = std::fs::remove_file(file);
            deleted = true;
        }
    }
    crate::audio::openvoice::invalidate_ref_cache();
    crate::audio::supertonic::clear_all_audio_cache();
    crate::audio::supertonic::TTS_SUPERVISOR.kill_now();
    if deleted {
        Json(json!({ "success": true, "message": "Reference voice deleted, restored to Supertonic default" })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No custom reference voice found to delete" })),
        )
            .into_response()
    }
}
