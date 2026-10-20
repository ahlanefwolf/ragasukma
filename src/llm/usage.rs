use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, Row, SqlitePool};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LlmUsageRecord {
    pub id: String,
    pub timestamp: i64,
    pub date_str: String,
    pub time_str: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub cached_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub duration_ms: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmUsagePaginatedResponse {
    pub items: Vec<LlmUsageRecord>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmUsageSummary {
    pub total_requests: usize,
    pub total_input_tokens: usize,
    pub total_cached_tokens: usize,
    pub total_output_tokens: usize,
    pub total_tokens: usize,
    pub cache_hit_rate_pct: f32,
    pub estimated_saved_tokens: usize,
}

#[allow(dead_code)]
static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub fn get_db_path() -> PathBuf {
    let candidate = crate::assets::find_file_in_candidates("ragasukma.db");
    if candidate.exists() {
        return candidate;
    }

    let dir = crate::config::get_app_data_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join("ragasukma.db")
}

pub async fn get_pool() -> &'static SqlitePool {
    POOL.get_or_init(|| async {
        let db_path = get_db_path();
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let opts = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .expect("Failed to initialize SQLite pool for LLM usage");

        // Run embedded migrations at startup
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run sqlx migrations for LLM usage");

        pool
    })
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn record_usage_async(
    provider: &str,
    model: &str,
    input_tokens: usize,
    cached_tokens: usize,
    output_tokens: usize,
    duration_ms: u64,
    status: &str,
) -> anyhow::Result<LlmUsageRecord> {
    let pool = get_pool().await;
    record_usage_to_pool(
        pool,
        provider,
        model,
        input_tokens,
        cached_tokens,
        output_tokens,
        duration_ms,
        status,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn record_usage_to_pool(
    pool: &SqlitePool,
    provider: &str,
    model: &str,
    input_tokens: usize,
    cached_tokens: usize,
    output_tokens: usize,
    duration_ms: u64,
    status: &str,
) -> anyhow::Result<LlmUsageRecord> {
    let (date_str, time_str) = crate::llm::history::get_local_date_and_time();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let id = format!("usage_{}_{}", timestamp, fastrand_hex(6));
    let total_tokens = (input_tokens + output_tokens) as i64;

    let record = LlmUsageRecord {
        id: id.clone(),
        timestamp,
        date_str: date_str.clone(),
        time_str: time_str.clone(),
        provider: provider.to_lowercase(),
        model: model.to_string(),
        input_tokens: input_tokens as i64,
        cached_tokens: cached_tokens as i64,
        output_tokens: output_tokens as i64,
        total_tokens,
        duration_ms: duration_ms as i64,
        status: status.to_lowercase(),
    };

    sqlx::query(
        r#"
        INSERT INTO usage_records (
            id, timestamp, date_str, time_str, provider, model,
            input_tokens, cached_tokens, output_tokens, total_tokens,
            duration_ms, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
    )
    .bind(&record.id)
    .bind(record.timestamp)
    .bind(&record.date_str)
    .bind(&record.time_str)
    .bind(&record.provider)
    .bind(&record.model)
    .bind(record.input_tokens)
    .bind(record.cached_tokens)
    .bind(record.output_tokens)
    .bind(record.total_tokens)
    .bind(record.duration_ms)
    .bind(&record.status)
    .execute(pool)
    .await?;

    Ok(record)
}

#[allow(clippy::too_many_arguments)]
pub fn record_usage(
    provider: &str,
    model: &str,
    input_tokens: usize,
    cached_tokens: usize,
    output_tokens: usize,
    duration_ms: u64,
    status: &str,
) {
    let provider = provider.to_string();
    let model = model.to_string();
    let status = status.to_string();

    let handle = tokio::runtime::Handle::try_current()
        .ok()
        .or_else(crate::app_state::get_tokio_handle);

    if let Some(h) = handle {
        h.spawn(async move {
            let _ = record_usage_async(
                &provider,
                &model,
                input_tokens,
                cached_tokens,
                output_tokens,
                duration_ms,
                &status,
            )
            .await;
        });
    }
}

pub async fn get_usage_summary(
    provider_filter: Option<&str>,
    date_filter: Option<&str>,
) -> LlmUsageSummary {
    let pool = get_pool().await;
    get_usage_summary_from_pool(pool, provider_filter, date_filter).await
}

pub async fn get_usage_summary_from_pool(
    pool: &SqlitePool,
    provider_filter: Option<&str>,
    date_filter: Option<&str>,
) -> LlmUsageSummary {
    let p_filter = provider_filter
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s != "all");
    let d_filter = date_filter
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "all");

    let mut query = String::from(
        r#"
        SELECT
            COUNT(*) as count,
            COALESCE(SUM(input_tokens), 0) as total_in,
            COALESCE(SUM(cached_tokens), 0) as total_cached,
            COALESCE(SUM(output_tokens), 0) as total_out,
            COALESCE(SUM(total_tokens), 0) as total_all
        FROM usage_records
        WHERE 1=1
        "#,
    );

    if p_filter.is_some() {
        query.push_str(" AND provider LIKE ? ");
    }
    if d_filter.is_some() {
        query.push_str(" AND date_str = ? ");
    }

    let mut q = sqlx::query(&query);
    if let Some(ref p) = p_filter {
        q = q.bind(format!("%{}%", p));
    }
    if let Some(d) = d_filter {
        q = q.bind(d);
    }

    match q.fetch_one(pool).await {
        Ok(row) => {
            let total_requests: i64 = row.try_get("count").unwrap_or(0);
            let total_input_tokens: i64 = row.try_get("total_in").unwrap_or(0);
            let total_cached_tokens: i64 = row.try_get("total_cached").unwrap_or(0);
            let total_output_tokens: i64 = row.try_get("total_out").unwrap_or(0);
            let total_tokens: i64 = row.try_get("total_all").unwrap_or(0);

            let total_requests = total_requests.max(0) as usize;
            let total_input_tokens = total_input_tokens.max(0) as usize;
            let total_cached_tokens = total_cached_tokens.max(0) as usize;
            let total_output_tokens = total_output_tokens.max(0) as usize;
            let total_tokens = total_tokens.max(0) as usize;

            let cache_hit_rate_pct = if total_input_tokens > 0 {
                ((total_cached_tokens as f64 / (total_input_tokens + total_cached_tokens) as f64)
                    * 100.0) as f32
            } else {
                0.0
            };

            LlmUsageSummary {
                total_requests,
                total_input_tokens,
                total_cached_tokens,
                total_output_tokens,
                total_tokens,
                cache_hit_rate_pct,
                estimated_saved_tokens: total_cached_tokens,
            }
        }
        Err(_) => LlmUsageSummary::default(),
    }
}

pub async fn get_usage_records_paginated(
    provider_filter: Option<&str>,
    date_filter: Option<&str>,
    page: usize,
    limit: usize,
) -> LlmUsagePaginatedResponse {
    let pool = get_pool().await;
    get_usage_records_paginated_from_pool(pool, provider_filter, date_filter, page, limit).await
}

pub async fn get_usage_records_paginated_from_pool(
    pool: &SqlitePool,
    provider_filter: Option<&str>,
    date_filter: Option<&str>,
    page: usize,
    limit: usize,
) -> LlmUsagePaginatedResponse {
    let p_filter = provider_filter
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s != "all");
    let d_filter = date_filter
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "all");

    let page = page.max(1);
    let limit = limit.clamp(1, 200);
    let offset = (page - 1) * limit;

    let mut count_query = String::from("SELECT COUNT(*) as cnt FROM usage_records WHERE 1=1");
    if p_filter.is_some() {
        count_query.push_str(" AND provider LIKE ? ");
    }
    if d_filter.is_some() {
        count_query.push_str(" AND date_str = ? ");
    }

    let mut cq = sqlx::query(&count_query);
    if let Some(ref p) = p_filter {
        cq = cq.bind(format!("%{}%", p));
    }
    if let Some(d) = d_filter {
        cq = cq.bind(d);
    }

    let total: usize = cq
        .fetch_one(pool)
        .await
        .ok()
        .and_then(|row| row.try_get::<i64, _>("cnt").ok())
        .map(|c| c.max(0) as usize)
        .unwrap_or(0);

    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(limit)
    };

    let items = get_usage_records_from_pool(pool, provider_filter, date_filter, limit, offset).await;

    LlmUsagePaginatedResponse {
        items,
        total,
        page,
        limit,
        total_pages,
    }
}

pub async fn get_usage_records_from_pool(
    pool: &SqlitePool,
    provider_filter: Option<&str>,
    date_filter: Option<&str>,
    limit: usize,
    offset: usize,
) -> Vec<LlmUsageRecord> {
    let p_filter = provider_filter
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s != "all");
    let d_filter = date_filter
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "all");

    let mut query = String::from("SELECT * FROM usage_records WHERE 1=1");

    if p_filter.is_some() {
        query.push_str(" AND provider LIKE ? ");
    }
    if d_filter.is_some() {
        query.push_str(" AND date_str = ? ");
    }

    query.push_str(" ORDER BY timestamp DESC, id DESC LIMIT ? OFFSET ?");

    let mut q = sqlx::query_as::<_, LlmUsageRecord>(&query);
    if let Some(ref p) = p_filter {
        q = q.bind(format!("%{}%", p));
    }
    if let Some(d) = d_filter {
        q = q.bind(d);
    }
    q = q.bind(limit as i64).bind(offset as i64);

    q.fetch_all(pool).await.unwrap_or_default()
}

pub async fn clear_usage() -> anyhow::Result<()> {
    let pool = get_pool().await;
    clear_usage_from_pool(pool).await
}

pub async fn clear_usage_from_pool(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM usage_records").execute(pool).await?;
    Ok(())
}

fn fastrand_hex(len: usize) -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new().build_hasher().finish();
    let hex = format!("{:016x}", s);
    hex.chars().take(len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlx_sqlite_usage_records_and_summary() {
        // Use an isolated in-memory pool so unit tests never clear or touch user's live database!
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        let rec1 = record_usage_to_pool(
            &pool,
            "antigravity",
            "gemini-2.5-flash",
            1000,
            200,
            150,
            450,
            "success",
        )
        .await
        .unwrap();
        assert_eq!(rec1.total_tokens, 1150);

        let rec2 = record_usage_to_pool(
            &pool,
            "codex",
            "gpt-5-codex",
            2000,
            0,
            300,
            800,
            "success",
        )
        .await
        .unwrap();
        assert_eq!(rec2.total_tokens, 2300);

        let summary = get_usage_summary_from_pool(&pool, None, None).await;
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_input_tokens, 3000);
        assert_eq!(summary.total_cached_tokens, 200);
        assert_eq!(summary.total_output_tokens, 450);
        assert_eq!(summary.total_tokens, 3450);

        let ag_summary = get_usage_summary_from_pool(&pool, Some("antigravity"), None).await;
        assert_eq!(ag_summary.total_requests, 1);
        assert_eq!(ag_summary.total_input_tokens, 1000);
        assert_eq!(ag_summary.total_cached_tokens, 200);

        let records = get_usage_records_from_pool(&pool, None, None, 10, 0).await;
        assert_eq!(records.len(), 2);

        let _ = clear_usage_from_pool(&pool).await;
        let empty_summary = get_usage_summary_from_pool(&pool, None, None).await;
        assert_eq!(empty_summary.total_requests, 0);
    }
}
