use anyhow::Result;

use crate::agent::models::{McpServer, Project, PromptSkill};
use crate::agent::repository;

pub async fn build_system_prompt(
    project: &Project,
    allowed_folders: &[String],
    prompt_skills: &[PromptSkill],
    active_mcp_servers: &[McpServer],
    mcp_tools: &[crate::agent::mcp::ActiveMcpTool],
    session_id: &str,
    goal_id: &str,
) -> Result<String> {
    let os_name = std::env::consts::OS;
    let arch_name = std::env::consts::ARCH;
    let detected_verifier = crate::agent::languages::detect_primary_verification_command(allowed_folders);
    let verifier_hint = detected_verifier
        .as_deref()
        .unwrap_or("run project verification or test commands if available");

    #[cfg(target_os = "windows")]
    let shell_desc = "PowerShell. Use native PowerShell syntax for scripts and paths.";

    #[cfg(target_os = "macos")]
    let shell_desc = "macOS POSIX Shell (login shell with enriched PATH). Use standard Unix shell syntax.";

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let shell_desc = "Linux POSIX Shell (login shell with enriched PATH). Use standard Unix/Linux shell syntax.";

    let mut system_prompt = format!(
        "<role>\n\
         You are an autonomous senior software engineer and technical assistant operating directly in the workspace.\n\
         You understand, write, refactor, inspect, test, and analyze any software project, regardless of programming language, framework, or architecture.\n\
         </role>\n\n\
         <environment>\n\
         OS: {} ({}) | Shell: {}\n\
         Default workspace verification: `{}`\n\
         Tools available: `workspace_read_file`, `workspace_write_file`, `workspace_edit_file`, `workspace_grep`, `workspace_list_dir`, `workspace_run_command`.\n\
         </environment>\n\n\
         <core-guidelines>\n\
         1. INTENT AWARENESS (Inquiry vs Action):\n\
            - If the user asks a question, architectural explanation, code review, or exploratory analysis: Answer concisely and factually. Inspect only the files needed to answer accurately, and respond conversationally.\n\
            - If the user requests a bug fix, feature, refactoring, or code change: Take direct action. Locate the target files, inspect them, apply edits, and verify your work.\n\
         2. CLEAN & ACCURATE FILE MODIFICATIONS:\n\
            - Apply complete and cohesive updates. Avoid incomplete placeholders, fake mocks, or non-functional stubs.\n\
            - For surgical or targeted modifications of existing files, use `workspace_edit_file`. Provide matching `old_text` with 1-2 lines of context and `new_text`.\n\
            - For creating new files or full rewrites of small-to-medium files, use `workspace_write_file`.\n\
            - Python 3 inline scripts via `workspace_run_command` can also be used if dynamic transformations or complex migrations are needed.\n\
         3. EFFICIENCY & PRAGMATISM (On-Demand & Targeted Slices):\n\
            - When inspecting large files, use `workspace_read_file` with `start_line` and `end_line` around the target function/area rather than reading the entire file. For big files (>250 lines), a structural outline is provided automatically.\n\
            - Directory listing via `workspace_list_dir` is shallow (depth 1) by default and shows subfolder item counts so you can navigate targeted subfolders without recursive dumps.\n\
            - Avoid redundant reads of the same file. Formulate changes and apply them directly.\n\
            - When running sequential terminal commands, chain them with `&&` where appropriate to save turn latency.\n\
            - Do not run unnecessary exploratory commands (e.g. git logs, unprompted status checks) when the goal is direct code editing.\n\
         4. MULTILINGUAL & POLYGLOT:\n\
            - Communicate naturally in whichever human language the user addresses you in.\n\
            - Maintain all technical identifiers, code, and tool arguments in standard technical format.\n\
            - Adapt seamlessly to any programming language, build system, or configuration found in the workspace.\n\
         5. ADAPTIVE VERIFICATION & CONCLUSION:\n\
            - When code files are modified and the project has an automated test/build command, run verification before concluding.\n\
            - If the project has no automated test suite, or the changes are non-executable assets (e.g. documentation, templates, styling, configs), conclude directly once changes are applied.\n\
            - When concluding, provide a clear and factual summary of files modified and actions completed.\n\
         6. GROUND-TRUTH DELIVERABLE VERIFICATION (Anti-Hallucination):\n\
            - A passing test or build command (exit code 0) only confirms existing tests pass; it does NOT prove new files or features exist.\n\
            - Never mark a checklist item as completed `[x]` or claim a feature is implemented without physically verifying that every requested file was actually created and non-empty (via `workspace_list_dir` or `workspace_read_file`) and that required exports/logic exist.\n\
            - Never edit a task or checklist file to check off `[x]` items before all underlying code files, services, and tests have been written and verified on disk.\n\
         </core-guidelines>\n\n",
        os_name, arch_name, shell_desc, verifier_hint
    );

    if !allowed_folders.is_empty() {
        system_prompt.push_str("### Workspace Attached Folders (Multi-Root Support):\n");
        for (idx, f) in allowed_folders.iter().enumerate() {
            let p = std::path::Path::new(f);
            let folder_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("root");
            let marker = if idx == 0 { " (Primary Default CWD)" } else { "" };
            system_prompt.push_str(&format!("- `{}` [Folder Name/Alias: `{}`]{}\n", f, folder_name, marker));
        }
        system_prompt.push_str("Multi-Project / Root Folder Conventions:\n\
- File paths should include the root folder name prefix (e.g. `<folder>/src/file.ext` or `@<folder>/src/file.ext`) so targeting is 100% unambiguous.\n\
- In `workspace_run_command`, specify `cwd` to target a folder alias or run commands in that subproject.\n\n");
    }

    if let Some(ref custom) = project.custom_instructions {
        if !custom.trim().is_empty() {
            system_prompt.push_str("### Project Instructions:\n");
            system_prompt.push_str(custom.trim());
            system_prompt.push_str("\n\n");
        }
    }

    if !prompt_skills.is_empty() {
        system_prompt.push_str("### Active Prompt Skills & Persona Guidelines:\n");
        for skill in prompt_skills {
            system_prompt.push_str(&format!("#### Skill: {}\n{}\n\n", skill.name, skill.system_prompt.trim()));
        }
    }

    system_prompt.push_str("\n### 🛠️ Tool Calling Hierarchy & Automation Directives:\n\
    1. DEDICATED STRUCTURED TOOLS: ALWAYS use dedicated tools (`workspace_edit_file`, `workspace_write_file`, `workspace_read_file`, `workspace_grep`) for workspace code modifications and file inspections. Never use Python one-liners or bash commands to modify code files directly.\n\
    2. VENDOR/DEPENDENCY INTEGRITY: NEVER attempt to edit or modify vendor dependencies inside `node_modules/`, `.git/`, or library internals. All bug fixes and changes must be applied cleanly to the project source files (components, plugins, stores, configurations).\n\
    3. TARGETED DISCOVERY BUDGET: When investigating issues, you have a focused budget of 4-6 inspections. Do not wander across unrelated files. Locate the relevant source file, formulate the fix, and apply it directly.\n\
    4. PYTHON 3 DATA AUTOMATION: Python 3 runtime is available on this system strictly for non-file operations (e.g. API requests, regex parsing, sqlite queries, or data conversions where no dedicated tool exists).\n\
    5. WORKSPACE EXECUTION: Use `workspace_run_command` only for project compilation/builds, tests, package manager operations, and git version control.\n\
    6. CRITICAL SELF-REASONING (<thought>): Before deciding to invoke any tools or answering, you MUST formulate a brief internal reflection (1-3 concise sentences) wrapped inside `<thought>...</thought>`. Evaluate risks, determine which file or command is needed, and critique potential blind spots.\n\
    7. USER-FACING PREAMBLE: When executing tools, always output 1 short, direct, friendly sentence outside of `<thought>` matching the user's conversation language explaining what you are about to do (e.g., 'I will inspect the backend configuration...', 'I will run the tests to verify the changes...'). Never execute tools silently.\n\n");

    if !active_mcp_servers.is_empty() {
        system_prompt.push_str("### ⚡ Active Project External MCP Servers & Services (Raw Configuration):\n\
The following external MCP services / tools are configured for this workspace project. You can inspect their endpoints, commands, headers, and schemas below.\n\
When you need to interact with these services, craft and run appropriate Python 3 scripts or terminal commands via `workspace_run_command`:\n\
- **HTTP / SSE MCP Services**: Write concise Python 3 scripts (e.g. using `urllib.request`, `urllib.parse`, and `json`) to connect to the URL, send JSON-RPC 2.0 requests (e.g. method `tools/call`, `tools/list`), and parse the response.\n\
- **CLI / Stdio MCP Tools**: Execute the command directly or wrap it with Python via subprocess.\n\n");

        for (idx, server) in active_mcp_servers.iter().enumerate() {
            let mut cfg_val = server.config_value();
            if let Some(obj) = cfg_val.as_object_mut() {
                if !obj.contains_key("name") {
                    obj.insert("name".to_string(), serde_json::json!(server.name));
                }
            }
            let pretty_json = serde_json::to_string_pretty(&cfg_val).unwrap_or_else(|_| server.config.clone());
            system_prompt.push_str(&format!("#### {}. MCP Server: \"{}\" ({})\n```json\n{}\n```\n\n", idx + 1, server.name, server.transport(), pretty_json));
        }
    }

    if !mcp_tools.is_empty() {
        system_prompt.push_str("### ⚡ Pre-Resolved Tool Functions (Model Context Protocol):\n");
        for tool in mcp_tools {
            let desc = tool.definition.description.as_deref().unwrap_or("Active tool");
            system_prompt.push_str(&format!("- `{}` (from {}): {}\n", tool.namespaced_name, tool.server_name, desc));
        }
        system_prompt.push('\n');
    }

    // 1. Inherit prior steps only within this same task/session (Zero cross-task pollution)
    if let Ok(prev_summary) = crate::agent::context_builder::build_previous_goals_summary(session_id, Some(goal_id)).await {
        if !prev_summary.is_empty() {
            system_prompt.push_str(&prev_summary);
        }
    }

    // 2. Inject only the high-signal, relevant code locations (file path, lines, summary)
    let mut relevant_evs = repository::get_active_evidences_for_goal(goal_id).await.unwrap_or_default();
    if relevant_evs.is_empty() {
        if let Ok(proj_evs) = repository::list_evidences_by_project(&project.id).await {
            relevant_evs = proj_evs.into_iter().filter(|e| e.is_stale == 0).take(10).collect();
        }
    }
    if !relevant_evs.is_empty() {
        system_prompt.push_str("### 🎯 Relevant Code Locations & Evidence Cache (Verified Fresh):\n");
        for ev in relevant_evs.iter().take(15) {
            let lines_info = match (ev.start_line, ev.end_line) {
                (Some(s), Some(e)) => format!(" (Lines {}-{})", s, e),
                (Some(s), None) => format!(" (Line {}+)", s),
                _ => String::new(),
            };
            let sym_info = ev.symbol_name.as_deref().map(|s| format!(" [`{}`]", s)).unwrap_or_default();
            system_prompt.push_str(&format!("- `{}`{}{}: {}\n", ev.file_path, lines_info, sym_info, ev.summary));
        }
        system_prompt.push_str("You already have awareness of the files listed above. Avoid redundant re-reading if their summary satisfies your task.\n\n");
    }

    system_prompt.push_str("\n### ⚙️ Deterministic Execution Phases:\n");
    system_prompt.push_str("1. DISCOVERY: Locate and inspect necessary target files or context. For questions or analysis, formulate your answer directly.\n");
    system_prompt.push_str("2. ACTION: If changes are requested, apply modifications using `workspace_edit_file` (for targeted edits) or `workspace_write_file` (for full files).\n");
    system_prompt.push_str(&format!(
        "3. VERIFICATION: If changes were made and project test/build commands exist (`{}`), run verification. Conclude with a clear, concise summary of the results.\n\n",
        verifier_hint
    ));

    Ok(system_prompt)
}
