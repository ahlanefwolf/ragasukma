use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::agent::models::{Message, Project, Session};
use crate::agent::repository;

pub struct TurnState {
    pub goal_id: String,
    pub current_phase: String,
    pub locked_targets: Vec<String>,
    pub discovery_counter: i64,
    pub iteration: usize,
    pub max_safety_iterations: usize,
    pub ephemeral_directive: Option<String>,
    pub modified_files: Vec<String>,
    pub executed_commands: Vec<String>,
    pub consecutive_command_count: usize,
    pub last_executed_command: Option<String>,
    pub consecutive_read_same_file: HashMap<String, usize>,
    pub recent_read_slices: HashSet<(String, Option<i64>, Option<i64>)>,
    pub global_inspection_streak: usize,
    pub last_command_had_error: bool,
    pub last_command_is_environment_error: bool,
    pub last_tool_had_error: bool,
    pub pending_error_files: HashSet<String>,
    pub files_read_in_full: HashSet<String>,
    pub force_final_text_only: bool,
    pub circuit_breaker_triggered: bool,
    pub has_passed_verification: bool,
    pub verification_attempts: usize,
    pub verification_prompted: bool,
    pub consecutive_empty_turns: usize,
}

impl TurnState {
    pub fn new(
        goal_id: String,
        current_phase: String,
        locked_targets: Vec<String>,
        discovery_counter: i64,
        prior_verifs_count: usize,
    ) -> Self {
        Self {
            goal_id,
            current_phase,
            locked_targets,
            discovery_counter,
            iteration: 0,
            max_safety_iterations: 50,
            ephemeral_directive: None,
            modified_files: Vec::new(),
            executed_commands: Vec::new(),
            consecutive_command_count: 0,
            last_executed_command: None,
            consecutive_read_same_file: HashMap::new(),
            recent_read_slices: HashSet::new(),
            global_inspection_streak: 0,
            last_command_had_error: false,
            last_command_is_environment_error: false,
            last_tool_had_error: false,
            pending_error_files: HashSet::new(),
            files_read_in_full: HashSet::new(),
            force_final_text_only: false,
            circuit_breaker_triggered: false,
            has_passed_verification: false,
            verification_attempts: prior_verifs_count,
            verification_prompted: false,
            consecutive_empty_turns: 0,
        }
    }

    pub fn trip_circuit_breaker(&mut self, directive_msg: &str) {
        self.circuit_breaker_triggered = true;
        self.force_final_text_only = true;
        self.ephemeral_directive = Some(directive_msg.to_string());
    }

    pub fn on_successful_edit(&mut self, file_path: &str) {
        self.last_tool_had_error = false;
        self.last_command_is_environment_error = false;
        self.has_passed_verification = false;
        self.verification_prompted = false;
        self.global_inspection_streak = 0;
        self.consecutive_command_count = 0;
        self.last_executed_command = None;
        self.recent_read_slices.clear();
        self.consecutive_read_same_file.clear();

        let clean = file_path.trim_start_matches("./");
        self.pending_error_files.retain(|ef| ef != clean && !clean.ends_with(ef) && !ef.ends_with(clean));
        if !self.modified_files.iter().any(|f| f == file_path) {
            self.modified_files.push(file_path.to_string());
        }
        if !self.locked_targets.iter().any(|f| f == file_path) {
            self.locked_targets.push(file_path.to_string());
        }
        self.current_phase = "verification".to_string();
    }
}

pub async fn resolve_or_create_goal(
    session: &Session,
    project: &Project,
    db_messages: &[Message],
    allowed_folders: &[String],
) -> Result<(String, String, Vec<String>, i64)> {
    let last_user_prompt = db_messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .and_then(|m| m.content.as_deref())
        .unwrap_or("")
        .to_string();

    let session_goals = repository::list_session_goals(&session.id).await.unwrap_or_default();
    let latest_existing_goal = session_goals.first().cloned();

    // Check if the latest goal was already completed or failed; if so, create a new one
    let goal_id = match latest_existing_goal {
        Some(ref g) if g.status != "completed" && g.status != "completed_unverified" && g.status != "failed" => {
            // Re-use current active goal if still in progress and user is continuing the task
            g.id.clone()
        }
        _ => {
            // New distinct goal for this user request with sub-second entropy
            let new_gid = repository::generate_id(&format!("goal_{}", session.id));
            let title = if last_user_prompt.chars().count() > 60 {
                format!("{}...", last_user_prompt.chars().take(57).collect::<String>())
            } else if !last_user_prompt.trim().is_empty() {
                last_user_prompt.trim().to_string()
            } else {
                session.title.clone()
            };
            let prompt_keywords: Vec<&str> = last_user_prompt
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            let _ = crate::agent::orchestrator::GoalOrchestrator::initialize_demand(
                &new_gid,
                &project.id,
                Some(&session.id),
                latest_existing_goal.as_ref().map(|g| g.id.as_str()),
                &title,
                &last_user_prompt,
                &[],
                &[],
                &prompt_keywords,
                allowed_folders,
            ).await?;
            new_gid
        }
    };

    // Revalidate evidence cache on disk
    let _ = crate::agent::context_builder::revalidate_evidence_cache(&project.id, allowed_folders).await;

    // Only reuse prior evidence if this goal is explicitly a continuation child of a parent goal
    let active_goal_opt = repository::get_goal(&goal_id).await.unwrap_or(None);
    if let Some(ref g) = active_goal_opt {
        if let Some(ref parent_gid) = g.parent_goal_id {
            let prompt_keywords: Vec<&str> = last_user_prompt
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            let _ = crate::agent::context_builder::reuse_evidence_from_parent(&goal_id, parent_gid, &prompt_keywords).await;
        }
    }

    // A fresh user turn MUST ALWAYS start with active execution phase ("discovery" or "action").
    let current_phase = active_goal_opt
        .as_ref()
        .map(|g| g.phase.as_str())
        .filter(|p| *p != "concluded")
        .unwrap_or("discovery")
        .to_string();
    let locked_targets: Vec<String> = active_goal_opt
        .as_ref()
        .and_then(|g| serde_json::from_str::<Vec<String>>(&g.target_files).ok())
        .unwrap_or_default();
    let discovery_counter: i64 = active_goal_opt
        .as_ref()
        .map(|g| g.discovery_count)
        .unwrap_or(0);

    Ok((goal_id, current_phase, locked_targets, discovery_counter))
}

pub async fn resolve_model(session: &Session, model_override: Option<&str>) -> (String, bool) {
    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .map(|m| m.to_string())
        .or_else(|| session.model.as_ref().filter(|m| !m.trim().is_empty()).cloned())
        .unwrap_or_else(|| {
            let u = crate::config::load_user_config();
            if !u.llm_model.trim().is_empty() {
                u.llm_model
            } else {
                "ag/gemini-2.5-flash".to_string()
            }
        });

    let _ = repository::update_session_model(&session.id, &model).await;
    let is_codex = model.starts_with("cx/") || model.to_lowercase().contains("codex");
    (model, is_codex)
}
