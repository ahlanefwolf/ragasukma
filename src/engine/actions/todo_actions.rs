/// Eksekusi perintah To-Do berbasis canonical action
pub fn execute_todo_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "add" => {
            let task = payload.trim();
            if task.is_empty() {
                return None;
            }
            match crate::engine::todo::add_task(task) {
                Ok(msg) => Some(("Catat Tugas".to_string(), msg)),
                Err(e) => Some(("Catat Tugas".to_string(), format!("Error: {e}"))),
            }
        }

        "done" => {
            let query = payload.trim();
            if query.is_empty() {
                return None;
            }
            match crate::engine::todo::complete_task(query) {
                Ok(msg) => Some(("Selesai Tugas".to_string(), msg)),
                Err(e) => Some(("Selesai Tugas".to_string(), format!("Error: {e}"))),
            }
        }

        _ => None,
    }
}
