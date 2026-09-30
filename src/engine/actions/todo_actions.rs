/// Eksekusi perintah To-Do berbasis canonical action
pub fn execute_todo_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "add" => {
            let task = payload.trim();
            if task.is_empty() {
                return None;
            }
            match crate::engine::todo::add_task(task) {
                Ok(msg) => Some(("Note Task".to_string(), msg)),
                Err(e) => Some(("Note Task".to_string(), format!("Error: {e}"))),
            }
        }

        "done" => {
            let query = payload.trim();
            if query.is_empty() {
                return None;
            }
            match crate::engine::todo::complete_task(query) {
                Ok(msg) => Some(("Task Done".to_string(), msg)),
                Err(e) => Some(("Task Done".to_string(), format!("Error: {e}"))),
            }
        }

        _ => None,
    }
}
