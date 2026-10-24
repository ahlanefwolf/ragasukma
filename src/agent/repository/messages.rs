use anyhow::Result;
use crate::agent::models::Message;
use super::{generate_id, now_secs};

// ============================================================================
// MESSAGES REPOSITORY
// ============================================================================

pub async fn list_messages(session_id: &str) -> Result<Vec<Message>> {
    let pool = crate::llm::usage::get_pool().await;
    let messages = sqlx::query_as::<_, Message>(
        "SELECT id, session_id, sequence_num, role, content, thought, tool_calls, tool_call_id, \
         status, token_usage_input, token_usage_output, created_at \
         FROM messages WHERE session_id = ?1 ORDER BY sequence_num ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(messages)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_message(
    session_id: &str,
    role: &str,
    content: Option<&str>,
    thought: Option<&str>,
    tool_calls: Option<&str>,
    tool_call_id: Option<&str>,
    status: &str,
    token_usage_input: Option<i64>,
    token_usage_output: Option<i64>,
) -> Result<Message> {
    let pool = crate::llm::usage::get_pool().await;
    let id = generate_id("msg");
    let now = now_secs();

    // Compute next sequence number atomically
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(sequence_num), 0) + 1 FROM messages WHERE session_id = ?1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    let sequence_num = row.0;

    sqlx::query(
        "INSERT INTO messages (id, session_id, sequence_num, role, content, thought, tool_calls, tool_call_id, \
         status, token_usage_input, token_usage_output, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(sequence_num)
    .bind(role)
    .bind(content)
    .bind(thought)
    .bind(tool_calls)
    .bind(tool_call_id)
    .bind(status)
    .bind(token_usage_input)
    .bind(token_usage_output)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Message {
        id,
        session_id: session_id.to_string(),
        sequence_num,
        role: role.to_string(),
        content: content.map(|s| s.to_string()),
        thought: thought.map(|s| s.to_string()),
        tool_calls: tool_calls.map(|s| s.to_string()),
        tool_call_id: tool_call_id.map(|s| s.to_string()),
        status: status.to_string(),
        token_usage_input,
        token_usage_output,
        created_at: now,
    })
}
