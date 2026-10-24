use anyhow::Result;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use super::models::{McpServer, Message, Project, PromptSkill, Session};

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn generate_id(prefix: &str) -> String {
    let s = RandomState::new().build_hasher().finish();
    let hex = format!("{:016x}", s);
    format!("{}_{}_{}", prefix, now_secs(), &hex[..6])
}

// ============================================================================
// PROJECTS REPOSITORY
// ============================================================================

pub async fn list_projects() -> Result<Vec<Project>> {
    let pool = crate::llm::usage::get_pool().await;
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, root_path, custom_instructions, created_at, updated_at \
         FROM projects ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(projects)
}

pub async fn get_project(id: &str) -> Result<Option<Project>> {
    let pool = crate::llm::usage::get_pool().await;
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, root_path, custom_instructions, created_at, updated_at \
         FROM projects WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(project)
}

pub async fn create_project(
    name: &str,
    description: &str,
    root_path: Option<&str>,
    custom_instructions: Option<&str>,
) -> Result<Project> {
    let pool = crate::llm::usage::get_pool().await;
    let id = generate_id("proj");
    let now = now_secs();

    sqlx::query(
        "INSERT INTO projects (id, name, description, root_path, custom_instructions, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(root_path)
    .bind(custom_instructions)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Project {
        id,
        name: name.to_string(),
        description: description.to_string(),
        root_path: root_path.map(|s| s.to_string()),
        custom_instructions: custom_instructions.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_project(
    id: &str,
    name: &str,
    description: &str,
    root_path: Option<&str>,
    custom_instructions: Option<&str>,
) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();

    sqlx::query(
        "UPDATE projects SET name = ?1, description = ?2, root_path = ?3, custom_instructions = ?4, updated_at = ?5 \
         WHERE id = ?6",
    )
    .bind(name)
    .bind(description)
    .bind(root_path)
    .bind(custom_instructions)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_project(id: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    sqlx::query("DELETE FROM projects WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// SESSIONS REPOSITORY
// ============================================================================

pub async fn list_sessions(project_id: &str) -> Result<Vec<Session>> {
    let pool = crate::llm::usage::get_pool().await;
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT id, project_id, title, status, model, temperature, source_chat_id, metadata, created_at, updated_at \
         FROM sessions WHERE project_id = ?1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(sessions)
}

pub async fn get_session(id: &str) -> Result<Option<Session>> {
    let pool = crate::llm::usage::get_pool().await;
    let session = sqlx::query_as::<_, Session>(
        "SELECT id, project_id, title, status, model, temperature, source_chat_id, metadata, created_at, updated_at \
         FROM sessions WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(session)
}

pub async fn create_session(
    project_id: &str,
    title: &str,
    model: Option<&str>,
    temperature: Option<f64>,
    source_chat_id: Option<&str>,
) -> Result<Session> {
    let pool = crate::llm::usage::get_pool().await;
    let id = generate_id("sess");
    let now = now_secs();
    let temp = temperature.unwrap_or(0.7);

    sqlx::query(
        "INSERT INTO sessions (id, project_id, title, status, model, temperature, source_chat_id, metadata, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 'idle', ?4, ?5, ?6, '{}', ?7, ?8)",
    )
    .bind(&id)
    .bind(project_id)
    .bind(title)
    .bind(model)
    .bind(temp)
    .bind(source_chat_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Session {
        id,
        project_id: project_id.to_string(),
        title: title.to_string(),
        status: "idle".to_string(),
        model: model.map(|s| s.to_string()),
        temperature: Some(temp),
        source_chat_id: source_chat_id.map(|s| s.to_string()),
        metadata: "{}".to_string(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_session_status(id: &str) -> Result<Option<String>> {
    let pool = crate::llm::usage::get_pool().await;
    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM sessions WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn update_session_status(id: &str, status: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(status)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_session_model(id: &str, model: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    sqlx::query("UPDATE sessions SET model = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(model)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_session(id: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    // 1. Clean up goal verifications and goal evidence for any goal in this session
    sqlx::query("DELETE FROM goal_verifications WHERE goal_id IN (SELECT id FROM goals WHERE session_id = ?1)")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM goal_evidence WHERE goal_id IN (SELECT id FROM goals WHERE session_id = ?1)")
        .bind(id)
        .execute(pool)
        .await?;
    // 2. Clean up goals belonging to this session
    sqlx::query("DELETE FROM goals WHERE session_id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    // 3. Purge orphaned evidence_cache entries no longer attached to any active goal
    sqlx::query("DELETE FROM evidence_cache WHERE id NOT IN (SELECT evidence_id FROM goal_evidence)")
        .execute(pool)
        .await?;
    // 4. Delete the session itself (messages cascade via foreign key)
    sqlx::query("DELETE FROM sessions WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// MESSAGES REPOSITORY
// ============================================================================

pub async fn list_messages(session_id: &str) -> Result<Vec<Message>> {
    let pool = crate::llm::usage::get_pool().await;
    let messages = sqlx::query_as::<_, Message>(
        "SELECT id, session_id, sequence_num, role, content, thought, tool_calls, tool_call_id, \
         status, token_usage_input, token_usage_output, created_at \
         FROM messages WHERE session_id = ?1 ORDER BY sequence_num ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(messages)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_message(
    session_id: &str,
    role: &str,
    content: Option<&str>,
    thought: Option<&str>,
    tool_calls: Option<&str>,
    tool_call_id: Option<&str>,
    status: &str,
    token_usage_input: Option<i64>,
    token_usage_output: Option<i64>,
) -> Result<Message> {
    let pool = crate::llm::usage::get_pool().await;
    let id = generate_id("msg");
    let now = now_secs();

    // Compute next sequence number atomically
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(sequence_num), 0) + 1 FROM messages WHERE session_id = ?1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    let sequence_num = row.0;

    sqlx::query(
        "INSERT INTO messages (id, session_id, sequence_num, role, content, thought, tool_calls, tool_call_id, \
         status, token_usage_input, token_usage_output, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(sequence_num)
    .bind(role)
    .bind(content)
    .bind(thought)
    .bind(tool_calls)
    .bind(tool_call_id)
    .bind(status)
    .bind(token_usage_input)
    .bind(token_usage_output)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Message {
        id,
        session_id: session_id.to_string(),
        sequence_num,
        role: role.to_string(),
        content: content.map(|s| s.to_string()),
        thought: thought.map(|s| s.to_string()),
        tool_calls: tool_calls.map(|s| s.to_string()),
        tool_call_id: tool_call_id.map(|s| s.to_string()),
        status: status.to_string(),
        token_usage_input,
        token_usage_output,
        created_at: now,
    })
}

// ============================================================================
// PROMPT SKILLS REPOSITORY
// ============================================================================

pub async fn list_prompt_skills() -> Result<Vec<PromptSkill>> {
    let pool = crate::llm::usage::get_pool().await;
    let skills = sqlx::query_as::<_, PromptSkill>(
        "SELECT id, name, slug, description, system_prompt, tags, is_active, created_at, updated_at \
         FROM prompt_skills ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(skills)
}

pub async fn create_prompt_skill(
    name: &str,
    slug: &str,
    description: &str,
    system_prompt: &str,
    tags_json: &str,
) -> Result<PromptSkill> {
    let pool = crate::llm::usage::get_pool().await;
    let id = generate_id("pskill");
    let now = now_secs();

    sqlx::query(
        "INSERT INTO prompt_skills (id, name, slug, description, system_prompt, tags, is_active, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
    )
    .bind(&id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(system_prompt)
    .bind(tags_json)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(PromptSkill {
        id,
        name: name.to_string(),
        slug: slug.to_string(),
        description: description.to_string(),
        system_prompt: system_prompt.to_string(),
        tags: tags_json.to_string(),
        is_active: 1,
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_prompt_skill(
    id: &str,
    name: &str,
    slug: &str,
    description: &str,
    system_prompt: &str,
    tags_json: &str,
) -> Result<PromptSkill> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();

    sqlx::query(
        "UPDATE prompt_skills \
         SET name = ?1, slug = ?2, description = ?3, system_prompt = ?4, tags = ?5, updated_at = ?6 \
         WHERE id = ?7",
    )
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(system_prompt)
    .bind(tags_json)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, PromptSkill>(
        "SELECT id, name, slug, description, system_prompt, tags, is_active, created_at, updated_at \
         FROM prompt_skills WHERE id = ?1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_prompt_skill(id: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    sqlx::query("DELETE FROM prompt_skills WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_project_skills(project_id: &str) -> Result<Vec<PromptSkill>> {
    let pool = crate::llm::usage::get_pool().await;
    let skills = sqlx::query_as::<_, PromptSkill>(
        "SELECT ps.id, ps.name, ps.slug, ps.description, ps.system_prompt, ps.tags, ps.is_active, ps.created_at, ps.updated_at \
         FROM prompt_skills ps \
         JOIN project_prompt_skills pps ON ps.id = pps.skill_id \
         WHERE pps.project_id = ?1 AND pps.is_enabled = 1 AND ps.is_active = 1 \
         ORDER BY pps.priority ASC, ps.name ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(skills)
}

pub async fn set_project_skills(project_id: &str, skill_ids: &[String]) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    sqlx::query("DELETE FROM project_prompt_skills WHERE project_id = ?1")
        .bind(project_id)
        .execute(pool)
        .await?;

    for (priority, sid) in skill_ids.iter().enumerate() {
        let _ = sqlx::query(
            "INSERT INTO project_prompt_skills (project_id, skill_id, priority, is_enabled) \
             VALUES (?1, ?2, ?3, 1)",
        )
        .bind(project_id)
        .bind(sid)
        .bind(priority as i64)
        .execute(pool)
        .await;
    }

    Ok(())
}

// ============================================================================
// MCP SERVERS REPOSITORY
// ============================================================================

pub async fn list_mcp_servers() -> Result<Vec<McpServer>> {
    let pool = crate::llm::usage::get_pool().await;
    let servers = sqlx::query_as::<_, McpServer>(
        "SELECT id, name, transport, command, args, env, url, is_enabled, created_at \
         FROM mcp_servers ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(servers)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_or_update_mcp_server(
    id: &str,
    name: &str,
    transport: &str,
    command: Option<&str>,
    args: &str,
    env: &str,
    url: Option<&str>,
    is_enabled: bool,
) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    let now = now_secs();
    let enabled_int = if is_enabled { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO mcp_servers (id, name, transport, command, args, env, url, is_enabled, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
         ON CONFLICT(id) DO UPDATE SET \
            name = excluded.name, \
            transport = excluded.transport, \
            command = excluded.command, \
            args = excluded.args, \
            env = excluded.env, \
            url = excluded.url, \
            is_enabled = excluded.is_enabled",
    )
    .bind(id)
    .bind(name)
    .bind(transport)
    .bind(command)
    .bind(args)
    .bind(env)
    .bind(url)
    .bind(enabled_int)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_mcp_server(id: &str) -> Result<()> {
    let pool = crate::llm::usage::get_pool().await;
    sqlx::query("DELETE FROM mcp_servers WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============================================================================
// GOAL STORE REPOSITORY
// ============================================================================

use super::models::{EvidenceCache, Goal, GoalVerification};

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_agent_tables_hierarchy() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        // 1. Create a Project
        let proj_id = "test_proj_1";
        sqlx::query("INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, 1000, 1000)")
            .bind(proj_id)
            .bind("Test AI Voice Project")
            .bind("Agent testing workspace")
            .execute(&pool)
            .await
            .unwrap();

        // 2. Create a Session
        let sess_id = "test_sess_1";
        sqlx::query("INSERT INTO sessions (id, project_id, title, status, created_at, updated_at) VALUES (?1, ?2, ?3, 'idle', 1001, 1001)")
            .bind(sess_id)
            .bind(proj_id)
            .bind("Refactor Audio Loop")
            .execute(&pool)
            .await
            .unwrap();

        // 3. Create Messages with sequence
        sqlx::query(
            "INSERT INTO messages (id, session_id, sequence_num, role, content, status, created_at) \
             VALUES ('msg_1', ?1, 1, 'user', 'Analyze latency', 'success', 1002)",
        )
        .bind(sess_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO messages (id, session_id, sequence_num, role, content, thought, status, created_at) \
             VALUES ('msg_2', ?1, 2, 'assistant', 'Here is the latency breakdown', 'Thinking steps...', 'success', 1003)",
        )
        .bind(sess_id)
        .execute(&pool)
        .await
        .unwrap();

        // 4. Query messages
        let msgs = sqlx::query_as::<_, Message>(
            "SELECT id, session_id, sequence_num, role, content, thought, tool_calls, tool_call_id, \
             status, token_usage_input, token_usage_output, created_at \
             FROM messages WHERE session_id = ?1 ORDER BY sequence_num ASC",
        )
        .bind(sess_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[0].sequence_num, 1);
        assert_eq!(msgs[1].role, "assistant");
        assert_eq!(msgs[1].sequence_num, 2);
        assert_eq!(msgs[1].thought.as_deref(), Some("Thinking steps..."));

        // 5. Test Cascade Deletion: deleting project should cascade to sessions and messages
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(proj_id)
            .execute(&pool)
            .await
            .unwrap();

        let remaining_msgs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE session_id = ?1")
            .bind(sess_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining_msgs.0, 0);
    }

    #[tokio::test]
    async fn test_goal_and_evidence_lifecycle() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        // 1. Setup project
        let proj_id = "proj_auth";
        sqlx::query("INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, 'Auth App', '', 1000, 1000)")
            .bind(proj_id)
            .execute(&pool)
            .await
            .unwrap();

        // 2. Demand 1 (G1): Fix login bug
        sqlx::query(
            "INSERT INTO goals (id, project_id, title, demand, success_criteria, constraints, status, created_at, updated_at) \
             VALUES ('G1', ?1, 'Fix login bug', 'Fix token validation', '[\"valid credentials 200\"]', '[]', 'discovery', 1001, 1001)",
        )
        .bind(proj_id)
        .execute(&pool)
        .await
        .unwrap();

        // Discovery loop: store evidence
        let ev1_id = "ev_handler_login";
        sqlx::query(
            "INSERT INTO evidence_cache (id, project_id, file_path, file_version, start_line, end_line, symbol_name, summary, is_stale, created_at, updated_at) \
             VALUES (?1, ?2, 'src/auth/handler.rs', 'v_hash_1', 20, 50, 'login_handler', 'Entry point POST /login', 0, 1002, 1002)",
        )
        .bind(ev1_id)
        .bind(proj_id)
        .execute(&pool)
        .await
        .unwrap();

        // Link evidence to G1
        sqlx::query("INSERT INTO goal_evidence (goal_id, evidence_id, relevance_score, is_active) VALUES ('G1', ?1, 1.0, 1)")
            .bind(ev1_id)
            .execute(&pool)
            .await
            .unwrap();

        // Verification & Conclusion of G1
        sqlx::query(
            "INSERT INTO goal_verifications (id, goal_id, iteration, test_command, test_output, is_passed, created_at) \
             VALUES ('verif_g1', 'G1', 1, 'cargo test test_login', 'test result: ok. 1 passed', 1, 1003)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("UPDATE goals SET status = 'completed', conclusion = 'Bug token validation fixed and verified', updated_at = 1004 WHERE id = 'G1'")
            .execute(&pool)
            .await
            .unwrap();

        // 3. Demand 2 (G2): Add rate limiting on login (Reuse evidence from G1)
        sqlx::query(
            "INSERT INTO goals (id, project_id, parent_goal_id, title, demand, success_criteria, constraints, status, created_at, updated_at) \
             VALUES ('G2', ?1, 'G1', 'Add rate limiting on login', 'Max 5 attempts', '[\"rate limit 429\"]', '[]', 'discovery', 1005, 1005)",
        )
        .bind(proj_id)
        .execute(&pool)
        .await
        .unwrap();

        // Reuse evidence ev1 for G2
        sqlx::query("INSERT INTO goal_evidence (goal_id, evidence_id, relevance_score, is_active) VALUES ('G2', ?1, 0.9, 1)")
            .bind(ev1_id)
            .execute(&pool)
            .await
            .unwrap();

        // Check active evidences for G2
        let g2_evidences = sqlx::query_as::<_, EvidenceCache>(
            "SELECT e.id, e.project_id, e.file_path, e.file_version, e.start_line, e.end_line, e.symbol_name, e.summary, e.raw_snippet, e.is_stale, e.created_at, e.updated_at \
             FROM evidence_cache e \
             JOIN goal_evidence ge ON e.id = ge.evidence_id \
             WHERE ge.goal_id = 'G2' AND ge.is_active = 1 AND e.is_stale = 0",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(g2_evidences.len(), 1);
        assert_eq!(g2_evidences[0].file_path, "src/auth/handler.rs");

        // Mark stale if handler file changed
        sqlx::query("UPDATE evidence_cache SET is_stale = 1 WHERE file_path = 'src/auth/handler.rs'")
            .execute(&pool)
            .await
            .unwrap();

        // G2 active fresh evidences should now be empty until revalidated
        let fresh_ev = sqlx::query_as::<_, EvidenceCache>(
            "SELECT e.id, e.project_id, e.file_path, e.file_version, e.start_line, e.end_line, e.symbol_name, e.summary, e.raw_snippet, e.is_stale, e.created_at, e.updated_at \
             FROM evidence_cache e \
             JOIN goal_evidence ge ON e.id = ge.evidence_id \
             WHERE ge.goal_id = 'G2' AND ge.is_active = 1 AND e.is_stale = 0",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(fresh_ev.len(), 0);
    }
}
