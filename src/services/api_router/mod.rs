use std::sync::{Arc, Mutex};
use axum::Router;

use crate::app_state::ServerSharedState;

pub mod automation;
pub mod llm;
pub mod media;
pub mod system;

pub use automation::*;
pub use llm::*;
pub use media::*;
pub use system::*;

pub fn register_api_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    let router = system::register_system_routes(router);
    let router = automation::register_automation_routes(router);
    let router = media::register_media_routes(router);
    llm::register_llm_routes(router)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::extract::{Query, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::Json;

    #[tokio::test]
    async fn test_axum_accounts_endpoints() {
        let res_ag = axum_ag_accounts().await.into_response();
        assert_eq!(res_ag.status(), StatusCode::OK);
        let bytes_ag = to_bytes(res_ag.into_body(), 1024 * 1024).await.unwrap();
        let val_ag: serde_json::Value = serde_json::from_slice(&bytes_ag).unwrap();
        assert!(val_ag.is_array());

        let res_cx = axum_cx_accounts().await.into_response();
        assert_eq!(res_cx.status(), StatusCode::OK);
        let bytes_cx = to_bytes(res_cx.into_body(), 1024 * 1024).await.unwrap();
        let val_cx: serde_json::Value = serde_json::from_slice(&bytes_cx).unwrap();
        assert!(val_cx.is_array());
    }

    #[tokio::test]
    async fn test_axum_models_endpoints() {
        let res_ag = axum_ag_models().await.into_response();
        assert_eq!(res_ag.status(), StatusCode::OK);
        let bytes_ag = to_bytes(res_ag.into_body(), 1024 * 1024).await.unwrap();
        let val_ag: serde_json::Value = serde_json::from_slice(&bytes_ag).unwrap();
        assert!(val_ag.get("models").is_some());

        let res_cx = axum_cx_models().await.into_response();
        assert_eq!(res_cx.status(), StatusCode::OK);
        let bytes_cx = to_bytes(res_cx.into_body(), 1024 * 1024).await.unwrap();
        let val_cx: serde_json::Value = serde_json::from_slice(&bytes_cx).unwrap();
        assert!(val_cx.is_array());
    }

    #[tokio::test]
    async fn test_axum_get_config() {
        let res = axum_get_config().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.get("llm_model").is_some());
    }

    #[tokio::test]
    async fn test_axum_get_model_status() {
        let res = axum_get_model_status().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.get("status").is_some());
    }

    #[tokio::test]
    async fn test_axum_get_skills() {
        let res = axum_get_skills().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.is_array());
    }

    #[tokio::test]
    async fn test_axum_get_recipes() {
        let res = axum_get_recipes().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.is_array());
    }

    #[tokio::test]
    async fn test_axum_get_actions_since() {
        let res = axum_get_actions_since(Query(LastIdQuery { last_id: Some(0) })).await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.get("actions").is_some());
        assert!(val.get("last_id").is_some());
    }

    #[tokio::test]
    async fn test_axum_menu_cmd_toggle_pause() {
        let shared = Arc::new(Mutex::new(ServerSharedState::default()));
        let res = axum_menu_cmd(
            State(Arc::clone(&shared)),
            Json(CmdPayload { cmd: "toggle_pause".to_string() }),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(shared.lock().unwrap().is_paused);
    }

    #[tokio::test]
    async fn test_axum_remote_active() {
        let res = axum_remote_active().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(val.get("success").and_then(|v| v.as_bool()), Some(true));
    }

    #[tokio::test]
    async fn test_axum_get_browser_command_next() {
        let res = axum_get_browser_command_next().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(val.get("has_command").is_some());
    }

    #[tokio::test]
    async fn test_axum_tts_state_ended() {
        let shared = Arc::new(Mutex::new(ServerSharedState::default()));
        let res = axum_tts_state(
            State(Arc::clone(&shared)),
            Json(TtsStatePayload {
                event: Some("ended".to_string()),
                ..Default::default()
            }),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_axum_tts_state_mobile_speaking() {
        let shared = Arc::new(Mutex::new(ServerSharedState::default()));
        let res = axum_tts_state(
            State(Arc::clone(&shared)),
            Json(TtsStatePayload {
                speaking: Some(true),
                state: Some("speaking".to_string()),
                emotion: Some("happy".to_string()),
                ..Default::default()
            }),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(shared.lock().unwrap().current_emotion, "happy");
    }

    #[tokio::test]
    async fn test_axum_ag_quota_get_missing_id() {
        let res = axum_ag_quota_get(Query(QuotaQuery { id: None })).await.into_response();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_axum_cx_quota_get_missing_id() {
        let res = axum_cx_quota_get(Query(QuotaQuery { id: None })).await.into_response();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
