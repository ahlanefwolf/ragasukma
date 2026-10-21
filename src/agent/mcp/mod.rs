// Agent MCP (Model Context Protocol) integration module

pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpToolDefinition {
        pub name: String,
        pub description: Option<String>,
        pub input_schema: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpCallToolRequest {
        pub name: String,
        pub arguments: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpToolResult {
        pub is_error: bool,
        pub content: Vec<serde_json::Value>,
    }
}
