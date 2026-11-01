use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

impl SkillItem {
    pub fn new(name: impl Into<String>, desc: impl Into<String>, cmd: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: desc.into(),
            command: cmd.into(),
        }
    }
}

pub async fn get_active_skills_from_db(
    pool: &SqlitePool,
) -> Result<Vec<SkillItem>, sqlx::Error> {
    sqlx::query_as::<_, SkillItem>(
        "SELECT name, description, command FROM skills WHERE is_active = 1 ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn save_skill_to_db(
    pool: &SqlitePool,
    skill: &SkillItem,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO skills (name, description, command, is_active, created_at) VALUES (?1, ?2, ?3, 1, ?4) ON CONFLICT(name) DO UPDATE SET description = excluded.description, command = excluded.command, is_active = 1",
    )
    .bind(&skill.name)
    .bind(&skill.description)
    .bind(&skill.command)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_skill_from_db(pool: &SqlitePool, name: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM skills WHERE name = ?1 COLLATE NOCASE")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}
