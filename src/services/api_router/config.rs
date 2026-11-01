use std::sync::{Arc, Mutex};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::app_state::ServerSharedState;

pub fn register_config_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router.route("/api/config", get(axum_get_config).post(axum_post_config))
}

pub(crate) async fn axum_get_config() -> impl IntoResponse {
    Json(crate::assets::load_config())
}

pub(crate) async fn axum_post_config(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(cfg): Json<crate::config::UserConfig>,
) -> impl IntoResponse {
    if let Err(e) = crate::assets::save_config(&cfg) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response()
    } else {
        if let Ok(mut lock) = shared.lock() {
            lock.config_updated = true;
        }
        Json(json!({ "success": true })).into_response()
    }
}
