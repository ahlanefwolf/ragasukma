use anyhow::Result;
use std::fs;
use std::path::Path;
use sha2::{Digest, Sha256};

use super::models::Goal;
use super::repository;

/// Computes a sha256 checksum of a file to detect any file changes / drifts.
pub fn compute_file_version<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Revalidates existing evidence cache against actual files on disk.
/// If file content changed or file no longer exists, marks evidence as stale.
pub async fn revalidate_evidence_cache(
    project_id: &str,
    workspace_roots: &[String],
) -> Result<usize> {
    if workspace_roots.is_empty() {
        return Ok(0);
    }

    let evidences = repository::list_evidences_by_project(project_id).await?;
    let mut stale_count = 0;

    for ev in evidences {
        if ev.is_stale == 1 {
            continue;
        }

        // Try to locate file in workspace roots
        let mut matched = false;
        let mut current_version = None;

        for root in workspace_roots {
            let full_path = Path::new(root).join(&ev.file_path);
            if full_path.exists() {
                matched = true;
                if let Ok(hash) = compute_file_version(&full_path) {
                    current_version = Some(hash);
                }
                break;
            }
        }

        // If file disappeared or hash does not match stored file_version
        if !matched || current_version.as_deref() != Some(&ev.file_version) {
            repository::mark_evidence_stale_by_file(project_id, &ev.file_path).await?;
            stale_count += 1;
        }
    }

    Ok(stale_count)
}

/// Discovers reusable evidence strictly from the parent goal or antecedent task in the same session.
/// Never leaks evidence from unrelated tasks or foreign sessions.
pub async fn reuse_evidence_from_parent(
    new_goal_id: &str,
    parent_goal_id: &str,
    keywords: &[&str],
) -> Result<usize> {
    let parent_evidences = repository::get_active_evidences_for_goal(parent_goal_id).await?;
    let mut linked_count = 0;

    for ev in parent_evidences {
        if ev.is_stale == 1 {
            continue;
        }

        let file_path_lower = ev.file_path.to_lowercase();
        let summary_lower = ev.summary.to_lowercase();
        let symbol_lower = ev.symbol_name.as_deref().unwrap_or("").to_lowercase();

        let mut matches = 0;
        for kw in keywords {
            let kw_lower = kw.to_lowercase();
            if file_path_lower.contains(&kw_lower)
                || summary_lower.contains(&kw_lower)
                || symbol_lower.contains(&kw_lower)
            {
                matches += 1;
            }
        }

        // Link if there is relevant keyword overlap, or if keywords list is empty (direct continuation)
        if matches > 0 || keywords.is_empty() {
            let relevance_score = if keywords.is_empty() {
                0.9
            } else {
                (matches as f64) / (keywords.len().max(1) as f64)
            };
            repository::link_evidence_to_goal(new_goal_id, &ev.id, relevance_score, true).await?;
            linked_count += 1;
        }
    }

    Ok(linked_count)
}

/// Builds a compact, high-signal summary of previous goals strictly within the SAME session/thread.
/// Prevents cross-task context pollution from other unrelated tasks.
pub async fn build_previous_goals_summary(
    session_id: &str,
    current_goal_id: Option<&str>,
) -> Result<String> {
    let session_goals = repository::list_session_goals(session_id).await?;
    let previous_goals: Vec<_> = session_goals
        .into_iter()
        .filter(|g| Some(g.id.as_str()) != current_goal_id && (g.status == "completed" || g.conclusion.is_some()))
        .take(3)
        .collect();

    if previous_goals.is_empty() {
        return Ok(String::new());
    }

    let mut out = String::new();
    out.push_str("### 📜 Prior Accomplished Steps in this Task Session:\n");

    for g in previous_goals {
        out.push_str(&format!("- **Step: \"{}\"** (Status: {})\n", g.title, g.status));
        if let Some(ref conc) = g.conclusion {
            let conc_preview = conc.lines().next().unwrap_or(conc).trim();
            out.push_str(&format!("  * Outcome: {}\n", conc_preview));
        }

        let evidences = repository::get_active_evidences_for_goal(&g.id).await.unwrap_or_default();
        if !evidences.is_empty() {
            out.push_str("  * Touched Files & Locations:\n");
            for ev in evidences.iter().take(4) {
                let lines_info = match (ev.start_line, ev.end_line) {
                    (Some(s), Some(e)) => format!(" (Lines {}-{})", s, e),
                    (Some(s), None) => format!(" (Line {}+)", s),
                    _ => String::new(),
                };
                out.push_str(&format!("    - `{}`{}: {}\n", ev.file_path, lines_info, ev.summary));
            }
        }
    }
    out.push('\n');

    Ok(out)
}

/// Builds the dynamic Goal Context and Active Evidence block for system prompt / active prompt.
pub async fn build_goal_context_prompt(goal: &Goal) -> Result<String> {
    let mut prompt = String::new();

    prompt.push_str("=== ACTIVE GOAL OBJECTIVE ===\n");
    prompt.push_str(&format!("Goal ID: {}\n", goal.id));
    prompt.push_str(&format!("Title: {}\n", goal.title));
    prompt.push_str(&format!("Demand: {}\n", goal.demand));
    prompt.push_str(&format!("Current State: {}\n\n", goal.status.to_uppercase()));

    // Success criteria
    if let Ok(criteria) = serde_json::from_str::<Vec<String>>(&goal.success_criteria) {
        if !criteria.is_empty() {
            prompt.push_str("Success Criteria (MUST all be verified before conclusion):\n");
            for (i, c) in criteria.iter().enumerate() {
                prompt.push_str(&format!("  {}. {}\n", i + 1, c));
            }
            prompt.push('\n');
        }
    }

    // Constraints
    if let Ok(constraints) = serde_json::from_str::<Vec<String>>(&goal.constraints) {
        if !constraints.is_empty() {
            prompt.push_str("Constraints & Guardrails:\n");
            for c in &constraints {
                prompt.push_str(&format!("  - {}\n", c));
            }
            prompt.push('\n');
        }
    }

    // Active Evidence Cache (only fresh & active evidence)
    let active_evidences = repository::get_active_evidences_for_goal(&goal.id).await?;
    if !active_evidences.is_empty() {
        prompt.push_str("=== ACTIVE EVIDENCE CACHE (Verified Relevant Code) ===\n");
        for ev in &active_evidences {
            let line_range = match (ev.start_line, ev.end_line) {
                (Some(s), Some(e)) => format!(" [Lines {}-{}]", s, e),
                (Some(s), None) => format!(" [Line {}+]", s),
                _ => String::new(),
            };
            let symbol = ev
                .symbol_name
                .as_deref()
                .map(|s| format!(" (Symbol: `{}`)", s))
                .unwrap_or_default();

            prompt.push_str(&format!(
                "- `{}`{}{}\n  Version: {}\n  Summary: {}\n",
                ev.file_path, line_range, symbol, ev.file_version, ev.summary
            ));

            if let Some(ref snippet) = ev.raw_snippet {
                if !snippet.trim().is_empty() {
                    prompt.push_str("  Snippet:\n```\n");
                    prompt.push_str(snippet.trim());
                    prompt.push_str("\n```\n");
                }
            }
            prompt.push('\n');
        }
    } else {
        prompt.push_str("=== ACTIVE EVIDENCE CACHE ===\n(No active evidence yet. Perform Discovery Loop first.)\n\n");
    }

    prompt.push_str("=== ORCHESTRATION INSTRUCTIONS ===\n");
    prompt.push_str("- Discovery: Gather and verify facts. Inspect only relevant files.\n");
    prompt.push_str("- Action: Apply surgical edits once sufficient evidence is gathered.\n");
    prompt.push_str("- Verification: Run unit/integration tests to confirm every success criterion.\n");
    prompt.push_str("- Conclusion: Conclude only when verified.\n");

    Ok(prompt)
}
