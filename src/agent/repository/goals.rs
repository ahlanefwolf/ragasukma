use anyhow::Result;
use crate::agent::models::{EvidenceCache, Goal, GoalVerification};
use super::{generate_id, now_secs};

// ============================================================================
// GOAL STORE REPOSITORY
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn create_goal(
    id: &str,
    project_id: &str,
    session_id: Option<&str>,
    parent_goal_id: Option<&str>,
    title: &str,
    demand: &str,
    success_criteria_json: &str,
    constraints_json: &str,
) -> Result<Goal> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();

    sqlx::query(
        "INSERT INTO goals (id, project_id, session_id, parent_goal_id, title, demand, success_criteria, constraints, phase, status, target_files, discovery_count, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'discovery', 'in_progress', '[]', 0, ?9, ?10) \
         ON CONFLICT(id) DO UPDATE SET \
            project_id = excluded.project_id, \
            session_id = excluded.session_id, \
            parent_goal_id = excluded.parent_goal_id, \
            title = excluded.title, \
            demand = excluded.demand, \
            success_criteria = excluded.success_criteria, \
            constraints = excluded.constraints, \
            phase = 'discovery', \
            status = 'in_progress', \
            target_files = '[]', \
            discovery_count = 0, \
            conclusion = NULL, \
            updated_at = excluded.updated_at",
    )
    .bind(id)
    .bind(project_id)
    .bind(session_id)
    .bind(parent_goal_id)
    .bind(title)
    .bind(demand)
    .bind(success_criteria_json)
    .bind(constraints_json)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Goal {
        id: id.to_string(),
        project_id: project_id.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        parent_goal_id: parent_goal_id.map(|s| s.to_string()),
        title: title.to_string(),
        demand: demand.to_string(),
        success_criteria: success_criteria_json.to_string(),
        constraints: constraints_json.to_string(),
        phase: "discovery".to_string(),
        status: "in_progress".to_string(),
        target_files: "[]".to_string(),
        discovery_count: 0,
        conclusion: None,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_goal(goal_id: &str) -> Result<Option<Goal>> {
    let pool = crate::llm::usage::get_pool().await;
    let goal = sqlx::query_as::<_, Goal>(
        "SELECT id, project_id, session_id, parent_goal_id, title, demand, success_criteria, constraints, phase, status, target_files, discovery_count, conclusion, created_at, updated_at \
         FROM goals WHERE id = ?1",
    )
    .bind(goal_id)
    .fetch_optional(pool)
    .await?;
    Ok(goal)
}

pub async fn list_session_goals(session_id: &str) -> Result<Vec<Goal>> {
    let pool = crate::llm::usage::get_pool().await;
    let goals = sqlx::query_as::<_, Goal>(
        "SELECT id, project_id, session_id, parent_goal_id, title, demand, success_criteria, constraints, phase, status, target_files, discovery_count, conclusion, created_at, updated_at \
         FROM goals WHERE session_id = ?1 ORDER BY created_at DESC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(goals)
}

pub async fn list_project_goals(project_id: &str) -> Result<Vec<Goal>> {
    let pool = crate::llm::usage::get_pool().await;
    let goals = sqlx::query_as::<_, Goal>(
        "SELECT id, project_id, session_id, parent_goal_id, title, demand, success_criteria, constraints, phase, status, target_files, discovery_count, conclusion, created_at, updated_at \
         FROM goals WHERE project_id = ?1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(goals)
}

pub async fn update_goal_phase(goal_id: &str, phase: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE goals SET phase = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(phase)
        .bind(now)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn increment_goal_discovery(goal_id: &str) -> Result<i64> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE goals SET discovery_count = discovery_count + 1, updated_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(goal_id)
        .execute(pool)
        .await?;

    let count: i64 = sqlx::query_scalar("SELECT discovery_count FROM goals WHERE id = ?1")
        .bind(goal_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    Ok(count)
}

pub async fn lock_goal_targets(goal_id: &str, target_files_json: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE goals SET target_files = ?1, phase = 'action', updated_at = ?2 WHERE id = ?3")
        .bind(target_files_json)
        .bind(now)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_goal_status(goal_id: &str, status: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE goals SET status = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(status)
        .bind(now)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn conclude_goal(goal_id: &str, status: &str, conclusion: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE goals SET status = ?1, phase = 'concluded', conclusion = ?2, updated_at = ?3 WHERE id = ?4")
        .bind(status)
        .bind(conclusion)
        .bind(now)
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// EVIDENCE CACHE REPOSITORY
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn save_evidence(
    project_id: &str,
    file_path: &str,
    file_version: &str,
    start_line: Option<i64>,
    end_line: Option<i64>,
    symbol_name: Option<&str>,
    summary: &str,
    raw_snippet: Option<&str>,
) -> Result<EvidenceCache> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    let id = generate_id("ev");

    sqlx::query(
        "INSERT INTO evidence_cache (id, project_id, file_path, file_version, start_line, end_line, symbol_name, summary, raw_snippet, is_stale, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11)",
    )
    .bind(&id)
    .bind(project_id)
    .bind(file_path)
    .bind(file_version)
    .bind(start_line)
    .bind(end_line)
    .bind(symbol_name)
    .bind(summary)
    .bind(raw_snippet)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(EvidenceCache {
        id,
        project_id: project_id.to_string(),
        file_path: file_path.to_string(),
        file_version: file_version.to_string(),
        start_line,
        end_line,
        symbol_name: symbol_name.map(|s| s.to_string()),
        summary: summary.to_string(),
        raw_snippet: raw_snippet.map(|s| s.to_string()),
        is_stale: 0,
        created_at: now,
        updated_at: now,
    })
}

pub async fn list_evidences_by_project(project_id: &str) -> Result<Vec<EvidenceCache>> {
    let pool = crate::llm::usage::get_pool().await;
    let items = sqlx::query_as::<_, EvidenceCache>(
        "SELECT id, project_id, file_path, file_version, start_line, end_line, symbol_name, summary, raw_snippet, is_stale, created_at, updated_at \
         FROM evidence_cache WHERE project_id = ?1 ORDER BY updated_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(items)
}

pub async fn mark_evidence_stale_by_file(project_id: &str, file_path: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE evidence_cache SET is_stale = 1, updated_at = ?1 WHERE project_id = ?2 AND file_path = ?3")
        .bind(now)
        .bind(project_id)
        .bind(file_path)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn link_evidence_to_goal(
    goal_id: &str,
    evidence_id: &str,
    relevance_score: f64,
    is_active: bool,
) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let active_int = if is_active { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO goal_evidence (goal_id, evidence_id, relevance_score, is_active) \
         VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(goal_id, evidence_id) DO UPDATE SET \
            relevance_score = excluded.relevance_score, \
            is_active = excluded.is_active",
    )
    .bind(goal_id)
    .bind(evidence_id)
    .bind(relevance_score)
    .bind(active_int)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_active_evidences_for_goal(goal_id: &str) -> Result<Vec<EvidenceCache>> {
    let pool = crate::llm::usage::get_pool().await;
    let items = sqlx::query_as::<_, EvidenceCache>(
        "SELECT e.id, e.project_id, e.file_path, e.file_version, e.start_line, e.end_line, e.symbol_name, e.summary, e.raw_snippet, e.is_stale, e.created_at, e.updated_at \
         FROM evidence_cache e \
         JOIN goal_evidence ge ON e.id = ge.evidence_id \
         WHERE ge.goal_id = ?1 AND ge.is_active = 1 AND e.is_stale = 0 \
         ORDER BY ge.relevance_score DESC, e.updated_at DESC",
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await?;
    Ok(items)
}

// ============================================================================
// GOAL VERIFICATIONS REPOSITORY
// ============================================================================

pub async fn record_verification(
    goal_id: &str,
    iteration: i64,
    test_command: &str,
    test_output: &str,
    is_passed: bool,
    feedback: Option<&str>,
) -> Result<GoalVerification> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    let id = generate_id("verif");
    let passed_int = if is_passed { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO goal_verifications (id, goal_id, iteration, test_command, test_output, is_passed, feedback, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&id)
    .bind(goal_id)
    .bind(iteration)
    .bind(test_command)
    .bind(test_output)
    .bind(passed_int)
    .bind(feedback)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(GoalVerification {
        id,
        goal_id: goal_id.to_string(),
        iteration,
        test_command: test_command.to_string(),
        test_output: test_output.to_string(),
        is_passed: passed_int,
        feedback: feedback.map(|s| s.to_string()),
        created_at: now,
    })
}

pub async fn list_goal_verifications(goal_id: &str) -> Result<Vec<GoalVerification>> {
    let pool = crate::llm::usage::get_pool().await;
    let rows = sqlx::query_as::<_, GoalVerification>(
        "SELECT id, goal_id, iteration, test_command, test_output, is_passed, feedback, created_at \
         FROM goal_verifications WHERE goal_id = ?1 ORDER BY iteration ASC",
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
