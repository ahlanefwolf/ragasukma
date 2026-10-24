use anyhow::{Context, Result};
use serde_json::json;
use std::collections::{HashMap, HashSet};

use super::models::{AgentSessionEvent, Message};
use super::repository;
use super::tools::{execute_tool_call, get_agent_tools_schema};
use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::codex::CodexProvider;
use crate::llm::v1_engine::{V1ChatRequest, V1Message, V1StreamEvent, V1ToolCall, V1ToolCallFunction};

// Re-export submodules for full backwards compatibility
pub use super::compressor::*;
pub use super::diagnostics::*;
pub use super::events::*;

pub fn parse_project_folders(root_path: Option<&str>) -> Vec<String> {
    let Some(raw) = root_path else {
        return Vec::new();
    };
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(raw) {
        return arr;
    }
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        vec![trimmed.to_string()]
    } else {
        Vec::new()
    }
}

pub async fn execute_session_turn(session_id: &str, model_override: Option<&str>) -> Result<String> {
    // 1. Load Session & Project
    let session = repository::get_session(session_id)
        .await?
        .context("Session not found")?;

    let project = repository::get_project(&session.project_id)
        .await?
        .context("Project not found")?;

    let allowed_folders = parse_project_folders(project.root_path.as_deref());
    for folder in &allowed_folders {
        let _ = super::tools::ensure_git_and_gitignore(std::path::Path::new(folder));
    }
    let prompt_skills = repository::get_project_skills(&project.id).await.unwrap_or_default();

    // 2. Mark session as running
    let _ = repository::update_session_status(session_id, "running").await;
    broadcast_session_event(
        session_id,
        AgentSessionEvent::StatusChange {
            status: "running".to_string(),
        },
    );

    // 3. Assemble Dense RFC 2119 System Prompt with Host OS & Environment Awareness
    let os_name = std::env::consts::OS;
    let arch_name = std::env::consts::ARCH;
    let detected_verifier = super::languages::detect_primary_verification_command(&allowed_folders);
    let verifier_hint = detected_verifier
        .as_deref()
        .unwrap_or("run appropriate test/check commands for this workspace");

    #[cfg(target_os = "windows")]
    let shell_desc = "PowerShell (ExecutionPolicy Bypass). Commands MUST use PowerShell syntax (e.g. `npm test`, `cargo check`, `$env:VAR`, `dir`).";

    #[cfg(target_os = "macos")]
    let shell_desc = "macOS POSIX Shell (login shell with enriched PATH including Homebrew, NVM, and Cargo). Commands MUST use Unix shell syntax.";

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let shell_desc = "Linux POSIX Shell (login shell with enriched PATH including NVM, FNM, Cargo, and local binaries). Commands MUST use standard Linux shell syntax.";

    let mut system_prompt = format!(
        "<system-conventions>\n\
         RFC 2119 applies: MUST, REQUIRED, NEVER, SHOULD, RECOMMENDED. NEVER and AVOID MUST be interpreted as aliases for MUST NOT and SHOULD NOT.\n\
         </system-conventions>\n\n\
         <role>\n\
         You are THE autonomous staff engineer and versatile coding assistant operating directly in the workspace environment (inspired by oh-my-pi architecture).\n\
         You handle any software engineering, refactoring, text-to-text generative task, writing, configuration, analysis, and data transformation.\n\
         You inspect files, explore directories, search patterns, run terminal commands, and modify code using provided workspace tools.\n\
         </role>\n\n\
         <environment>\n\
         Operating System: {} ({})\n\
         Default Shell: {}\n\
         Terminal Capability: `workspace_run_command` executes directly in this shell with access to installed development tools (npm, npx, pnpm, yarn, node, cargo, python, pip, git, etc.). Adapt command syntax, flags, and paths specifically to this operating system.\n\
         </environment>\n\
         <engineering-philosophy>\n\
         - Evidence-first terse engineer: every sentence fact, decision, or concrete risk. Zero filler, zero ceremony, zero marketing, zero hedging, zero over-politeness.\n\
         - Correctness first, then maintainability. Clean cutover: update every caller, remove obsolete code/imports/deprecated paths, no temporary shims.\n\
         - User-reported errors or observations are ground truth; NEVER rerun commands just to re-confirm what the user already stated.\n\
         - Pragmatic execution bias: when given an operational directive (e.g. push, commit, build, test, run, deploy), execute immediately with chained commands. Do not delay with redundant exploration or deferential questions.\n\
         </engineering-philosophy>\n\n\
         <completeness-contract>\n\
         - MUST keep full objective intact across turns. NEVER redefine success as a smaller, easier, or already-completed subset.\n\
         - When instructed to perform global tasks (e.g. \"all text to i18n\", \"all components\", \"refactor all endpoints\"), you MUST discover and update EVERY target artifact thoroughly. NEVER stop after touching just 1 or 2 files!\n\
         - NEVER deliver unfinished work: stubs, mocks, no-ops, fake fallbacks, `TODO: implement`, or misleading \"scaffold/MVP\" claims.\n\
         - Audit current repo state against every deliverable before concluding. If any part lacks direct current-state code edits, keep working!\n\
         - Strict goal completion: NEVER declare a goal complete prematurely. Ensure all operations, edits, and chained commands have executed with 0 errors before concluding.\n\
         </completeness-contract>\n\n\
         <multilingual-interaction>\n\
         - Fully multilingual: The user may interact in any language (e.g. Indonesian, English, Spanish, Japanese). Understand intent accurately regardless of phrasing, idioms, or rhetorical style.\n\
         - Respond in the language used by the user, while keeping all code, comments, variables, identifiers, tool arguments, and technical execution in standard English.\n\
         </multilingual-interaction>\n\n\
         <critical>\n\
         1. UNRESTRICTED WORKSPACE AUTONOMY: You have full, unrestricted permission to perform any actions within the workspace. You can inspect files, run terminal commands, and modify files directly without asking for permission.\n\
         2. IMMEDIATE DIRECT ACTION & EDITING: When instructed to generate, create, fix, modify, transform, or add content (code, text, config, docs, markdown, translations), you MUST apply the changes directly using `workspace_edit_file` or `workspace_write_file`. Once you identify the target files or locations, apply the modifications IMMEDIATELY. Do NOT hesitate or postpone edits across multiple turns.\n\
         3. DISTINGUISH BUG FIXING VS FEATURE IMPLEMENTATION (NO SPECULATIVE PRE-BUILDS):\n\
            - BUG / ERROR REPORTS: When investigating any bug, compiler failure, broken export, runtime crash, or user-reported error: Run the project compiler, typechecker, or test command directly (`{verifier_hint}`) via `workspace_run_command` as your FIRST step to localize exact line numbers in seconds.\n\
            - FEATURE IMPLEMENTATION / TASK SPECS / REFACTORING: When instructed to add a feature, create/modify functionality, refactor code, or implement requirements from a task spec (without any reported compiler/runtime error): NEVER run preliminary build/test commands before editing! Running build/test on untouched code wastes turns and tokens. Go STRAIGHT to inspecting the relevant source file with `workspace_read_file` and applying changes with `workspace_edit_file`. Only run verification AFTER modifications are made!\n\
         4. IMMEDIATE ACTION BIAS (NO IDLE BROWSING OR REDUNDANT RE-READING):\n\
            - Never perform more than 2 consecutive file reads before applying an edit or running a command.\n\
            - Once a target file is located and read, your NEXT tool call MUST be `workspace_edit_file` or `workspace_write_file`.\n\
            - Formulate your changes and apply them immediately. Do NOT re-read the same file multiple times before editing.\n\
         5. BUG AND ERROR REPORTS REQUIRE IMMEDIATE FIXES: When the user reports an error, crash, bug, or unexpected behavior (regardless of language, phrasing, or question marks), NEVER ask 'Do you want me to fix this?' or provide only a conversational diagnosis. You MUST immediately find the file, edit the code using `workspace_edit_file` or `workspace_write_file`, and verify the fix.\n\
         6. COMMAND CHAINING & PIPELINING WITH '&&': For any sequential multi-step terminal operation (e.g. `git add -A && git commit -m \"...\" && git push`, `npm run build && npm test`, `cargo check && cargo test`, or running setup then test), ALWAYS chain them into a single `workspace_run_command` invocation using `&&` (or shell equivalent). NEVER split sequential commands across separate turns when they can be safely chained. This drastically saves tokens, minimizes turn latency, and completes operations in a single swift execution.\n\
         7. PRAGMATIC DIRECT EXECUTION (NO OVER-POLITENESS OR HESITATION): When given a direct operational directive (e.g. \"push\", \"commit and push\", \"run test\", \"build\", \"format\", \"deploy\", \"clean\"):\n\
            - Execute the chained command sequence IMMEDIATELY on your very first tool call.\n\
            - NEVER run 4-5 exploratory read-only commands (like checking git remotes, branch status, commit logs, configs, or directory trees) when the action is straightforward.\n\
            - NEVER be overly polite, deferential, or ask trivial confirmation (e.g. 'Should I push now?'). Directly chain and execute the necessary operational sequence (e.g. `git add -A && git commit -m \"...\" && git push && git push gitlab main` or `git status -s && git add -A && git commit -m \"...\" && git push`).\n\
            - Once the command succeeds with exit code 0, state the factual result concisely and conclude immediately.\n\
         8. OPERATIONAL FOCUS & NO PRELIMINARY GIT INSPECTION: NEVER run exploratory commands like `git status`, `git log`, `git branch`, or file listings before starting a coding or operational task unless the user explicitly requested git inspection. When a task or prompt provides file paths or clear objectives, jump directly to reading or editing those files!\n\
         9. FOCUS DIRECTLY ON SOURCE CODE (NO UNNECESSARY CONFIG BROWSING): When tasked with implementing logic, UI elements, API endpoints, or features, do NOT read `Cargo.toml`, `package.json`, `.gitignore`, `tsconfig.json`, or lockfiles unless you explicitly need to add a brand new external library dependency. Inspect only the relevant source code files where the logic lives!\n\
         10. SINGLE-PASS FILE UPDATES: When modifying a file to fulfill a feature or task, apply ALL required changes to that file in ONE single operation. For files under 500 lines (e.g. `main.rs`, `App.tsx`), write the complete updated file via `workspace_write_file` or update all structs, handlers, and tests in a single comprehensive `workspace_edit_file`. NEVER make 3-4 separate partial edits across multiple turns to the same file!\n\
         11. FULL COMPONENT/SECTION READS & BATCHED EDITS:\n\
            - NEVER read files sequentially in tiny 10-30 line slices across multiple turns.\n\
            - Read the entire component, section, or file in a single `workspace_read_file` call (omit `start_line`/`end_line` for files under 500 lines).\n\
            - When a file has multiple compiler/test errors, formulate ALL changes for that file and apply them in ONE comprehensive `workspace_edit_file` or `workspace_write_file` call.\n\
            - When multiple files are broken, you CAN and SHOULD issue multiple `workspace_edit_file` tool calls in parallel within the same turn to fix them simultaneously!\n\
            - When editing structured dictionaries or files with duplicated keys, provide 3-5 lines of context in `old_text` so the target match is 100% unique.\n\
         12. ACCURATE EDITING & EDIT RECOVERY: When calling `workspace_edit_file`, provide 3-6 actual lines as `old_text`. If `workspace_edit_file` reports that 'old_text' was not found or matched multiple locations, immediately inspect the tool error message or surrounding lines and re-apply the edit with a unique enclosing block, or call `workspace_write_file` to write the updated file directly. NEVER wander off into scanning directories or reading unrelated files.\n\
         13. INFORMATIONAL QUESTIONS VS ACTION: Only treat a request as informational if the user is asking for general explanations without indicating that anything needs fixing, creating, or changing. If the user merely asks an informational question, inspect at most 1-2 files or greps and reply directly. But any problem or error report requires editing the code!\n\
         14. NEVER CLAIM WORK DONE WITHOUT EDITING (ZERO HALLUCINATION): You MUST NEVER claim in conversational text that a bug is fixed, a component is updated, or a build passed unless you ACTUALLY executed `workspace_edit_file` or `workspace_write_file` to apply the changes to the disk! Describing solutions without executing the edit tool when asked to fix or build is a critical violation.\n\
         15. NEVER USE SHELL SCRIPTS FOR READING: NEVER invoke shell commands like `sed`, `cat`, `head`, `tail`, or custom script one-liners via `workspace_run_command` just to read or view file lines. Always use `workspace_read_file` or `workspace_grep`.\n\
         16. SUMMARY OF ACCOMPLISHMENTS: Once actions are executed and verified, provide a concise and clear summary of the files modified and commands executed.\n\
         17. NEVER CLAIM SUCCESS ON FAILING CHECKS (ZERO TOLERANCE FOR FALSE SUCCESS): If a compiler, build, or test command printed any errors or returned a non-zero exit code, you MUST NEVER claim success, say 'all done', or conclude your turn. You are strictly forbidden from claiming a task is complete while errors remain in terminal or compiler output. You MUST fix the root causes and re-run verification until it passes cleanly with 0 errors.\n\
         18. MANDATORY VERIFICATION BEFORE CONCLUSION: When you modify or add code, verify your work using `workspace_run_command` (e.g. `{}`) before concluding if a verification command exists. Only conclude when verification checks succeed.\n\
         19. STRICT GOAL COMPLETION (NO PREMATURE GOAL CLAIMS): While direct action and command chaining are required, NEVER conclude early or claim a goal is accomplished if the underlying task has not actually succeeded. Verify that terminal commands exited with code 0 (`success: true`) and requested files on disk are truly modified before declaring success. If any command in a chain fails or reports errors, immediately inspect the failure, fix the root cause, and re-run until it passes.\n\
         20. MULTI-COMPONENT & FULL-STACK COMPLETENESS: When a task specifies work across multiple folders, components, or services (e.g. `backend` AND `frontend`), you MUST complete the deliverables for ALL specified components. NEVER conclude after editing only one component when other components remain untouched. Verify each modified component with its corresponding test/build command (e.g. `cargo test` for backend and `npm run check && npm run build` for frontend), and conclude only when all components are implemented and verified.\n\
         21. NO UNREQUESTED GIT COMMITS OR DUPLICATE TEST RERUNS: Once all modified components are verified with 0 errors (e.g. backend tests pass AND frontend build passes), your work is COMPLETE. Do NOT execute `git init`, `git add`, `git commit`, or duplicate test reruns unless the user explicitly requested git operations in their prompt. Conclude your turn immediately with your final summary!\n\
         </critical>\n\n",
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
        for skill in &prompt_skills {
            system_prompt.push_str(&format!("#### Skill: {}\n{}\n\n", skill.name, skill.system_prompt.trim()));
        }
    }

    // 3b. Determine Goal lifecycle: state-driven without brittle word lists
    let mut db_messages = repository::list_messages(session_id).await?;
    let last_user_prompt = db_messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .and_then(|m| m.content.as_deref())
        .unwrap_or("")
        .to_string();

    // State-driven task indicator: syntactical markers (file paths, @tags, file extensions)
    let is_task_request = last_user_prompt.contains('@')
        || last_user_prompt.contains('/')
        || last_user_prompt.contains('\\')
        || [".md", ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".json", ".toml", ".yaml", ".yml", ".go", ".c", ".cpp", ".java"]
            .iter()
            .any(|ext| last_user_prompt.contains(ext));

    let session_goals = repository::list_session_goals(session_id).await.unwrap_or_default();
    let latest_existing_goal = session_goals.first().cloned();

    // Check if the latest goal is still active / unresolved
    let has_unresolved_active_goal = latest_existing_goal
        .as_ref()
        .map(|g| g.status == "in_progress" || g.status == "initialized" || g.phase == "action" || g.phase == "verification")
        .unwrap_or(false);

    // Check if the latest goal was already completed or failed; if so, create a new one
    let goal_id = match latest_existing_goal {
        Some(ref g) if g.status != "completed" && g.status != "completed_unverified" && g.status != "failed" => {
            // Re-use current active goal if still in progress and user is continuing the task
            g.id.clone()
        }
        _ => {
            // New distinct goal for this user request
            let new_gid = format!("goal_{}_{}", session_id, repository::now_secs());
            let title = if last_user_prompt.chars().count() > 60 {
                format!("{}...", last_user_prompt.chars().take(57).collect::<String>())
            } else if !last_user_prompt.trim().is_empty() {
                last_user_prompt.trim().to_string()
            } else {
                session.title.clone()
            };
            let _ = repository::create_goal(
                &new_gid,
                &project.id,
                Some(session_id),
                latest_existing_goal.as_ref().map(|g| g.id.as_str()),
                &title,
                &last_user_prompt,
                "[]",
                "[]",
            ).await?;
            new_gid
        }
    };

    // 1. INHERIT PRIOR STEPS ONLY WITHIN THIS SAME TASK/SESSION (Zero cross-task pollution)
    if let Ok(prev_summary) = super::context_builder::build_previous_goals_summary(session_id, Some(&goal_id)).await {
        if !prev_summary.is_empty() {
            system_prompt.push_str(&prev_summary);
        }
    }

    // 2. Revalidate evidence cache on disk
    let _ = super::context_builder::revalidate_evidence_cache(&project.id, &allowed_folders).await;

    // 3. Only reuse prior evidence if this goal is explicitly a continuation child of a parent goal
    let active_goal_opt = repository::get_goal(&goal_id).await.unwrap_or(None);
    if let Some(ref g) = active_goal_opt {
        if let Some(ref parent_gid) = g.parent_goal_id {
            let prompt_keywords: Vec<&str> = last_user_prompt
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            let _ = super::context_builder::reuse_evidence_from_parent(&goal_id, parent_gid, &prompt_keywords).await;
        }
    }

    // 3. Inject only the high-signal, relevant code locations (file path, lines, summary)
    // Checks both goal-linked evidence AND general fresh project evidence cache (so recent file discoveries are immediately visible)
    let mut relevant_evs = repository::get_active_evidences_for_goal(&goal_id).await.unwrap_or_default();
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

    // Fetch active goal info to coordinate deterministic phase machine
    let active_goal_opt = repository::get_goal(&goal_id).await.unwrap_or(None);
    // CRITICAL FIX: A fresh user turn MUST ALWAYS start with active execution phase ("discovery" or "action").
    // NEVER inherit a stale "concluded" phase from a prior completed turn, which caused false "Verification has already succeeded" rejections!
    let mut current_phase = active_goal_opt
        .as_ref()
        .map(|g| g.phase.as_str())
        .filter(|p| *p != "concluded")
        .unwrap_or("discovery")
        .to_string();
    let mut locked_targets: Vec<String> = active_goal_opt
        .as_ref()
        .and_then(|g| serde_json::from_str::<Vec<String>>(&g.target_files).ok())
        .unwrap_or_default();
    let mut discovery_counter: i64 = active_goal_opt
        .as_ref()
        .map(|g| g.discovery_count)
        .unwrap_or(0);

    system_prompt.push_str("\n### ⚙️ Deterministic Execution Phases:\n");
    system_prompt.push_str("1. DISCOVERY: Locate and inspect necessary target files. Once located, proceed directly to modifying them.\n");
    system_prompt.push_str("2. ACTION: Apply targeted file modifications using `workspace_edit_file` or `workspace_write_file`.\n");
    system_prompt.push_str(&format!(
        "3. VERIFICATION: After modifying files, verify changes via test or build commands (e.g. `{}`). Once it passes with 0 errors, summarize your completed work.\n\n",
        verifier_hint
    ));

    // 4. Autonomous Tool Execution Loop (aligned with oh-my-pi architecture)
    // Runs autonomously until model has no more tool calls or user pauses/cancels
    let max_safety_iterations = 50;
    let mut iteration = 0;
    let mut ephemeral_directive: Option<String> = None;
    // Reset modified_files and executed_commands for this specific turn so reports only reflect ACTUAL successful edits
    let mut modified_files: Vec<String> = Vec::new();
    let mut executed_commands: Vec<String> = Vec::new();
    let mut consecutive_read_same_file: HashMap<String, usize> = HashMap::new();
    let mut recent_read_slices: HashSet<(String, Option<i64>, Option<i64>)> = HashSet::new();
    let mut global_inspection_streak: usize = 0;
    let mut last_command_had_error: bool = false;
    let mut last_tool_had_error: bool = false;
    let mut pending_error_files: HashSet<String> = HashSet::new();
    let mut files_read_in_full: HashSet<String> = HashSet::new();


    let prior_verifs = repository::list_goal_verifications(&goal_id).await.unwrap_or_default();
    // Fresh turn execution always starts unverified until verified in this active execution loop
    let mut has_passed_verification = false;
    let mut verification_attempts = prior_verifs.len();
    let mut verification_prompted = false;
    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .map(|m| m.to_string())
        .or_else(|| session.model.filter(|m| !m.trim().is_empty()))
        .unwrap_or_else(|| {
            let u = crate::config::load_user_config();
            if !u.llm_model.trim().is_empty() {
                u.llm_model
            } else {
                "ag/gemini-2.5-flash".to_string()
            }
        });

    // Update session model to the latest chosen model
    let _ = repository::update_session_model(session_id, &model).await;
    let is_codex = model.starts_with("cx/") || model.to_lowercase().contains("codex");

    while iteration < max_safety_iterations {
        iteration += 1;

        // Check if session was stopped or aborted externally via lightweight single-column query
        if let Ok(Some(status)) = repository::get_session_status(session_id).await {
            if status != "running" {
                println!("[Agent] Session {} status changed to '{}', breaking iteration loop.", session_id, status);
                break;
            }
        }

        broadcast_session_event(
            session_id,
            AgentSessionEvent::StepInfo {
                iteration,
                action: "thinking".to_string(),
                details: Some(format!(
                    "Model {} is thinking & selecting actions...",
                    model
                )),
            },
        );

        // Calculate total tool messages from in-memory history
        let total_tool_msgs = db_messages.iter().filter(|m| m.role == "tool").count();
        let mut tool_counter = 0;

        let mut v1_messages = Vec::new();

        // Push System prompt first
        v1_messages.push(V1Message {
            role: "system".to_string(),
            content: Some(json!(system_prompt)),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            thought_signature: None,
        });

        // Dynamic Token Budget & Sliding Window Pruning:
        // Instead of a brittle fixed message count (e.g. >18), evaluate total estimated tokens.
        // Budget target for history messages: ~32,000 tokens (leaving ample room for system prompt & output).
        const MAX_HISTORY_TOKEN_BUDGET: usize = 32_000;
        let total_history_tokens: usize = db_messages.iter().map(estimate_message_tokens).sum();

        let mut evicted_older_messages: Vec<&Message> = Vec::new();
        let messages_to_send: Vec<&Message> = if total_history_tokens > MAX_HISTORY_TOKEN_BUDGET || db_messages.len() > 24 {
            let mut pruned = Vec::new();
            if let Some(first_user_msg) = db_messages.iter().find(|m| m.role == "user") {
                pruned.push(first_user_msg);
            }

            // Find tail start index by working backwards from the newest messages until token budget is met
            let mut accumulated_tail_tokens = 0;
            let mut tail_start_idx = db_messages.len().saturating_sub(1);
            while tail_start_idx > 0 {
                let msg_tok = estimate_message_tokens(&db_messages[tail_start_idx]);
                if accumulated_tail_tokens + msg_tok > MAX_HISTORY_TOKEN_BUDGET && (db_messages.len() - tail_start_idx) >= 10 {
                    break;
                }
                accumulated_tail_tokens += msg_tok;
                tail_start_idx = tail_start_idx.saturating_sub(1);
            }

            // Never start tail right on a "tool" message (which would orphan it without its parent assistant)
            while tail_start_idx > 0 && db_messages.get(tail_start_idx).map(|m| m.role.as_str()) == Some("tool") {
                tail_start_idx = tail_start_idx.saturating_sub(1);
            }

            // Collect all evicted messages between first_user_msg and tail_start_idx for rolling compaction
            let first_user_id = db_messages.iter().find(|m| m.role == "user").map(|m| &m.id);
            for m in &db_messages[..tail_start_idx] {
                if Some(&m.id) != first_user_id {
                    evicted_older_messages.push(m);
                }
            }

            for m in &db_messages[tail_start_idx..] {
                if !pruned.iter().any(|existing| existing.id == m.id) {
                    pruned.push(m);
                }
            }
            if let Some(last_user_msg) = db_messages.iter().rev().find(|m| m.role == "user") {
                if !pruned.iter().any(|existing| existing.id == last_user_msg.id) {
                    pruned.push(last_user_msg);
                }
            }
            // Strict sequence sort so user prompt is NEVER before older assistant/tool messages
            pruned.sort_by_key(|m| m.sequence_num);
            pruned
        } else {
            db_messages.iter().collect()
        };

        // If older messages were evicted by the sliding window, inject a rolling compact summary
        // so the model does not suffer from context amnesia!
        if !evicted_older_messages.is_empty() {
            let eviction_summary = summarize_evicted_messages(&evicted_older_messages);
            if !eviction_summary.is_empty() {
                v1_messages.push(V1Message {
                    role: "user".to_string(),
                    content: Some(json!(eviction_summary)),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    thought_signature: None,
                });
            }
        }

        // Collect all valid tool_call IDs emitted by assistant messages in this window to prevent orphaned tool_results
        let mut known_tool_call_ids = std::collections::HashSet::new();
        for msg in &messages_to_send {
            if msg.role == "assistant" {
                if let Some(ref tc_raw) = msg.tool_calls {
                    if let Ok(calls) = serde_json::from_str::<Vec<V1ToolCall>>(tc_raw) {
                        for c in calls {
                            known_tool_call_ids.insert(c.id);
                        }
                    }
                }
            }
        }

        for msg in messages_to_send {
            // Drop orphaned tool messages whose parent assistant tool_call was pruned out
            if msg.role == "tool" {
                if let Some(ref tcid) = msg.tool_call_id {
                    if !known_tool_call_ids.contains(tcid) {
                        continue;
                    }
                }
            }

            let tc_parsed: Option<Vec<V1ToolCall>> = msg
                .tool_calls
                .as_deref()
                .and_then(|raw| serde_json::from_str(raw).ok());

            let content_val = if msg.role == "user" {
                msg.content.as_ref().map(|c| {
                    let expanded = super::tools::expand_file_tags(c, &allowed_folders);
                    json!(expanded)
                })
            } else if msg.role == "tool" {
                tool_counter += 1;
                let is_recent_tail = tool_counter > total_tool_msgs.saturating_sub(3);
                msg.content.as_ref().map(|c| {
                    let compacted = compact_tool_output(c, is_recent_tail);
                    json!(compacted)
                })
            } else {
                msg.content.as_ref().map(|c| json!(c))
            };

            v1_messages.push(V1Message {
                role: msg.role.clone(),
                content: content_val,
                tool_calls: tc_parsed,
                tool_call_id: msg.tool_call_id.clone(),
                name: None,
                thought_signature: None,
            });
        }

        // Ephemeral in-memory prodding directive without polluting DB messages
        if let Some(directive) = ephemeral_directive.take() {
            v1_messages.push(V1Message {
                role: "user".to_string(),
                content: Some(json!(directive)),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                thought_signature: None,
            });
        }

        let tools_schema = get_agent_tools_schema();
        let chat_req = V1ChatRequest {
            model: model.clone(),
            messages: v1_messages,
            temperature: session.temperature.map(|t| t as f32).or(Some(0.7)),
            max_tokens: Some(4096),
            stream: Some(true),
            tools: Some(tools_schema),
            tool_choice: None,
        };

        let input_chars: usize = chat_req.messages.iter().map(|m| {
            m.content.as_ref().map(|c| c.to_string().len()).unwrap_or(0)
        }).sum();
        let estimated_input_tokens = (input_chars / 4).max(1);
        let turn_start_time = std::time::Instant::now();

        // Call Provider inside spawn_blocking with automatic retry (up to 3 attempts)
        let req_clone = chat_req.clone();
        let sess_id_clone = session_id.to_string();
        let current_iteration = iteration;
        let call_res = tokio::task::spawn_blocking(move || {
            let mut last_err = None;
            for attempt in 1..=3 {
                let mut accumulated_content = String::new();
                let mut accumulated_thought = String::new();
                let mut collected_tool_calls: Vec<V1ToolCall> = Vec::new();

                let attempt_result = if is_codex {
                    let mut announced_tool_indices: HashSet<usize> = HashSet::new();
                    let mut provider = CodexProvider::new();
                    let res = provider.chat_stream_v1(&req_clone, |event| {
                        match event {
                            V1StreamEvent::ContentDelta(delta) => {
                                accumulated_content.push_str(&delta);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::Delta { delta },
                                );
                            }
                            V1StreamEvent::ThoughtDelta(thought) => {
                                accumulated_thought.push_str(&thought);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::ThoughtDelta { delta: thought },
                                );
                            }
                            V1StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                                if index >= collected_tool_calls.len() {
                                    collected_tool_calls.resize(
                                        index + 1,
                                        V1ToolCall {
                                            id: id.clone().unwrap_or_else(|| format!("call_{}", index)),
                                            call_type: "function".to_string(),
                                            function: V1ToolCallFunction {
                                                name: name.clone().unwrap_or_default(),
                                                arguments: String::new(),
                                            },
                                            thought_signature: None,
                                            extra_content: None,
                                            provider_specific_fields: None,
                                        },
                                    );
                                }
                                if let Some(tc_id) = id {
                                    collected_tool_calls[index].id = tc_id;
                                }
                                if let Some(tc_name) = name {
                                    if !tc_name.is_empty() && announced_tool_indices.insert(index) {
                                        broadcast_session_event(
                                            &sess_id_clone,
                                            AgentSessionEvent::StepInfo {
                                                iteration: current_iteration,
                                                action: format!("tool:{}", tc_name),
                                                details: Some(format!("Preparing `{}`...", tc_name)),
                                            },
                                        );
                                    }
                                    collected_tool_calls[index].function.name = tc_name;
                                }
                                collected_tool_calls[index].function.arguments.push_str(&arguments);
                            }
                            _ => {}
                        }
                        Ok(())
                    });
                    res.map(|v1_res| {
                        let tc = if v1_res.tool_calls.is_some() {
                            v1_res.tool_calls
                        } else if !collected_tool_calls.is_empty() {
                            Some(collected_tool_calls)
                        } else {
                            None
                        };
                        (accumulated_content, accumulated_thought, tc)
                    })
                } else {
                    let mut announced_tool_indices: HashSet<usize> = HashSet::new();
                    let mut provider = AntigravityProvider::new();
                    let res = provider.chat_stream_v1(&req_clone, |event| {
                        match event {
                            V1StreamEvent::ContentDelta(delta) => {
                                accumulated_content.push_str(&delta);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::Delta { delta },
                                );
                            }
                            V1StreamEvent::ThoughtDelta(thought) => {
                                accumulated_thought.push_str(&thought);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::ThoughtDelta { delta: thought },
                                );
                            }
                            V1StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                                if index >= collected_tool_calls.len() {
                                    collected_tool_calls.resize(
                                        index + 1,
                                        V1ToolCall {
                                            id: id.clone().unwrap_or_else(|| format!("call_{}", index)),
                                            call_type: "function".to_string(),
                                            function: V1ToolCallFunction {
                                                name: name.clone().unwrap_or_default(),
                                                arguments: String::new(),
                                            },
                                            thought_signature: None,
                                            extra_content: None,
                                            provider_specific_fields: None,
                                        },
                                    );
                                }
                                if let Some(tc_id) = id {
                                    collected_tool_calls[index].id = tc_id;
                                }
                                if let Some(tc_name) = name {
                                    if !tc_name.is_empty() && announced_tool_indices.insert(index) {
                                        broadcast_session_event(
                                            &sess_id_clone,
                                            AgentSessionEvent::StepInfo {
                                                iteration: current_iteration,
                                                action: format!("tool:{}", tc_name),
                                                details: Some(format!("Preparing `{}`...", tc_name)),
                                            },
                                        );
                                    }
                                    collected_tool_calls[index].function.name = tc_name;
                                }
                                collected_tool_calls[index].function.arguments.push_str(&arguments);
                            }
                            _ => {}
                        }
                        Ok(())
                    });
                    res.map(|v1_res| {
                        let tc = if v1_res.tool_calls.is_some() {
                            v1_res.tool_calls
                        } else if !collected_tool_calls.is_empty() {
                            Some(collected_tool_calls)
                        } else {
                            None
                        };
                        (accumulated_content, accumulated_thought, tc)
                    })
                };

                match attempt_result {
                    Ok(val) => return Ok(val),
                    Err(e) => {
                        let err_str = e.to_string();
                        let is_retryable = err_str.contains("429")
                            || err_str.contains("503")
                            || err_str.contains("502")
                            || err_str.contains("timeout")
                            || err_str.contains("connection closed")
                            || err_str.contains("broken pipe");

                        last_err = Some(e);
                        if attempt < 3 && is_retryable {
                            std::thread::sleep(std::time::Duration::from_millis(800 * attempt as u64));
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Unknown upstream connection error")))
        })
        .await?;

        let (final_content, final_thought, tool_calls_opt) = match call_res {
            Ok(triplet) => triplet,
            Err(e) => {
                let err_str = e.to_string();
                let err_msg_content = format!(
                    "⚠️ **Error communicating with AI model**: {}\n\nYou can switch the model using the dropdown above or send another prompt to retry.",
                    err_str
                );

                if let Ok(err_msg) = repository::create_message(
                    session_id,
                    "assistant",
                    Some(&err_msg_content),
                    None,
                    None,
                    None,
                    "error",
                    None,
                    None,
                )
                .await
                {
                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::MessageCreated(err_msg),
                    );
                }

                let _ = repository::update_session_status(session_id, "idle").await;
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::StatusChange {
                        status: "idle".to_string(),
                    },
                );
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::Error {
                        message: err_str.clone(),
                    },
                );

                return Ok(err_msg_content);
            }
        };

        let duration_ms = turn_start_time.elapsed().as_millis() as u64;
        let tool_chars: usize = tool_calls_opt.as_ref().map(|calls| {
            calls.iter().map(|tc| tc.function.name.len() + tc.function.arguments.len()).sum()
        }).unwrap_or(0);
        let output_chars = final_content.len() + final_thought.len() + tool_chars;
        let estimated_output_tokens = (output_chars / 4).max(1);

        // Record turn usage to llm_usage table
        let provider_name = if is_codex { "codex" } else { "antigravity" };
        let _ = crate::llm::usage::record_usage_async(
            provider_name,
            &model,
            estimated_input_tokens,
            0,
            estimated_output_tokens,
            duration_ms,
            "success",
        )
        .await;

        let thought_opt = if final_thought.trim().is_empty() {
            None
        } else {
            Some(final_thought.as_str())
        };

        // Check if model returned tool calls
        if let Some(tool_calls) = tool_calls_opt {
            if !tool_calls.is_empty() {
                // 1. Save assistant message containing the tool calls
                let tc_json = serde_json::to_string(&tool_calls).unwrap_or_default();
                let assistant_text = if final_content.is_empty() {
                    None
                } else {
                    Some(final_content.as_str())
                };

                let assistant_msg = repository::create_message(
                    session_id,
                    "assistant",
                    assistant_text,
                    thought_opt,
                    Some(&tc_json),
                    None,
                    "success",
                    Some(estimated_input_tokens as i64),
                    Some(estimated_output_tokens as i64),
                )
                .await?;

                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::MessageCreated(assistant_msg.clone()),
                );
                db_messages.push(assistant_msg);

                // 2. Execute each tool call and save results
                for tc in &tool_calls {
                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::StepInfo {
                            iteration,
                            action: format!("tool:{}", tc.function.name),
                            details: Some(format!("Executing `{}`...", tc.function.name)),
                        },
                    );

                    let mut args_val = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                        .unwrap_or_else(|_| json!({}));

                    let is_edit_op = tc.function.name == "workspace_write_file"
                        || tc.function.name == "workspace_edit_file";

                    let is_inspection = tc.function.name == "workspace_read_file"
                        || tc.function.name == "workspace_grep"
                        || tc.function.name == "workspace_list_dir"
                        || (tc.function.name == "workspace_run_command" && {
                            let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim().to_lowercase();
                            cmd.starts_with("cat ") || cmd.starts_with("ls") || cmd.starts_with("head ") || cmd.starts_with("tail ") || cmd.starts_with("grep ") || cmd.starts_with("find ")
                        });

                    let target_arg_file = args_val.get("file_path").or_else(|| args_val.get("path")).and_then(|p| p.as_str()).map(|s| s.to_string());
                    let is_target_in_error_list = target_arg_file.as_ref().map(|p| {
                        let clean = p.trim_start_matches("./");
                        pending_error_files.iter().any(|ef| ef == clean || clean.ends_with(ef) || ef.ends_with(clean))
                    }).unwrap_or(false);

                    // HYBRID TARGETED SLICE RULE:
                    // When inspecting a file with compiler/verification errors, load the entire file on FIRST read
                    // so the model gets full context (imports, struct definitions, matching braces).
                    // On subsequent reads (or after edits), if the model consciously requests a line slice,
                    // honor the slice with safety padding (+/- 15 lines) so it can inspect shifted lines without
                    // wasting tokens re-reading the entire file.
                    if tc.function.name == "workspace_read_file" {
                        if let Some(ref path) = target_arg_file {
                            let clean = path.trim_start_matches("./").to_string();
                            let is_first_read = !files_read_in_full.contains(&clean);
                            if is_target_in_error_list && is_first_read {
                                if let Some(obj) = args_val.as_object_mut() {
                                    obj.remove("start_line");
                                    obj.remove("end_line");
                                }
                            } else if let Some(obj) = args_val.as_object_mut() {
                                let parse_line_val = |v: Option<&serde_json::Value>| -> Option<i64> {
                                    v.and_then(|val| {
                                        val.as_i64()
                                            .or_else(|| val.as_u64().map(|n| n as i64))
                                            .or_else(|| val.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
                                    })
                                };
                                let has_start = parse_line_val(obj.get("start_line"));
                                let has_end = parse_line_val(obj.get("end_line"));

                                if let (Some(s), Some(e)) = (has_start, has_end) {
                                    let padded_start = (s - 15).max(1);
                                    let padded_end = e + 15;
                                    obj.insert("start_line".to_string(), json!(padded_start));
                                    obj.insert("end_line".to_string(), json!(padded_end));
                                } else if let Some(s) = has_start {
                                    let padded_start = (s - 15).max(1);
                                    obj.insert("start_line".to_string(), json!(padded_start));
                                } else if let Some(e) = has_end {
                                    let padded_end = e + 15;
                                    obj.insert("end_line".to_string(), json!(padded_end));
                                }
                            }
                        }
                    }

                    let is_accessing_unmodified_target = target_arg_file.as_ref().map(|p| {
                        let clean = p.trim_start_matches("./");
                        !modified_files.iter().any(|mf| {
                            let mf_clean = mf.trim_start_matches("./");
                            mf_clean == clean || mf_clean.ends_with(clean) || clean.ends_with(mf_clean)
                        })
                    }).unwrap_or(false);

                    let is_work_on_new_target = (tc.function.name == "workspace_read_file"
                        || tc.function.name == "workspace_edit_file"
                        || tc.function.name == "workspace_write_file")
                        && is_accessing_unmodified_target;

                    let is_post_verification_terminal_command = has_passed_verification
                        && !modified_files.is_empty()
                        && !last_command_had_error
                        && pending_error_files.is_empty()
                        && tc.function.name == "workspace_run_command"
                        && {
                            let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim().to_lowercase();
                            cmd.starts_with("git ") || cmd.contains("git commit") || cmd.contains("git add") || executed_commands.iter().any(|ec| ec.contains(&cmd) || cmd.contains(ec))
                        };

                    // HARD GATES & CIRCUIT BREAKERS (oh-my-pi inspired anti-loop protection)
                    // Circuit breaker only triggers AFTER verification has actually been tested and passed with 0 errors in THIS turn,
                    // AND the agent is NOT trying to read or edit another untouched deliverable file!
                    let tool_output = if has_passed_verification && !modified_files.is_empty() && !last_command_had_error && pending_error_files.is_empty() && !is_work_on_new_target && (tc.function.name != "workspace_run_command" || is_post_verification_terminal_command) {
                        "Verification has already succeeded with 0 errors for all modified file(s). All active errors resolved. Conclude your turn immediately and provide your summary to the user now.".to_string()
                    } else if is_inspection && !is_target_in_error_list && global_inspection_streak >= 4 {
                        format!(
                            "Tool execution rejected by inspection circuit breaker: You have executed {} consecutive inspection operations without editing any code. Stop browsing files. Apply your changes using `workspace_edit_file` or `workspace_write_file` NOW, or run verification via `workspace_run_command`.",
                            global_inspection_streak
                        )
                    } else if tc.function.name == "workspace_read_file" && {
                        let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                        let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                        let path = target_arg_file.clone().unwrap_or_default();
                        let is_exact_duplicate = recent_read_slices.contains(&(path.clone(), start_line, end_line));
                        let is_streak_exceeded = target_arg_file.as_ref().map(|p| consecutive_read_same_file.get(p).copied().unwrap_or(0) >= 3).unwrap_or(false);
                        is_exact_duplicate || is_streak_exceeded
                    } {
                        let path = target_arg_file.as_deref().unwrap_or("target");
                        format!(
                            "Tool execution rejected: File `{}` has already been inspected in this turn without modification. You already have its contents in your conversation history. Formulate your changes and call `workspace_edit_file` or `workspace_write_file` now.",
                            path
                        )
                    } else {
                        match execute_tool_call(
                            &tc.function.name,
                            &args_val,
                            &allowed_folders,
                        ) {
                            Ok(out) => {
                                if tc.function.name == "workspace_read_file" {
                                    if let Some(ref p) = target_arg_file {
                                        let clean = p.trim_start_matches("./").to_string();
                                        let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                                        let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                                        if start_line.is_none() && end_line.is_none() {
                                            files_read_in_full.insert(clean);
                                        }
                                        recent_read_slices.insert((p.clone(), start_line, end_line));
                                    }
                                }
                                out
                            }
                            Err(e) => format!("Tool execution error: {}", e),
                        }
                    };

                    if is_inspection {
                        global_inspection_streak += 1;
                    }

                    if is_inspection {
                        if current_phase == "discovery" {
                            discovery_counter = repository::increment_goal_discovery(&goal_id).await.unwrap_or(discovery_counter + 1);
                            if let Some(ref path) = target_arg_file {
                                // Only lock individual files, never whole directories
                                let is_dir_inspection = tc.function.name == "workspace_list_dir"
                                    || path.ends_with('/')
                                    || !path.contains('.');
                                if !is_dir_inspection && !locked_targets.contains(path) {
                                    locked_targets.push(path.clone());
                                }
                            }
                        }
                    }

                    // Auto-invalidation and tracking on file modification (ONLY on actual SUCCESSFUL edit)
                    if is_edit_op {
                        let is_edit_failed = tool_output.starts_with("Tool execution error")
                            || tool_output.starts_with("Phase Gate")
                            || tool_output.contains("'old_text' not found")
                            || tool_output.contains("failed to apply edit");

                        if is_edit_failed {
                            last_tool_had_error = true;
                        } else {
                            last_tool_had_error = false;
                            has_passed_verification = false; // Reset verification so newly modified files must be verified!
                            verification_prompted = false; // Reset so verification gate triggers for newly modified files!
                            global_inspection_streak = 0;
                            recent_read_slices.clear();
                            consecutive_read_same_file.clear();
                            if let Some(path) = target_arg_file.as_ref() {
                                let clean = path.trim_start_matches("./");
                                pending_error_files.retain(|ef| ef != clean && !clean.ends_with(ef) && !ef.ends_with(clean));
                                if !modified_files.contains(path) {
                                    modified_files.push(path.clone());
                                }
                                if !locked_targets.contains(path) {
                                    locked_targets.push(path.clone());
                                }
                                let _ = repository::mark_evidence_stale_by_file(&project.id, path).await;
                            }

                            // Automatic transition to Verification phase upon editing
                            current_phase = "verification".to_string();
                            let targets_json = serde_json::to_string(&locked_targets).unwrap_or_else(|_| "[]".to_string());
                            let _ = repository::lock_goal_targets(&goal_id, &targets_json).await;
                            let _ = repository::update_goal_phase(&goal_id, "verification").await;

                            let relevant_verifiers = super::languages::detect_verifiers_for_modified_files(&allowed_folders, &modified_files);
                            let primary_verify = if !relevant_verifiers.is_empty() {
                                relevant_verifiers.join(" && ")
                            } else {
                                super::languages::detect_primary_verification_command(&allowed_folders)
                                    .unwrap_or_else(|| "test/build runner".to_string())
                            };
                            ephemeral_directive = Some(
                                format!(
                                    "ACTION EXECUTED: Target file updated successfully. You are now in VERIFICATION phase. Run the verification command using `workspace_run_command` (e.g. `{}`) to verify your changes.",
                                    primary_verify
                                )
                            );
                        }
                    } else if tc.function.name == "workspace_run_command" {
                        global_inspection_streak = 0;
                        if let Some(cmd) = args_val.get("command").and_then(|c| c.as_str()) {
                            if !executed_commands.contains(&cmd.to_string()) {
                                executed_commands.push(cmd.to_string());
                            }

                            let is_err = is_command_output_error(&tool_output);

                            if is_err {
                                last_command_had_error = true;
                                has_passed_verification = false;

                                let (grouped_diags, general_errs) = group_diagnostics_by_file(&tool_output);
                                for g in &grouped_diags {
                                    pending_error_files.insert(g.file_path.clone());
                                }
                                let mut diag_bulletin = String::new();

                                if !grouped_diags.is_empty() {
                                    diag_bulletin.push_str("\n\n🚨 TARGET FILES WITH COMPILER ERRORS (Batch-edit these files):\n");
                                    for g in grouped_diags.iter().take(12) {
                                        let lines_str = if !g.lines.is_empty() {
                                            format!(" (lines: {})", g.lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", "))
                                        } else {
                                            String::new()
                                        };
                                        diag_bulletin.push_str(&format!("- `{}` -> {} error(s){}\n", g.file_path, g.error_count, lines_str));
                                        for s in &g.sample_messages {
                                            diag_bulletin.push_str(&format!("    * {}\n", s));
                                        }
                                    }
                                } else {
                                    let extracted_diags = extract_command_diagnostics(&tool_output);
                                    if !extracted_diags.is_empty() {
                                        diag_bulletin.push_str(&format!(
                                            "\n\n🚨 EXTRACTED COMPILER / RUNTIME DIAGNOSTICS:\n{}\n",
                                            extracted_diags.iter().map(|d| format!("- {}", d)).collect::<Vec<_>>().join("\n")
                                        ));
                                    }
                                }

                                if !general_errs.is_empty() {
                                    diag_bulletin.push_str(&format!("General: {}\n", general_errs.join(" | ")));
                                }

                                ephemeral_directive = Some(format!(
                                    "COMMAND FAILED: Command `{cmd}` reported errors or a non-zero exit code.{diag_bulletin}\n\nCRITICAL BATCH-ACTION REQUIRED:\n1. Read the target files listed above (or use the lines indicated).\n2. Apply ALL fixes for each target file in ONE comprehensive `workspace_edit_file` or `workspace_write_file` call.\n3. You may call `workspace_edit_file` multiple times in parallel for multiple files in this same turn.\n4. Do NOT fix line-by-line in separate turns. Resolve all errors before re-running verification."
                                ));
                            }

                            if super::languages::is_verification_command(cmd) {
                                let is_passed = !is_err;
                                verification_attempts += 1;
                                let _ = repository::record_verification(
                                    &goal_id,
                                    verification_attempts as i64,
                                    cmd,
                                    &tool_output,
                                    is_passed,
                                    if is_passed { None } else { Some(&tool_output) },
                                ).await;

                                // Verification CANNOT be considered satisfied if the user requested changes but 0 files were modified
                                if is_passed && !modified_files.is_empty() {
                                    has_passed_verification = true;
                                    last_command_had_error = false;
                                    pending_error_files.clear();
                                    current_phase = "concluded".to_string();
                                    let _ = repository::update_goal_phase(&goal_id, "concluded").await;
                                    let _ = repository::update_goal_status(&goal_id, "completed").await;
                                    ephemeral_directive = Some(
                                        "VERIFICATION PASSED: The build/test command succeeded with 0 errors! If all parts of the user objective across all components (e.g. backend and frontend) are complete, conclude your turn now with a clear summary. If remaining deliverables or other files still need changes, proceed directly to edit them.".to_string()
                                    );
                                } else if is_passed && modified_files.is_empty() && has_unresolved_active_goal {
                                    // Build passed but no files were modified yet for this in-progress goal
                                    ephemeral_directive = Some(
                                        "VERIFICATION RAN WITHOUT CODE EDITS: The build/test command passed, but you have not modified any files for this task yet. Ensure you apply any needed changes to the target files before concluding.".to_string()
                                    );
                                } else if !is_passed {
                                    has_passed_verification = false;
                                    current_phase = "action".to_string();
                                    let _ = repository::update_goal_phase(&goal_id, "action").await;

                                    let (grouped_diags, _) = group_diagnostics_by_file(&tool_output);
                                    for g in &grouped_diags {
                                        pending_error_files.insert(g.file_path.clone());
                                    }
                                    let mut diag_bulletin = String::new();
                                    if !grouped_diags.is_empty() {
                                        diag_bulletin.push_str("\n\n🚨 VERIFICATION FAILURE TARGET FILES:\n");
                                        for g in grouped_diags.iter().take(10) {
                                            diag_bulletin.push_str(&format!("- `{}` ({} errors at lines {:?})\n", g.file_path, g.error_count, g.lines));
                                        }
                                    }

                                    ephemeral_directive = Some(format!(
                                        "VERIFICATION FAILED: Verification command `{cmd}` reported errors.{diag_bulletin}\n\nACTION REQUIRED: Read the failing files, modify them using `workspace_edit_file`, and re-verify before concluding."
                                    ));
                                }
                            }
                        }
                    }



                    // If edit failed due to text mismatch or duplicate match, guide model directly back to target
                    if tc.function.name == "workspace_edit_file" {
                        if tool_output.contains("'old_text' not found") {
                            ephemeral_directive = Some(
                                "CRITICAL EDIT RECOVERY: Your `workspace_edit_file` failed because 'old_text' did not match disk. The error message includes the exact lines currently on disk! EITHER copy those exact lines into `old_text` with 2-3 lines of context, OR immediately call `workspace_write_file` to write the whole file directly. You MUST apply this fix now; do NOT attempt to conclude or wander off.".to_string()
                            );
                        } else if tool_output.contains("matches ") || tool_output.contains("matched ") {
                            ephemeral_directive = Some(
                                "CRITICAL: 'old_text' matched multiple locations in the file. Include the parent object key, function declaration, or 2-3 additional surrounding lines in 'old_text' to make it 100% unique, or use `workspace_write_file` directly. Apply the change now.".to_string()
                            );
                        }
                    }

                    // Check inspection streaks
                    if global_inspection_streak >= 2 {
                        ephemeral_directive = Some(format!(
                            "DIRECTIVE: You have completed {} consecutive inspection steps without modifying code. Gathered context is sufficient. Stop browsing files. Apply your changes using `workspace_edit_file` or `workspace_write_file` NOW, or run verification.",
                            global_inspection_streak
                        ));
                    }

                    // Prevent inspection loops after verification passed
                    if has_passed_verification && is_inspection && !is_accessing_unmodified_target {
                        ephemeral_directive = Some(
                            "STOP: Verification has already passed for modified files! Do not perform redundant inspections. Conclude your response or proceed directly to edit remaining target files.".to_string()
                        );
                    }

                    // Track consecutive reads/greps on the same file to prevent repetitive inspection loops across the whole turn
                    let is_file_inspection = tc.function.name == "workspace_read_file" || tc.function.name == "workspace_grep";
                    if is_file_inspection {
                        if let Some(path) = args_val.get("file_path").or_else(|| args_val.get("path")).and_then(|p| p.as_str()) {
                            let count = consecutive_read_same_file.entry(path.to_string()).or_insert(0);
                            *count += 1;
                            if *count >= 4 {
                                ephemeral_directive = Some(format!(
                                    "DIRECTIVE: You have inspected `{}` {} times this turn. You have sufficient information. STOP CALLING READ TOOLS ON THIS FILE. If your task requires code modifications, call `workspace_edit_file` or `workspace_write_file` now. If answering a question, respond directly to the user.",
                                    path, count
                                ));
                            }
                        }
                    }

                    // Auto-cache discovered file contents into Evidence Cache for reuse in future turns & tasks
                    if tc.function.name == "workspace_read_file" && !tool_output.starts_with("Tool execution error") {
                        if let Some(file_path) = args_val.get("file_path").or_else(|| args_val.get("path")).and_then(|p| p.as_str()) {
                            let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                            let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                            let preview_summary = format!("Read {} (lines {:?}-{:?})", file_path, start_line.unwrap_or(1), end_line.unwrap_or(0));
                            if let Ok(saved_ev) = repository::save_evidence(
                                &project.id,
                                file_path,
                                "live_mtime",
                                start_line,
                                end_line,
                                None,
                                &preview_summary,
                                None, // Keep lean, no raw code bloat
                            ).await {
                                let _ = repository::link_evidence_to_goal(&goal_id, &saved_ev.id, 0.8, true).await;
                            }
                        }
                    } else if is_edit_op && !tool_output.starts_with("Tool execution error") {
                        if let Some(file_path) = args_val.get("file_path").or_else(|| args_val.get("path")).and_then(|p| p.as_str()) {
                            let preview_summary = format!("Modified/Created file {}", file_path);
                            if let Ok(saved_ev) = repository::save_evidence(
                                &project.id,
                                file_path,
                                "live_mtime",
                                None,
                                None,
                                None,
                                &preview_summary,
                                None, // Keep lean, no raw code bloat
                            ).await {
                                let _ = repository::link_evidence_to_goal(&goal_id, &saved_ev.id, 1.0, true).await;
                            }
                        }
                    }

                    // Sanitize large tool output before DB persistence so SQLite stays lean
                    let tool_output_for_db = sanitize_tool_output_for_db(&tool_output);

                    let tool_msg = repository::create_message(
                        session_id,
                        "tool",
                        Some(&tool_output_for_db),
                        None,
                        None,
                        Some(&tc.id),
                        "success",
                        None,
                        None,
                    )
                    .await?;

                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::MessageCreated(tool_msg.clone()),
                    );

                    // Cap in-memory tool output to max 32 KB so massive raw outputs never inflate LLM context
                    let mut in_memory_tool_msg = tool_msg;
                    in_memory_tool_msg.content = Some(sanitize_tool_output_for_memory(&tool_output));
                    db_messages.push(in_memory_tool_msg);
                }

// Circuit breaker for post-edit wandering removed: allow autonomous agent to edit multiple files across components without premature interruption

                // Approaching iteration safety limit: remind model to summarize and conclude
                if iteration >= max_safety_iterations.saturating_sub(2) {
                    ephemeral_directive = Some(
                        "TURN LIMIT APPROACHING: Please finalize your actions now. Summarize your findings, code modifications, or answer the user directly.".to_string()
                    );
                }

                // Loop again with tool results
                continue;
            }
        }

        // If the model returned no conversational content to the user yet:
        if final_content.trim().is_empty() && iteration < max_safety_iterations {
            ephemeral_directive = Some(
                "You have finished running tools or inspecting the workspace. Now provide a clear, comprehensive summary of the work done, code changes, or answers to the user's request.".to_string(),
            );
            continue;
        }

        // Action Gate (State-driven & language-agnostic):
        // If a command or edit tool returned an error in this turn, the agent must fix it before concluding
        let must_apply_changes = last_command_had_error || last_tool_had_error;

        if must_apply_changes && modified_files.is_empty() && iteration < max_safety_iterations - 2 {
            ephemeral_directive = Some(
                "CRITICAL REJECTION — UNRESOLVED ERROR WITHOUT REPAIR: A terminal command or tool execution failed with an error, but you have NOT successfully modified any files to fix it! You cannot conclude without applying the fix. You MUST invoke `workspace_edit_file` or `workspace_write_file` NOW to resolve the issue on disk.".to_string()
            );
            continue;
        }

        // State-driven task gate: If the turn was working on a code/task deliverable,
        // or if the assistant output mentions workspace tool calls in text instead of invoking them,
        // do not conclude without actually executing the modifications.
        let model_mentions_tools_in_text = final_content.contains("workspace_");
        let has_active_inspections = global_inspection_streak > 0 || !consecutive_read_same_file.is_empty();
        let is_unfulfilled_task = (is_task_request || model_mentions_tools_in_text) && has_active_inspections;

        if is_unfulfilled_task && modified_files.is_empty() && iteration < max_safety_iterations - 2 {
            ephemeral_directive = Some(
                "ACTION REQUIRED — NO CODE CHANGES APPLIED YET: You have inspected the workspace, but no files have been modified on disk yet! Do NOT merely narrate plans, next steps, or thoughts in text. You MUST call `workspace_read_file`, `workspace_edit_file`, or `workspace_write_file` NOW to complete the required modifications.".to_string()
            );
            continue;
        }

        // Hard Verification Enforcement Gate:
        // NEVER allow conclusion while verification failed or active command/tool errors exist
        let has_failing_checks = (!has_passed_verification && verification_attempts > 0)
            || last_command_had_error
            || last_tool_had_error;
        if has_failing_checks && iteration < max_safety_iterations - 2 {
            ephemeral_directive = Some(
                "CRITICAL REJECTION — FAILING CHECKS OR UNRESOLVED ERRORS: A verification check, terminal command, or edit operation failed! You are strictly forbidden from concluding or claiming success while errors remain. Inspect the error messages, fix the code using `workspace_edit_file` or `workspace_write_file`, and re-run verification until all checks pass cleanly.".to_string()
            );
            continue;
        }

        // Verification Gate: If files were modified in this session, ensure at least one verification check was run
        if !modified_files.is_empty() && !has_passed_verification && !verification_prompted && iteration < max_safety_iterations - 1 {
            verification_prompted = true;
            let relevant_verifiers = super::languages::detect_verifiers_for_modified_files(&allowed_folders, &modified_files);
            let verifier_hint = if !relevant_verifiers.is_empty() {
                relevant_verifiers.join(" && ")
            } else if let Some(ref pv) = detected_verifier {
                pv.clone()
            } else {
                "appropriate build/test commands for the modified components".to_string()
            };
            ephemeral_directive = Some(format!(
                "VERIFICATION REQUIRED: You modified {} file(s) ({}) but have not verified them yet. Run `{}` using `workspace_run_command` to ensure there are no compilation or test errors before concluding.",
                modified_files.len(),
                modified_files.join(", "),
                verifier_hint
            ));
            continue;
        }

        // Final assistant turn reached (no tool calls)
        let generated_summary = if !modified_files.is_empty() {
            let mut summary_lines = Vec::new();
            summary_lines.push("### 📋 Successfully Applied File Modifications:".to_string());
            for f in &modified_files {
                summary_lines.push(format!("- `{}`", f));
            }
            Some(summary_lines.join("\n"))
        } else {
            None
        };

        // If there were unresolved errors, override the assistant's claim to prevent false success statements
        let is_unresolved = last_command_had_error || last_tool_had_error || (!has_passed_verification && !modified_files.is_empty());
        let composed_content = if is_unresolved {
            let error_details = if last_command_had_error {
                "Terminal verification/build command failed with errors."
            } else if last_tool_had_error {
                "A file modification or tool execution failed."
            } else {
                "Required verification check has not passed cleanly."
            };
            format!(
                "⚠️ **Execution Ended with Unresolved Errors**\n\n{}\n\n{}",
                error_details,
                if final_content.trim().is_empty() {
                    "Please review the error outputs above or provide guidance."
                } else {
                    final_content.trim()
                }
            )
        } else {
            match (final_content.trim().is_empty(), generated_summary.as_ref()) {
                (false, Some(summary)) => format!("{}\n\n{}", final_content.trim(), summary),
                (false, None) => final_content.trim().to_string(),
                (true, Some(summary)) => summary.clone(),
                (true, None) if !final_thought.trim().is_empty() => final_thought.trim().to_string(),
                (true, None) => "I have completed the operations and workspace inspection as requested.".to_string(),
            }
        };

        let content_to_save = composed_content.as_str();

        // Conclude the active goal with the final summary so subsequent goals inherit it
        let final_goal_status = if ((must_apply_changes || is_task_request) && modified_files.is_empty())
            || last_command_had_error
            || last_tool_had_error
        {
            "failed"
        } else if !modified_files.is_empty() && !has_passed_verification {
            "completed_unverified"
        } else {
            "completed"
        };
        let _ = repository::conclude_goal(&goal_id, final_goal_status, content_to_save).await;

        let final_msg = repository::create_message(
            session_id,
            "assistant",
            Some(content_to_save),
            thought_opt,
            None,
            None,
            "success",
            Some(estimated_input_tokens as i64),
            Some(estimated_output_tokens as i64),
        )
        .await?;

        broadcast_session_event(
            session_id,
            AgentSessionEvent::MessageCreated(final_msg),
        );
        broadcast_session_event(
            session_id,
            AgentSessionEvent::Done {
                reply: content_to_save.to_string(),
            },
        );

        let _ = repository::update_session_status(session_id, "idle").await;
        broadcast_session_event(
            session_id,
            AgentSessionEvent::StatusChange {
                status: "idle".to_string(),
            },
        );
        return Ok(content_to_save.to_string());
    }

    // Fallback: If iteration limit reached while still running tools, ensure we accurately report status
    // so the UI never displays false complete claims or hangs!
    let fallback_summary = if !modified_files.is_empty() {
        format!(
            "### ⚠️ Iteration Limit Reached — Partial Changes Applied\n\nThe following files were updated in this turn:\n{}\n\nSome sections or files may still need further modifications. Please review the current workspace changes or prompt me to continue with the remaining parts.",
            modified_files.iter().map(|f| format!("- `{}`", f)).collect::<Vec<_>>().join("\n")
        )
    } else {
        format!(
            "### ⚠️ Completed Without File Modifications\n\nNo files were modified in this turn.\n- Commands executed: {}\n\nPlease provide specific instructions if there are components or lines you want to change.",
            if executed_commands.is_empty() { "None".to_string() } else { executed_commands.join(", ") }
        )
    };

    let final_fallback_msg = repository::create_message(
        session_id,
        "assistant",
        Some(&fallback_summary),
        None,
        None,
        None,
        "success",
        None,
        None,
    )
    .await?;

    let final_fallback_status = if !modified_files.is_empty() {
        "in_progress"
    } else {
        "failed"
    };
    let _ = repository::conclude_goal(&goal_id, final_fallback_status, &fallback_summary).await;

    broadcast_session_event(
        session_id,
        AgentSessionEvent::MessageCreated(final_fallback_msg),
    );

    let _ = repository::update_session_status(session_id, "idle").await;
    broadcast_session_event(
        session_id,
        AgentSessionEvent::StatusChange {
            status: "idle".to_string(),
        },
    );
    broadcast_session_event(
        session_id,
        AgentSessionEvent::Done {
            reply: fallback_summary.clone(),
        },
    );
    Ok(fallback_summary)
}

/// Delegated project verification detection for backward compatibility
pub fn detect_project_verification_commands(allowed_folders: &[String]) -> Option<String> {
    super::languages::detect_primary_verification_command(allowed_folders)
}

/// Delegated verification command check for backward compatibility
pub fn is_verification_command(cmd: &str) -> bool {
    super::languages::is_verification_command(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_folders() {
        assert_eq!(parse_project_folders(None), Vec::<String>::new());
        assert_eq!(
            parse_project_folders(Some("[\"/path/a\", \"/path/b\"]")),
            vec!["/path/a".to_string(), "/path/b".to_string()]
        );
        assert_eq!(
            parse_project_folders(Some("/single/path")),
            vec!["/single/path".to_string()]
        );
    }
}
