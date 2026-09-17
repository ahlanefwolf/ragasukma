use super::constants::*;
use super::oauth::{complete_oauth_flow, ensure_project_id, refresh_account_token};
use super::types::{
    AntigravityAccount, AntigravityAccountInfo, AntigravityQuotaResponse, ModelQuotaInfo,
    current_timestamp_secs, generate_pseudo_uuid,
};
use crate::llm::types::ChatMessage;
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AntigravityProvider {
    pub accounts: Vec<AntigravityAccount>,
    pub current_index: usize,
    pub client: reqwest::blocking::Client,
    pub config_file: PathBuf,
}

impl AntigravityProvider {
    pub fn new() -> Self {
        let config_file = crate::assets::find_file_in_candidates("antigravity_accounts.json");

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        let mut provider = Self {
            accounts: Vec::new(),
            current_index: 0,
            client,
            config_file,
        };

        let _ = provider.load_accounts();
        crate::term_log!(
            "✨ [Antigravity] Initialized provider with {} account(s) loaded.",
            provider.accounts.len()
        );
        provider
    }

    pub fn load_accounts(&mut self) -> Result<()> {
        if !self.config_file.exists() {
            let alt = crate::config::get_app_config_dir()
                .join("assets")
                .join("antigravity_accounts.json");
            if alt.exists() {
                self.config_file = alt;
            } else {
                return Ok(());
            }
        }

        let content = fs::read_to_string(&self.config_file)?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if let Ok(accs) = serde_json::from_str::<Vec<AntigravityAccount>>(trimmed) {
            self.accounts = accs;
            return Ok(());
        }

        if let Ok(token_strings) = serde_json::from_str::<Vec<String>>(trimmed) {
            let mut list = Vec::new();
            for (idx, tok) in token_strings.into_iter().enumerate() {
                if !tok.trim().is_empty() {
                    list.push(AntigravityAccount {
                        id: format!("ag-acc-{}", idx + 1),
                        email: None,
                        refresh_token: tok.trim().to_string(),
                        access_token: None,
                        expires_at: 0,
                        project_id: None,
                        tier_id: None,
                        rate_limited_until: 0,
                        consecutive_errors: 0,
                        is_active: true,
                    });
                }
            }
            self.accounts = list;
            return Ok(());
        }

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let items = val
                .get("accounts")
                .or_else(|| val.get("tokens"))
                .or_else(|| val.get("connections"))
                .and_then(|v| v.as_array());

            if let Some(arr) = items {
                let mut list = Vec::new();
                for (idx, item) in arr.iter().enumerate() {
                    if let Ok(acc) = serde_json::from_value::<AntigravityAccount>(item.clone()) {
                        list.push(acc);
                    } else if let Some(tok) = item
                        .pointer("/credentials/refreshToken")
                        .or_else(|| item.pointer("/credentials/refresh_token"))
                        .or_else(|| item.pointer("/refreshToken"))
                        .or_else(|| item.pointer("/refresh_token"))
                        .and_then(|v| v.as_str())
                    {
                        let email = item
                            .pointer("/credentials/email")
                            .or_else(|| item.pointer("/email"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let project_id = item
                            .pointer("/credentials/projectId")
                            .or_else(|| item.pointer("/projectId"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        list.push(AntigravityAccount {
                            id: format!("ag-acc-{}", idx + 1),
                            email,
                            refresh_token: tok.trim().to_string(),
                            access_token: None,
                            expires_at: 0,
                            project_id,
                            tier_id: None,
                            rate_limited_until: 0,
                            consecutive_errors: 0,
                            is_active: true,
                        });
                    }
                }
                self.accounts = list;
            }
        }

        Ok(())
    }

    pub fn save_accounts(&self) -> Result<()> {
        if let Some(parent) = self.config_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&self.accounts)?;
        fs::write(&self.config_file, json)?;
        Ok(())
    }

    pub fn add_oauth_account(&mut self, code: &str, redirect_uri: &str) -> Result<String> {
        let account = complete_oauth_flow(&self.client, code, redirect_uri)?;

        if let Some(existing) = self
            .accounts
            .iter_mut()
            .find(|a| a.refresh_token == account.refresh_token)
        {
            existing.is_active = true;
            existing.access_token = account.access_token;
            existing.expires_at = account.expires_at;
            if account.email.is_some() {
                existing.email = account.email;
            }
            if account.project_id.is_some() {
                existing.project_id = account.project_id;
            }
            let id = existing.id.clone();
            self.save_accounts()?;
            return Ok(id);
        }

        let id = account.id.clone();
        self.accounts.push(account);
        self.save_accounts()?;
        Ok(id)
    }

    pub fn add_account(
        &mut self,
        refresh_token: &str,
        email: Option<String>,
        project_id: Option<String>,
    ) -> Result<String> {
        let clean_token = refresh_token.trim().to_string();
        if clean_token.is_empty() {
            bail!("Refresh token cannot be empty");
        }

        if let Some(existing) = self
            .accounts
            .iter_mut()
            .find(|a| a.refresh_token == clean_token)
        {
            existing.is_active = true;
            if email.is_some() {
                existing.email = email;
            }
            if project_id.is_some() {
                existing.project_id = project_id;
            }
            let id = existing.id.clone();
            self.save_accounts()?;
            return Ok(id);
        }

        let id = format!("ag-acc-{}", self.accounts.len() + 1);
        let mut account = AntigravityAccount {
            id: id.clone(),
            email,
            refresh_token: clean_token,
            access_token: None,
            expires_at: 0,
            project_id,
            tier_id: None,
            rate_limited_until: 0,
            consecutive_errors: 0,
            is_active: true,
        };

        let _ = refresh_account_token(&self.client, &mut account);
        let _ = ensure_project_id(&self.client, &mut account);

        self.accounts.push(account);
        self.save_accounts()?;
        Ok(id)
    }

    pub fn remove_account(&mut self, id_or_email: &str) -> bool {
        let initial_len = self.accounts.len();
        self.accounts.retain(|a| {
            a.id != id_or_email && a.email.as_deref().unwrap_or("") != id_or_email
        });
        if self.accounts.len() != initial_len {
            let _ = self.save_accounts();
            true
        } else {
            false
        }
    }

    pub fn list_accounts(&self) -> Vec<AntigravityAccountInfo> {
        let now = current_timestamp_secs();
        self.accounts
            .iter()
            .map(|a| {
                let is_rate_limited = a.rate_limited_until > now;
                let rate_limited_seconds_left = if is_rate_limited {
                    a.rate_limited_until.saturating_sub(now)
                } else {
                    0
                };
                AntigravityAccountInfo {
                    id: a.id.clone(),
                    email: a.email.clone(),
                    project_id: a.project_id.clone(),
                    is_active: a.is_active,
                    is_rate_limited,
                    rate_limited_seconds_left,
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn refresh_account_token(&self, account: &mut AntigravityAccount) -> Result<String> {
        refresh_account_token(&self.client, account)
    }

    #[allow(dead_code)]
    pub fn ensure_project_id(&self, account: &mut AntigravityAccount) -> Result<String> {
        ensure_project_id(&self.client, account)
    }

    pub fn refresh_account_by_id(&mut self, id: &str) -> Result<String> {
        let client = self.client.clone();
        if let Some(account) = self.accounts.iter_mut().find(|a| a.id == id) {
            let _ = refresh_account_token(&client, account)?;
            let _ = ensure_project_id(&client, account);
            account.rate_limited_until = 0;
            let pid = account.project_id.clone().unwrap_or_default();
            let _ = self.save_accounts();
            Ok(pid)
        } else {
            bail!("Account not found");
        }
    }

    pub fn toggle_account_by_id(&mut self, id: &str) -> Result<bool> {
        if let Some(account) = self.accounts.iter_mut().find(|a| a.id == id) {
            account.is_active = !account.is_active;
            let active = account.is_active;
            let _ = self.save_accounts();
            Ok(active)
        } else {
            bail!("Account not found");
        }
    }

    pub fn get_account_quota(&mut self, id: &str) -> Result<AntigravityQuotaResponse> {
        let client = self.client.clone();
        let (access_token, project_id, email, tier_id) = {
            let account = match self.accounts.iter_mut().find(|a| a.id == id) {
                Some(a) => a,
                None => bail!("Account with ID '{id}' not found"),
            };

            let now = current_timestamp_secs();
            if account.access_token.is_none() || account.expires_at <= now {
                let _ = refresh_account_token(&client, account)?;
            }

            let _ = ensure_project_id(&client, account);
            (
                account.access_token.clone().unwrap_or_default(),
                account.project_id.clone(),
                account.email.clone(),
                account.tier_id.clone(),
            )
        };

        let _ = self.save_accounts();

        let body = if let Some(ref pid) = project_id {
            serde_json::json!({ "project": pid })
        } else {
            serde_json::json!({})
        };

        // 1. Fetch available models with quota info
        let mut models_quota = Vec::new();
        let mut fetch_err: Option<String> = None;

        let models_resp = client
            .post(ANTIGRAVITY_FETCH_MODELS_URL)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_USER_AGENT)
            .header("X-Client-Name", "antigravity")
            .header("X-Client-Version", "2.11.0")
            .json(&body)
            .send();

        match models_resp {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    if let Ok(val) = resp.json::<serde_json::Value>() {
                        if let Some(models_obj) = val.get("models").and_then(|v| v.as_object()) {
                            for (model_key, model_val) in models_obj {
                                let is_internal = model_val
                                    .get("isInternal")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                if is_internal {
                                    continue;
                                }

                                if let Some(q_info) = model_val.get("quotaInfo") {
                                    let frac = q_info
                                        .get("remainingFraction")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(0.0);
                                    let pct = (frac * 1000.0).round() / 10.0;
                                    let reset_time = q_info
                                        .get("resetTime")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let display_name = model_val
                                        .get("displayName")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(model_key)
                                        .to_string();

                                    models_quota.push(ModelQuotaInfo {
                                        model_id: model_key.clone(),
                                        display_name,
                                        remaining_fraction: frac,
                                        remaining_percentage: pct,
                                        reset_time,
                                    });
                                }
                            }
                        }
                    }
                } else {
                    let err_txt = resp.text().unwrap_or_default();
                    fetch_err = Some(format!("HTTP {status}: {err_txt}"));
                }
            }
            Err(e) => {
                fetch_err = Some(format!("Request failed: {e}"));
            }
        }

        // Sort models_quota to put priority models first
        let priority_order = [
            "gemini-3.8-flash-high",
            "gemini-3.8-flash-medium",
            "gemini-3.8-flash-low",
            "gemini-3.7-flash-high",
            "gemini-3.7-flash-medium",
            "gemini-3.7-flash-low",
            "gemini-3.6-flash-high",
            "gemini-3.6-flash-medium",
            "gemini-3.6-flash-low",
            "gemini-3.5-flash-high",
            "gemini-3.5-flash-low",
            "gemini-pro-agent",
            "claude-sonnet-4-6",
            "claude-opus-4-6-thinking",
            "gpt-oss-120b-medium",
            "gemini-3.1-flash-image",
        ];

        models_quota.sort_by(|a, b| {
            let pos_a = priority_order.iter().position(|&m| m == a.model_id);
            let pos_b = priority_order.iter().position(|&m| m == b.model_id);
            match (pos_a, pos_b) {
                (Some(pa), Some(pb)) => pa.cmp(&pb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.model_id.cmp(&b.model_id),
            }
        });

        // 2. Fetch weekly quota summary
        let mut weekly_quota = Vec::new();
        let weekly_resp = client
            .post(ANTIGRAVITY_QUOTA_SUMMARY_URL)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_USER_AGENT)
            .header("X-Client-Name", "antigravity")
            .header("X-Client-Version", "2.11.0")
            .json(&body)
            .send();

        if let Ok(w_res) = weekly_resp {
            if w_res.status().is_success() {
                if let Ok(w_val) = w_res.json::<serde_json::Value>() {
                    let groups = w_val
                        .get("groups")
                        .or_else(|| w_val.pointer("/quotaSummary/groups"))
                        .and_then(|v| v.as_array());

                    if let Some(groups_arr) = groups {
                        for g in groups_arr {
                            let g_name = g.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
                            let buckets = g.get("buckets").and_then(|v| v.as_array());
                            if let Some(b_arr) = buckets {
                                for b in b_arr {
                                    let b_id = b.get("bucketId").and_then(|v| v.as_str()).unwrap_or("");
                                    let b_name = b.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
                                    let b_text = format!("{b_id} {b_name}").to_lowercase();
                                    if !b_text.contains("weekly") {
                                        continue;
                                    }
                                    if b.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false) {
                                        continue;
                                    }
                                    if let Some(frac) = b.get("remainingFraction").and_then(|v| v.as_f64()) {
                                        let d_name = if g_name.to_lowercase().contains("gemini") {
                                            "Gemini (Weekly)".to_string()
                                        } else if g_name.to_lowercase().contains("claude") || g_name.to_lowercase().contains("gpt") {
                                            "Claude & GPT (Weekly)".to_string()
                                        } else {
                                            format!("{g_name} (Weekly)")
                                        };
                                        let reset_time = b.get("resetTime").and_then(|v| v.as_str()).map(|s| s.to_string());
                                        let pct = (frac * 1000.0).round() / 10.0;
                                        weekly_quota.push(ModelQuotaInfo {
                                            model_id: b_id.to_string(),
                                            display_name: d_name,
                                            remaining_fraction: frac,
                                            remaining_percentage: pct,
                                            reset_time,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Tier / Plan name
        let plan = tier_id.as_deref().map(|t| match t {
            "g1-pro-tier" => "Google AI Pro".to_string(),
            "free-tier" | "starter" => "Starter (Free)".to_string(),
            other => other.to_string(),
        });

        let status = if fetch_err.is_some() && models_quota.is_empty() && weekly_quota.is_empty() {
            "error".to_string()
        } else {
            "success".to_string()
        };

        Ok(AntigravityQuotaResponse {
            status,
            account_id: id.to_string(),
            email,
            project_id,
            plan,
            models: models_quota,
            weekly: weekly_quota,
            error: fetch_err,
        })
    }

    fn select_available_account_index(&self) -> Option<usize> {
        let total = self.accounts.len();
        if total == 0 {
            return None;
        }

        let now = current_timestamp_secs();
        for i in 0..total {
            let idx = (self.current_index + i) % total;
            let acc = &self.accounts[idx];
            if acc.is_active && acc.rate_limited_until <= now {
                return Some(idx);
            }
        }
        None
    }

    pub fn chat_stream<F>(
        &mut self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
        mut on_sentence: F,
    ) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        if self.accounts.is_empty() {
            bail!("Antigravity has no configured accounts. Please add an account first.");
        }

        let total_accounts = self.accounts.len();
        let mut attempts = 0;
        let now = current_timestamp_secs();

        while attempts < total_accounts {
            let acc_idx = match self.select_available_account_index() {
                Some(idx) => idx,
                None => {
                    let min_wait = self
                        .accounts
                        .iter()
                        .filter(|a| a.is_active && a.rate_limited_until > now)
                        .map(|a| a.rate_limited_until - now)
                        .min()
                        .unwrap_or(60);
                    bail!(
                        "All Antigravity accounts are rate-limited or inactive. Please wait {} seconds.",
                        min_wait
                    );
                }
            };

            self.current_index = (acc_idx + 1) % total_accounts;
            attempts += 1;

            let now = current_timestamp_secs();
            let need_token = {
                let acc = &self.accounts[acc_idx];
                acc.access_token.is_none() || acc.expires_at <= now
            };

            if need_token {
                if let Err(e) = refresh_account_token(&self.client, &mut self.accounts[acc_idx]) {
                    self.accounts[acc_idx].consecutive_errors += 1;
                    crate::term_err!(
                        "⚠️ [Antigravity] Failed to refresh token for account {}: {}",
                        self.accounts[acc_idx].id,
                        e
                    );
                    continue;
                }
            }

            let project_id = match ensure_project_id(&self.client, &mut self.accounts[acc_idx]) {
                Ok(pid) => pid,
                Err(_) => "cloudaicompanion-default".to_string(),
            };

            let access_token = self.accounts[acc_idx]
                .access_token
                .clone()
                .unwrap_or_default();

            let result = self.execute_stream_request(
                &access_token,
                &project_id,
                model,
                messages,
                temperature,
                max_tokens,
                &mut on_sentence,
            );

            match result {
                Ok(reply) => {
                    self.accounts[acc_idx].consecutive_errors = 0;
                    let _ = self.save_accounts();
                    return Ok(reply);
                }
                Err(err) => {
                    let err_str = err.to_string();
                    let is_rate_limit = err_str.contains("429")
                        || err_str.to_lowercase().contains("resource_exhausted")
                        || err_str.to_lowercase().contains("quota");

                    if is_rate_limit {
                        let cooldown = 15 * 60;
                        self.accounts[acc_idx].rate_limited_until =
                            current_timestamp_secs() + cooldown;
                        crate::term_err!(
                            "⚠️ [Antigravity] Account {} hit 429 quota limit. Rotating to next account...",
                            self.accounts[acc_idx].id
                        );
                        let _ = self.save_accounts();
                        continue;
                    } else if err_str.contains("401") || err_str.contains("403") {
                        self.accounts[acc_idx].access_token = None;
                        self.accounts[acc_idx].expires_at = 0;
                        crate::term_err!(
                            "⚠️ [Antigravity] Account {} auth error: {}. Retrying next...",
                            self.accounts[acc_idx].id,
                            err_str
                        );
                        let _ = self.save_accounts();
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        bail!("All Antigravity accounts failed during execution.")
    }

    fn execute_stream_request<F>(
        &self,
        access_token: &str,
        project_id: &str,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
        on_sentence: &mut F,
    ) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        let endpoint = format!("{}/v1internal:streamGenerateContent?alt=sse", ANTIGRAVITY_BASE_URL);

        let mut contents = Vec::new();
        for msg in messages {
            let role = match msg.role.as_str() {
                "assistant" => "model",
                _ => "user",
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let session_id = generate_pseudo_uuid("ag-sess");
        let conversation_id = generate_pseudo_uuid("ag-conv");
        let trajectory_id = generate_pseudo_uuid("ag-traj");
        let step = (messages.len() * 2).max(1);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let request_id = format!("agent/{conversation_id}/{now_ms}/{trajectory_id}/{step}");

        let clean_model = if let Some(stripped) = model.strip_prefix("ag/") {
            stripped
        } else {
            model
        };

        let payload = serde_json::json!({
            "project": project_id,
            "model": clean_model,
            "userAgent": "antigravity",
            "requestType": "agent",
            "requestId": request_id,
            "request": {
                "contents": contents,
                "generationConfig": {
                    "temperature": temperature,
                    "topP": 0.95,
                    "topK": 40,
                    "maxOutputTokens": max_tokens.min(8192)
                },
                "sessionId": session_id
            }
        });

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_USER_AGENT)
            .header("X-Client-Name", "antigravity")
            .header("X-Client-Version", "2.11.0")
            .json(&payload)
            .send()
            .context("Failed to send request to Antigravity stream endpoint")?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().unwrap_or_default();
            bail!("Antigravity HTTP error {status}: {err_text}");
        }

        let reader = std::io::BufReader::new(response);
        let mut full_reply = String::new();
        let mut current_sentence = String::new();
        let mut active_emotion = "neutral";

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if let Some(data_str) = trimmed.strip_prefix("data: ") {
                if data_str.trim() == "[DONE]" {
                    break;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                    let candidates = val
                        .pointer("/response/candidates")
                        .or_else(|| val.pointer("/candidates"))
                        .and_then(|c| c.as_array());

                    if let Some(cands) = candidates {
                        for cand in cands {
                            if let Some(parts) =
                                cand.pointer("/content/parts").and_then(|p| p.as_array())
                            {
                                for part in parts {
                                    if part
                                        .get("thought")
                                        .and_then(|t| t.as_bool())
                                        .unwrap_or(false)
                                    {
                                        continue;
                                    }

                                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                        full_reply.push_str(text);
                                        current_sentence.push_str(text);

                                        let has_skill_tag = current_sentence.contains("[SKILL:");
                                        let has_skill_closed =
                                            has_skill_tag && current_sentence.contains(']');

                                        if !has_skill_tag || has_skill_closed {
                                            while let Some(pos) =
                                                current_sentence.find(['.', '?', '!', '\n'])
                                            {
                                                if let Some(skill_idx) =
                                                    current_sentence.find("[SKILL:")
                                                {
                                                    if pos > skill_idx {
                                                        break;
                                                    }
                                                }
                                                let sentence = current_sentence[..=pos].to_string();
                                                current_sentence =
                                                    current_sentence[pos + 1..].to_string();

                                                if let Some(emo) =
                                                    crate::ui::renderer::expressions::detect_emotion(
                                                        &sentence,
                                                    )
                                                {
                                                    active_emotion = emo;
                                                }

                                                let clean =
                                                    crate::llm::cleaner::clean_ai_text(&sentence);
                                                if !clean.is_empty() {
                                                    on_sentence(&clean, active_emotion)?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let clean_rest = crate::llm::cleaner::clean_ai_text(&current_sentence);
        if !clean_rest.is_empty() {
            if let Some(emo) =
                crate::ui::renderer::expressions::detect_emotion(&current_sentence)
            {
                active_emotion = emo;
            }
            on_sentence(&clean_rest, active_emotion)?;
        }

        Ok(full_reply)
    }

    pub fn chat_stream_raw<F>(
        &mut self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
        mut on_chunk: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        if self.accounts.is_empty() {
            bail!("Antigravity has no configured accounts. Please add an account first.");
        }

        let total_accounts = self.accounts.len();
        let mut attempts = 0;
        let now = current_timestamp_secs();

        while attempts < total_accounts {
            let acc_idx = match self.select_available_account_index() {
                Some(idx) => idx,
                None => {
                    let min_wait = self
                        .accounts
                        .iter()
                        .filter(|a| a.is_active && a.rate_limited_until > now)
                        .map(|a| a.rate_limited_until - now)
                        .min()
                        .unwrap_or(60);
                    bail!(
                        "All Antigravity accounts are rate-limited or inactive. Please wait {} seconds.",
                        min_wait
                    );
                }
            };

            self.current_index = (acc_idx + 1) % total_accounts;
            attempts += 1;

            let now = current_timestamp_secs();
            let need_token = {
                let acc = &self.accounts[acc_idx];
                acc.access_token.is_none() || acc.expires_at <= now
            };

            if need_token {
                if let Err(e) = refresh_account_token(&self.client, &mut self.accounts[acc_idx]) {
                    self.accounts[acc_idx].consecutive_errors += 1;
                    crate::term_err!(
                        "⚠️ [Antigravity] Failed to refresh token for account {}: {}",
                        self.accounts[acc_idx].id,
                        e
                    );
                    continue;
                }
            }

            let project_id = match ensure_project_id(&self.client, &mut self.accounts[acc_idx]) {
                Ok(pid) => pid,
                Err(_) => "cloudaicompanion-default".to_string(),
            };

            let access_token = self.accounts[acc_idx]
                .access_token
                .clone()
                .unwrap_or_default();

            let result = self.execute_raw_stream_request(
                &access_token,
                &project_id,
                model,
                messages,
                temperature,
                max_tokens,
                &mut on_chunk,
            );

            match result {
                Ok(reply) => {
                    self.accounts[acc_idx].consecutive_errors = 0;
                    let _ = self.save_accounts();
                    return Ok(reply);
                }
                Err(err) => {
                    let err_str = err.to_string();
                    let is_rate_limit = err_str.contains("429")
                        || err_str.to_lowercase().contains("resource_exhausted")
                        || err_str.to_lowercase().contains("quota");

                    if is_rate_limit {
                        let cooldown = 15 * 60;
                        self.accounts[acc_idx].rate_limited_until =
                            current_timestamp_secs() + cooldown;
                        crate::term_err!(
                            "⚠️ [Antigravity] Account {} hit 429 quota limit. Rotating to next account...",
                            self.accounts[acc_idx].id
                        );
                        let _ = self.save_accounts();
                        continue;
                    } else if err_str.contains("401") || err_str.contains("403") {
                        self.accounts[acc_idx].access_token = None;
                        self.accounts[acc_idx].expires_at = 0;
                        crate::term_err!(
                            "⚠️ [Antigravity] Account {} auth error: {}. Retrying next...",
                            self.accounts[acc_idx].id,
                            err_str
                        );
                        let _ = self.save_accounts();
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        bail!("All Antigravity accounts failed during execution.")
    }

    fn execute_raw_stream_request<F>(
        &self,
        access_token: &str,
        project_id: &str,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
        on_chunk: &mut F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        let endpoint = format!("{}/v1internal:streamGenerateContent?alt=sse", ANTIGRAVITY_BASE_URL);

        let mut contents = Vec::new();
        for msg in messages {
            let role = match msg.role.as_str() {
                "assistant" => "model",
                _ => "user",
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let session_id = generate_pseudo_uuid("ag-sess");
        let conversation_id = generate_pseudo_uuid("ag-conv");
        let trajectory_id = generate_pseudo_uuid("ag-traj");
        let step = (messages.len() * 2).max(1);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let request_id = format!("agent/{conversation_id}/{now_ms}/{trajectory_id}/{step}");

        let clean_model = if let Some(stripped) = model.strip_prefix("ag/") {
            stripped
        } else {
            model
        };

        let payload = serde_json::json!({
            "project": project_id,
            "model": clean_model,
            "userAgent": "antigravity",
            "requestType": "agent",
            "requestId": request_id,
            "request": {
                "contents": contents,
                "generationConfig": {
                    "temperature": temperature,
                    "topP": 0.95,
                    "topK": 40,
                    "maxOutputTokens": max_tokens.min(8192)
                },
                "sessionId": session_id
            }
        });

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_USER_AGENT)
            .header("X-Client-Name", "antigravity")
            .header("X-Client-Version", "2.11.0")
            .json(&payload)
            .send()
            .context("Failed to send request to Antigravity stream endpoint")?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().unwrap_or_default();
            bail!("Antigravity HTTP error {status}: {err_text}");
        }

        let reader = std::io::BufReader::new(response);
        let mut full_reply = String::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if let Some(data_str) = trimmed.strip_prefix("data: ") {
                if data_str.trim() == "[DONE]" {
                    break;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                    let candidates = val
                        .pointer("/response/candidates")
                        .or_else(|| val.pointer("/candidates"))
                        .and_then(|c| c.as_array());

                    if let Some(cands) = candidates {
                        for cand in cands {
                            if let Some(parts) =
                                cand.pointer("/content/parts").and_then(|p| p.as_array())
                            {
                                for part in parts {
                                    if part
                                        .get("thought")
                                        .and_then(|t| t.as_bool())
                                        .unwrap_or(false)
                                    {
                                        continue;
                                    }

                                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                        full_reply.push_str(text);
                                        on_chunk(text)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(full_reply)
    }
}
