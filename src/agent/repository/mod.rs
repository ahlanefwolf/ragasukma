use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod goals;
pub mod messages;
pub mod projects_and_sessions;

pub use goals::*;
pub use messages::*;
pub use projects_and_sessions::*;

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn generate_id(prefix: &str) -> String {
    let s = RandomState::new().build_hasher().finish();
    let hex = format!("{:016x}", s);
    format!("{}_{}_{}", prefix, now_secs(), &hex[..6])
}

#[cfg(test)]
mod tests {
    use crate::agent::models::EvidenceCache;
    use crate::agent::models::Message;
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
