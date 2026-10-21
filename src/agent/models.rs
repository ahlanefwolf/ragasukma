use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PromptSkill {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub system_prompt: String,
    pub tags: String, // Stored as JSON string in SQLite
    pub is_active: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: String, // Stored as JSON string
    pub env: String,  // Stored as JSON string
    pub url: Option<String>,
    pub is_enabled: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub root_path: Option<String>,
    pub custom_instructions: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectPromptSkill {
    pub project_id: String,
    pub skill_id: String,
    pub priority: i64,
    pub is_enabled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectMcpServer {
    pub project_id: String,
    pub server_id: String,
    pub is_enabled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub source_chat_id: Option<String>,
    pub metadata: String, // JSON
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub sequence_num: i64,
    pub role: String,
    pub content: Option<String>,
    pub thought: Option<String>,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
    pub status: String,
    pub token_usage_input: Option<i64>,
    pub token_usage_output: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentSessionEvent {
    #[serde(rename = "status")]
    StatusChange { status: String },
    #[serde(rename = "message")]
    MessageCreated(Message),
    #[serde(rename = "step")]
    StepInfo {
        iteration: usize,
        action: String,
        details: Option<String>,
    },
    #[serde(rename = "done")]
    Done { reply: String },
    #[serde(rename = "error")]
    Error { message: String },
}
