use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TodoItem {
    pub id: i64,
    pub task: String,
    pub is_done: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

static TODO_CACHE: Mutex<Option<Vec<TodoItem>>> = Mutex::new(None);
static TODO_MUTEX: Mutex<()> = Mutex::new(());

fn get_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

use crate::app_state::spawn_db_task;

pub fn parse_todo_items_from_markdown(content: &str) -> Vec<TodoItem> {
    let mut items = Vec::new();
    let mut now = get_now_secs();
    let mut id_counter = 1;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") {
            let task = trimmed.trim_start_matches("- [ ]").trim();
            if !task.is_empty() {
                items.push(TodoItem {
                    id: id_counter,
                    task: task.to_string(),
                    is_done: false,
                    created_at: now,
                    completed_at: None,
                });
                id_counter += 1;
                now += 1;
            }
        } else if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let task = trimmed
                .trim_start_matches("- [x]")
                .trim_start_matches("- [X]")
                .trim();
            if !task.is_empty() {
                items.push(TodoItem {
                    id: id_counter,
                    task: task.to_string(),
                    is_done: true,
                    created_at: now,
                    completed_at: Some(now),
                });
                id_counter += 1;
                now += 1;
            }
        }
    }
    items
}

pub fn format_todo_markdown(items: &[TodoItem]) -> String {
    let mut out = String::from("# 📋 Tasks & Notes\n\n## ⏳ In Progress / To-Do\n");
    for item in items.iter().filter(|i| !i.is_done) {
        out.push_str(&format!("- [ ] {}\n", item.task));
    }
    out.push_str("\n## ✅ Done\n");
    for item in items.iter().filter(|i| i.is_done) {
        out.push_str(&format!("- [x] {}\n", item.task));
    }
    out
}

fn ensure_cache_loaded() {
    let mut guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(Vec::new());

        // Schedule async sync from SQLite DB
        spawn_db_task(async {
            let _ = sync_todos_from_db().await;
        });
    }
}

pub async fn sync_todos_from_db() -> Vec<TodoItem> {
    let pool = crate::db::get_pool().await;
    let rows = sqlx::query_as::<_, TodoItem>(
        "SELECT id, task, is_done, created_at, completed_at FROM todos ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(rows.clone());
    rows
}

pub fn get_active_todos() -> Vec<TodoItem> {
    ensure_cache_loaded();
    let guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .map(|list| list.iter().filter(|i| !i.is_done).cloned().collect())
        .unwrap_or_default()
}

pub fn get_all_todos() -> Vec<TodoItem> {
    ensure_cache_loaded();
    let guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().cloned().unwrap_or_default()
}

pub fn format_todo_summary() -> String {
    let active = get_active_todos();
    if active.is_empty() {
        return String::new();
    }

    let mut out = String::from("\nUSER TO-DO / NOTES:\n");
    for (i, t) in active.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, t.task));
    }
    out.push('\n');
    out
}

pub fn add_task(task_text: &str) -> std::io::Result<String> {
    let _sync_guard = TODO_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let task = task_text.trim();
    if task.is_empty() {
        return Ok("Task text must not be empty".to_string());
    }

    ensure_cache_loaded();
    let now = get_now_secs();

    {
        let mut guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let list = guard.get_or_insert_with(Vec::new);
        let next_id = list.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        let item = TodoItem {
            id: next_id,
            task: task.to_string(),
            is_done: false,
            created_at: now,
            completed_at: None,
        };
        list.push(item);
    }

    // Persist to SQLite
    let t_string = task.to_string();
    spawn_db_task(async move {
        let pool = crate::db::get_pool().await;
        let _ = sqlx::query(
            "INSERT INTO todos (task, is_done, created_at) VALUES (?1, 0, ?2)",
        )
        .bind(&t_string)
        .bind(now)
        .execute(pool)
        .await;
    });

    Ok(format!("Task '{}' successfully saved to to-do list", task))
}

pub fn complete_task(query: &str) -> std::io::Result<String> {
    let _sync_guard = TODO_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let q = query.trim();
    if q.is_empty() {
        return Ok("Task keyword must not be empty".to_string());
    }

    ensure_cache_loaded();
    let q_lower = q.to_lowercase();
    let now = get_now_secs();

    let mut found_id = None;
    let mut matched_title = String::new();

    {
        let mut guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let list = match guard.as_mut() {
            Some(l) => l,
            None => return Ok("To-do list has not been created yet".to_string()),
        };

        // 1. Exact or Substring Match
        for item in list.iter() {
            if !item.is_done && item.task.to_lowercase().contains(&q_lower) {
                found_id = Some(item.id);
                matched_title = item.task.clone();
                break;
            }
        }

        // 2. Fuzzy Similarity Fallback
        if found_id.is_none() {
            let mut best_sim = 0.0f32;
            for item in list.iter() {
                if !item.is_done {
                    let sim = crate::engine::perception::string_similarity(q, &item.task);
                    if sim >= 0.65 && sim > best_sim {
                        best_sim = sim;
                        found_id = Some(item.id);
                        matched_title = item.task.clone();
                    }
                }
            }
        }

        if let Some(target_id) = found_id {
            if let Some(item) = list.iter_mut().find(|i| i.id == target_id) {
                item.is_done = true;
                item.completed_at = Some(now);
            }
        }
    }

    if let Some(target_id) = found_id {
        let m_title = matched_title.clone();
        spawn_db_task(async move {
            let pool = crate::db::get_pool().await;
            let _ = sqlx::query(
                "UPDATE todos SET is_done = 1, completed_at = ?1 WHERE id = ?2 OR (task = ?3 AND is_done = 0)",
            )
            .bind(now)
            .bind(target_id)
            .bind(&m_title)
            .execute(pool)
            .await;
        });

        Ok(format!("Task '{}' successfully marked as done", matched_title))
    } else {
        Ok(format!("Task with keyword '{}' not found in active to-do list", q))
    }
}

pub fn delete_task(id: i64) -> bool {
    let _sync_guard = TODO_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    ensure_cache_loaded();

    let mut removed = false;
    {
        let mut guard = TODO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(list) = guard.as_mut() {
            let before = list.len();
            list.retain(|i| i.id != id);
            removed = list.len() < before;
        }
    }

    if removed {
        spawn_db_task(async move {
            let pool = crate::db::get_pool().await;
            let _ = sqlx::query("DELETE FROM todos WHERE id = ?1")
                .bind(id)
                .execute(pool)
                .await;
        });
    }
    removed
}

// -----------------------------------------------------------------------------
// Legacy String Parsers (for backward compatibility and pure string tests)
// -----------------------------------------------------------------------------
pub fn add_task_to_content(content: &str, task: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let target_item = format!("- [ ] {}", task);

    let mut insert_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let l = line.to_lowercase();
        if l.starts_with("## ")
            && (l.contains("in progress") || l.contains("to-do") || l.contains("todo"))
        {
            insert_idx = Some(i + 1);
            break;
        }
    }

    if let Some(idx) = insert_idx {
        lines.insert(idx, target_item);
    } else {
        lines.push(String::new());
        lines.push("##  In Progress / To-Do".to_string());
        lines.push(target_item);
    }

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content
}

pub fn complete_task_in_content(content: &str, query: &str) -> Option<(String, String)> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found_idx = None;
    let mut matched_title = String::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") {
            let task_text = trimmed.trim_start_matches("- [ ]").trim();
            if task_text.to_lowercase().contains(&q) {
                found_idx = Some(i);
                matched_title = task_text.to_string();
                break;
            }
        }
    }

    // Fallback: Fuzzy Similarity Matching for to-do items
    if found_idx.is_none() {
        let mut best_sim = 0.0f32;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ]") {
                let task_text = trimmed.trim_start_matches("- [ ]").trim();
                let sim = crate::engine::perception::string_similarity(query, task_text);
                if sim >= 0.65 && sim > best_sim {
                    best_sim = sim;
                    found_idx = Some(i);
                    matched_title = task_text.to_string();
                }
            }
        }
    }

    let found_idx = found_idx?;
    lines.remove(found_idx);

    let completed_item = format!("- [x] {}", matched_title);
    let mut done_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let l = line.to_lowercase();
        if l.starts_with("## ") && l.contains("done") {
            done_idx = Some(i + 1);
            break;
        }
    }

    if let Some(idx) = done_idx {
        lines.insert(idx, completed_item);
    } else {
        lines.push(String::new());
        lines.push("##  Done".to_string());
        lines.push(completed_item);
    }

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    Some((new_content, matched_title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_complete_todo() {
        let initial = "#  Tasks\n\n##  In Progress / To-Do\n- [ ] Buy milk\n\n##  Done\n";
        let updated = add_task_to_content(initial, "Undangan nikahan Yusuf");
        assert!(updated.contains("- [ ] Undangan nikahan Yusuf"));
        assert!(updated.contains("- [ ] Buy milk"));

        let (done, matched) =
            complete_task_in_content(&updated, "nikahan").expect("should find task");
        assert_eq!(matched, "Undangan nikahan Yusuf");
        assert!(!done.contains("- [ ] Undangan nikahan Yusuf"));
        assert!(done.contains("- [x] Undangan nikahan Yusuf"));
        assert!(done.contains("- [ ] Buy milk"));
    }

    #[test]
    fn test_complete_todo_fuzzy() {
        let initial =
            "#  Tasks\n\n##  In Progress / To-Do\n- [ ] Pay electricity bill\n\n##  Done\n";
        let (done, matched) = complete_task_in_content(initial, "electricity bill")
            .expect("should find task with fuzzy");
        assert_eq!(matched, "Pay electricity bill");
        assert!(done.contains("- [x] Pay electricity bill"));
    }

    #[test]
    fn test_todo_crud_and_formatting() {
        let _ = add_task("Beli kopi espresso");
        let active = get_active_todos();
        assert!(active.iter().any(|t| t.task == "Beli kopi espresso"));

        let summary = format_todo_summary();
        assert!(summary.contains("Beli kopi espresso"));

        let complete_res = complete_task("kopi");
        assert!(complete_res.is_ok());

        let target_id = active.iter().find(|t| t.task == "Beli kopi espresso").map(|t| t.id);
        if let Some(id) = target_id {
            delete_task(id);
        }
    }
}
