use super::diagnostics::is_command_output_error;
use super::models::Message;
use super::tools::truncate_smart;
use crate::llm::v1_engine::V1ToolCall;

/// Estimates token count from a message for budget calculations (approx 4 chars per token)
pub fn estimate_message_tokens(msg: &Message) -> usize {
    let mut total = 0;
    if let Some(ref c) = msg.content {
        total += c.len();
    }
    if let Some(ref tc) = msg.tool_calls {
        total += tc.len();
    }
    (total / 4).max(1)
}

/// Generates a rolling hierarchical semantic summary of older messages evicted from the active sliding window,
/// preserving user directives, file mutations, actions, and decisions without memory amnesia across long multi-turn sessions.
pub fn summarize_evicted_messages(evicted: &[&Message]) -> String {
    let mut user_goals = Vec::new();
    let mut modified_files = Vec::new();
    let mut executed_cmds = Vec::new();
    let mut key_read_files = Vec::new();
    let mut decisions = Vec::new();

    for msg in evicted {
        if msg.role == "user" {
            if let Some(ref c) = msg.content {
                let trimmed = c.trim();
                // Filter out synthetic or tool-injected eviction summaries
                if !trimmed.is_empty() && !trimmed.starts_with("### 🧠") && !trimmed.starts_with("VERIFICATION") {
                    let first_line = trimmed.lines().next().unwrap_or(trimmed);
                    let short: String = first_line.chars().take(120).collect();
                    if !user_goals.contains(&short) {
                        user_goals.push(short);
                    }
                }
            }
        } else if msg.role == "assistant" {
            if let Some(ref thought) = msg.thought {
                let trimmed = thought.trim();
                if !trimmed.is_empty() {
                    let first_line = trimmed.lines().next().unwrap_or(trimmed);
                    let short: String = first_line.chars().take(140).collect();
                    if !decisions.contains(&short) && decisions.len() < 4 {
                        decisions.push(short);
                    }
                }
            }
            if let Some(ref tc_raw) = msg.tool_calls {
                if let Ok(calls) = serde_json::from_str::<Vec<V1ToolCall>>(tc_raw) {
                    for call in calls {
                        if let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.function.arguments) {
                            if call.function.name == "workspace_edit_file" || call.function.name == "workspace_write_file" {
                                if let Some(p) = args.get("file_path").or_else(|| args.get("path")).and_then(|v| v.as_str()) {
                                    if !modified_files.contains(&p.to_string()) {
                                        modified_files.push(p.to_string());
                                    }
                                }
                            } else if call.function.name == "workspace_run_command" {
                                if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                                    let clean_cmd = cmd.trim().to_string();
                                    if !executed_cmds.contains(&clean_cmd) {
                                        executed_cmds.push(clean_cmd);
                                    }
                                }
                            } else if call.function.name == "workspace_read_file" {
                                if let Some(p) = args.get("file_path").or_else(|| args.get("path")).and_then(|v| v.as_str()) {
                                    if !key_read_files.contains(&p.to_string()) {
                                        key_read_files.push(p.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut summary = String::new();
    summary.push_str("### 🧠 Hierarchical Memory Distillation (Earlier Evicted Turns):\n");
    if !user_goals.is_empty() {
        summary.push_str(&format!("- **User Directives**: {}\n", user_goals.join(" | ")));
    }
    if !modified_files.is_empty() {
        summary.push_str(&format!("- **Files Modified**: {}\n", modified_files.join(", ")));
    }
    if !executed_cmds.is_empty() {
        summary.push_str(&format!("- **Key Commands**: {}\n", executed_cmds.iter().take(4).cloned().collect::<Vec<_>>().join(" && ")));
    }
    if !key_read_files.is_empty() {
        summary.push_str(&format!("- **Inspected Files**: {}\n", key_read_files.iter().take(5).cloned().collect::<Vec<_>>().join(", ")));
    }
    if !decisions.is_empty() {
        summary.push_str(&format!("- **Prior Reasoning & Decisions**: {}\n", decisions.join(" | ")));
    }
    summary.push_str("- Retain this context and build directly upon it without repeating earlier operations.\n\n");
    summary
}

/// Sanitizes tool output for persistent storage in database.
/// If tool output is extremely large (e.g. read_file > 32,000 chars), we truncate the middle
/// to keep SQLite and frontend state lightweight, while keeping live buffer intact in memory.
pub fn sanitize_tool_output_for_db(content: &str) -> String {
    if content.len() > 32_000 {
        let prefix: String = content.chars().take(16_000).collect();
        let suffix: String = content.chars().rev().take(8_000).collect::<Vec<_>>().into_iter().rev().collect();
        format!(
            "{}\n\n[... Output truncated for database efficiency ({} total bytes) ...]\n\n{}",
            prefix,
            content.len(),
            suffix
        )
    } else {
        content.to_string()
    }
}

pub fn sanitize_tool_output_for_memory(content: &str) -> String {
    if content.len() > 32_000 {
        let prefix: String = content.chars().take(16_000).collect();
        let suffix: String = content
            .chars()
            .rev()
            .take(8_000)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!(
            "{}\n\n[... Large tool output truncated for in-memory context budget ({} total bytes) ...]\n\n{}",
            prefix,
            content.len(),
            suffix
        )
    } else {
        content.to_string()
    }
}

/// Prunes tool output results to optimize context window, token cost, and LLM inference latency.
/// Preserves the live tail intact while eliding bulky older outputs.
/// Retains compiler error diagnostics up to 12,000 chars while heavily compacting successful command runs
/// and replacing historical file reads with lightweight DB cache references.
pub fn compact_tool_output(content: &str, is_recent_tail: bool) -> String {
    compact_tool_output_smart(content, None, is_recent_tail)
}

/// Context-aware tool output compactor that differentiates by tool type.
pub fn compact_tool_output_smart(content: &str, tool_name: Option<&str>, is_recent_tail: bool) -> String {
    // 1. Preserve compiler/test error diagnostics up to 12,000 chars so actionable error messages are never lost
    if is_command_output_error(content) {
        if content.len() > 12_000 {
            return truncate_smart(content, 4_000, 8_000);
        }
        return content.to_string();
    }

    match tool_name {
        Some("workspace_run_command") => {
            // For successful commands (exit 0, tests passing, builds, etc.):
            if is_recent_tail {
                // Keep head + tail up to 1,500 chars (500 head + 1,000 tail)
                truncate_smart(content, 500, 1_000)
            } else {
                // Older successful command runs: condense to 1-2 summary lines
                let first_line = content.lines().next().unwrap_or("").trim();
                let last_line = content.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
                if content.len() > 300 {
                    format!(
                        "{}\n[... {} characters of successful command output omitted; exit code: 0 ...]\n{}",
                        first_line,
                        content.len(),
                        last_line
                    )
                } else {
                    content.to_string()
                }
            }
        }
        Some("workspace_read_file") => {
            if is_recent_tail {
                // In active recent turns: keep up to 8,000 chars for live edits
                truncate_smart(content, 3_000, 5_000)
            } else {
                // In older historical turns: file is already indexed in SQLite evidence cache
                let first_line = content.lines().next().unwrap_or("").trim();
                if content.len() > 400 {
                    format!(
                        "{}\n[... {} characters of file content indexed in database evidence cache; consult active evidence or re-read slice if needed ...]",
                        first_line,
                        content.len()
                    )
                } else {
                    content.to_string()
                }
            }
        }
        Some("workspace_list_dir") => {
            if is_recent_tail {
                truncate_smart(content, 1_000, 1_500)
            } else if content.len() > 300 {
                let first_line = content.lines().next().unwrap_or("").trim();
                format!(
                    "{}\n[... {} characters of directory listing omitted ...]",
                    first_line,
                    content.len()
                )
            } else {
                content.to_string()
            }
        }
        Some("workspace_grep") => {
            if is_recent_tail {
                truncate_smart(content, 1_500, 2_500)
            } else if content.len() > 400 {
                let first_line = content.lines().next().unwrap_or("").trim();
                format!(
                    "{}\n[... {} characters of grep search results omitted ...]",
                    first_line,
                    content.len()
                )
            } else {
                content.to_string()
            }
        }
        _ => {
            // General fallback
            if is_recent_tail {
                truncate_smart(content, 3_000, 5_000)
            } else if content.len() > 500 {
                let first_line = content.lines().next().unwrap_or("").trim();
                format!(
                    "{}\n[... {} characters of historical tool output compacted to preserve token budget ...]",
                    first_line,
                    content.len()
                )
            } else {
                content.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_tool_output_for_memory_capping() {
        let small = "{\"status\": \"ok\", \"count\": 5}";
        assert_eq!(sanitize_tool_output_for_memory(small), small);

        let huge = "X".repeat(100_000);
        let sanitized = sanitize_tool_output_for_memory(&huge);
        assert!(sanitized.len() < 35_000);
        assert!(sanitized.contains("Large tool output truncated for in-memory context budget"));
    }

    #[test]
    fn test_compact_tool_output_caps_large_error_diagnostics() {
        let huge_error = format!("❌ COMMAND FAILED with exit code 1\n{}", "error: type mismatch\n".repeat(2_000));
        assert!(huge_error.len() > 30_000);
        let compacted = compact_tool_output(&huge_error, true);
        assert!(compacted.len() <= 15_000);
    }

    #[test]
    fn test_compact_successful_command_output() {
        let successful_build = format!("Compiling crates...\n{}Finished release [optimized] target(s) in 12.3s", "   Compiling dep v0.1\n".repeat(200));
        assert!(successful_build.len() > 4_000);

        // Recent tail should cap to <= 1,600 chars
        let tail_compacted = compact_tool_output_smart(&successful_build, Some("workspace_run_command"), true);
        assert!(tail_compacted.len() <= 1_600);

        // Historical turn should distill to minimal summary
        let hist_compacted = compact_tool_output_smart(&successful_build, Some("workspace_run_command"), false);
        assert!(hist_compacted.len() < 400);
        assert!(hist_compacted.contains("successful command output omitted"));
    }

    #[test]
    fn test_compact_historical_file_reads() {
        let file_content = format!("// Header\n{}", "fn do_something() { println!(\"hello\"); }\n".repeat(100));
        assert!(file_content.len() > 3_000);

        let hist_compacted = compact_tool_output_smart(&file_content, Some("workspace_read_file"), false);
        assert!(hist_compacted.len() < 300);
        assert!(hist_compacted.contains("database evidence cache"));
    }
}
