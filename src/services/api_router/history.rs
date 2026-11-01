use std::sync::{Arc, Mutex};
use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::ServerSharedState;

pub fn register_history_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        .route("/api/history/dates", get(axum_get_history_dates))
        .route("/api/history/view", get(axum_get_history_view))
        .route("/api/history/delete", post(axum_delete_history))
        .route("/api/chat/events", get(axum_chat_events_sse))
}

pub(crate) async fn axum_get_history_dates() -> impl IntoResponse {
    let dates = crate::repository::chat_history::get_history_dates().await;
    Json(dates)
}

#[derive(Deserialize)]
pub(crate) struct HistoryViewQuery {
    #[serde(default)]
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) page: Option<usize>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
    #[serde(default)]
    pub(crate) order: Option<String>,
}

pub(crate) async fn axum_get_history_view(Query(q): Query<HistoryViewQuery>) -> impl IntoResponse {
    let target_date = match q.date {
        Some(ref d) if !d.trim().is_empty() => d.trim().to_string(),
        _ => {
            let dates = crate::repository::chat_history::get_history_dates().await;
            dates.first().cloned().unwrap_or_default()
        }
    };

    if target_date.is_empty() {
        return Json(json!({
            "date": "",
            "turns": [],
            "total_turns": 0,
            "page": 1,
            "limit": 20,
            "total_pages": 1
        }))
        .into_response();
    }

    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(20);
    let paginated = crate::repository::chat_history::get_history_by_date_paginated(
        &target_date,
        page,
        limit,
        q.order.as_deref(),
    )
    .await;
    Json(paginated).into_response()
}

#[derive(Deserialize)]
pub(crate) struct DeleteHistoryPayload {
    pub(crate) date: String,
}

pub(crate) async fn axum_delete_history(
    Json(payload): Json<DeleteHistoryPayload>,
) -> impl IntoResponse {
    let clean_date = payload.date.trim();
    if clean_date.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing date parameter" })),
        )
            .into_response();
    }

    match crate::repository::chat_history::delete_history_by_date(clean_date).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "History record not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to delete history: {e}") })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_chat_events_sse() -> impl IntoResponse {
    let sender = crate::repository::chat_history::get_chat_event_sender();
    let mut rx = sender.subscribe();

    let stream = async_stream::stream! {
        let init_json = serde_json::json!({
            "type": "connected"
        });
        yield Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", init_json));

        // Persistent keep-alive ping interval (prevents reallocating timer futures on every token)
        let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(3));
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(item) => {
                            let evt = serde_json::json!({
                                "type": "chat_turn",
                                "item": item
                            });
                            if let Ok(json_str) = serde_json::to_string(&evt) {
                                yield Ok(format!("data: {}\n\n", json_str));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                            eprintln!("[Chat SSE] Recv lagged by {} messages", missed);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    // Send periodic keep-alive event every 3s to flush Cloudflare & proxy buffers
                    yield Ok("event: ping\ndata: {}\n\n".to_string());
                }
            }
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache, no-transform")
        .header("Connection", "keep-alive")
        .header("X-Accel-Buffering", "no")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from_stream(stream))
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create SSE stream",
            )
                .into_response()
        })
}
