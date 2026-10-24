use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

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

/// Inspects project indicator files and generates a tailored .gitignore template.
pub fn generate_smart_gitignore(root_path: &Path) -> String {
    let has_node = root_path.join("package.json").exists()
        || root_path.join("pnpm-lock.yaml").exists()
        || root_path.join("yarn.lock").exists()
        || root_path.join("bun.lockb").exists();

    let has_rust = root_path.join("Cargo.toml").exists();

    let has_python = root_path.join("pyproject.toml").exists()
        || root_path.join("requirements.txt").exists()
        || root_path.join("Pipfile").exists()
        || root_path.join("setup.py").exists();

    let has_go = root_path.join("go.mod").exists();
    let has_php = root_path.join("composer.json").exists();
    let has_java = root_path.join("pom.xml").exists()
        || root_path.join("build.gradle").exists()
        || root_path.join("build.gradle.kts").exists();
    let has_c_cpp = root_path.join("CMakeLists.txt").exists() || root_path.join("Makefile").exists();

    let mut out = String::new();
    out.push_str("# Universal Workspace .gitignore (Auto-generated by Ragasukma Agent)\n\n");

    out.push_str("# Operating System & Editor Artifacts\n");
    out.push_str(".DS_Store\n");
    out.push_str("Thumbs.db\n");
    out.push_str("*.swp\n");
    out.push_str("*.swo\n");
    out.push_str("*~\n");
    out.push_str(".idea/\n");
    out.push_str(".vscode/\n");
    out.push_str("*.sublime-project\n");
    out.push_str("*.sublime-workspace\n\n");

    out.push_str("# Environment & Secret Variables\n");
    out.push_str(".env\n");
    out.push_str(".env.*\n");
    out.push_str("!.env.example\n\n");

    if has_node {
        out.push_str("# Node.js / Web / Frontend\n");
        out.push_str("node_modules/\n");
        out.push_str("dist/\n");
        out.push_str("build/\n");
        out.push_str(".next/\n");
        out.push_str(".nuxt/\n");
        out.push_str(".turbo/\n");
        out.push_str(".cache/\n");
        out.push_str("coverage/\n");
        out.push_str("*.log\n");
        out.push_str("npm-debug.log*\n");
        out.push_str("yarn-debug.log*\n");
        out.push_str("yarn-error.log*\n");
        out.push_str("pnpm-debug.log*\n\n");
    }

    if has_rust {
        out.push_str("# Rust / Cargo\n");
        out.push_str("/target/\n");
        out.push_str("**/*.rs.bk\n");
        out.push_str("*.pdb\n\n");
    }

    if has_python {
        out.push_str("# Python\n");
        out.push_str("__pycache__/\n");
        out.push_str("*.py[cod]\n");
        out.push_str("*$py.class\n");
        out.push_str("*.so\n");
        out.push_str(".venv/\n");
        out.push_str("venv/\n");
        out.push_str("env/\n");
        out.push_str("ENV/\n");
        out.push_str("build/\n");
        out.push_str("dist/\n");
        out.push_str("*.egg-info/\n");
        out.push_str(".pytest_cache/\n");
        out.push_str(".mypy_cache/\n");
        out.push_str(".ruff_cache/\n\n");
    }

    if has_go {
        out.push_str("# Go\n");
        out.push_str("/bin/\n");
        out.push_str("/dist/\n");
        out.push_str("vendor/\n\n");
    }

    if has_php {
        out.push_str("# PHP / Composer\n");
        out.push_str("/vendor/\n\n");
    }

    if has_java {
        out.push_str("# Java / Gradle / Maven\n");
        out.push_str("target/\n");
        out.push_str("build/\n");
        out.push_str(".gradle/\n\n");
    }

    if has_c_cpp {
        out.push_str("# C / C++ / Build\n");
        out.push_str("build/\n");
        out.push_str("bin/\n");
        out.push_str("*.o\n");
        out.push_str("*.a\n");
        out.push_str("*.so\n");
        out.push_str("*.dylib\n\n");
    }

    if !has_node && !has_rust && !has_python && !has_go && !has_php && !has_java && !has_c_cpp {
        out.push_str("# Generic Common Ignore Patterns\n");
        out.push_str("node_modules/\n");
        out.push_str("dist/\n");
        out.push_str("build/\n");
        out.push_str("target/\n");
        out.push_str(".cache/\n");
        out.push_str("__pycache__/\n");
        out.push_str(".venv/\n");
        out.push_str("venv/\n");
        out.push_str("*.log\n\n");
    }

    out
}

/// Ensures that a workspace root has a git repository initialized and a tailored .gitignore file.
/// If .git does not exist, runs `git init`.
/// If .gitignore does not exist, inspects project files and generates a smart .gitignore.
pub fn ensure_git_and_gitignore(root_path: &Path) -> Result<()> {
    if !root_path.exists() || !root_path.is_dir() {
        return Ok(());
    }

    // 1. Check & init git if not present
    let git_dir = root_path.join(".git");
    if !git_dir.exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("init").current_dir(root_path);
        crate::engine::skills::configure_subprocess(&mut cmd);
        match cmd.output() {
            Ok(out) => {
                if out.status.success() {
                    println!("[Agent Workspace] Initialized git repo at: {:?}", root_path);
                } else {
                    eprintln!(
                        "[Agent Workspace] Failed to run 'git init' at {:?}: {}",
                        root_path,
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
            }
            Err(e) => {
                eprintln!("[Agent Workspace] Could not execute 'git init' at {:?}: {}", root_path, e);
            }
        }
    }

    // 2. Check & create smart .gitignore if missing
    let gitignore_path = root_path.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = generate_smart_gitignore(root_path);
        if let Err(e) = std::fs::write(&gitignore_path, gitignore_content) {
            eprintln!("[Agent Workspace] Failed to write smart .gitignore at {:?}: {}", gitignore_path, e);
        } else {
            println!("[Agent Workspace] Created smart .gitignore at: {:?}", gitignore_path);
        }
    }

    Ok(())
}

/// Builds a cross-platform shell Command with an enriched PATH environment so tools like
/// npm, npx, pnpm, yarn, node, cargo, python, etc. are reliably found across Linux, macOS, and Windows.
pub fn create_workspace_command(command_str: &str) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("powershell");
        crate::engine::skills::configure_subprocess(&mut cmd);
        cmd.arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-WindowStyle")
            .arg("Hidden")
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
