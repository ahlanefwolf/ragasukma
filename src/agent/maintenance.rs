//! Database maintenance: bounded retention for high-churn agent telemetry.
//!
//! `messages` and `agent_audit_logs` grow with every model turn and every tool call.
//! Left alone they grow without bound. A sweep runs once at startup and then
//! periodically, so long-lived sessions are also pruned.
//!
//! Retention windows and the sweep interval are read from the environment so the
//! policy stays a runtime knob rather than a value baked into SQL:
//!
//! - `RAGASUKMA_AGENT_MESSAGE_RETENTION_DAYS`    (default 30, `0` disables)
//! - `RAGASUKMA_AGENT_AUDIT_RETENTION_DAYS`      (default 30, `0` disables)
//! - `RAGASUKMA_AGENT_SESSION_RETENTION_DAYS`    (default 90, `0` disables)
//! - `RAGASUKMA_AGENT_RETENTION_INTERVAL_HOURS`  (default 6, `0` = startup only)

use crate::agent::repository;

const ENV_MESSAGE_RETENTION_DAYS: &str = "RAGASUKMA_AGENT_MESSAGE_RETENTION_DAYS";
const ENV_AUDIT_RETENTION_DAYS: &str = "RAGASUKMA_AGENT_AUDIT_RETENTION_DAYS";
const ENV_RETENTION_INTERVAL_HOURS: &str = "RAGASUKMA_AGENT_RETENTION_INTERVAL_HOURS";
const ENV_SESSION_RETENTION_DAYS: &str = "RAGASUKMA_AGENT_SESSION_RETENTION_DAYS";

const DEFAULT_MESSAGE_RETENTION_DAYS: u64 = 30;
const DEFAULT_AUDIT_RETENTION_DAYS: u64 = 30;
const DEFAULT_RETENTION_INTERVAL_HOURS: u64 = 6;
const DEFAULT_SESSION_RETENTION_DAYS: u64 = 90;

const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;

/// Parses a retention window in days from a raw value, falling back to `default`.
/// Returns `None` when retention is explicitly disabled (`0`).
fn parse_retention_days(raw: Option<&str>, default: u64) -> Option<u64> {
    let days = raw
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(default);

    if days == 0 {
        None
    } else {
        Some(days)
    }
}

/// Reads a retention window in days from the environment, falling back to `default`.
fn retention_days(env_key: &str, default: u64) -> Option<u64> {
    let raw = std::env::var(env_key).ok();
    parse_retention_days(raw.as_deref(), default)
}

/// Converts a day count into an absolute epoch-second cutoff relative to now.
fn cutoff_epoch_secs(days: u64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now.saturating_sub(days.saturating_mul(SECONDS_PER_DAY)) as i64
}

/// Prunes expired `messages` and `agent_audit_logs` rows.
///
/// This is best-effort: failures are logged and never abort startup, because losing
/// maintenance should not stop the application from running.
pub async fn run_retention_sweep() {
    if let Some(days) = retention_days(ENV_MESSAGE_RETENTION_DAYS, DEFAULT_MESSAGE_RETENTION_DAYS) {
        let cutoff = cutoff_epoch_secs(days);
        match repository::delete_expired_messages(cutoff).await {
            Ok(0) => {}
            Ok(n) => crate::term_log!("[Retention] Pruned {} message row(s) older than {}d", n, days),
            Err(e) => crate::term_err!("[Retention] Message sweep failed: {e}"),
        }
    }

    if let Some(days) = retention_days(ENV_AUDIT_RETENTION_DAYS, DEFAULT_AUDIT_RETENTION_DAYS) {
        let cutoff = cutoff_epoch_secs(days);
        match repository::delete_expired_audit_logs(cutoff).await {
            Ok(0) => {}
            Ok(n) => crate::term_log!("[Retention] Pruned {} audit row(s) older than {}d", n, days),
            Err(e) => crate::term_err!("[Retention] Audit sweep failed: {e}"),
        }
    }

    // Stale sessions carry their goals and evidence with them, so this also reclaims
    // everything that hung off abandoned conversations.
    if let Some(days) = retention_days(ENV_SESSION_RETENTION_DAYS, DEFAULT_SESSION_RETENTION_DAYS) {
        let cutoff = cutoff_epoch_secs(days);
        match repository::delete_expired_sessions(cutoff).await {
            Ok(0) => {}
            Ok(n) => crate::term_log!("[Retention] Pruned {} session(s) idle for over {}d", n, days),
            Err(e) => crate::term_err!("[Retention] Session sweep failed: {e}"),
        }
    }

    // Projects with no sessions left hold nothing recoverable. Sweeping them keeps the
    // project list meaningful instead of accumulating abandoned experiments forever.
    match repository::delete_orphan_projects().await {
        Ok(0) => {}
        Ok(n) => crate::term_log!("[Retention] Pruned {} orphaned project(s)", n),
        Err(e) => crate::term_err!("[Retention] Orphan project sweep failed: {e}"),
    }
}

/// Runs retention once immediately, then keeps re-running it on a configurable interval.
///
/// This is meant to be spawned once at startup. A single sweep at boot is not enough for
/// long-lived sessions, which would otherwise accumulate rows forever.
pub async fn run_retention_loop() {
    // Always sweep at least once so a machine that has been off for weeks is cleaned at boot.
    run_retention_sweep().await;

    let hours = std::env::var(ENV_RETENTION_INTERVAL_HOURS)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_RETENTION_INTERVAL_HOURS);

    // `0` means "startup only": honour it instead of spinning.
    if hours == 0 {
        return;
    }

    let interval = std::time::Duration::from_secs(hours.saturating_mul(SECONDS_PER_HOUR));
    loop {
        tokio::time::sleep(interval).await;
        run_retention_sweep().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_days_zero_disables() {
        assert_eq!(parse_retention_days(Some("0"), 30), None);
    }

    #[test]
    fn retention_days_falls_back_to_default() {
        assert_eq!(parse_retention_days(None, 30), Some(30));
        assert_eq!(parse_retention_days(Some("not-a-number"), 30), Some(30));
    }

    #[test]
    fn retention_days_honors_override() {
        assert_eq!(parse_retention_days(Some("7"), 30), Some(7));
        assert_eq!(parse_retention_days(Some(" 14 "), 30), Some(14));
    }

    #[tokio::test]
    async fn sweep_deletes_rows_older_than_cutoff() {
        let _guard = crate::agent::repository::testsupport::db_test_lock().await;
        let pool = crate::llm::usage::get_pool().await;

        let project = repository::create_project("Retention Test", "", None, None)
            .await
            .unwrap();
        let session = repository::create_session(&project.id, "S", None, None, None)
            .await
            .unwrap();

        // Two messages: one ancient, one fresh.
        //
        // The fresh row sits well inside the retention window: placing it exactly on the cutoff
        // boundary makes the test race the wall clock, since the sweep recomputes `now` itself.
        let ancient = 0i64;
        let fresh = super::cutoff_epoch_secs(0);
        sqlx::query("INSERT INTO messages (id, session_id, sequence_num, role, content, status, created_at) VALUES ('ret_old', ?1, 1, 'user', 'old', 'success', ?2)")
            .bind(&session.id)
            .bind(ancient)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO messages (id, session_id, sequence_num, role, content, status, created_at) VALUES ('ret_new', ?1, 2, 'user', 'new', 'success', ?2)")
            .bind(&session.id)
            .bind(fresh)
            .execute(pool)
            .await
            .unwrap();

        // The sweep is global, so assert on this fixture's rows rather than a global count.
        repository::delete_expired_messages(cutoff_epoch_secs(1)).await.unwrap();

        let survivors: Vec<String> =
            sqlx::query_scalar("SELECT id FROM messages WHERE id IN ('ret_old','ret_new')")
                .fetch_all(pool)
                .await
                .unwrap();
        assert_eq!(survivors, vec!["ret_new".to_string()], "only the fresh message must survive");
    }

    #[tokio::test]
    async fn audit_sweep_deletes_rows_older_than_cutoff() {
        let _guard = crate::agent::repository::testsupport::db_test_lock().await;
        let pool = crate::llm::usage::get_pool().await;
        let ancient = 0i64;
        // Keep the fresh row strictly inside the window so a clock tick cannot expire it.
        let fresh = super::cutoff_epoch_secs(0);
        sqlx::query("INSERT INTO agent_audit_logs (id, tool_name, status, created_at) VALUES ('aud_old', 'workspace_read_file', 'executed', ?1)")
            .bind(ancient)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO agent_audit_logs (id, tool_name, status, created_at) VALUES ('aud_new', 'workspace_read_file', 'executed', ?1)")
            .bind(fresh)
            .execute(pool)
            .await
            .unwrap();

        // The sweep is global: other tests may own audit rows too, so assert on this fixture's
        // rows specifically instead of on a global delete count.
        repository::delete_expired_audit_logs(cutoff_epoch_secs(1)).await.unwrap();

        let survivors: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM agent_audit_logs WHERE id IN ('aud_old','aud_new')",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        assert_eq!(survivors, vec!["aud_new".to_string()], "only the fresh audit row must survive");
    }
}

#[cfg(test)]
mod retention_scope_tests {
    use super::*;

    // This test asserts on global sweeps (`delete_orphan_projects` removes every project
    // with no sessions), so it must not overlap with other DB tests that create their own
    // session-less fixtures mid-run.
    #[tokio::test]
    async fn expired_sessions_and_orphan_projects_are_pruned() {
        let _guard = crate::agent::repository::testsupport::db_test_lock().await;
        let stale_project = repository::create_project("Stale", "", None, None).await.unwrap();
        let stale_session = repository::create_session(&stale_project.id, "old", None, None, None)
            .await
            .unwrap();

        let live_project = repository::create_project("Live", "", None, None).await.unwrap();
        let live_session = repository::create_session(&live_project.id, "new", None, None, None)
            .await
            .unwrap();

        let pool = crate::llm::usage::get_pool().await;
        sqlx::query("UPDATE sessions SET updated_at = 1 WHERE id = ?1")
            .bind(&stale_session.id)
            .execute(pool)
            .await
            .unwrap();

        let cutoff = cutoff_epoch_secs(1);
        let removed = repository::delete_expired_sessions(cutoff).await.unwrap();
        assert!(removed >= 1, "stale session should be pruned");

        let orphans = repository::delete_orphan_projects().await.unwrap();
        assert!(orphans >= 1, "orphan project should be pruned");

        let gone: Option<i64> = sqlx::query_scalar("SELECT 1 FROM projects WHERE id = ?1")
            .bind(&stale_project.id)
            .fetch_optional(pool)
            .await
            .unwrap();
        assert!(gone.is_none(), "stale project must be gone");

        let survived: Option<i64> = sqlx::query_scalar("SELECT 1 FROM sessions WHERE id = ?1")
            .bind(&live_session.id)
            .fetch_optional(pool)
            .await
            .unwrap();
        assert!(survived.is_some(), "live session must survive");
        let _ = live_project;
    }
}
