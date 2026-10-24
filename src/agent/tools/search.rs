use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceFileInfo {
    pub relative_path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub line_count: Option<usize>,
}

/// Execute workspace grep across files with line-length capping and total output bounding.
pub fn run_workspace_grep(
    resolved: &Path,
    query: &str,
    allowed_roots: &[String],
) -> Result<serde_json::Value> {
    let query_lower = query.to_lowercase();
    let re_opt = regex::RegexBuilder::new(query)
        .case_insensitive(true)
        .build()
        .ok();

    let max_results = 25;
    let max_total_chars = 16_000;
    let mut current_total_chars = 0;
    let mut matches = Vec::new();
    let mut truncated = false;

    let check_line_match = |line: &str| -> bool {
        if let Some(ref re) = re_opt {
            re.is_match(line)
        } else {
            line.to_lowercase().contains(&query_lower)
        }
    };

    if resolved.is_file() {
        if let Ok(content) = fs::read_to_string(resolved) {
            let display_file = super::fs::to_workspace_folder_path(resolved, allowed_roots);
            for (line_idx, line) in content.lines().enumerate() {
                if check_line_match(line) {
                    let trimmed = line.trim();
                    let content_snippet = if trimmed.chars().count() > 600 {
                        let prefix: String = trimmed.chars().take(600).collect();
                        format!("{} ... [truncated]", prefix)
                    } else {
                        trimmed.to_string()
                    };

                    current_total_chars += content_snippet.len();
                    matches.push(json!({
                        "file": display_file,
                        "line": line_idx + 1,
                        "content": content_snippet
                    }));

                    if matches.len() >= max_results || current_total_chars >= max_total_chars {
                        truncated = true;
                        break;
                    }
                }
            }
        }
        return Ok(json!({
            "query": query,
            "matches_found": matches.len(),
            "truncated": truncated,
            "results": matches
        }));
    }

    let mut builder = ignore::WalkBuilder::new(resolved);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true)
        .max_depth(Some(12));

    let mut overrides = ignore::overrides::OverrideBuilder::new(resolved);
    let _ = overrides.add("!.git");
    let _ = overrides.add("!.git/**");
    let _ = overrides.add("!node_modules");
    let _ = overrides.add("!node_modules/**");
    let _ = overrides.add("!target");
    let _ = overrides.add("!target/**");
    let _ = overrides.add("!dist");
    let _ = overrides.add("!dist/**");
    let _ = overrides.add("!build");
    let _ = overrides.add("!build/**");
    let _ = overrides.add("!.venv");
    let _ = overrides.add("!.venv/**");
    let _ = overrides.add("!venv");
    let _ = overrides.add("!venv/**");
    let _ = overrides.add("!__pycache__");
    let _ = overrides.add("!__pycache__/**");
    let _ = overrides.add("!.cache");
    let _ = overrides.add("!.cache/**");
    let _ = overrides.add("!*.min.js");
    let _ = overrides.add("!*.min.css");
    let _ = overrides.add("!*.map");
    let _ = overrides.add("!*.bundle.js");
    let _ = overrides.add("!.DS_Store");
    let _ = overrides.add("!Thumbs.db");
    if let Ok(ov) = overrides.build() {
        builder.overrides(ov);
    }

    for result in builder.build() {
        if matches.len() >= max_results || current_total_chars >= max_total_chars {
            truncated = true;
            break;
        }

        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Skip binary or oversized files (> 2 MB)
            if metadata.len() > 2_000_000 {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy();
            if file_name.ends_with(".min.js")
                || file_name.ends_with(".min.css")
                || file_name.ends_with(".map")
                || file_name.ends_with(".bundle.js")
            {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let display_file = super::fs::to_workspace_folder_path(path, allowed_roots);
                for (line_idx, line) in content.lines().enumerate() {
                    if check_line_match(line) {
                        let trimmed = line.trim();
                        let content_snippet = if trimmed.chars().count() > 600 {
                            let prefix: String = trimmed.chars().take(600).collect();
                            format!("{} ... [truncated]", prefix)
                        } else {
                            trimmed.to_string()
                        };

                        current_total_chars += content_snippet.len();
                        matches.push(json!({
                            "file": display_file,
                            "line": line_idx + 1,
                            "content": content_snippet
                        }));

                        if matches.len() >= max_results || current_total_chars >= max_total_chars {
                            truncated = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "query": query,
        "matches_found": matches.len(),
        "truncated": truncated,
        "results": matches
    }))
}

/// Recursively list files across all allowed root folders in the project,
/// strictly respecting .gitignore, git global/exclude rules, and filtering out heavy build artifacts.
pub fn list_workspace_files(allowed_roots: &[String]) -> Result<Vec<WorkspaceFileInfo>> {
    let mut files = Vec::new();
    let max_items = 2500;
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

        let mut builder = ignore::WalkBuilder::new(r_path);
        builder
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .parents(true)
            .max_depth(Some(12));

        let mut overrides = ignore::overrides::OverrideBuilder::new(r_path);
        let _ = overrides.add("!.git");
        let _ = overrides.add("!.git/**");
        let _ = overrides.add("!node_modules");
        let _ = overrides.add("!node_modules/**");
        let _ = overrides.add("!target");
        let _ = overrides.add("!target/**");
        let _ = overrides.add("!dist");
        let _ = overrides.add("!dist/**");
        let _ = overrides.add("!build");
        let _ = overrides.add("!build/**");
        let _ = overrides.add("!.venv");
        let _ = overrides.add("!.venv/**");
        let _ = overrides.add("!venv");
        let _ = overrides.add("!venv/**");
        let _ = overrides.add("!__pycache__");
        let _ = overrides.add("!__pycache__/**");
        let _ = overrides.add("!.cache");
        let _ = overrides.add("!.cache/**");
        let _ = overrides.add("!.DS_Store");
        let _ = overrides.add("!Thumbs.db");
        if let Ok(ov) = overrides.build() {
            builder.overrides(ov);
        }

        for result in builder.build() {
            if files.len() >= max_items {
                break;
            }
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.depth() == 0 {
                continue;
            }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name == ".git" || name == ".DS_Store" || name == "Thumbs.db" {
                continue;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            let inner_rel = path
                .strip_prefix(r_path)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| name.clone().replace('\\', "/"));

            let rel_path = if folder_name.is_empty() {
                inner_rel
            } else if inner_rel.is_empty() {
                folder_name.clone()
            } else {
                format!("{}/{}", folder_name, inner_rel)
            };

            let size = if is_dir {
                0
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            let line_count = if !is_dir && size < 500_000 {
                fs::read_to_string(path).ok().map(|c| c.lines().count())
            } else {
                None
            };

            files.push(WorkspaceFileInfo {
                relative_path: rel_path,
                name,
                is_dir,
                size,
                line_count,
            });
        }
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
