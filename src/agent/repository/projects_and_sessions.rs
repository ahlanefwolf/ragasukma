use anyhow::Result;
use crate::agent::models::{McpServer, Project, PromptSkill, Session};
use super::{generate_id, now_secs};

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
    let temp = temperature.unwrap_or(0.1);

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
