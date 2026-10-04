use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpStream;

use crate::server::send_json_response;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatTurnItem {
    pub time: String,
    pub speaker: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryDay {
    pub date: String,
    pub turns: Vec<ChatTurnItem>,
}

fn parse_history_file(content: &str, date_str: &str) -> ChatHistoryDay {
    let mut turns = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        // Check for format: ### [00:01:48] Speaker
        if line.starts_with("### [") {
            if let Some(end_bracket) = line.find(']') {
                let time = line[5..end_bracket].trim().to_string();
                let speaker = line[end_bracket + 1..].trim().to_string();
                i += 1;
                let mut msg_lines = Vec::new();
                while i < lines.len() {
                    let next = lines[i];
                    if next.trim().starts_with("### [") || next.trim() == "---" {
                        break;
                    }
                    msg_lines.push(next);
                    i += 1;
                }
                let message = msg_lines.join("\n").trim().to_string();
                if !message.is_empty() {
                    turns.push(ChatTurnItem {
                        time,
                        speaker,
                        message,
                    });
                }
                continue;
            }
        }
        i += 1;
    }

    ChatHistoryDay {
        date: date_str.to_string(),
        turns,
    }
}

/// GET /api/history/dates
pub fn handle_get_history_dates(stream: &mut TcpStream) {
    let histories_dir = crate::assets::get_assets_dir().join("histories");
    let mut dates: Vec<String> = Vec::new();

    if histories_dir.exists() {
        if let Ok(entries) = fs::read_dir(&histories_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        dates.push(stem.to_string());
                    }
                }
            }
        }
    }

    dates.sort();
    dates.reverse(); // Most recent date first
    let json = serde_json::to_string(&dates).unwrap_or_else(|_| "[]".to_string());
    send_json_response(stream, 200, &json);
}

/// GET /api/history/view?date=YYYY-MM-DD
pub fn handle_get_history_by_date(full_url: &str, stream: &mut TcpStream) {
    let date_param = full_url.split('?').nth(1).and_then(|query| {
        query.split('&').find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next() == Some("date") {
                parts.next().map(|s| s.to_string())
            } else {
                None
            }
        })
    });

    let histories_dir = crate::assets::get_assets_dir().join("histories");
    let target_date = match date_param {
        Some(d) if !d.trim().is_empty() => d.trim().to_string(),
        _ => {
            // Pick most recent date
            let mut dates: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(&histories_dir) {
                for entry in entries.flatten() {
                    if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        dates.push(stem.to_string());
                    }
                }
            }
            dates.sort();
            dates.into_iter().last().unwrap_or_default()
        }
    };

    if target_date.is_empty() {
        send_json_response(stream, 200, "{\"date\":\"\",\"turns\":[]}");
        return;
    }

    // Sanitize target_date to avoid path traversal (only allow alphanumeric and hyphen)
    let sanitized_date: String = target_date.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect();
    let file_path = histories_dir.join(format!("{sanitized_date}.md"));

    if file_path.exists() {
        if let Ok(content) = fs::read_to_string(&file_path) {
            let parsed = parse_history_file(&content, &sanitized_date);
            let json = serde_json::to_string(&parsed).unwrap_or_else(|_| "{}".to_string());
            send_json_response(stream, 200, &json);
            return;
        }
    }

    send_json_response(stream, 200, &format!("{{\"date\":\"{sanitized_date}\",\"turns\":[]}}"));
}

/// POST /api/history/delete
pub fn handle_delete_history(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct DeleteHistoryPayload {
        date: String,
    }

    if let Ok(item) = serde_json::from_str::<DeleteHistoryPayload>(body_str) {
        let sanitized_date: String = item.date.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect();
        let file_path = crate::assets::get_assets_dir().join("histories").join(format!("{sanitized_date}.md"));
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
            crate::term_log!("[History Service] Deleted history file for {}", sanitized_date);
            send_json_response(stream, 200, "{\"success\":true}");
        } else {
            send_json_response(stream, 404, "{\"error\":\"History not found\"}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid delete history request\"}");
    }
}
