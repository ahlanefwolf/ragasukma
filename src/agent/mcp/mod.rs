// Agent MCP (Model Context Protocol) integration module
// Fully language-agnostic implementation supporting stdio and HTTP/SSE JSON-RPC 2.0 transports.

pub mod types {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpToolDefinition {
        pub name: String,
        pub description: Option<String>,
        #[serde(rename = "inputSchema", default = "default_schema")]
        pub input_schema: serde_json::Value,
    }

    fn default_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpCallToolRequest {
        pub name: String,
        #[serde(default)]
        pub arguments: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpToolResult {
        pub is_error: bool,
        pub content: Vec<serde_json::Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpServerConfig {
        pub id: String,
        pub name: String,
        pub transport: String, // "stdio", "sse", "http"
        pub command: Option<String>,
        #[serde(default)]
        pub args: Vec<String>,
        #[serde(default)]
        pub env: HashMap<String, String>,
        pub url: Option<String>,
        pub is_enabled: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpTestConnectionResult {
        pub success: bool,
        pub server_name: String,
        pub transport: String,
        pub tools_found: usize,
        pub tools: Vec<String>,
        pub error: Option<String>,
        pub latency_ms: u64,
    }
}

use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub use types::*;

/// Execute JSON-RPC 2.0 request over stdio transport
async fn call_stdio_rpc(
    command_str: &str,
    args: &[String],
    env: &HashMap<String, String>,
    request_json: &serde_json::Value,
    timeout_duration: Duration,
) -> Result<serde_json::Value, String> {
    let payload = serde_json::to_string(request_json).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(command_str);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn process '{}': {}", command_str, e))?;

    let mut stdin = child.stdin.take().ok_or_else(|| "Failed to capture stdin of MCP process".to_string())?;
    let stdout = child.stdout.take().ok_or_else(|| "Failed to capture stdout of MCP process".to_string())?;

    let line_writer = tokio::spawn(async move {
        stdin.write_all(payload.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    });

    let mut reader = BufReader::new(stdout).lines();

    let rpc_result = tokio::select! {
        res = line_writer => {
            if let Err(e) = res {
                let _ = child.kill().await;
                return Err(format!("Failed writing to stdin: {:?}", e));
            }
            // Read response line
            tokio::select! {
                line = reader.next_line() => {
                    match line {
                        Ok(Some(line_str)) => {
                            serde_json::from_str::<serde_json::Value>(&line_str)
                                .map_err(|e| format!("Invalid JSON received from MCP process: {} (raw: {})", e, line_str))
                        }
                        Ok(None) => Err("MCP process closed stdout without response".to_string()),
                        Err(e) => Err(format!("Error reading from MCP stdout: {}", e)),
                    }
                }
                _ = tokio::time::sleep(timeout_duration) => {
                    let _ = child.kill().await;
                    Err("Timed out waiting for MCP process response".to_string())
                }
            }
        }
        _ = tokio::time::sleep(timeout_duration) => {
            let _ = child.kill().await;
            Err("Timed out spawning or writing to MCP process".to_string())
        }
    };

    let _ = child.kill().await;
    rpc_result
}

/// Execute JSON-RPC 2.0 request over HTTP/SSE endpoint
async fn call_http_rpc(
    url: &str,
    request_json: &serde_json::Value,
    timeout_duration: Duration,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(timeout_duration)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(request_json)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed to {}: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("MCP HTTP server responded with status: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse HTTP JSON response: {}", e))?;

    Ok(body)
}

/// Generic MCP client instance that handles tool listing and execution
pub struct McpClient {
    pub config: McpServerConfig,
    pub timeout: Duration,
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            timeout: Duration::from_secs(15),
        }
    }

    /// Run initialization handshake
    pub async fn initialize(&self) -> Result<serde_json::Value, String> {
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "ragasukma-agent",
                    "version": "1.0.0"
                }
            }
        });

        self.send_rpc(&init_req).await
    }

    /// List available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>, String> {
        let list_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let resp = self.send_rpc(&list_req).await?;

        if let Some(err) = resp.get("error") {
            return Err(format!("MCP tools/list error: {}", err));
        }

        let tools_val = resp
            .get("result")
            .and_then(|r| r.get("tools"))
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));

        let tools: Vec<McpToolDefinition> = serde_json::from_value(tools_val)
            .map_err(|e| format!("Failed to parse MCP tools definition: {}", e))?;

        Ok(tools)
    }

    /// Call a specific tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<McpToolResult, String> {
        let call_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let resp = self.send_rpc(&call_req).await?;

        if let Some(err) = resp.get("error") {
            return Ok(McpToolResult {
                is_error: true,
                content: vec![err.clone()],
            });
        }

        let result_val = resp.get("result").unwrap_or(&serde_json::Value::Null);
        let is_error = result_val.get("isError").and_then(|b| b.as_bool()).unwrap_or(false);
        let content = result_val
            .get("content")
            .and_then(|c| c.as_array())
            .cloned()
            .unwrap_or_else(|| vec![result_val.clone()]);

        Ok(McpToolResult { is_error, content })
    }

    /// Dispatch JSON-RPC call based on configured transport
    async fn send_rpc(&self, request: &serde_json::Value) -> Result<serde_json::Value, String> {
        let transport = self.config.transport.to_lowercase();
        match transport.as_str() {
            "stdio" => {
                let cmd = self.config.command.as_deref().ok_or_else(|| {
                    "MCP stdio transport requires a 'command' to execute".to_string()
                })?;
                call_stdio_rpc(cmd, &self.config.args, &self.config.env, request, self.timeout).await
            }
            "sse" | "http" => {
                let url = self.config.url.as_deref().ok_or_else(|| {
                    "MCP HTTP/SSE transport requires a valid 'url'".to_string()
                })?;
                call_http_rpc(url, request, self.timeout).await
            }
            other => Err(format!("Unsupported MCP transport '{}'. Supported: stdio, sse, http", other)),
        }
    }

    /// Test server connectivity and retrieve available tools
    pub async fn test_connection(&self) -> McpTestConnectionResult {
        let start = Instant::now();
        let name = self.config.name.clone();
        let transport = self.config.transport.clone();

        match self.list_tools().await {
            Ok(tools) => {
                let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
                McpTestConnectionResult {
                    success: true,
                    server_name: name,
                    transport,
                    tools_found: tools.len(),
                    tools: tool_names,
                    error: None,
                    latency_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(e) => {
                // If tools/list failed, check if initialize succeeds
                match self.initialize().await {
                    Ok(_) => McpTestConnectionResult {
                        success: true,
                        server_name: name,
                        transport,
                        tools_found: 0,
                        tools: vec![],
                        error: Some(format!("Handshake ok, but tools/list returned: {}", e)),
                        latency_ms: start.elapsed().as_millis() as u64,
                    },
                    Err(init_err) => McpTestConnectionResult {
                        success: false,
                        server_name: name,
                        transport,
                        tools_found: 0,
                        tools: vec![],
                        error: Some(format!("Connection failed: {}", init_err)),
                        latency_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }
        }
    }
}
