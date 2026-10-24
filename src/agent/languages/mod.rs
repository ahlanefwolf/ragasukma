pub mod c_cpp;
pub mod csharp;
pub mod dart;
pub mod elixir;
pub mod golang;
pub mod java;
pub mod kotlin;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;
pub mod zig;

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectLanguageInfo {
    pub language: &'static str,
    pub primary_verifier: String,
    pub secondary_verifiers: Vec<String>,
}

pub trait LanguageDetector: Send + Sync {
    fn detect(&self, root: &Path) -> Option<ProjectLanguageInfo>;
}

/// Registry of all language detectors
fn get_detectors() -> Vec<Box<dyn LanguageDetector>> {
    vec![
        Box::new(rust::RustDetector),
        Box::new(node::NodeDetector),
        Box::new(python::PythonDetector),
        Box::new(golang::GoDetector),
        Box::new(dart::DartDetector),
        Box::new(csharp::CSharpDetector),
        Box::new(java::JavaDetector),
        Box::new(kotlin::KotlinDetector),
        Box::new(c_cpp::CCppDetector),
        Box::new(php::PhpDetector),
        Box::new(ruby::RubyDetector),
        Box::new(elixir::ElixirDetector),
        Box::new(swift::SwiftDetector),
        Box::new(zig::ZigDetector),
    ]
}

/// Scans workspace roots and identifies all project languages present,
/// including root manifests and immediate 1-level subprojects (e.g. backend, frontend, server, client, web).
pub fn detect_all_languages(allowed_folders: &[String]) -> Vec<ProjectLanguageInfo> {
    let detectors = get_detectors();
    let mut results = Vec::new();

    for folder in allowed_folders {
        let root = Path::new(folder);
        if !root.exists() || !root.is_dir() {
            continue;
        }

        // 1. First, check root directory directly
        for detector in &detectors {
            if let Some(info) = detector.detect(root) {
                if !results.iter().any(|r: &ProjectLanguageInfo| r.primary_verifier == info.primary_verifier) {
                    results.push(info);
                }
            }
        }

        // 2. Scan immediate 1-level subdirectories (e.g. backend/, frontend/, client/, server/, web/, etc.)
        // This is essential for monorepos, fullstack apps, and multi-component workspaces
        if let Ok(entries) = std::fs::read_dir(root) {
            let mut subdirs = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.')
                        && name != "node_modules"
                        && name != "target"
                        && name != "dist"
                        && name != "build"
                        && name != "vendor"
                        && name != "venv"
                        && name != ".venv"
                        && name != "cache"
                        && name != "tmp"
                    {
                        subdirs.push((name, path));
                    }
                }
            }
            subdirs.sort_by(|a, b| a.0.cmp(&b.0));

            for (sub_name, sub_path) in subdirs {
                for detector in &detectors {
                    if let Some(mut info) = detector.detect(&sub_path) {
                        info.primary_verifier = format!("cd {} && {}", sub_name, info.primary_verifier);
                        info.secondary_verifiers = info
                            .secondary_verifiers
                            .into_iter()
                            .map(|v| format!("cd {} && {}", sub_name, v))
                            .collect();

                        if !results.iter().any(|r: &ProjectLanguageInfo| r.primary_verifier == info.primary_verifier) {
                            results.push(info);
                        }
                    }
                }
            }
        }
    }

    results
}

/// Returns the primary verification command for the detected workspace.
/// For multi-component workspaces, chains each component verifier in a subshell.
pub fn detect_primary_verification_command(allowed_folders: &[String]) -> Option<String> {
    let languages = detect_all_languages(allowed_folders);
    if languages.is_empty() {
        None
    } else if languages.len() == 1 {
        Some(languages[0].primary_verifier.clone())
    } else {
        // Multi-component: chain them together in subshells so cwd changes do not interfere
        let chained = languages
            .into_iter()
            .map(|l| {
                if l.primary_verifier.starts_with("cd ") {
                    format!("({})", l.primary_verifier)
                } else {
                    l.primary_verifier
                }
            })
            .collect::<Vec<_>>()
            .join(" && ");
        Some(chained)
    }
}

/// Returns the verification commands relevant to the modified files, or all detected primary verifiers.
pub fn detect_verifiers_for_modified_files(
    allowed_folders: &[String],
    modified_files: &[String],
) -> Vec<String> {
    let languages = detect_all_languages(allowed_folders);
    if languages.is_empty() {
        return Vec::new();
    }

    if modified_files.is_empty() {
        return languages.into_iter().map(|l| l.primary_verifier).collect();
    }

    let mut relevant = Vec::new();
    for lang in &languages {
        if let Some(rest) = lang.primary_verifier.strip_prefix("cd ") {
            if let Some((sub_name, _)) = rest.split_once(" && ") {
                let sub_trimmed = sub_name.trim();
                let is_modified = modified_files.iter().any(|f| {
                    let clean = f.trim_start_matches("./");
                    clean.starts_with(sub_trimmed)
                        || clean.contains(&format!("/{}/", sub_trimmed))
                        || clean.contains(&format!("{}/", sub_trimmed))
                });
                if is_modified && !relevant.contains(&lang.primary_verifier) {
                    relevant.push(lang.primary_verifier.clone());
                }
            }
        } else {
            // Root project verifier
            if !relevant.contains(&lang.primary_verifier) {
                relevant.push(lang.primary_verifier.clone());
            }
        }
    }

    if relevant.is_empty() {
        languages.into_iter().map(|l| l.primary_verifier).collect()
    } else {
        relevant
    }
}

/// Checks if an executed shell command is a verification / test / typecheck command
pub fn is_verification_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();

    // Handle chained shell commands e.g. "cd simulations/python_scenario && python3 -m unittest discover -s tests"
    let sub_commands: Vec<&str> = lower
        .split(['&', ';', '|'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    for sub in sub_commands {
        let sub_trimmed = sub.trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace());

        let known_prefixes = [
            "cargo test", "cargo check", "cargo clippy", "cargo build",
            "npm test", "npm run test", "npm run build", "npm run check", "npm run typecheck", "npm run lint",
            "pnpm test", "pnpm run test", "pnpm run build", "pnpm run typecheck",
            "yarn test", "yarn run test", "yarn run build", "yarn run typecheck",
            "bun test", "bun run build", "bun run check",
            "deno test", "deno check", "deno lint",
            "node --test", "node --experimental-strip-types --test", "node test",
            "npx tsc", "tsc",
            "vite build",
            "pytest",
            "python -m unittest", "python3 -m unittest", "py -m unittest",
            "python -m pytest", "python3 -m pytest", "py -m pytest",
            "python -m", "python3 -m",
            "uv run pytest", "poetry run pytest", "pipenv run pytest",
            "uv run", "poetry run",
            "go test", "go vet", "go build",
            "flutter test", "flutter analyze", "dart test", "dart analyze",
            "dotnet test", "dotnet build",
            "gradle test", "./gradlew test", "mvn test", "mvn verify",
            "ctest", "cmake --build", "make test", "make check", "meson test",
            "php artisan test", "./vendor/bin/phpunit", "composer test", "./vendor/bin/pest",
            "bundle exec rspec", "bundle exec rake test", "rake test",
            "mix test",
            "swift test", "swift build",
            "zig build test", "zig build",
            "vitest", "jest",
        ];

        if known_prefixes.iter().any(|p| sub_trimmed.starts_with(p)) {
            return true;
        }

        // Also check if executing a test script directly, e.g. "python3 tests/test_billing.py" or "node tests/cart.test.js"
        if (sub_trimmed.starts_with("python") || sub_trimmed.starts_with("python3") || sub_trimmed.starts_with("node"))
            && (sub_trimmed.contains("test") || sub_trimmed.contains("unittest") || sub_trimmed.contains("pytest") || sub_trimmed.contains("check"))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rust_detection() {
        let temp = std::env::temp_dir().join("test_agent_rust_proj");
        let _ = fs::create_dir_all(&temp);
        fs::write(temp.join("Cargo.toml"), "[package]\nname = \"foo\"").unwrap();

        let detected = detect_all_languages(&[temp.to_string_lossy().to_string()]);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].language, "Rust");
        assert_eq!(detected[0].primary_verifier, "cargo test");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_is_verification_command() {
        assert!(is_verification_command("cargo test --lib"));
        assert!(is_verification_command("npm test"));
        assert!(is_verification_command("npx tsc --noEmit"));
        assert!(is_verification_command("pytest tests/"));
        assert!(is_verification_command("python3 -m unittest discover -s tests"));
        assert!(is_verification_command("cd simulations/python_scenario && python3 -m unittest discover -s tests"));
        assert!(is_verification_command("node --test tests/*.test.js"));
        assert!(is_verification_command("go test ./..."));
        assert!(is_verification_command("flutter test"));
        assert!(is_verification_command("dotnet test"));
        assert!(is_verification_command("(cd backend && cargo test) && (cd frontend && npm run build)"));
        assert!(!is_verification_command("git status"));
        assert!(!is_verification_command("ls -la"));

        let failing_json = r#"{"command":"npx tsc --noEmit","cwd":"/home/headway/MyWork/project/portofolio","error_banner":"❌ COMMAND FAILED with exit code 2.","exit_code":2,"status":"FAILED","stderr":"","stdout":"src/App.tsx(28,5): error TS2741","success":false}"#;
        assert!(crate::agent::executor::is_command_output_error(failing_json));

        let success_json = r#"{"command":"npx tsc --noEmit","cwd":"/home/headway/MyWork/project/portofolio","exit_code":0,"stderr":"","stdout":"","success":true}"#;
        assert!(!crate::agent::executor::is_command_output_error(success_json));
    }

    #[test]
    fn test_multi_subproject_detection() {
        let temp = std::env::temp_dir().join("test_agent_fullstack_proj");
        let backend = temp.join("backend");
        let frontend = temp.join("frontend");
        let _ = fs::create_dir_all(&backend);
        let _ = fs::create_dir_all(&frontend);

        fs::write(backend.join("Cargo.toml"), "[package]\nname = \"backend\"").unwrap();
        fs::write(
            frontend.join("package.json"),
            r#"{"name": "frontend", "scripts": {"check": "tsc --noEmit", "build": "vite build"}}"#,
        )
        .unwrap();

        let allowed = vec![temp.to_string_lossy().to_string()];
        let detected = detect_all_languages(&allowed);
        assert_eq!(detected.len(), 2);
        assert_eq!(detected[0].primary_verifier, "cd backend && cargo test");
        assert_eq!(detected[1].primary_verifier, "cd frontend && npm run check && npm run build");

        let primary_chained = detect_primary_verification_command(&allowed);
        assert_eq!(
            primary_chained,
            Some("(cd backend && cargo test) && (cd frontend && npm run check && npm run build)".to_string())
        );

        // Test file scoping
        let mod_frontend = vec!["frontend/src/App.tsx".to_string()];
        let scoped_front = detect_verifiers_for_modified_files(&allowed, &mod_frontend);
        assert_eq!(scoped_front, vec!["cd frontend && npm run check && npm run build".to_string()]);

        let mod_both = vec!["backend/src/main.rs".to_string(), "frontend/src/App.tsx".to_string()];
        let scoped_both = detect_verifiers_for_modified_files(&allowed, &mod_both);
        assert_eq!(
            scoped_both,
            vec![
                "cd backend && cargo test".to_string(),
                "cd frontend && npm run check && npm run build".to_string()
            ]
        );

        let _ = fs::remove_dir_all(&temp);
    }
}
