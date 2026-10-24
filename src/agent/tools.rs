use anyhow::{bail, Context, Result};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
                "description": "Write or create a file in the workspace.",
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
                "description": "Replace a specific section or lines in an existing file. TIP: Keep 'old_text' short and unique (only 2-6 lines around the target change) so it matches reliably.",
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
                            "description": "Full shell command to execute"
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

/// Verify that a target path is allowed within one of the project's root folders.
/// Seamlessly supports multi-root workspaces, folder aliases, and relative paths across all platforms.
pub fn resolve_allowed_path(raw_path: &str, allowed_roots: &[String]) -> Result<PathBuf> {
    if allowed_roots.is_empty() {
        bail!("Workspace has no registered root folders.");
    }

    let input_path = Path::new(raw_path);

    // 1. If absolute path: verify it starts with an allowed root
    if input_path.is_absolute() {
        let canonical = if input_path.exists() {
            input_path.canonicalize().unwrap_or_else(|_| input_path.to_path_buf())
        } else if let Some(parent) = input_path.parent() {
            if parent.exists() {
                parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()).join(input_path.file_name().unwrap_or_default())
            } else {
                input_path.to_path_buf()
            }
        } else {
            input_path.to_path_buf()
        };

        let is_inside = allowed_roots.iter().any(|r| {
            let r_path = Path::new(r);
            let r_canon = r_path.canonicalize().unwrap_or_else(|_| r_path.to_path_buf());
            canonical.starts_with(&r_canon)
        });

        if !is_inside {
            bail!(
                "Access denied: path '{}' is outside allowed workspace folders ({:?})",
                raw_path,
                allowed_roots
            );
        }
        return Ok(canonical);
    }

    // 2. Relative path: Check if path matches an existing file/dir in any allowed root directly
    for root in allowed_roots {
        let candidate = Path::new(root).join(input_path);
        if candidate.exists() {
            return Ok(candidate.canonicalize().unwrap_or(candidate));
        }
    }

    // 3. Multi-root folder alias matching:
    // e.g. input_path is "frontend" or "frontend/src/App.tsx", matching root ".../frontend"
    for root in allowed_roots {
        let root_p = Path::new(root);
        if let Some(folder_name) = root_p.file_name().and_then(|n| n.to_str()) {
            if input_path == Path::new(folder_name) {
                return Ok(root_p.canonicalize().unwrap_or_else(|_| root_p.to_path_buf()));
            }
            if let Ok(rel) = input_path.strip_prefix(folder_name) {
                let candidate = root_p.join(rel);
                if candidate.exists() {
                    return Ok(candidate.canonicalize().unwrap_or(candidate));
                }
            }
        }
    }

    // 4. For new files or directories that do not exist yet (e.g. write_file)
    for root in allowed_roots {
        let root_p = Path::new(root);
        if let Some(folder_name) = root_p.file_name().and_then(|n| n.to_str()) {
            if let Ok(rel) = input_path.strip_prefix(folder_name) {
                return Ok(root_p.join(rel));
            }
        }
    }

    // 5. Default fallback to primary root folder
    let target = Path::new(&allowed_roots[0]).join(input_path);
    let target_canonical = if target.exists() {
        target.canonicalize().unwrap_or(target)
    } else {
        target
    };

    Ok(target_canonical)
}

/// Convert any full path into a clean, workspace-relative path prefixed with its registered root folder name.
/// e.g. `/home/user/project/portofolio/src/App.tsx` -> `portofolio/src/App.tsx`
pub fn to_workspace_folder_path(path: &Path, allowed_roots: &[String]) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for root in allowed_roots {
        let root_p = Path::new(root);
        let root_canon = root_p.canonicalize().unwrap_or_else(|_| root_p.to_path_buf());
        if let Ok(rel) = canonical.strip_prefix(&root_canon) {
            let folder_name = root_p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if rel.as_os_str().is_empty() {
                return folder_name.to_string();
            }
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if folder_name.is_empty() {
                return rel_str;
            }
            return format!("{}/{}", folder_name, rel_str);
        }
    }

    path.to_string_lossy().replace('\\', "/")
}

/// Execute a tool call safely
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

            let mut entries = Vec::new();
            for entry in fs::read_dir(&resolved)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type()?.is_dir();
                entries.push(format!("{}{}", file_name, if is_dir { "/" } else { "" }));
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

            let content = fs::read_to_string(&resolved)
                .context(format!("Failed to read file '{}'", file_path))?;

            let start = args.get("start_line").and_then(|v| v.as_u64()).map(|n| n as usize);
            let end = args.get("end_line").and_then(|v| v.as_u64()).map(|n| n as usize);

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

            let old_content = fs::read_to_string(&resolved).ok();
            let is_new_file = old_content.is_none();

            if let Some(parent) = resolved.parent() {
                let _ = fs::create_dir_all(parent);
            }

            fs::write(&resolved, content)?;

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

            let content = fs::read_to_string(&resolved)
                .context(format!("Failed to read file '{}' for editing", file_path))?;

            let updated_content = apply_resilient_edit(&content, old_text, new_text, file_path)?;

            fs::write(&resolved, &updated_content)?;

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

            let mut matches = Vec::new();
            grep_dir_recursive(&resolved, query, &mut matches, 50, allowed_roots)?;

            Ok(json!({ "query": query, "matches_found": matches.len(), "results": matches }).to_string())
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

            let mut proc = create_workspace_command(command_str);

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

fn grep_dir_recursive(
    target_path: &Path,
    query: &str,
    matches: &mut Vec<serde_json::Value>,
    max_results: usize,
    allowed_roots: &[String],
) -> Result<()> {
    if matches.len() >= max_results {
        return Ok(());
    }

    let query_lower = query.to_lowercase();

    // Direct support if target_path is a single file!
    if target_path.is_file() {
        if let Ok(content) = fs::read_to_string(target_path) {
            let display_file = to_workspace_folder_path(target_path, allowed_roots);
            for (line_idx, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    matches.push(json!({
                        "file": display_file,
                        "line": line_idx + 1,
                        "content": line.trim()
                    }));
                    if matches.len() >= max_results {
                        return Ok(());
                    }
                }
            }
        }
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(target_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip noisy directories
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                let _ = grep_dir_recursive(&path, query, matches, max_results, allowed_roots);
            } else if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let display_file = to_workspace_folder_path(&path, allowed_roots);
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&query_lower) {
                            matches.push(json!({
                                "file": display_file,
                                "line": line_idx + 1,
                                "content": line.trim()
                            }));
                            if matches.len() >= max_results {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceFileInfo {
    pub relative_path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub line_count: Option<usize>,
}

/// Recursively list files across all allowed root folders in the project
pub fn list_workspace_files(allowed_roots: &[String]) -> Result<Vec<WorkspaceFileInfo>> {
    let mut files = Vec::new();
    let max_items = 1500;
    for root in allowed_roots {
        let r_path = Path::new(root);
        if !r_path.exists() || !r_path.is_dir() {
            continue;
        }

        let folder_name = r_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Also add the root folder itself to the file list so UI/agent sees it
        if !folder_name.is_empty() {
            files.push(WorkspaceFileInfo {
                relative_path: folder_name.clone(),
                name: folder_name.clone(),
                is_dir: true,
                size: 0,
                line_count: None,
            });
        }

        collect_files_recursive(r_path, &folder_name, r_path, &mut files, 0, max_items)?;
    }

    files.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.relative_path.cmp(&b.relative_path)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(files)
}

fn collect_files_recursive(
    base_root: &Path,
    folder_prefix: &str,
    current_dir: &Path,
    results: &mut Vec<WorkspaceFileInfo>,
    depth: usize,
    max_items: usize,
) -> Result<()> {
    if depth > 10 || results.len() >= max_items {
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(current_dir) {
        let mut sorted: Vec<_> = entries.flatten().collect();
        sorted.sort_by_key(|e| e.file_name());

        for entry in sorted {
            if results.len() >= max_items {
                break;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = path.is_dir();

            // Skip heavy build artifact, cache, and VCS folders
            if is_dir {
                if name == ".git"
                    || name == ".cache"
                    || name == ".turbo"
                    || name == ".next"
                    || name == ".nuxt"
                    || name == ".venv"
                    || name == "venv"
                    || name == "__pycache__"
                    || name == "node_modules"
                    || name == "target"
                    || name == "dist"
                    || name == "build"
                    || name == "vendor"
                {
                    continue;
                }
            } else {
                // Skip OS junk files, but keep important dotfiles like .gitignore, .env, .editorconfig
                if name == ".DS_Store" || name == "Thumbs.db" || name == "desktop.ini" {
                    continue;
                }
            }

            let inner_rel = path
                .strip_prefix(base_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| name.clone().replace('\\', "/"));

            let rel_path = if folder_prefix.is_empty() {
                inner_rel
            } else if inner_rel.is_empty() {
                folder_prefix.to_string()
            } else {
                format!("{}/{}", folder_prefix, inner_rel)
            };

            let size = if is_dir {
                0
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            let line_count = if !is_dir && size < 500_000 {
                fs::read_to_string(&path).ok().map(|c| c.lines().count())
            } else {
                None
            };

            results.push(WorkspaceFileInfo {
                relative_path: rel_path,
                name,
                is_dir,
                size,
                line_count,
            });

            if is_dir {
                collect_files_recursive(base_root, folder_prefix, &path, results, depth + 1, max_items)?;
            }
        }
    }
    Ok(())
}

/// Applies surgical edits to file content with a multi-tier resilient matcher:
/// 1. Literal exact match
/// 2. Trimmed leading/trailing newline match
/// 3. Whitespace & indentation tolerant line-by-line sliding window
/// 4. Quote & semicolon relaxed matching (single quotes vs double quotes, trailing semicolons/commas)
/// 5. Anchor matching (first and last non-empty line match uniquely)
/// 6. High-similarity fuzzy window matching (similarity >= 0.85)
/// 7. Detailed contextual diagnostic reporting exact file lines if all matchers fail.
pub fn apply_resilient_edit(
    content: &str,
    old_text: &str,
    new_text: &str,
    file_path: &str,
) -> Result<String> {
    let normalized_content = content.replace("\r\n", "\n");
    let normalized_old = old_text.replace("\r\n", "\n");
    let normalized_new = new_text.replace("\r\n", "\n");

    // Tier 1: Exact literal match
    if normalized_content.contains(&normalized_old) {
        let count = normalized_content.matches(&normalized_old).count();
        if count == 1 {
            return Ok(normalized_content.replacen(&normalized_old, &normalized_new, 1));
        } else if count > 1 {
            bail!(
                "'old_text' matches {} times in file '{}'. Provide 2-3 additional surrounding lines to make it unique.",
                count, file_path
            );
        }
    }

    // Tier 2: Trimmed leading/trailing blank lines
    let trimmed_old = normalized_old.trim_matches('\n');
    let trimmed_new = normalized_new.trim_matches('\n');
    if !trimmed_old.is_empty() && trimmed_old != normalized_old && normalized_content.contains(trimmed_old) {
        let count = normalized_content.matches(trimmed_old).count();
        if count == 1 {
            return Ok(normalized_content.replacen(trimmed_old, trimmed_new, 1));
        }
    }

    let content_lines: Vec<&str> = normalized_content.lines().collect();
    let raw_old_lines: Vec<&str> = normalized_old.lines().collect();

    // Strip leading & trailing empty lines from old_lines
    let start_empty = raw_old_lines.iter().take_while(|l| l.trim().is_empty()).count();
    let end_empty = raw_old_lines.iter().rev().take_while(|l| l.trim().is_empty()).count();
    let old_lines: Vec<&str> = if start_empty + end_empty < raw_old_lines.len() {
        raw_old_lines[start_empty..raw_old_lines.len() - end_empty].to_vec()
    } else {
        raw_old_lines
    };

    if old_lines.is_empty() {
        bail!("'old_text' cannot be empty or only whitespace");
    }

    // Tier 3: Whitespace-tolerant line-by-line matching
    if content_lines.len() >= old_lines.len() {
        let mut match_indices = Vec::new();
        for i in 0..=(content_lines.len() - old_lines.len()) {
            let window = &content_lines[i..i + old_lines.len()];
            if window.iter().zip(old_lines.iter()).all(|(c, o)| c.trim() == o.trim()) {
                match_indices.push(i);
            }
        }

        if match_indices.len() == 1 {
            let start_idx = match_indices[0];
            let mut result_lines = Vec::new();
            result_lines.extend_from_slice(&content_lines[..start_idx]);
            result_lines.push(&normalized_new);
            result_lines.extend_from_slice(&content_lines[start_idx + old_lines.len()..]);
            return Ok(result_lines.join("\n"));
        } else if match_indices.len() > 1 {
            bail!(
                "'old_text' matched {} different sections with whitespace tolerance in '{}'. Provide more surrounding lines.",
                match_indices.len(), file_path
            );
        }
    }

    // Tier 4: Punctuation & quote relaxed matching (single quotes vs double quotes, trailing semicolons/commas)
    fn relax_line(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && *c != ';' && *c != ',')
            .map(|c| if c == '\'' || c == '`' { '"' } else { c })
            .collect()
    }

    if content_lines.len() >= old_lines.len() {
        let relaxed_old: Vec<String> = old_lines.iter().map(|l| relax_line(l)).collect();
        let mut relaxed_matches = Vec::new();
        for i in 0..=(content_lines.len() - old_lines.len()) {
            let window = &content_lines[i..i + old_lines.len()];
            let matches = window
                .iter()
                .zip(relaxed_old.iter())
                .all(|(c, o)| relax_line(c) == *o);
            if matches {
                relaxed_matches.push(i);
            }
        }

        if relaxed_matches.len() == 1 {
            let start_idx = relaxed_matches[0];
            let mut result_lines = Vec::new();
            result_lines.extend_from_slice(&content_lines[..start_idx]);
            result_lines.push(&normalized_new);
            result_lines.extend_from_slice(&content_lines[start_idx + old_lines.len()..]);
            return Ok(result_lines.join("\n"));
        }
    }

    // Tier 5: Anchor matching with meaningful non-trivial anchors
    if old_lines.len() >= 3 {
        // Find first non-trivial line (not just a solitary brace or tag like `</div>` or `}`)
        let first_meaningful = old_lines
            .iter()
            .find(|l| {
                let t = l.trim();
                t.len() > 3 && t != "</div>" && t != "}" && t != "};" && t != "</button>" && t != "</div>;"
            })
            .map(|l| l.trim());

        let last_meaningful = old_lines
            .iter()
            .rev()
            .find(|l| {
                let t = l.trim();
                t.len() > 3 && t != "</div>" && t != "}" && t != "};" && t != "</button>" && t != "</div>;"
            })
            .map(|l| l.trim());

        if let (Some(first_anchor), Some(last_anchor)) = (first_meaningful, last_meaningful) {
            let mut anchor_matches = Vec::new();
            for i in 0..content_lines.len() {
                if content_lines[i].trim() == first_anchor {
                    let min_end = i + 1;
                    let max_end = (i + old_lines.len() + 15).min(content_lines.len());
                    for (j, line) in content_lines.iter().enumerate().take(max_end).skip(min_end) {
                        if line.trim() == last_anchor {
                            anchor_matches.push((i, j));
                        }
                    }
                }
            }

            if !anchor_matches.is_empty() {
                let target_span = old_lines.len();
                let mut best_anchor = anchor_matches[0];
                let mut min_diff = (anchor_matches[0].1 - anchor_matches[0].0 + 1).abs_diff(target_span);
                let mut second_diff = usize::MAX;

                for &(start, end) in &anchor_matches[1..] {
                    let diff = (end - start + 1).abs_diff(target_span);
                    if diff < min_diff {
                        second_diff = min_diff;
                        min_diff = diff;
                        best_anchor = (start, end);
                    } else if diff < second_diff {
                        second_diff = diff;
                    }
                }

                if anchor_matches.len() == 1 || min_diff < second_diff {
                    let (start_idx, end_idx) = best_anchor;
                    let mut result_lines = Vec::new();
                    result_lines.extend_from_slice(&content_lines[..start_idx]);
                    result_lines.push(&normalized_new);
                    result_lines.extend_from_slice(&content_lines[end_idx + 1..]);
                    return Ok(result_lines.join("\n"));
                }
            }
        }
    }

    // Tier 6: Elastic similarity window matching (handles line count variations +/- 3 lines)
    if !content_lines.is_empty() && !old_lines.is_empty() {
        let min_len = old_lines.len().saturating_sub(3).max(1);
        let max_len = (old_lines.len() + 3).min(content_lines.len());

        let mut best_score = 0.0f64;
        let mut second_best_score = 0.0f64;
        let mut best_range = None;

        for window_len in min_len..=max_len {
            if content_lines.len() < window_len {
                continue;
            }
            for i in 0..=(content_lines.len() - window_len) {
                let window = &content_lines[i..i + window_len];
                let mut score_sum = 0.0;
                let pair_count = window.len().min(old_lines.len());
                for (c, o) in window.iter().take(pair_count).zip(old_lines.iter().take(pair_count)) {
                    let c_trim = c.trim();
                    let o_trim = o.trim();
                    if c_trim == o_trim {
                        score_sum += 1.0;
                    } else if c_trim.to_lowercase() == o_trim.to_lowercase() {
                        score_sum += 0.95;
                    } else {
                        let c_chars: std::collections::HashSet<char> = c_trim.chars().filter(|ch| !ch.is_whitespace()).collect();
                        let o_chars: std::collections::HashSet<char> = o_trim.chars().filter(|ch| !ch.is_whitespace()).collect();
                        let intersect = c_chars.intersection(&o_chars).count();
                        let union = c_chars.union(&o_chars).count();
                        if union > 0 {
                            score_sum += (intersect as f64) / (union as f64);
                        }
                    }
                }
                let avg_score = score_sum / (old_lines.len().max(window_len) as f64);
                if avg_score > best_score {
                    second_best_score = best_score;
                    best_score = avg_score;
                    best_range = Some((i, i + window_len));
                } else if avg_score > second_best_score {
                    second_best_score = avg_score;
                }
            }
        }

        if best_score >= 0.65 && (best_score - second_best_score) >= 0.04 {
            if let Some((start_idx, end_idx)) = best_range {
                let mut result_lines = Vec::new();
                result_lines.extend_from_slice(&content_lines[..start_idx]);
                result_lines.push(&normalized_new);
                result_lines.extend_from_slice(&content_lines[end_idx..]);
                return Ok(result_lines.join("\n"));
            }
        }
    }

    // Tier 7: Diagnostics reporting exact file lines
    let mut best_hint_idx = None;
    let mut highest_keyword_match = 0;

    for (idx, line) in content_lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        let matches = old_lines
            .iter()
            .filter(|o| o.trim().len() >= 4 && line_lower.contains(&o.trim().to_lowercase()))
            .count();
        if matches > highest_keyword_match {
            highest_keyword_match = matches;
            best_hint_idx = Some(idx);
        }
    }

    let hint_block = if let Some(center_idx) = best_hint_idx {
        let start = center_idx.saturating_sub(4);
        let end = (center_idx + 6).min(content_lines.len());
        let snippet = content_lines[start..end]
            .iter()
            .enumerate()
            .map(|(offset, l)| format!("{:>4} | {}", start + offset + 1, l))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\nClosest actual content in '{}' (lines {}-{}):\n```\n{}\n```\nAction: Copy the exact lines above to form `old_text` or use `workspace_write_file` if replacing a large block.",
            file_path, start + 1, end, snippet
        )
    } else {
        format!(
            "\nUse `workspace_read_file` to inspect the exact lines of '{}' before editing.",
            file_path
        )
    };

    bail!("'old_text' not found in file '{}'.{}", file_path, hint_block)
}

/// Builds a cross-platform shell Command with an enriched PATH environment so tools like
/// npm, npx, pnpm, yarn, node, cargo, python, etc. are reliably found across Linux, macOS, and Windows.
fn create_workspace_command(command_str: &str) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("powershell");
        cmd.arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(command_str);

        // Enrich Windows PATH if needed
        let current_path = std::env::var("PATH").unwrap_or_default();
        let mut path_entries: Vec<PathBuf> = std::env::split_paths(&current_path).collect();

        if let Ok(appdata) = std::env::var("APPDATA") {
            let npm_roaming = PathBuf::from(&appdata).join("npm");
            if npm_roaming.exists() && !path_entries.contains(&npm_roaming) {
                path_entries.insert(0, npm_roaming);
            }
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let cargo_bin = PathBuf::from(&userprofile).join(".cargo").join("bin");
            if cargo_bin.exists() && !path_entries.contains(&cargo_bin) {
                path_entries.insert(0, cargo_bin);
            }
        }
        let node_prog = PathBuf::from(r"C:\Program Files\nodejs");
        if node_prog.exists() && !path_entries.contains(&node_prog) {
            path_entries.insert(0, node_prog);
        }

        if let Ok(new_path) = std::env::join_paths(path_entries) {
            cmd.env("PATH", new_path);
        }
        cmd
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Enrich Unix PATH with common Node, Python, and Rust manager directories
        let current_path = std::env::var("PATH").unwrap_or_default();
        let mut path_entries: Vec<PathBuf> = std::env::split_paths(&current_path).collect();

        if let Ok(home) = std::env::var("HOME") {
            let home_path = Path::new(&home);
            let candidates = [
                home_path.join(".local/bin"),
                home_path.join(".cargo/bin"),
                home_path.join(".bun/bin"),
                home_path.join(".volta/bin"),
                home_path.join(".fnm/current/bin"),
                home_path.join(".asdf/shims"),
                home_path.join(".asdf/bin"),
                PathBuf::from("/usr/local/bin"),
                PathBuf::from("/opt/homebrew/bin"),
            ];

            for candidate in candidates {
                if candidate.exists() && !path_entries.contains(&candidate) {
                    path_entries.insert(0, candidate);
                }
            }

            // NVM active directory or latest installed node version
            if let Ok(nvm_bin) = std::env::var("NVM_BIN") {
                let nvm_path = PathBuf::from(nvm_bin);
                if nvm_path.exists() && !path_entries.contains(&nvm_path) {
                    path_entries.insert(0, nvm_path);
                }
            }

            let nvm_node_dir = home_path.join(".nvm/versions/node");
            if nvm_node_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&nvm_node_dir) {
                    let mut versions: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path().join("bin"))
                        .filter(|p| p.is_dir())
                        .collect();
                    versions.sort();
                    if let Some(latest) = versions.pop() {
                        if !path_entries.contains(&latest) {
                            path_entries.insert(0, latest);
                        }
                    }
                }
            }
        }

        let new_path = std::env::join_paths(path_entries).unwrap_or_else(|_| current_path.into());

        let shell_var = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let (shell_bin, is_login) = if Path::new(&shell_var).exists() {
            let login = shell_var.ends_with("bash") || shell_var.ends_with("zsh");
            (shell_var, login)
        } else if Path::new("/bin/bash").exists() {
            ("/bin/bash".to_string(), true)
        } else {
            ("sh".to_string(), false)
        };

        let mut cmd = std::process::Command::new(&shell_bin);
        if is_login {
            cmd.arg("-l").arg("-c").arg(command_str);
        } else {
            cmd.arg("-c").arg(command_str);
        }
        cmd.env("PATH", new_path);
        cmd
    }
}

/// Automatically parse `@file` or `@file:start-end` tags in user messages
/// and append the referenced file snippet lines directly to the turn prompt.
pub fn expand_file_tags(text: &str, allowed_roots: &[String]) -> String {
    if !text.contains('@') || allowed_roots.is_empty() {
        return text.to_string();
    }

    // Match @filepath or @filepath:start or @filepath:start-end (cached regex)
    static FILE_TAG_RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
    let re_opt = FILE_TAG_RE.get_or_init(|| {
        regex::Regex::new(r#"@([a-zA-Z0-9_\-\./\\]+)(?::(\d+)(?:-(\d+))?)?"#).ok()
    });

    let re = match re_opt.as_ref() {
        Some(r) => r,
        None => return text.to_string(),
    };

    let mut attached_snippets: Vec<String> = Vec::new();
    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    for caps in re.captures_iter(text) {
        let raw_file = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if raw_file.is_empty() {
            continue;
        }

        // Try to resolve in allowed roots
        let resolved = match resolve_allowed_path(raw_file, allowed_roots) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if !resolved.exists() {
            continue;
        }

        // If it's a directory tag (@folder), attach directory listing context
        if resolved.is_dir() {
            let snippet_key = format!("dir:{}", raw_file);
            if seen_keys.contains(&snippet_key) {
                continue;
            }
            seen_keys.insert(snippet_key);

            let mut dir_files: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(&resolved) {
                let mut sorted: Vec<_> = entries.flatten().collect();
                sorted.sort_by_key(|e| e.file_name());
                for entry in sorted {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.starts_with('.')
                        || fname == "node_modules"
                        || fname == "target"
                        || fname == "dist"
                        || fname == "build"
                    {
                        continue;
                    }
                    let is_d = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    if is_d {
                        dir_files.push(format!("  [dir]  {}/", fname));
                    } else {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        dir_files.push(format!("  [file] {} ({} bytes)", fname, size));
                    }
                }
            }

            let display_path = to_workspace_folder_path(&resolved, allowed_roots);
            attached_snippets.push(format!(
                "📁 Attached Directory Context for `@{}`:\n```\nPath: {}\nItems:\n{}\n```",
                raw_file,
                display_path,
                if dir_files.is_empty() {
                    "  (empty folder)".to_string()
                } else {
                    dir_files.join("\n")
                }
            ));
            continue;
        }

        if !resolved.is_file() {
            continue;
        }

        let start_line = caps.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
        let end_line = caps.get(3).and_then(|m| m.as_str().parse::<usize>().ok());

        let snippet_key = format!(
            "{}:{}:{}",
            raw_file,
            start_line.unwrap_or(0),
            end_line.unwrap_or(0)
        );
        if seen_keys.contains(&snippet_key) {
            continue;
        }
        seen_keys.insert(snippet_key);

        if let Ok(content) = fs::read_to_string(&resolved) {
            let lines: Vec<&str> = content.lines().collect();
            let total = lines.len();

            let (sliced, range_label) = match (start_line, end_line) {
                (Some(s), Some(e)) => {
                    let s_idx = s.saturating_sub(1);
                    let e_idx = e.min(total);
                    if s_idx < e_idx {
                        let numbered = lines[s_idx..e_idx]
                            .iter()
                            .enumerate()
                            .map(|(i, l)| format!("{:4} | {}", s + i, l))
                            .collect::<Vec<_>>()
                            .join("\n");
                        (numbered, format!("(lines {}-{})", s, e_idx))
                    } else {
                        (String::new(), format!("(lines {}-{})", s, e))
                    }
                }
                (Some(s), None) => {
                    let s_idx = s.saturating_sub(1);
                    if s_idx < total {
                        (format!("{:4} | {}", s, lines[s_idx]), format!("(line {})", s))
                    } else {
                        (String::new(), format!("(line {})", s))
                    }
                }
                (None, Some(e)) => {
                    let e_idx = e.min(total);
                    let numbered = lines[..e_idx]
                        .iter()
                        .enumerate()
                        .map(|(i, l)| format!("{:4} | {}", i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n");
                    (numbered, format!("(lines 1-{})", e_idx))
                }
                (None, None) => {
                    let numbered = lines
                        .iter()
                        .take(300)
                        .enumerate()
                        .map(|(i, l)| format!("{:4} | {}", i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let label = if total > 300 {
                        format!("(lines 1-300 of {})", total)
                    } else {
                        format!("(all {} lines)", total)
                    };
                    (numbered, label)
                }
            };

            if !sliced.is_empty() {
                attached_snippets.push(format!(
                    "### [Referenced File Context: `{}` {}]:\n```\n{}\n```",
                    raw_file, range_label, sliced
                ));
            }
        }
    }

    if attached_snippets.is_empty() {
        text.to_string()
    } else {
        format!(
            "{}\n\n### Attached File Snippets (via @ references):\n{}",
            text,
            attached_snippets.join("\n\n")
        )
    }
}

/// Open native OS folder selection dialog (File Manager)
pub fn pick_folder_dialog(initial_path: Option<&str>) -> Result<Option<String>> {
    let _ = &initial_path;
    #[cfg(target_os = "linux")]
    {
        // 1. Try zenity (standard on GNOME, Fedora, Ubuntu, Sway/Wayland, etc.)
        let mut cmd = std::process::Command::new("zenity");
        cmd.arg("--file-selection")
            .arg("--directory")
            .arg("--title=Select Workspace Folder")
            .arg("--modal")
            .arg("--width=700")
            .arg("--height=500");

        if let Some(init) = initial_path {
            let trimmed = init.trim();
            if !trimmed.is_empty() {
                let formatted = if trimmed.ends_with('/') {
                    trimmed.to_string()
                } else {
                    format!("{}/", trimmed)
                };
                cmd.arg(format!("--filename={}", formatted));
            }
        }

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    return Ok(Some(stdout));
                }
            } else if output.status.code() == Some(1) {
                // Exit code 1 in zenity means the user clicked Cancel or closed the window
                return Ok(None);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                eprintln!("[pick_folder_dialog] Zenity non-zero exit ({:?}): {}", output.status.code(), stderr);
            }
        }

        // 2. Try kdialog (standard on KDE Plasma)
        let mut kcmd = std::process::Command::new("kdialog");
        kcmd.arg("--getexistingdirectory");
        if let Some(init) = initial_path {
            let trimmed = init.trim();
            if !trimmed.is_empty() {
                kcmd.arg(trimmed);
            }
        }
        if let Ok(output) = kcmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    return Ok(Some(stdout));
                }
            } else if output.status.code() == Some(1) {
                return Ok(None);
            }
        }

        bail!("Unable to open file manager dialog (zenity or kdialog). Ensure GUI is available or enter the folder path manually.")
    }

    #[cfg(target_os = "windows")]
    {
        let selected_path_init = if let Some(init) = initial_path {
            let trimmed = init.trim().replace('"', "`\"");
            if !trimmed.is_empty() {
                format!("$dialog.SelectedPath = \"{}\"", trimmed)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let ps_script = format!(
            r#"
        Add-Type -AssemblyName System.Windows.Forms
        $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
        $dialog.Description = "Select Workspace Folder"
        $dialog.ShowNewFolderButton = $true
        {}
        if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{
            Write-Output $dialog.SelectedPath
        }}
        "#,
            selected_path_init
        );
        let output = std::process::Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(ps_script)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                return Ok(Some(stdout));
            }
        }
        Ok(None)
    }

    #[cfg(target_os = "macos")]
    {
        let script = if let Some(init) = initial_path {
            let trimmed = init.trim().replace('"', "\\\"");
            if !trimmed.is_empty() {
                format!(
                    r#"POSIX path of (choose folder with prompt "Select Workspace Folder" default location (POSIX file "{}"))"#,
                    trimmed
                )
            } else {
                r#"POSIX path of (choose folder with prompt "Select Workspace Folder")"#.to_string()
            }
        } else {
            r#"POSIX path of (choose folder with prompt "Select Workspace Folder")"#.to_string()
        };
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stdout.is_empty() {
                return Ok(Some(stdout));
            }
        }
        Ok(None)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        bail!("This operating system does not support native folder dialogs yet.")
    }
}

/// Open an existing folder or file directly in the OS file manager (e.g. Thunar, Nautilus, Explorer, Finder)
pub fn open_path_in_file_manager(path_str: &str) -> Result<()> {
    let p = std::path::Path::new(path_str);
    if !p.exists() {
        bail!("Folder or file not found on disk: {}", path_str);
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(path_str);
        crate::engine::skills::configure_subprocess(&mut cmd);
        cmd.spawn()?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer.exe");
        cmd.arg(path_str);
        crate::engine::skills::configure_subprocess(&mut cmd);
        cmd.spawn()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(path_str);
        crate::engine::skills::configure_subprocess(&mut cmd);
        cmd.spawn()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        bail!("This platform does not support opening file manager natively yet.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
