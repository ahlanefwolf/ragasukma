use anyhow::{bail, Context, Result};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns tool definitions for OpenAI / Antigravity / Codex V1 tool calling
pub fn get_agent_tools_schema() -> Vec<serde_json::Value> {
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
                "description": "Read text content of a file in the workspace.",
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
                "description": "Execute a terminal/shell command (e.g. `cargo check`, `npm run build`, `npm test`, `git status`, etc.) in the workspace directory.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Full shell command to execute"
                        },
                        "cwd": {
                            "type": "string",
                            "description": "Working subdirectory relative to workspace (optional, defaults to workspace root)"
                        }
                    },
                    "required": ["command"]
                }
            }
        }),
    ]
}

/// Verify that a target path is allowed within one of the project's root folders
pub fn resolve_allowed_path(raw_path: &str, allowed_roots: &[String]) -> Result<PathBuf> {
    if allowed_roots.is_empty() {
        bail!("Workspace has no registered root folders.");
    }

    let input_path = Path::new(raw_path);

    // If path is relative, resolve against the first allowed root
    let target = if input_path.is_relative() {
        Path::new(&allowed_roots[0]).join(input_path)
    } else {
        input_path.to_path_buf()
    };

    let target_canonical = if target.exists() {
        target.canonicalize().unwrap_or(target)
    } else {
        // If file doesn't exist yet (for write_file), canonicalize parent
        if let Some(parent) = target.parent() {
            if parent.exists() {
                parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()).join(target.file_name().unwrap_or_default())
            } else {
                target
            }
        } else {
            target
        }
    };

    // Check if target is inside at least one allowed root
    let is_inside_allowed = allowed_roots.iter().any(|root| {
        let r_path = Path::new(root);
        let r_canonical = r_path.canonicalize().unwrap_or_else(|_| r_path.to_path_buf());
        target_canonical.starts_with(&r_canonical)
    });

    if !is_inside_allowed {
        bail!(
            "Access denied: path '{}' is outside allowed workspace folders ({:?})",
            raw_path,
            allowed_roots
        );
    }

    Ok(target_canonical)
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
            Ok(json!({ "path": resolved.to_string_lossy(), "entries": entries }).to_string())
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

            let result_json = serde_json::json!({
                "status": "ok",
                "path": resolved.display().to_string(),
                "file_path": file_path,
                "bytes_written": content.len(),
                "is_new_file": is_new_file,
                "old_content": old_content,
                "new_content": content,
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
            grep_dir_recursive(&resolved, query, &mut matches, 50)?;

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

            #[cfg(target_os = "windows")]
            let mut proc = {
                let mut cmd = std::process::Command::new("powershell");
                cmd.arg("-NoProfile").arg("-Command").arg(command_str);
                cmd
            };

            #[cfg(not(target_os = "windows"))]
            let mut proc = {
                let mut cmd = std::process::Command::new("sh");
                cmd.arg("-c").arg(command_str);
                cmd
            };

            proc.current_dir(&work_dir);

            // Execute command with output capture
            let output = proc.output().with_context(|| {
                format!("Failed to execute shell command: '{}'", command_str)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            // Limit output size to prevent context overflow (max 20,000 characters)
            let trimmed_stdout = if stdout.len() > 20_000 {
                format!("{}... [Output truncated, too long]", &stdout[..20_000])
            } else {
                stdout
            };

            let trimmed_stderr = if stderr.len() > 10_000 {
                format!("{}... [Error output truncated]", &stderr[..10_000])
            } else {
                stderr
            };

            let res_json = serde_json::json!({
                "command": command_str,
                "cwd": work_dir.display().to_string(),
                "exit_code": exit_code,
                "success": output.status.success(),
                "stdout": trimmed_stdout,
                "stderr": trimmed_stderr,
            });

            Ok(res_json.to_string())
        }

        _ => bail!("Unknown tool '{}'", name),
    }
}

fn grep_dir_recursive(
    dir: &Path,
    query: &str,
    matches: &mut Vec<serde_json::Value>,
    max_results: usize,
) -> Result<()> {
    if matches.len() >= max_results {
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip noisy directories
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                let _ = grep_dir_recursive(&path, query, matches, max_results);
            } else if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(query) {
                            matches.push(json!({
                                "file": path.to_string_lossy(),
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
    for root in allowed_roots {
        let r_path = Path::new(root);
        if !r_path.exists() || !r_path.is_dir() {
            continue;
        }
        collect_files_recursive(r_path, r_path, &mut files, 0, 500)?;
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

            // Skip noisy folders and files
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "dist"
                || name == "build"
                || name == "vendor"
                || name == ".git"
            {
                continue;
            }

            let rel_path = path
                .strip_prefix(base_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| name.clone().replace('\\', "/"));

            let is_dir = path.is_dir();
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
                collect_files_recursive(base_root, &path, results, depth + 1, max_items)?;
            }
        }
    }
    Ok(())
}

/// Automatically parse `@file` or `@file:start-end` tags in user messages
/// and append the referenced file snippet lines directly to the turn prompt.
pub fn expand_file_tags(text: &str, allowed_roots: &[String]) -> String {
    if !text.contains('@') || allowed_roots.is_empty() {
        return text.to_string();
    }

    // Match @filepath or @filepath:start or @filepath:start-end
    let re = match regex::Regex::new(r#"@([a-zA-Z0-9_\-\./\\]+)(?::(\d+)(?:-(\d+))?)?"#) {
        Ok(r) => r,
        Err(_) => return text.to_string(),
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

            attached_snippets.push(format!(
                "📁 Attached Directory Context for `@{}`:\n```\nPath: {}\nItems:\n{}\n```",
                raw_file,
                resolved.display(),
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
    fn test_expand_file_tags_no_tags() {
        let roots = vec!["/tmp".to_string()];
        let input = "hello world without tags";
        assert_eq!(expand_file_tags(input, &roots), input);
    }
}
