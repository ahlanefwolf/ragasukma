use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::thread;

use crate::services::browser_session_service::get_latest_snapshot;

use super::runner::{execute_single_step, wait_for_dom_change, wait_for_target_page};
use super::types::{AutomationRecipe, InitialAction, RecipeStep, TrainStepDecision, get_recipes_dir};

/// Menghitung hash representatif dari state DOM untuk mendeteksi apakah halaman benar-benar berubah
pub fn compute_dom_hash(snap: &crate::services::browser_session_service::BrowserDomSnapshot) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    snap.url.hash(&mut hasher);
    snap.title.hash(&mut hasher);
    snap.buttons.len().hash(&mut hasher);
    for b in snap.buttons.iter().take(20) {
        b.selector.hash(&mut hasher);
        b.text.hash(&mut hasher);
        b.aria_label.hash(&mut hasher);
    }
    snap.inputs.len().hash(&mut hasher);
    for inp in snap.inputs.iter().take(10) {
        inp.selector.hash(&mut hasher);
        inp.placeholder.hash(&mut hasher);
        inp.text.hash(&mut hasher);
    }
    snap.texts.len().hash(&mut hasher);
    for t in snap.texts.iter().take(15) {
        t.hash(&mut hasher);
    }
    hasher.finish()
}

/// Helper function to strip markdown code blocks from LLM JSON output
pub fn extract_json_from_llm(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json") {
        if let Some(end) = stripped.rfind("```") {
            return stripped[..end].trim();
        }
        return stripped.trim();
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        if let Some(end) = stripped.rfind("```") {
            return stripped[..end].trim();
        }
        return stripped.trim();
    }
    trimmed
}

/// Memperbaiki kesalahan umum output JSON dari LLM:
/// - Escaped quotes liar di array: e.g. `\"button[...]\"`
/// - Trailing commas pada array/object: e.g. `[1, 2,]`
/// - Unquoted string values: e.g. "recipe_name": Video Title,
pub fn repair_json_string_values(json_str: &str) -> String {
    // 1. Bersihkan escaped quotes liar yang ditaruh di luar string
    let mut cleaned = json_str.to_string();
    cleaned = cleaned.replace(", \\\"", ", \"");
    cleaned = cleaned.replace("[\\\"", "[\"");
    cleaned = cleaned.replace("\\\"]", "\"]");
    cleaned = cleaned.replace("\\\" ", "\" ");
    cleaned = cleaned.replace("\\\",", "\",");
    cleaned = cleaned.replace(": \\\"", ": \"");
    cleaned = cleaned.replace("\\\", ", "\", ");

    // 2. Bersihkan trailing commas: `, ]` -> `]` dan `, }` -> `}`
    while cleaned.contains(",\n]") || cleaned.contains(",\n }") || cleaned.contains(",]") || cleaned.contains(",}") {
        cleaned = cleaned.replace(",\n]", "\n]");
        cleaned = cleaned.replace(",\n }", "\n }");
        cleaned = cleaned.replace(",]", "]");
        cleaned = cleaned.replace(",}", "}");
    }

    let mut output = Vec::new();
    for line in cleaned.lines() {
        let trimmed = line.trim();
        if let Some((key_part, val_part)) = trimmed.split_once(':') {
            let key_trim = key_part.trim();
            if key_trim.starts_with('"') && key_trim.ends_with('"') {
                let val_trim = val_part.trim();
                let (val_body, has_comma) = if let Some(stripped) = val_trim.strip_suffix(',') {
                    (stripped.trim(), true)
                } else {
                    (val_trim, false)
                };

                let is_already_valid = val_body.starts_with('"')
                    || val_body.starts_with('{')
                    || val_body.starts_with('[')
                    || val_body == "true"
                    || val_body == "false"
                    || val_body == "null"
                    || val_body.parse::<f64>().is_ok();

                if !is_already_valid && !val_body.is_empty() {
                    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                    let comma_str = if has_comma { "," } else { "" };
                    output.push(format!("{indent}{key_trim}: \"{val_body}\"{comma_str}"));
                    continue;
                }
            }
        }
        output.push(line.to_string());
    }
    output.join("\n")
}

/// Fallback parser toleran (Lenient Regex / Lexer):
/// Jika serde_json gagal mem-parse format LLM yang hancur, ekstrak field utama secara manual
pub fn parse_lenient_train_decision(raw: &str) -> Option<TrainStepDecision> {
    let clean = extract_json_from_llm(raw);

    // Helper untuk mengekstrak string value berdasarkan key
    let extract_str = |key: &str| -> Option<String> {
        let pattern = format!(r#""{key}""#);
        let start_idx = clean.find(&pattern)?;
        let after_key = &clean[start_idx + pattern.len()..];
        let colon_idx = after_key.find(':')?;
        let after_colon = after_key[colon_idx + 1..].trim_start();

        if let Some(inner) = after_colon.strip_prefix('"') {
            let mut result = String::new();
            let mut escape = false;
            for c in inner.chars() {
                if escape {
                    result.push(c);
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    return Some(result);
                } else {
                    result.push(c);
                }
            }
            Some(result)
        } else {
            // Unquoted string fallback
            let end_idx = after_colon.find([',', '\n', '}']).unwrap_or(after_colon.len());
            let val = after_colon[..end_idx].trim().trim_matches('"').to_string();
            if val == "null" || val.is_empty() { None } else { Some(val) }
        }
    };

    let status = extract_str("status").unwrap_or_else(|| {
        if clean.to_uppercase().contains(r#""DONE""#) || clean.to_uppercase().contains("STATUS: DONE") {
            "DONE".to_string()
        } else {
            "CONTINUE".to_string()
        }
    });

    let reasoning = extract_str("reasoning").unwrap_or_default();
    let recipe_id = extract_str("recipe_id");
    let recipe_name = extract_str("recipe_name");

    // Jika status DONE, tidak perlu next_step
    if status.eq_ignore_ascii_case("DONE") {
        return Some(TrainStepDecision {
            reasoning,
            status,
            recipe_id,
            recipe_name,
            required_params: None,
            next_step: None,
        });
    }

    // Ekstrak next_step
    let action = extract_str("action");
    if let Some(act) = action {
        let selector = extract_str("selector").unwrap_or_default();
        let target_text = extract_str("target_text");
        let value = extract_str("value");

        // Ekstrak fallback_selectors dari array jika ada
        let mut fallback_selectors = Vec::new();
        if let Some(fb_start) = clean.find(r#""fallback_selectors""#) {
            let after_fb = &clean[fb_start..];
            if let Some(open_b) = after_fb.find('[') {
                if let Some(close_b) = after_fb[open_b..].find(']') {
                    let array_content = &after_fb[open_b + 1..open_b + close_b];
                    for item in array_content.split(',') {
                        let trimmed = item.trim().trim_matches(|c| c == '"' || c == '\\' || c == '\'' || c == ' ');
                        if !trimmed.is_empty() {
                            fallback_selectors.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        return Some(TrainStepDecision {
            reasoning,
            status,
            recipe_id,
            recipe_name,
            required_params: if clean.contains("{{query}}") { Some(vec!["query".to_string()]) } else { None },
            next_step: Some(RecipeStep {
                step: 0,
                action: act,
                selector,
                fallback_selectors,
                target_text,
                value,
            }),
        });
    }

    None
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RecipeDiagnosisResponse {
    #[serde(default)]
    pub diagnosis: String,
    pub optimized_steps: Vec<RecipeStep>,
}

/// Diagnoses, prunes, and optimizes recorded steps before writing to recipe JSON
fn diagnose_and_optimize_steps(
    llm: &mut crate::llm::LlmClient,
    user_goal: &str,
    domain: &str,
    initial_url: &str,
    steps: Vec<RecipeStep>,
) -> Vec<RecipeStep> {
    if steps.len() <= 1 {
        return steps;
    }

    let steps_json = match serde_json::to_string_pretty(&steps) {
        Ok(j) => j,
        Err(_) => return steps,
    };

    let system_prompt = "You are an expert Web Automation Quality Engineer and Recipe Optimizer.
Your task is to audit, diagnose, and streamline recorded interactive browser automation steps to achieve the target user goal with minimal, reliable actions.

OPTIMIZATION CRITERIA:
1. Prune Redundant Clicks & Misclicks:
   - If an input field is clicked immediately before a 'type' action on that same input field, eliminate the redundant click (typing automatically focuses the element).
   - Eliminate duplicate clicks on the exact same element with no intermediate state change.
   - Eliminate stray clicks on irrelevant containers, empty page areas, or decorative elements.
   - Remove redundant or unnecessary scroll actions unless strictly required to reveal a hidden element.
2. Maintain Crucial Functional Flow:
   - Ensure the sequence contains all essential steps required to fulfill the goal (e.g. typing query, submitting form/search, clicking desired result item, navigation).
   - Do not drop critical interaction steps needed to complete the user's objective.
3. Parameter Templates:
   - Preserve parameter templates such as {{query}}, {{keyword}}, etc., in 'value' attributes.
4. Keep Selectors Intact:
   - For every retained step, preserve its primary 'selector', 'fallback_selectors', and 'target_text' so execution remains robust.
5. Re-index Steps:
   - Renumber all retained steps consecutively starting from 1 (1, 2, 3, ...).

OUTPUT INSTRUCTIONS:
- You MUST respond with ONLY a valid raw JSON object.
- Do NOT output conversational text, explanations outside JSON, or markdown ticks outside the JSON.
- JSON Schema:
{
  \"diagnosis\": \"Clear English summary of redundancies/misclicks detected and how the recipe was optimized\",
  \"optimized_steps\": [
    {
      \"step\": 1,
      \"action\": \"type\",
      \"selector\": \"input[name='q']\",
      \"fallback_selectors\": [\"#search\"],
      \"target_text\": \"Search\",
      \"value\": \"{{query}}\"
    }
  ]
}";

    let user_prompt = format!(
        "USER GOAL: \"{user_goal}\"\nTARGET DOMAIN: \"{domain}\"\nINITIAL URL: \"{initial_url}\"\n\nRECORDED RAW STEPS ({count} steps):\n{steps_json}\n\nDiagnose these steps, remove all redundancies, misclicks, and unnecessary actions, and return the streamlined minimal step sequence in the specified JSON format.",
        count = steps.len()
    );

    let messages = vec![
        crate::llm::types::ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        crate::llm::types::ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    crate::term_log!("🔍 [Recipe Diagnostic Pass] Auditing and optimizing {} recorded steps...", steps.len());

    match llm.complete_messages_silent(&messages) {
        Ok(reply) => {
            let clean_json = extract_json_from_llm(&reply);
            let repaired = repair_json_string_values(clean_json);
            if let Ok(diag) = serde_json::from_str::<RecipeDiagnosisResponse>(&repaired) {
                if !diag.optimized_steps.is_empty() {
                    crate::term_log!("✨ [Recipe Diagnosis Summary]: {}", diag.diagnosis);
                    let mut reindexed = diag.optimized_steps;
                    for (i, st) in reindexed.iter_mut().enumerate() {
                        st.step = i + 1;
                    }
                    crate::term_log!(
                        "⚡ [Recipe Optimized]: Refined steps from {} -> {}",
                        steps.len(),
                        reindexed.len()
                    );
                    reindexed
                } else {
                    crate::term_log!("⚠️ [Recipe Diagnosis]: Diagnostic returned empty steps array, falling back to recorded steps.");
                    steps
                }
            } else {
                crate::term_log!("⚠️ [Recipe Diagnosis]: Failed to parse diagnosis JSON, falling back to recorded steps.");
                steps
            }
        }
        Err(e) => {
            crate::term_err!("⚠️ [Recipe Diagnosis]: Failed to complete diagnosis prompt: {e}, falling back to recorded steps.");
            steps
        }
    }
}

/// Worker Train / Interactive Recorder (Agentic Training Loop):
/// Mengeksplorasi halaman web secara interaktif langkah-demi-langkah (Reasoning Loop):
/// 1. Ambil snapshot DOM aktual setiap langkah.
/// 2. LLM Planner bernalar (reasoning) menentukan 1 aksi berikutnya.
/// 3. Aksi dieksekusi langsung di browser nyata.
/// 4. Halaman berubah -> loop lanjut membaca DOM baru hingga goal tercapai (status: "DONE").
/// 5. Seluruh jejak langkah dikompilasi jadi 1 blueprint AutomationRecipe JSON untuk eksekusi 0-token di masa depan!
pub fn record_recipe_action(payload: &str) -> Option<(String, String)> {
    let clean = payload.trim();
    if clean.is_empty() {
        return None;
    }

    let (target_url, user_goal) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(clean) {
        let u = val["url"].as_str().unwrap_or("").to_string();
        let g = val["goal"].as_str().unwrap_or("").to_string();
        (u, g)
    } else if let Some((u, g)) = clean.split_once('|') {
        (u.trim().to_string(), g.trim().to_string())
    } else {
        (clean.to_string(), "Automate page task".to_string())
    };

    if target_url.is_empty() {
        crate::term_err!("⚠️ [Recipe Train] Missing target URL in payload: {payload}");
        return None;
    }

    crate::term_log!(
        "🧠 [Recipe Train Loop] Starting interactive training loop for URL: '{}' | Goal: '{}'",
        target_url,
        user_goal
    );

    let url_for_thread = target_url.clone();
    let goal_for_thread = user_goal.clone();

    thread::spawn(move || {
        let initial_ts = get_latest_snapshot().map(|s| s.timestamp).unwrap_or(0);

        // Step 0: Buka Tab Baru Bersih (Isolated Clean State)
        crate::term_log!("🌐 [Recipe Train Loop] Step 0: Opening clean tab to: {}", url_for_thread);
        crate::services::browser_session_service::smart_open_or_switch_tab(
            &url_for_thread,
            None,
            None,
        );

        let clean_domain = url_for_thread
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.")
            .split('/')
            .next()
            .unwrap_or("")
            .to_string();

        // Tunggu halaman awal termuat sesuai target domain
        wait_for_target_page(&clean_domain, initial_ts, 10);

        let cfg = crate::assets::load_config();
        let is_id = cfg.lang.to_lowercase().starts_with("id");
        let start_speech = if is_id {
            format!("Mulai mempelajari alur untuk {clean_domain}.")
        } else {
            format!("Starting to learn workflow for {clean_domain}.")
        };
        crate::audio::tts::speak_assistant(&start_speech, "excited");

        let mut recorded_steps: Vec<RecipeStep> = Vec::new();
        let mut history_trace: Vec<String> = Vec::new();
        let mut overall_reasoning: Vec<String> = Vec::new();
        let mut suggested_recipe_id: Option<String> = None;
        let mut suggested_recipe_name: Option<String> = None;
        let mut suggested_required_params: Option<Vec<String>> = None;
        let mut last_warning_feedback: Option<String> = None;

        let max_iterations = 12;
        let mut llm = crate::llm::client::LlmClient::new();

        for iteration in 0..max_iterations {
            crate::term_log!(
                "🔄 [Recipe Train Loop] Iteration {}/{} — Inspecting current DOM state...",
                iteration + 1,
                max_iterations
            );

            // Ambil snapshot DOM terkini dari halaman aktif
            let snap = match get_latest_snapshot() {
                Some(s) => s,
                None => {
                    crate::term_err!("❌ [Recipe Train Loop] Failed to acquire page snapshot from browser");
                    break;
                }
            };

            // Rangkum elemen interaktif saat ini
            let mut buttons_summary = Vec::new();
            for b in snap.buttons.iter().take(30) {
                let label = b.text.as_deref().or(b.aria_label.as_deref()).unwrap_or("").trim();
                if !label.is_empty() {
                    buttons_summary.push(format!(
                        "- button: \"{}\" | selector: \"{}\"",
                        label.replace('\n', " "),
                        b.selector.as_deref().unwrap_or("")
                    ));
                }
            }

            let mut inputs_summary = Vec::new();
            for inp in snap.inputs.iter().take(15) {
                let label = inp.placeholder.as_deref().or(inp.aria_label.as_deref()).unwrap_or("").trim();
                inputs_summary.push(format!(
                    "- input: \"{}\" | selector: \"{}\"",
                    label.replace('\n', " "),
                    inp.selector.as_deref().unwrap_or("")
                ));
            }

            let texts_sample: Vec<String> = snap.texts.iter().take(15).map(|t| format!("- \"{}\"", t.trim())).collect();

            let history_text = if history_trace.is_empty() {
                "(None — this is the first step on the landing page)".to_string()
            } else {
                history_trace.join("\n")
            };

            let page_context = format!(
                "PAGE CONTEXT:\nDomain: {}\nTitle: {}\nCurrent URL: {}\n\nPREVIOUSLY EXECUTED STEPS:\n{}\n\nDETECTED INTERACTIVE INPUTS:\n{}\n\nDETECTED BUTTONS:\n{}\n\nDETECTED TEXTS:\n{}",
                snap.domain,
                snap.title,
                snap.url,
                history_text,
                if inputs_summary.is_empty() { "(none)".to_string() } else { inputs_summary.join("\n") },
                if buttons_summary.is_empty() { "(none)".to_string() } else { buttons_summary.join("\n") },
                if texts_sample.is_empty() { "(none)".to_string() } else { texts_sample.join("\n") },
            );

            let system_prompt = r#"You are an expert interactive browser automation trainer (Agentic Recorder).
Your role: Look at the current live web page, inspect what has been executed so far, and decide the NEXT SINGLE ACTION to progress towards fulfilling the user goal.

DECISION FORMAT (JSON ONLY):
{
  "reasoning": "Explain step-by-step what you observe on the screen and why this specific action is the optimal next move",
  "status": "CONTINUE" or "DONE",
  "recipe_id": "lowercase_snake_case_id (provide on first iteration or when finished)",
  "recipe_name": "Short Human Readable Recipe Name (MUST be double quoted string)",
  "required_params": ["query"] or [],
  "next_step": {
    "action": "click|type|enter|scroll",
    "selector": "CSS selector chosen from the detected elements list (or empty string)",
    "value": "{{query}} (use only if step inputs dynamic user search/data; otherwise literal text or null)"
  }
}

RULES:
1. If the user goal is already fully achieved (NOTE: if the goal is to play/watch a video or open a product/detail, you MUST click the target video/item first; do NOT stop at the search results list!), return status: "DONE" with next_step: null.
2. FORM / SEARCH AUTOCOMPLETE: After typing a city, airport, song, or search query into an input, check if a suggestions dropdown/list popped up (look at DETECTED BUTTONS for items with roles like option/menuitem or airport codes like "CGK", "Jakarta", etc.). If a suggestion list appears, your NEXT action MUST be clicking the target suggestion item or pressing enter! DO NOT re-type into the same or previous input!
3. If the recipe needs a dynamic query/keyword from user (e.g. searching songs, flights, products), use value: "{{query}}" and set required_params: ["query"].
4. If the recipe is static (clicks, open tab, static navigation), DO NOT use template variables and set required_params: [].
5. Output STRICTLY valid JSON. All string values MUST be strictly enclosed in double quotes (e.g. \"recipe_name\": \"My Recipe\"). Do NOT output unquoted strings.
6. NO surrounding conversation or explanations outside JSON. Keep output minimal and compact."#;

            let warning_block = if let Some(ref w) = last_warning_feedback {
                format!("\nCRITICAL OBSERVATION FROM LAST STEP:\n⚠️ {}\n", w)
            } else {
                String::new()
            };

            let user_prompt = format!(
                "OVERALL USER GOAL: \"{}\"\n\n{}{}",
                goal_for_thread,
                page_context,
                warning_block
            );

            let messages = vec![
                crate::llm::types::ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                crate::llm::types::ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ];

            match llm.complete_messages_silent(&messages) {
                Ok(llm_reply) => {
                    let clean_json = extract_json_from_llm(&llm_reply);
                    let repaired_json = repair_json_string_values(clean_json);
                    let decision_opt = serde_json::from_str::<TrainStepDecision>(&repaired_json)
                        .ok()
                        .or_else(|| {
                            crate::term_log!("⚠️ [Recipe Train Loop] Standard JSON parser failed, attempting lenient regex parser...");
                            parse_lenient_train_decision(&repaired_json)
                        })
                        .or_else(|| parse_lenient_train_decision(clean_json));

                    match decision_opt {
                        Some(decision) => {
                            crate::term_log!(
                                "  💡 [Train Iteration {} Reasoning]: {}",
                                iteration + 1,
                                decision.reasoning
                            );
                            overall_reasoning.push(format!("Step {}: {}", iteration + 1, decision.reasoning));

                            if decision.recipe_id.is_some() && suggested_recipe_id.is_none() {
                                suggested_recipe_id = decision.recipe_id;
                            }
                            if decision.recipe_name.is_some() && suggested_recipe_name.is_none() {
                                suggested_recipe_name = decision.recipe_name;
                            }
                            if decision.required_params.is_some() && suggested_required_params.is_none() {
                                suggested_required_params = decision.required_params;
                            }

                            // Cek apakah LLM menyatakan alur sudah selesai
                            if decision.status.eq_ignore_ascii_case("DONE") {
                                crate::term_log!("🎉 [Recipe Train Loop] Goal accomplished according to LLM reasoning!");
                                break;
                            }

                            // Eksekusi aksi berikutnya jika status CONTINUE
                            if let Some(mut step) = decision.next_step {
                                // 1. Deteksi duplikasi "type" berturut-turut pada elemen yang sama tanpa aksi submit
                                let is_duplicate_type = if step.action == "type" {
                                    recorded_steps.last().map(|last| {
                                        last.action == "type"
                                            && (last.selector == step.selector || (last.selector.is_empty() && step.selector.is_empty()))
                                    }).unwrap_or(false)
                                } else {
                                    false
                                };

                                if is_duplicate_type {
                                    crate::term_log!("⚠️ [Train Validator] Skipped duplicate 'type' step on same element without intermediate submit/click.");
                                    last_warning_feedback = Some("The query text is ALREADY typed into the search box in the previous step! Do NOT type again. Your next step MUST be submitting the search (e.g. click the search button with aria-label='Search' or action: 'enter').".to_string());
                                    continue;
                                }

                                // Auto-adopt target_text dan fallback_selectors langsung dari snapshot DOM
                                if step.action == "click" {
                                    if let Some(b) = snap.buttons.iter().find(|b| {
                                        b.selector.as_deref() == Some(&step.selector)
                                            || b.fallbacks.iter().any(|f| f == &step.selector)
                                    }) {
                                        if step.target_text.is_none() {
                                            step.target_text = b.text.clone().or_else(|| b.aria_label.clone());
                                        }
                                        for fb in &b.fallbacks {
                                            if fb != &step.selector && !step.fallback_selectors.contains(fb) {
                                                step.fallback_selectors.push(fb.clone());
                                            }
                                        }
                                    }
                                } else if step.action == "type" {
                                    if let Some(inp) = snap.inputs.iter().find(|inp| {
                                        inp.selector.as_deref() == Some(&step.selector)
                                            || inp.fallbacks.iter().any(|f| f == &step.selector)
                                    }) {
                                        if step.target_text.is_none() {
                                            step.target_text = inp.placeholder.clone().or_else(|| inp.aria_label.clone());
                                        }
                                        for fb in &inp.fallbacks {
                                            if fb != &step.selector && !step.fallback_selectors.contains(fb) {
                                                step.fallback_selectors.push(fb.clone());
                                            }
                                        }
                                    }
                                }

                                step.step = recorded_steps.len() + 1;
                                crate::term_log!(
                                    "  ▶️ [Executing Live Action] Step {}: {} ({})",
                                    step.step,
                                    step.action,
                                    step.selector
                                );

                                let before_step_ts = snap.timestamp;
                                let prev_hash = compute_dom_hash(&snap);

                                // Eksekusi aksi nyata di browser
                                let mut dummy_params = HashMap::new();
                                dummy_params.insert("query".to_string(), goal_for_thread.clone());
                                dummy_params.insert("keyword".to_string(), goal_for_thread.clone());
                                execute_single_step(&step, &dummy_params, Some(&snap));

                                // Tunggu secara reaktif hingga timestamp DOM diperbarui oleh background extension (maks 2500ms)
                                let new_snap_opt = wait_for_dom_change(before_step_ts, 2500);
                                let new_hash = new_snap_opt.as_ref().map(compute_dom_hash).unwrap_or(prev_hash);
                                let dom_changed = new_hash != prev_hash;

                                if let Some(ref new_snap) = new_snap_opt {
                                    if new_snap.timestamp > before_step_ts {
                                        crate::term_log!("  ⚡ [DOM Reactive] New DOM snapshot detected (ts: {}), advancing immediately!", new_snap.timestamp);
                                    }
                                }

                                // 2. Validasi perubahan DOM untuk aksi 'click': jika tidak ada perubahan, jangan masukkan ke steps!
                                if step.action == "click" && !dom_changed {
                                    crate::term_log!(
                                        "⚠️ [Train Validator] Click on '{}' did NOT produce any DOM/URL change (hash unchanged: {}). Skipping step from recording.",
                                        step.selector,
                                        prev_hash
                                    );
                                    last_warning_feedback = Some(format!(
                                        "Clicking on '{}' did NOT change the page DOM or URL. The element might not be interactive or didn't respond. Try a different selector/button from the detected list, or use action: 'enter'.",
                                        step.selector
                                    ));
                                    continue;
                                }

                                // Aksi valid dan membuahkan hasil: catat ke recipe!
                                last_warning_feedback = None;
                                history_trace.push(format!(
                                    "Step {}: {} ({})",
                                    step.step, step.action, step.selector
                                ));

                                let step_desc = step.target_text.clone().unwrap_or_else(|| step.selector.clone());
                                let step_voice = if is_id {
                                    format!("Langkah {} selesai, {}", step.step, step_desc)
                                } else {
                                    format!("Step {} completed, {}", step.step, step_desc)
                                };
                                crate::audio::tts::speak_assistant(&step_voice, "happy");

                                recorded_steps.push(step);
                            } else {
                                crate::term_log!("⚠️ [Recipe Train Loop] No next_step provided, ending loop.");
                                break;
                            }
                        }
                        None => {
                            crate::term_err!("❌ [Recipe Train Loop] Failed to parse decision JSON from LLM.\nRaw: {clean_json}");
                            break;
                        }
                    }
                }
                Err(e) => {
                    crate::term_err!("❌ [Recipe Train Loop] LLM call failed: {e}");
                    break;
                }
            }
        }

        // Kompilasi seluruh langkah yang berhasil direkam menjadi Recipe utuh
        if !recorded_steps.is_empty() {
            // Fase Diagnosa & Optimasi Langkah (Pruning redundant clicks/inputs)
            let recorded_steps = diagnose_and_optimize_steps(
                &mut llm,
                &goal_for_thread,
                &clean_domain,
                &url_for_thread,
                recorded_steps,
            );

            let final_recipe_id = suggested_recipe_id.unwrap_or_else(|| {
                format!(
                    "recipe_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                )
            });
            let final_recipe_name = suggested_recipe_name.unwrap_or_else(|| goal_for_thread.clone());

            // Tentukan required_params secara ketat: periksa template {{...}} di semua langkah
            let mut final_params = suggested_required_params.unwrap_or_default();
            let auto_extracted = super::types::extract_template_params(&recorded_steps);
            for p in auto_extracted {
                if !final_params.contains(&p) {
                    final_params.push(p);
                }
            }

            let compiled_recipe = AutomationRecipe {
                recipe_id: final_recipe_id.clone(),
                name: final_recipe_name,
                domain: clean_domain.clone(),
                version: 1,
                required_params: final_params,
                initial_action: InitialAction {
                    action: "open_new_tab".to_string(),
                    url: url_for_thread.clone(),
                },
                steps: recorded_steps,
            };

            let recipe_file_path = get_recipes_dir().join(format!("{}.json", final_recipe_id));
            let _ = fs::create_dir_all(get_recipes_dir());
            if let Ok(formatted) = serde_json::to_string_pretty(&compiled_recipe) {
                if fs::write(&recipe_file_path, formatted).is_ok() {
                    crate::term_log!(
                        "🏆 [Recipe Train Complete] Compiled {} interactive steps into '{:?}'! Future runs will cost 0 tokens.",
                        compiled_recipe.steps.len(),
                        recipe_file_path
                    );
                    let done_speech = if is_id {
                        format!("Semua langkah berhasil dipelajari dan disimpan ke resep {}!", compiled_recipe.name)
                    } else {
                        format!("All steps successfully learned and saved to recipe {}!", compiled_recipe.name)
                    };
                    crate::audio::tts::speak_assistant(&done_speech, "excited");
                }
            }
        } else {
            crate::term_err!("⚠️ [Recipe Train Complete] No steps were recorded.");
        }
    });

    Some((
        format!("Train Recipe: {user_goal}"),
        format!("Interactive training loop started for {target_url}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_unquoted_recipe_name_json() {
        let malformed_json = r#"{
  "reasoning": "Looking for video",
  "status": "CONTINUE",
  "recipe_id": "search_and_play_youtube_video",
  "recipe_name": Search and play video on YouTube,
  "required_params": ["query"],
  "next_step": {
    "action": "click",
    "selector": "[aria-label='Search']",
    "target_text": "Search"
  }
}"#;

        let repaired = repair_json_string_values(malformed_json);
        assert!(repaired.contains(r#""recipe_name": "Search and play video on YouTube","#));

        let parsed = serde_json::from_str::<TrainStepDecision>(&repaired);
        assert!(parsed.is_ok(), "Failed to parse repaired json: {:?}", parsed.err());
        let decision = parsed.unwrap();
        assert_eq!(decision.recipe_name.as_deref(), Some("Search and play video on YouTube"));
        assert_eq!(decision.status, "CONTINUE");
    }
}
