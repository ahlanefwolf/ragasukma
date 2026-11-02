use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq, Eq)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

impl SkillItem {
    pub fn new(name: impl Into<String>, description: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            command: command.into(),
        }
    }
}

static SKILLS_CACHE: Mutex<Option<Vec<SkillItem>>> = Mutex::new(None);
static SKILLS_MUTEX: Mutex<()> = Mutex::new(());

fn get_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

use crate::app_state::spawn_db_task;

pub fn format_skills_markdown(skills: &[SkillItem]) -> String {
    let mut out = String::from(
        "#  Skill Command Registry\n\n\
List of quick commands and system automations that can be triggered directly via voice, menu, or AI automation.\n\
This file can be edited directly or managed via the Skill Commands UI.\n\n\
| Skill Name | Description | Command / Script |\n\
| :--- | :--- | :--- |\n",
    );

    for s in skills {
        out.push_str(&format!(
            "| {} | {} | `{}` |\n",
            s.name.trim(),
            s.description.trim(),
            s.command.trim()
        ));
    }
    out
}

fn ensure_cache_loaded() {
    let mut guard = SKILLS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(Vec::new());

        spawn_db_task(async {
            let _ = sync_skills_from_db().await;
        });
    }
}

pub async fn sync_skills_from_db() -> Vec<SkillItem> {
    let pool = crate::llm::usage::get_pool().await;
    let rows = sqlx::query_as::<_, SkillItem>(
        "SELECT name, description, command FROM skills WHERE is_active = 1 ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut guard = SKILLS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(rows.clone());
    rows
}

pub async fn save_skill_to_db(skill: &SkillItem) -> Result<(), String> {
    let pool = crate::llm::usage::get_pool().await;
    let now = get_now_secs();

    sqlx::query(
        "INSERT INTO skills (name, description, command, is_active, created_at) VALUES (?1, ?2, ?3, 1, ?4) ON CONFLICT(name) DO UPDATE SET description = excluded.description, command = excluded.command, is_active = 1",
    )
    .bind(&skill.name)
    .bind(&skill.description)
    .bind(&skill.command)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let _ = sync_skills_from_db().await;
    Ok(())
}

pub async fn delete_skill_from_db(name: &str) -> Result<(), String> {
    let pool = crate::llm::usage::get_pool().await;

    sqlx::query("DELETE FROM skills WHERE name = ?1 COLLATE NOCASE")
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    let _ = sync_skills_from_db().await;
    Ok(())
}

pub fn parse_skills_markdown() -> Vec<SkillItem> {
    ensure_cache_loaded();
    let guard = SKILLS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().cloned().unwrap_or_default()
}

pub fn save_skills_markdown(skills: &[SkillItem]) -> std::io::Result<()> {
    let _sync_guard = SKILLS_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    {
        let mut guard = SKILLS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(skills.to_vec());
    }

    let items_clone = skills.to_vec();
    let now = get_now_secs();
    spawn_db_task(async move {
        let pool = crate::llm::usage::get_pool().await;
        for s in &items_clone {
            let _ = sqlx::query(
                "INSERT INTO skills (name, description, command, is_active, created_at) VALUES (?1, ?2, ?3, 1, ?4) ON CONFLICT(name) DO UPDATE SET description = excluded.description, command = excluded.command, is_active = 1",
            )
            .bind(&s.name)
            .bind(&s.description)
            .bind(&s.command)
            .bind(now)
            .execute(pool)
            .await;
        }
    });

    Ok(())
}

/// Utility helper to configure subprocess to be isolated from AppImage runtime and prevent console flash on Windows
pub use crate::agent::engine::subprocess::configure_subprocess;

pub fn execute_command_by_os(cmd: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let trimmed = cmd.trim();
        // If command is a direct executable or file (e.g. calc.exe, notepad.exe, explorer.exe)
        // and does not contain shell pipes or redirects, run directly without cmd /C
        if !trimmed.contains('|')
            && !trimmed.contains('&')
            && !trimmed.contains('>')
            && !trimmed.contains('<')
        {
            if let Some((prog, args)) = trimmed.split_once(' ') {
                let clean_prog = prog.trim_matches('"');
                let clean_args = args.trim_matches('"');
                let mut direct_cmd = std::process::Command::new(clean_prog);
                direct_cmd.arg(clean_args);
                configure_subprocess(&mut direct_cmd);
                if direct_cmd.spawn().is_ok() {
                    return Ok(());
                }
            } else {
                let clean_prog = trimmed.trim_matches('"');
                let mut direct_cmd = std::process::Command::new(clean_prog);
                configure_subprocess(&mut direct_cmd);
                if direct_cmd.spawn().is_ok() {
                    return Ok(());
                }
            }
        }

        let mut command = std::process::Command::new("cmd");
        // Using /S /C "<cmd>" so that parser cmd.exe not memotong tanda kutip ganda internal
        command.args(["/S", "/C", &format!("\"{trimmed}\"")]);
        configure_subprocess(&mut command);
        command.spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", cmd]);
        configure_subprocess(&mut command);
        command.spawn()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_item_new_and_format() {
        let skill = SkillItem::new("Open Terminal", "Opens bash terminal", "alacritty");
        assert_eq!(skill.name, "Open Terminal");
        assert_eq!(skill.description, "Opens bash terminal");
        assert_eq!(skill.command, "alacritty");

        let formatted = format_skills_markdown(&[skill]);
        assert!(formatted.contains("| Open Terminal | Opens bash terminal | `alacritty` |"));
    }
}