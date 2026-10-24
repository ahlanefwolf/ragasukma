use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;

use super::models::AgentSessionEvent;

static SESSION_BROADCASTERS: OnceLock<Mutex<HashMap<String, broadcast::Sender<AgentSessionEvent>>>> =
    OnceLock::new();

fn get_broadcasters_map() -> &'static Mutex<HashMap<String, broadcast::Sender<AgentSessionEvent>>> {
    SESSION_BROADCASTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_session_sender(session_id: &str) -> broadcast::Sender<AgentSessionEvent> {
    let mut map = get_broadcasters_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(sender) = map.get(session_id) {
        sender.clone()
    } else {
        // High capacity channel to prevent Lagged errors during rapid streaming / large tool calls
        let (sender, _) = broadcast::channel(2048);
        map.insert(session_id.to_string(), sender.clone());
        sender
    }
}

pub fn broadcast_session_event(session_id: &str, event: AgentSessionEvent) {
    let sender = get_session_sender(session_id);
    let _ = sender.send(event);
}
