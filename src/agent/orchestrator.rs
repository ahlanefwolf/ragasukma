use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::context_builder::{build_goal_context_prompt, revalidate_evidence_cache, reuse_evidence_from_parent};
use super::models::{EvidenceCache, Goal, GoalVerification};
use super::repository;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalState {
    Initialized,
    Discovery,
    Action,
    Verification,
    Completed,
    Failed,
}

impl GoalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            GoalState::Initialized => "initialized",
            GoalState::Discovery => "discovery",
            GoalState::Action => "action",
            GoalState::Verification => "verification",
            GoalState::Completed => "completed",
            GoalState::Failed => "failed",
        }
    }
}

impl std::str::FromStr for GoalState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "discovery" => GoalState::Discovery,
            "action" => GoalState::Action,
            "verification" => GoalState::Verification,
            "completed" => GoalState::Completed,
            "failed" => GoalState::Failed,
            _ => GoalState::Initialized,
        })
    }
}

pub struct GoalOrchestrator {
    pub goal_id: String,
    pub project_id: String,
    pub max_discovery_iterations: usize,
    pub max_verification_loops: usize,
}

impl GoalOrchestrator {
    pub fn new(goal_id: &str, project_id: &str) -> Self {
        Self {
            goal_id: goal_id.to_string(),
            project_id: project_id.to_string(),
            max_discovery_iterations: 15,
            max_verification_loops: 5,
        }
    }

    /// Step 1: Initialize Goal and prepare initial reusable context
    #[allow(clippy::too_many_arguments)]
    pub async fn initialize_demand(
        goal_id: &str,
        project_id: &str,
        session_id: Option<&str>,
        parent_goal_id: Option<&str>,
        title: &str,
        demand: &str,
        success_criteria: &[String],
        constraints: &[String],
        reuse_keywords: &[&str],
        workspace_roots: &[String],
    ) -> Result<Goal> {
        let sc_json = serde_json::to_string(success_criteria)?;
        let const_json = serde_json::to_string(constraints)?;

        let goal = repository::create_goal(
            goal_id,
            project_id,
            session_id,
            parent_goal_id,
            title,
            demand,
            &sc_json,
            &const_json,
        )
        .await?;

        // 1. Revalidate any stale cache on disk
        let _ = revalidate_evidence_cache(project_id, workspace_roots).await;

        // 2. Reuse fresh evidence strictly if parent goal is linked
        if let Some(parent_id) = parent_goal_id {
            if !reuse_keywords.is_empty() {
                let _ = reuse_evidence_from_parent(goal_id, parent_id, reuse_keywords).await;
            }
        }

        // 3. Move state to discovery
        repository::update_goal_status(goal_id, GoalState::Discovery.as_str()).await?;
        let mut goal = goal;
        goal.status = GoalState::Discovery.as_str().to_string();

        Ok(goal)
    }

    /// Step 2: Record discovered evidence during Discovery Loop
    #[allow(clippy::too_many_arguments)]
    pub async fn record_discovery(
        &self,
        file_path: &str,
        file_version: &str,
        start_line: Option<i64>,
        end_line: Option<i64>,
        symbol_name: Option<&str>,
        summary: &str,
        raw_snippet: Option<&str>,
    ) -> Result<EvidenceCache> {
        let ev = repository::save_evidence(
            &self.project_id,
            file_path,
            file_version,
            start_line,
            end_line,
            symbol_name,
            summary,
            raw_snippet,
        )
        .await?;

        // Automatically link this evidence to current goal as active
        repository::link_evidence_to_goal(&self.goal_id, &ev.id, 1.0, true).await?;

        Ok(ev)
    }

    /// Step 3: Transition to Action state when discovery is determined to be sufficient
    pub async fn transition_to_action(&self) -> Result<()> {
        repository::update_goal_status(&self.goal_id, GoalState::Action.as_str()).await
    }

    /// Step 4: Transition to Verification state after action/edits are applied
    pub async fn transition_to_verification(&self) -> Result<()> {
        repository::update_goal_status(&self.goal_id, GoalState::Verification.as_str()).await
    }

    /// Step 5: Verification Loop execution & evaluation
    pub async fn verify_iteration(
        &self,
        iteration: i64,
        test_command: &str,
        test_output: &str,
        is_passed: bool,
        feedback: Option<&str>,
    ) -> Result<GoalVerification> {
        let verif = repository::record_verification(
            &self.goal_id,
            iteration,
            test_command,
            test_output,
            is_passed,
            feedback,
        )
        .await?;

        if !is_passed {
            // If failed, orchestrator checks if we should loop back to Action or Discovery
            if iteration < self.max_verification_loops as i64 {
                // Loop back to Action by default, or Discovery if feedback implies missing facts
                let next_state = if feedback.map(|f| f.contains("not found") || f.contains("unknown")).unwrap_or(false) {
                    GoalState::Discovery
                } else {
                    GoalState::Action
                };
                repository::update_goal_status(&self.goal_id, next_state.as_str()).await?;
            } else {
                // Reached max retries
                repository::update_goal_status(&self.goal_id, GoalState::Failed.as_str()).await?;
            }
        }

        Ok(verif)
    }

    /// Step 6: Conclusion - marks goal completed only if verified
    pub async fn conclude(
        &self,
        conclusion_summary: &str,
    ) -> Result<()> {
        let goal = repository::get_goal(&self.goal_id)
            .await?
            .context("Goal not found")?;

        let verifs = repository::list_goal_verifications(&self.goal_id).await?;
        let has_passing_verif = verifs.iter().any(|v| v.is_passed == 1);

        if !has_passing_verif && goal.status != "completed" {
            anyhow::bail!("Cannot conclude goal {}: No passing verification record found.", self.goal_id);
        }

        repository::conclude_goal(&self.goal_id, GoalState::Completed.as_str(), conclusion_summary).await?;
        Ok(())
    }

    /// Step 6b: Conclude goal with specific state (Completed, Failed, etc.) via orchestrator
    pub async fn conclude_with_status(
        &self,
        status: GoalState,
        conclusion_summary: &str,
    ) -> Result<()> {
        repository::conclude_goal(&self.goal_id, status.as_str(), conclusion_summary).await?;
        Ok(())
    }

    /// Compiles current active prompt for this goal
    pub async fn get_current_prompt(&self) -> Result<String> {
        let goal = repository::get_goal(&self.goal_id)
            .await?
            .context("Goal not found")?;
        build_goal_context_prompt(&goal).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_simulation_g1_and_g2() {
        // Setup project via repository so it uses the active pool
        let project = repository::create_project("Web Auth App", "Test project", None, None)
            .await
            .unwrap();
        let project_id = &project.id;

        let session = repository::create_session(project_id, "Simulation Session", None, None, None)
            .await
            .unwrap();
        let test_session_id = &session.id;

        // Clean up any stale records from previous runs to ensure clean idempotency
        let pool = crate::llm::usage::get_pool().await;
        let _ = sqlx::query("DELETE FROM goal_verifications WHERE goal_id IN ('G1', 'G2')").execute(pool).await;
        let _ = sqlx::query("DELETE FROM goal_evidence WHERE goal_id IN ('G1', 'G2')").execute(pool).await;
        let _ = sqlx::query("DELETE FROM goals WHERE id IN ('G1', 'G2')").execute(pool).await;
        let _ = sqlx::query("DELETE FROM evidence_cache WHERE project_id = ?1").bind(project_id).execute(pool).await;

        // ====================================================================
        // DEMAND 1 (G1): Fix login bug
        // ====================================================================

        // 1. Goal Initialization
        let g1_criteria = vec![
            "Login succeeds for valid credentials (HTTP 200)".to_string(),
            "Login fails properly for invalid credentials (HTTP 401)".to_string(),
        ];
        let g1_constraints = vec!["Do not modify JWT response payload".to_string()];

        let g1 = GoalOrchestrator::initialize_demand(
            "G1",
            project_id,
            Some(test_session_id),
            None,
            "Fix login bug",
            "User reported valid token rejected",
            &g1_criteria,
            &g1_constraints,
            &["login", "auth"],
            &[],
        )
        .await
        .unwrap();

        assert_eq!(g1.id, "G1");
        assert_eq!(g1.status, "discovery");

        let orch_g1 = GoalOrchestrator::new("G1", project_id);

        // 2. Discovery Loop
        // Save discovery result to Evidence Cache
        orch_g1
            .record_discovery(
                "src/auth/handler.rs",
                "sha256_v1_handler",
                Some(25),
                Some(60),
                Some("login_endpoint"),
                "Login handler receives JSON payload and calls AuthService",
                Some("pub async fn login_endpoint(payload: Json<LoginRequest>) -> Response { ... }"),
            )
            .await
            .unwrap();

        orch_g1
            .record_discovery(
                "src/auth/token.rs",
                "sha256_v1_token",
                Some(10),
                Some(45),
                Some("validate_token"),
                "Token validation logic has off-by-one check on expiry timestamp",
                Some("if now > exp { return Err(AuthError::Expired); }"),
            )
            .await
            .unwrap();

        // Context summary prompt verified
        let prompt_g1 = orch_g1.get_current_prompt().await.unwrap();
        assert!(prompt_g1.contains("Fix login bug"));
        assert!(prompt_g1.contains("src/auth/handler.rs"));
        assert!(prompt_g1.contains("src/auth/token.rs"));

        // 3. Goal Action
        orch_g1.transition_to_action().await.unwrap();
        let current_g1 = repository::get_goal("G1").await.unwrap().unwrap();
        assert_eq!(current_g1.status, "action");

        // 4. Verification Loop
        orch_g1.transition_to_verification().await.unwrap();
        let verif_g1 = orch_g1
            .verify_iteration(
                1,
                "cargo test auth::test_login",
                "test auth::test_valid_login ... ok\ntest auth::test_invalid_login ... ok\n2 passed; 0 failed",
                true,
                None,
            )
            .await
            .unwrap();
        assert_eq!(verif_g1.is_passed, 1);

        // 5. Conclusion G1
        orch_g1
            .conclude("Expired token bug fix completed and verified with 2 unit tests passing.")
            .await
            .unwrap();

        let concluded_g1 = repository::get_goal("G1").await.unwrap().unwrap();
        assert_eq!(concluded_g1.status, "completed");
        assert!(concluded_g1.conclusion.unwrap().contains("fix completed"));

        // ====================================================================
        // DEMAND 2 (G2): Add rate limiting on login
        // ====================================================================

        // 1. Goal Initialization G2 (does not inherit 'completed' status from G1)
        let g2_criteria = vec![
            "Limit login attempts to max 5 per minute".to_string(),
            "Return HTTP 429 Too Many Requests when threshold exceeded".to_string(),
            "Rate limiting unit tests pass".to_string(),
        ];
        let g2_constraints = vec!["Use in-memory sliding window rate limiter".to_string()];

        let g2 = GoalOrchestrator::initialize_demand(
            "G2",
            project_id,
            Some(test_session_id),
            Some("G1"), // optional relation
            "Add rate limiting on login",
            "Protect login endpoint against brute force attacks",
            &g2_criteria,
            &g2_constraints,
            &["login", "handler"], // Evidence reuse keywords
            &[],
        )
        .await
        .unwrap();

        assert_eq!(g2.id, "G2");
        assert_eq!(g2.status, "discovery"); // G2 status starts at discovery, not completed!

        let orch_g2 = GoalOrchestrator::new("G2", project_id);

        // 2 & 3. Evidence Reuse & Context Summary G2
        // Handler login from G1 is automatically linked to G2 via keyword match "login"
        let prompt_g2 = orch_g2.get_current_prompt().await.unwrap();
        assert!(prompt_g2.contains("Add rate limiting on login"));
        assert!(prompt_g2.contains("src/auth/handler.rs")); // Reused from G1!

        // G1 token expiry detail does not match 'handler'/'rate limit', so it doesn't pollute G2 context
        let g2_active_evs = repository::get_active_evidences_for_goal("G2").await.unwrap();
        assert_eq!(g2_active_evs.len(), 1);
        assert_eq!(g2_active_evs[0].file_path, "src/auth/handler.rs");

        // 4. Follow-up Discovery Loop G2 (discovering rate limit middleware)
        orch_g2
            .record_discovery(
                "src/middleware/rate_limit.rs",
                "sha256_v1_rl",
                Some(1),
                Some(35),
                Some("RateLimiter"),
                "Rate limiting middleware layer for Axum",
                Some("pub struct RateLimiter { max_requests: usize }"),
            )
            .await
            .unwrap();

        let updated_g2_evs = repository::get_active_evidences_for_goal("G2").await.unwrap();
        assert_eq!(updated_g2_evs.len(), 2);

        // 5. Goal Action G2
        orch_g2.transition_to_action().await.unwrap();

        // 6. Verification Loop G2
        orch_g2.transition_to_verification().await.unwrap();
        let verif_g2 = orch_g2
            .verify_iteration(
                1,
                "cargo test auth::test_rate_limit",
                "test auth::test_5_requests_allowed ... ok\ntest auth::test_6th_request_429 ... ok\n2 passed; 0 failed",
                true,
                None,
            )
            .await
            .unwrap();
        assert_eq!(verif_g2.is_passed, 1);

        // 7. Conclusion G2
        orch_g2
            .conclude("Rate limiting middleware active on login endpoint and verification passed.")
            .await
            .unwrap();

        let concluded_g2 = repository::get_goal("G2").await.unwrap().unwrap();
        assert_eq!(concluded_g2.status, "completed");

        // Ensure G1 and G2 remain isolated and have their own distinct conclusions
        let final_g1 = repository::get_goal("G1").await.unwrap().unwrap();
        assert_eq!(final_g1.status, "completed");
        assert_ne!(final_g1.conclusion, concluded_g2.conclusion);

        // Ensure G3 reading historical context ONLY receives compact file metadata without raw code bloat
        let prev_summary = super::super::context_builder::build_previous_goals_summary(test_session_id, Some("G3"))
            .await
            .unwrap();
        assert!(prev_summary.contains("Fix login bug"));
        assert!(prev_summary.contains("src/auth/handler.rs"));
        assert!(prev_summary.contains("Lines 25-60"));
        // Confirm no raw verbose code snippets are leaked
        assert!(!prev_summary.contains("pub async fn login_endpoint"));
    }
}
