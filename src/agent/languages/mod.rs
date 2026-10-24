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

/// Scans workspace roots and identifies all project languages present
pub fn detect_all_languages(allowed_folders: &[String]) -> Vec<ProjectLanguageInfo> {
    let detectors = get_detectors();
    let mut results = Vec::new();

    for folder in allowed_folders {
        let root = Path::new(folder);
        if !root.exists() || !root.is_dir() {
            continue;
        }

        for detector in &detectors {
            if let Some(info) = detector.detect(root) {
                if !results.iter().any(|r: &ProjectLanguageInfo| r.language == info.language) {
                    results.push(info);
                }
            }
        }
    }

    results
}

/// Returns the primary verification command for the detected workspace
pub fn detect_primary_verification_command(allowed_folders: &[String]) -> Option<String> {
    let languages = detect_all_languages(allowed_folders);
    languages.first().map(|l| l.primary_verifier.clone())
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
        let sub_trimmed = sub.trim_start_matches(|c: char| c == '(' || c.is_whitespace());

        let known_prefixes = [
            "cargo test", "cargo check", "cargo clippy", "cargo build",
            "npm test", "npm run test", "npm run build", "npm run check", "npm run typecheck", "npm run lint",
            "pnpm test", "pnpm run test", "pnpm run build", "pnpm run typecheck",
            "yarn test", "yarn run test", "yarn run build", "yarn run typecheck",
            "bun test", "bun run build",
            "deno test", "deno check", "deno lint",
            "node --test", "node --experimental-strip-types --test", "node test",
            "npx tsc", "tsc",
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
        assert!(!is_verification_command("git status"));
        assert!(!is_verification_command("ls -la"));

        let failing_json = r#"{"command":"npx tsc --noEmit","cwd":"/home/headway/MyWork/project/portofolio","error_banner":"❌ COMMAND FAILED with exit code 2.","exit_code":2,"status":"FAILED","stderr":"","stdout":"src/App.tsx(28,5): error TS2741","success":false}"#;
        assert!(crate::agent::executor::is_command_output_error(failing_json));

        let success_json = r#"{"command":"npx tsc --noEmit","cwd":"/home/headway/MyWork/project/portofolio","exit_code":0,"stderr":"","stdout":"","success":true}"#;
        assert!(!crate::agent::executor::is_command_output_error(success_json));
    }
}
