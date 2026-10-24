pub mod dialogs;
pub mod editor;
pub mod fs;
pub mod search;
pub mod tags;

use anyhow::{bail, Context, Result};
use serde_json::json;
use std::fs as std_fs;
use std::path::Path;
use std::sync::OnceLock;

pub use dialogs::{open_path_in_file_manager, pick_folder_dialog};
pub use editor::apply_resilient_edit;
pub use fs::{
    create_workspace_command, ensure_git_and_gitignore, generate_smart_gitignore,
    resolve_allowed_path, to_workspace_folder_path,
};
pub use search::{list_workspace_files, run_workspace_grep, WorkspaceFileInfo};
pub use tags::expand_file_tags;

/// Smartly truncates output keeping both the head and tail, omitting the middle.
/// Perfectly Unicode/char-safe without mid-character slice panics.
pub fn truncate_smart(s: &str, max_head: usize, max_tail: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_head + max_tail {
        return s.to_string();
    }
    let head: String = s.chars().take(max_head).collect();
    let tail_skip = char_count.saturating_sub(max_tail);
    let tail: String = s.chars().skip(tail_skip).collect();
    let omitted = char_count - max_head - max_tail;
    format!(
        "{}\n\n... [{} characters omitted to preserve token budget] ...\n\n{}",
        head, omitted, tail
    )
}

/// Checks if a tool is strictly read-only and free of side effects.
pub fn is_read_only_tool(name: &str) -> bool {
    matches!(
        name,
        "workspace_read_file" | "workspace_list_dir" | "workspace_grep"
    )
}

/// Returns tool definitions for OpenAI / Antigravity / Codex V1 tool calling (cached)
pub fn get_agent_tools_schema() -> Vec<serde_json::Value> {
    static CACHED_SCHEMA: OnceLock<Vec<serde_json::Value>> = OnceLock::new();
    CACHED_SCHEMA
        .get_or_init(|| {
            vec![
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_list_dir",
                        "description": "List files and subdirectories in the specified workspace directory.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Relative or absolute path to the directory within the workspace"
                                }
                            },
                            "required": ["path"]
                        }
                    }
                }),
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_read_file",
                        "description": "Read text content of a file in the workspace. You can read the full file by omitting start_line and end_line, or specify lines around a target. AVOID reading files in repetitive 50-100 line chunks; read the target section directly in one call.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Path to the file to read"
                                },
                                "start_line": {
                                    "type": "integer",
                                    "description": "Start line number (1-indexed, optional)"
                                },
                                "end_line": {
                                    "type": "integer",
                                    "description": "End line number (1-indexed, optional)"
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                }),
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_write_file",
                        "description": "Write or create a file in the workspace. TIP: Prefer this tool over workspace_edit_file for small files (< 100 lines) or major rewrites to ensure 100% accuracy and speed.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Target file path to write"
                                },
                                "content": {
                                    "type": "string",
                                    "description": "Full text content to write to the file"
                                }
                            },
                            "required": ["file_path", "content"]
                        }
                    }
                }),
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_edit_file",
                        "description": "Replace a specific section or lines in an existing file. For small files (< 100 lines), prefer workspace_write_file directly. TIP: Keep 'old_text' short and unique (only 2-6 lines around the target change) so it matches instantly.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Path to the file to edit"
                                },
                                "old_text": {
                                    "type": "string",
                                    "description": "Exact text or lines to find and replace. Keep it minimal and concise (2 to 6 lines) for exact matching."
                                },
                                "new_text": {
                                    "type": "string",
                                    "description": "Replacement text"
                                }
                            },
                            "required": ["file_path", "old_text", "new_text"]
                        }
                    }
                }),
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_grep",
                        "description": "Search for string or regex across workspace files.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Search query or regex string"
                                },
                                "path": {
                                    "type": "string",
                                    "description": "Subdirectory to search (optional, defaults to first root folder)"
                                }
                            },
                            "required": ["query"]
                        }
                    }
                }),
                json!({
                    "type": "function",
                    "function": {
                        "name": "workspace_run_command",
                        "description": format!(
                            "Execute a terminal/shell command on host OS '{}' using {}. Has full access to user development tools (npm, npx, pnpm, yarn, node, cargo, python, pip, git, etc.). Specify 'cwd' to run in a specific project folder or directory. Commands support full shell syntax and chaining.",
                            std::env::consts::OS,
                            if cfg!(target_os = "windows") { "PowerShell" } else { "user login shell (bash/zsh)" }
                        ),
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "Full shell command to execute. Supports chaining multiple commands with '&&' (e.g. 'git add -A && git commit -m \"...\" && git push' or 'npm run build && npm test'). Always chain related sequential commands into a single call rather than running them across multiple turns."
                                },
                                "cwd": {
                                    "type": "string",
                                    "description": "Working directory relative to workspace or folder alias (optional, defaults to primary workspace root)"
                                }
                            },
                            "required": ["command"]
                        }
                    }
                }),
            ]
        })
        .clone()
}

/// Execute a tool call safely
/// Normalizes terminal commands by stripping redundant root folder prefixes in `cd` calls.
/// e.g. when `work_dir` is `/home/user/scenario`, `cd scenario/backend` is automatically normalized to `cd backend`.
pub fn normalize_workspace_command(command: &str, work_dir: &Path, allowed_roots: &[String]) -> String {
    let mut normalized = command.to_string();

    let mut root_folder_names = Vec::new();
    if let Some(name) = work_dir.file_name().and_then(|n| n.to_str()) {
        root_folder_names.push(name.to_string());
    }
    for root in allowed_roots {
        let p = Path::new(root);
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if !root_folder_names.contains(&name.to_string()) {
                root_folder_names.push(name.to_string());
            }
        }
    }

    for folder_name in root_folder_names {
        if !work_dir.join(&folder_name).exists() {
            let pat1 = format!("cd {}/", folder_name);
            let pat2 = format!("cd ./{}/", folder_name);
            normalized = normalized.replace(&pat2, "cd ");
            normalized = normalized.replace(&pat1, "cd ");

            let pat3 = format!("cd {}", folder_name);
            let pat4 = format!("cd ./{}", folder_name);
            for delim in &[" ", ";", "&&", ")", "\n"] {
                normalized = normalized.replace(&format!("{}{}", pat4, delim), &format!("cd .{}", delim));
                normalized = normalized.replace(&format!("{}{}", pat3, delim), &format!("cd .{}", delim));
            }
            if normalized.trim() == pat3 || normalized.trim() == pat4 {
                normalized = "cd .".to_string();
            }
        }
    }

    normalized
}

pub fn execute_tool_call(
    name: &str,
    args: &serde_json::Value,
    allowed_roots: &[String],
) -> Result<String> {
    match name {
        "workspace_list_dir" => {
            let path_str = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let resolved = resolve_allowed_path(path_str, allowed_roots)?;

            if !resolved.exists() {
                bail!("Directory '{}' not found", path_str);
            }
            if !resolved.is_dir() {
                bail!("'{}' is not a directory", path_str);
            }

            let mut builder = ignore::WalkBuilder::new(&resolved);
            builder
                .max_depth(Some(1))
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .parents(true);

            let mut overrides = ignore::overrides::OverrideBuilder::new(&resolved);
            let _ = overrides.add("!.git");
            let _ = overrides.add("!.git/**");
            let _ = overrides.add("!.DS_Store");
            let _ = overrides.add("!Thumbs.db");
            if let Ok(ov) = overrides.build() {
                builder.overrides(ov);
            }

            let mut entries = Vec::new();
            for entry in builder.build().flatten() {
                if entry.depth() == 0 {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".git" || name == ".DS_Store" || name == "Thumbs.db" {
                    continue;
                }
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                entries.push(format!("{}{}", name, if is_dir { "/" } else { "" }));
            }
            entries.sort();
            let display_path = to_workspace_folder_path(&resolved, allowed_roots);
            Ok(json!({ "path": display_path, "entries": entries }).to_string())
        }

        "workspace_read_file" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .context("Parameter 'file_path' is required")?;
            let resolved = resolve_allowed_path(file_path, allowed_roots)?;

            if !resolved.exists() {
                bail!("File '{}' not found", file_path);
            }
            if resolved.is_dir() {
                bail!("'{}' is a directory, not a file", file_path);
            }

            let content = std_fs::read_to_string(&resolved)
                .context(format!("Failed to read file '{}'", file_path))?;

            let parse_line = |key: &str| -> Option<usize> {
                args.get(key).and_then(|v| {
                    v.as_u64()
                        .map(|n| n as usize)
                        .or_else(|| v.as_i64().filter(|&n| n > 0).map(|n| n as usize))
                        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<usize>().ok()))
                })
            };
            let start = parse_line("start_line");
            let end = parse_line("end_line");

            if start.is_some() || end.is_some() {
                let lines: Vec<&str> = content.lines().collect();
                let start_idx = start.unwrap_or(1).saturating_sub(1);
                let end_idx = end.unwrap_or(lines.len()).min(lines.len());
                let sliced = if start_idx < end_idx {
                    lines[start_idx..end_idx].join("\n")
                } else {
                    String::new()
                };
                Ok(sliced)
            } else {
                Ok(content)
            }
        }

        "workspace_write_file" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .context("Parameter 'file_path' is required")?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .context("Parameter 'content' is required")?;
            let resolved = resolve_allowed_path(file_path, allowed_roots)?;

            let old_content = std_fs::read_to_string(&resolved).ok();
            let is_new_file = old_content.is_none();

            if let Some(parent) = resolved.parent() {
                let _ = std_fs::create_dir_all(parent);
            }

            std_fs::write(&resolved, content)?;

            let display_path = to_workspace_folder_path(&resolved, allowed_roots);
            let result_json = serde_json::json!({
                "status": "ok",
                "path": display_path,
                "file_path": file_path,
                "bytes_written": content.len(),
                "is_new_file": is_new_file,
                "old_content": old_content,
                "new_content": content,
            });

            Ok(result_json.to_string())
        }

        "workspace_edit_file" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .context("Parameter 'file_path' is required")?;
            let old_text = args
                .get("old_text")
                .and_then(|v| v.as_str())
                .context("Parameter 'old_text' is required")?;
            let new_text = args
                .get("new_text")
                .and_then(|v| v.as_str())
                .context("Parameter 'new_text' is required")?;
            let resolved = resolve_allowed_path(file_path, allowed_roots)?;

            if !resolved.exists() {
                bail!("File '{}' not found to edit", file_path);
            }
            if resolved.is_dir() {
                bail!("'{}' is a directory, not a file", file_path);
            }

            let content = std_fs::read_to_string(&resolved)
                .context(format!("Failed to read file '{}' for editing", file_path))?;

            let updated_content = apply_resilient_edit(&content, old_text, new_text, file_path)?;

            std_fs::write(&resolved, &updated_content)?;

            let display_path = to_workspace_folder_path(&resolved, allowed_roots);
            let result_json = serde_json::json!({
                "status": "ok",
                "path": display_path,
                "file_path": file_path,
                "bytes_written": updated_content.len(),
                "old_text_len": old_text.len(),
                "new_text_len": new_text.len(),
            });

            Ok(result_json.to_string())
        }

        "workspace_grep" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .context("Parameter 'query' is required")?;
            let sub_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let resolved = resolve_allowed_path(sub_path, allowed_roots)?;

            let grep_result = run_workspace_grep(&resolved, query, allowed_roots)?;
            Ok(grep_result.to_string())
        }

        "workspace_run_command" => {
            let command_str = args
                .get("command")
                .and_then(|v| v.as_str())
                .context("Parameter 'command' is required")?;
            let sub_path = args
                .get("cwd")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let work_dir = resolve_allowed_path(sub_path, allowed_roots)?;

            if !work_dir.exists() || !work_dir.is_dir() {
                bail!("Working directory '{}' not found", sub_path);
            }

            let effective_command = normalize_workspace_command(command_str, &work_dir, allowed_roots);

            let mut proc = create_workspace_command(&effective_command);

            proc.current_dir(&work_dir);

            // Execute command with output capture
            let output = proc.output().with_context(|| {
                format!("Failed to execute shell command: '{}'", command_str)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            let success = output.status.success() && exit_code == 0;

            // Preserve tail where compiler diagnostics and error traces live (4k head + 16k tail)
            let trimmed_stdout = truncate_smart(&stdout, 4_000, 16_000);
            let trimmed_stderr = truncate_smart(&stderr, 2_000, 8_000);

            let mut res_map = serde_json::Map::new();
            res_map.insert("command".to_string(), serde_json::json!(command_str));
            res_map.insert("cwd".to_string(), serde_json::json!(work_dir.display().to_string()));
            res_map.insert("exit_code".to_string(), serde_json::json!(exit_code));
            res_map.insert("success".to_string(), serde_json::json!(success));

            if !success {
                res_map.insert("status".to_string(), serde_json::json!("FAILED"));

                let mut combined_output = String::new();
                if !stdout.is_empty() {
                    combined_output.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    combined_output.push('\n');
                    combined_output.push_str(&stderr);
                }

                let diags = super::executor::extract_command_diagnostics(&combined_output);
                let banner = if !diags.is_empty() {
                    format!(
                        "❌ COMMAND FAILED with exit code {exit_code}.\nIdentified Errors:\n{}\n\nReview the stdout/stderr below, edit the offending files, and re-run verification.",
                        diags.iter().map(|d| format!("  - {}", d)).collect::<Vec<_>>().join("\n")
                    )
                } else {
                    format!(
                        "❌ COMMAND FAILED with exit code {exit_code}. Review stdout and stderr below to locate errors, fix the offending files, and re-run verification."
                    )
                };

                res_map.insert("error_banner".to_string(), serde_json::json!(banner));
            } else {
                res_map.insert("status".to_string(), serde_json::json!("SUCCESS"));
            }

            res_map.insert("stdout".to_string(), serde_json::json!(trimmed_stdout));
            res_map.insert("stderr".to_string(), serde_json::json!(trimmed_stderr));

            let res_json = serde_json::Value::Object(res_map);

            Ok(res_json.to_string())
        }

        _ => bail!("Unknown tool '{}'", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_generate_smart_gitignore_node() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_test_node_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(temp_dir.join("package.json"), "{}");

        let gitignore = generate_smart_gitignore(&temp_dir);
        assert!(gitignore.contains("node_modules/"));
        assert!(gitignore.contains("dist/"));
        assert!(gitignore.contains(".DS_Store"));
        assert!(gitignore.contains(".env"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_smart_gitignore_rust() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_test_rust_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"foo\"");

        let gitignore = generate_smart_gitignore(&temp_dir);
        assert!(gitignore.contains("/target/"));
        assert!(gitignore.contains(".DS_Store"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_smart_gitignore_python() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_test_py_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(temp_dir.join("requirements.txt"), "flask==2.0.0");

        let gitignore = generate_smart_gitignore(&temp_dir);
        assert!(gitignore.contains("__pycache__/"));
        assert!(gitignore.contains(".venv/"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ensure_git_and_gitignore_initializes() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_test_init_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(temp_dir.join("package.json"), "{}");

        let res = ensure_git_and_gitignore(&temp_dir);
        assert!(res.is_ok());
        assert!(temp_dir.join(".git").exists());
        assert!(temp_dir.join(".gitignore").exists());

        let content = fs::read_to_string(temp_dir.join(".gitignore")).unwrap();
        assert!(content.contains("node_modules/"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_run_workspace_grep_line_truncation() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_test_grep_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let long_line = format!("const TARGET_STRING = '{}';", "A".repeat(1200));
        let test_file = temp_dir.join("sample.js");
        let _ = fs::write(&test_file, &long_line);

        let roots = vec![temp_dir.to_string_lossy().to_string()];
        let val = run_workspace_grep(&temp_dir, "TARGET_STRING", &roots).unwrap();
        let matches = val.get("results").and_then(|r| r.as_array()).unwrap();
        assert_eq!(matches.len(), 1);
        let snippet = matches[0].get("content").and_then(|c| c.as_str()).unwrap();
        assert!(snippet.contains("[truncated]"));
        assert!(snippet.len() < 700);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_to_workspace_folder_path() {
        let roots = vec!["/tmp/my-project".to_string()];
        let path = Path::new("/tmp/my-project/src/components/Button.tsx");
        assert_eq!(
            to_workspace_folder_path(path, &roots),
            "my-project/src/components/Button.tsx"
        );
    }

    #[test]
    fn test_normalize_workspace_command() {
        let temp_dir = std::env::temp_dir().join(format!("ragasukma_cmd_norm_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let roots = vec![temp_dir.to_string_lossy().to_string()];
        let folder_name = temp_dir.file_name().unwrap().to_str().unwrap();

        let cmd = format!("(cd {}/backend && cargo test) && (cd {}/frontend && npm test)", folder_name, folder_name);
        let normalized = normalize_workspace_command(&cmd, &temp_dir, &roots);
        assert_eq!(normalized, "(cd backend && cargo test) && (cd frontend && npm test)");

        let single_cmd = format!("cd {} && cargo test", folder_name);
        let normalized_single = normalize_workspace_command(&single_cmd, &temp_dir, &roots);
        assert_eq!(normalized_single, "cd . && cargo test");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_expand_file_tags_no_tags() {
        let roots = vec!["/tmp".to_string()];
        let input = "hello world without tags";
        assert_eq!(expand_file_tags(input, &roots), input);
    }

    #[test]
    fn test_create_workspace_command_execution() {
        let mut cmd = create_workspace_command("echo '__agent_shell_ok__'");
        let output = cmd.output().expect("Failed to execute test command");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("__agent_shell_ok__"));
    }

    #[test]
    fn test_apply_resilient_edit_whitespace_and_indentation() {
        let content = "function foo() {\n    const a = 1;\n    const b = 2;\n    return a + b;\n}";
        // AI sent 2 spaces instead of 4 spaces
        let old_text = "  const a = 1;\n  const b = 2;";
        let new_text = "  const a = 10;\n  const b = 20;";
        let updated = apply_resilient_edit(content, old_text, new_text, "test.ts").unwrap();
        assert!(updated.contains("const a = 10;"));
        assert!(updated.contains("return a + b;"));
    }

    #[test]
    fn test_apply_resilient_edit_quote_and_semicolon_tolerance() {
        let content = "export const config = {\n  mode: 'production',\n  port: 8080\n};";
        // AI used double quotes and trailing comma
        let old_text = "mode: \"production\",";
        let new_text = "mode: 'staging',";
        let updated = apply_resilient_edit(content, old_text, new_text, "config.ts").unwrap();
        assert!(updated.contains("mode: 'staging'"));
    }

    #[test]
    fn test_apply_resilient_edit_anchor_matching() {
        let content = "class Service {\n  init() {\n    // comment here\n    setup();\n  }\n}";
        // AI skipped the comment inside
        let old_text = "init() {\n  setup();\n}";
        let new_text = "init() {\n  initV2();\n}";
        let updated = apply_resilient_edit(content, old_text, new_text, "service.ts").unwrap();
        assert!(updated.contains("initV2();"));
    }

    #[test]
    fn test_apply_resilient_edit_diagnostic_hint() {
        let content = "line 1\nline 2\nconst specialFeature = true;\nline 4\nline 5";
        let err = apply_resilient_edit(content, "function completelyDifferentNonExistentFeature() {\n  return 999;\n}", "new", "test.ts").unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("'old_text' not found"));
        assert!(err_msg.contains("Use `workspace_read_file`") || err_msg.contains("Closest actual content in 'test.ts'"));
    }
}
