use rust_embed::RustEmbed;
use std::sync::{Arc, Mutex};


use crate::app_state::*;

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct EmbeddedWebDist;

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
    payload: Option<axum::Json<EvictAudioPayload>>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(axum::Json(p)) = payload {
        if let Some(id) = p.id {
            if !id.is_empty() {
                crate::audio::supertonic::evict_audio(&id);
            }
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

async fn axum_music_stream_path_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let clean_filename = decode_url_component(&path);
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

pub fn serve_audio_bytes_with_range(
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

fn serve_static_asset(url: &str) -> axum::response::Response {
    use axum::response::IntoResponse;

    let clean_path = url.trim_start_matches('/');
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
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "vrm" => "model/gltf-binary",
        _ => "application/octet-stream",
    };

    // 1. Cek file fisik langsung di disk (Development / Live dist update)
    let disk_file = std::path::Path::new("web/dist").join(target_path);
    let disk_index = std::path::Path::new("web/dist/index.html");

    if disk_file.is_file() {
        if let Ok(bytes) = std::fs::read(&disk_file) {
            let mime = mime_for(target_path);
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    } else if disk_index.is_file() && !clean_path.starts_with("api/") && !clean_path.starts_with("assets/") {
        if let Ok(bytes) = std::fs::read(disk_index) {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    }

    // 2. Fallback jika tidak ada folder web/dist di disk (Binary standalone embedded)
    if let Some(file) = EmbeddedWebDist::get(target_path) {
        let mime_type = mime_for(target_path);
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("Content-Type", mime_type)
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .body(axum::body::Body::from(file.data))
            .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    } else if let Some(index) = EmbeddedWebDist::get("index.html") {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .body(axum::body::Body::from(index.data))
            .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    axum::response::Response::builder()
        .status(axum::http::StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(axum::body::Body::from("404 Not Found"))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn handle_axum_fallback(
    req: axum::extract::Request,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let method = req.method().as_str();
    let url = req.uri().path();

    // Fast-path untuk static web asset (HTML, JS, CSS, gambar, SPA):
    // Langsung disajikan dari memori/disk tanpa spawn_blocking dan VirtualHttpStream overhead
    if method == "GET" && !url.starts_with("/api") && !url.starts_with("/v1") {
        return serve_static_asset(url);
    }

    if url.starts_with("/api") || url.starts_with("/v1") {
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Endpoint not found" })),
        )
            .into_response();
    }

    if method == "GET" {
        serve_static_asset("index.html")
    } else {
        (
            axum::http::StatusCode::METHOD_NOT_ALLOWED,
            axum::Json(serde_json::json!({ "error": "Method not allowed" })),
        )
            .into_response()
    }
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    let rt = crate::app_state::get_global_tokio_runtime();
    let handle = rt.handle().clone();
    crate::app_state::set_tokio_handle(handle.clone());

    handle.spawn(async move {
        let addr_str = "127.0.0.1:42128";

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

            // Start background callback listener for OpenAI Codex OAuth (port 1455)
            crate::services::llm_providers::codex_service::ensure_port_1455_listener();

            use crate::services::openai_v1_service::{
                axum_v1_chat_completions_handler, axum_v1_models_handler, axum_v1_options_handler,
                axum_v1_root_handler,
            };

            let app = axum::Router::new()
                .route("/api/ping", axum::routing::get(|| async {
                    axum::Json(serde_json::json!({ "app": "ragasukma", "status": "running" }))
                }))
                .route("/api/ws", axum::routing::get(crate::services::events_service::axum_ws_handler))
                .route("/auth/callback", axum::routing::get(crate::services::llm_providers::codex_service::axum_codex_oauth_callback))
                .route("/callback", axum::routing::get(crate::services::llm_providers::codex_service::axum_codex_oauth_callback))
                .route("/api/tts/audio", axum::routing::get(axum_tts_audio_handler))
                .route("/api/tts/evict", axum::routing::post(axum_tts_evict_handler))
                .route("/api/tts/ack", axum::routing::post(axum_tts_evict_handler))
                .route("/api/music/stream", axum::routing::get(axum_music_stream_handler))
                .route("/music/*path", axum::routing::get(axum_music_stream_path_handler))
                // Native OpenAI-compatible routes (Real-time SSE Streaming & Low-latency Routing)
                .route(
                    "/v1/chat/completions",
                    axum::routing::post(axum_v1_chat_completions_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/api/v1/chat/completions",
                    axum::routing::post(axum_v1_chat_completions_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/v1/completions",
                    axum::routing::post(axum_v1_chat_completions_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/api/v1/completions",
                    axum::routing::post(axum_v1_chat_completions_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/v1/models",
                    axum::routing::get(axum_v1_models_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/api/v1/models",
                    axum::routing::get(axum_v1_models_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/v1",
                    axum::routing::get(axum_v1_root_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/v1/",
                    axum::routing::get(axum_v1_root_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/api/v1",
                    axum::routing::get(axum_v1_root_handler).options(axum_v1_options_handler),
                )
                .route(
                    "/api/v1/",
                    axum::routing::get(axum_v1_root_handler).options(axum_v1_options_handler),
                );

            let app = crate::services::api_router::register_api_routes(app)
                .layer(cors)
                .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))
                .fallback(handle_axum_fallback)
                .with_state(shared);

            if let Err(e) = axum::serve(tokio_listener, app).await {
                eprintln!("[Web Server] Axum server error: {}", e);
            }
    });
}
