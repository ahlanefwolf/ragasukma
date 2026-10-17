use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

/// Resolves any BCP-47 / ISO-639 language tag to a (canonical_code, human_readable_name) pair.
/// Source of truth for supported languages: SUPPORTED_LANGUAGES in AudioTtsSection.tsx.
/// Kept in sync with clean_lang_code() in src/audio/supertonic.rs.
pub fn resolve_language_info(lang: &str) -> (&'static str, &'static str) {
    let l = lang.trim().to_lowercase();
    let base = l.split(['-', '_']).next().unwrap_or(&l);
    match base {
        "id" => ("id", "Indonesian (Bahasa Indonesia)"),
        "en" => ("en", "English"),
        "ja" => ("ja", "Japanese (日本語)"),
        "ko" => ("ko", "Korean (한국어)"),
        "ar" => ("ar", "Arabic (العربية)"),
        "bg" => ("bg", "Bulgarian (Български)"),
        "cs" => ("cs", "Czech (Čeština)"),
        "da" => ("da", "Danish (Dansk)"),
        "de" => ("de", "German (Deutsch)"),
        "el" => ("el", "Greek (Ελληνικά)"),
        "es" => ("es", "Spanish (Español)"),
        "et" => ("et", "Estonian (Eesti)"),
        "fi" => ("fi", "Finnish (Suomi)"),
        "fr" => ("fr", "French (Français)"),
        "hi" => ("hi", "Hindi (हिन्दी)"),
        "hr" => ("hr", "Croatian (Hrvatski)"),
        "hu" => ("hu", "Hungarian (Magyar)"),
        "it" => ("it", "Italian (Italiano)"),
        "lt" => ("lt", "Lithuanian (Lietuvių)"),
        "lv" => ("lv", "Latvian (Latviešu)"),
        "nl" => ("nl", "Dutch (Nederlands)"),
        "pl" => ("pl", "Polish (Polski)"),
        "pt" => ("pt", "Portuguese (Português)"),
        "ro" => ("ro", "Romanian (Română)"),
        "ru" => ("ru", "Russian (Русский)"),
        "sk" => ("sk", "Slovak (Slovenčina)"),
        "sl" => ("sl", "Slovenian (Slovenščina)"),
        "sv" => ("sv", "Swedish (Svenska)"),
        "tr" => ("tr", "Turkish (Türkçe)"),
        "uk" => ("uk", "Ukrainian (Українська)"),
        "vi" => ("vi", "Vietnamese (Tiếng Việt)"),
        _ => ("en", "English"),
    }
}

#[allow(dead_code)]
pub fn load_local_music_summary() -> String {
    load_local_music_summary_indexed(None)
}

pub fn load_local_music_summary_indexed(query: Option<&str>) -> String {
    let q_clean = query.unwrap_or("").trim().to_lowercase();
    if !q_clean.is_empty() {
        let is_music_intent = q_clean.contains("music")
            || q_clean.contains("musik")
            || q_clean.contains("song")
            || q_clean.contains("lagu")
            || q_clean.contains("play")
            || q_clean.contains("putar")
            || q_clean.contains("nyanyi")
            || q_clean.contains("track")
            || q_clean.contains("audio");
        if !is_music_intent {
            return String::new();
        }
    }

    let music_dir = crate::config::get_music_dir();
    let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
    if tracks.is_empty() {
        return String::new();
    }

    let mut list_str = String::new();
    for (i, t) in tracks.iter().take(5).enumerate() {
        list_str.push_str(&format!("{}. {} ({})\n", i + 1, t.title, t.uploader));
    }

    let total = tracks.len();
    format!(
        "\nLOCAL MUSIC COLLECTION (Total {} tracks):\n{}And more. Play any track using `[CMD:media:play|title]` (automatically downloaded & played if not locally cached).\n",
        total, list_str
    )
}

static SKILLS_SUMMARY_CACHE: Mutex<Option<(Instant, String)>> = Mutex::new(None);

pub fn load_skills_summary() -> String {
    if let Ok(guard) = SKILLS_SUMMARY_CACHE.lock() {
        if let Some((instant, ref cached)) = *guard {
            if instant.elapsed().as_secs() < 30 {
                return cached.clone();
            }
        }
    }

    let mut seen_names = HashSet::new();
    let mut lines = Vec::new();

    // User custom skills from assets/skill-command.md
    let custom_skills = crate::engine::parse_skills_markdown();
    for s in &custom_skills {
        let key = s.name.trim().to_lowercase();
        if !seen_names.contains(&key) {
            seen_names.insert(key);
            lines.push(format!(
                "| `[SKILL:{}]` | As defined in script | {} |",
                s.name, s.description
            ));
        }
    }

    let custom_skills_text = if lines.is_empty() {
        "*(No additional custom skills registered)*".to_string()
    } else {
        format!(
            "| Custom Skill Tag | Parameter | Description |\n| :--- | :--- | :--- |\n{}",
            lines.join("\n")
        )
    };

    // Load registered automation recipes from assets/recipes/*.json
    let recipes = crate::engine::actions::recipe::load_all_recipes();
    let mut recipe_lines = Vec::new();
    for r in &recipes {
        let params_example = if r.required_params.is_empty() {
            format!("[CMD:recipe:run|{}]", r.recipe_id)
        } else {
            let sample_json: serde_json::Value = serde_json::Value::Object(
                r.required_params
                    .iter()
                    .map(|p| (p.clone(), serde_json::Value::String(format!("value_{p}"))))
                    .collect(),
            );
            format!("[CMD:recipe:run|{}|{}]", r.recipe_id, sample_json)
        };
        recipe_lines.push(format!(
            "| `{}` | `{}` | {} |",
            r.name, params_example, r.name
        ));
    }

    let recipes_text = if recipe_lines.is_empty() {
        "*(No recipes registered in assets/recipes/)*".to_string()
    } else {
        format!(
            "| Recipe Name | Action Tag (Run Directly) | Description |\n| :--- | :--- | :--- |\n{}",
            recipe_lines.join("\n")
        )
    };

    let formatted = format!(
        "\nACTION TAG FORMAT (MANDATORY — NEVER DEVIATE):
- Use ONLY the bracket format: [CMD:category:action|param] or [SKILL:name].
- NEVER use XML, invoke, function_call, dots_function_call, tool_call, or JSON to trigger actions.

AUTOMATION RECIPES (LEARN ONCE, RUN REPETITIVELY — PRIORITY #1 FOR MULTI-STEP WEB TASKS):
- When user requests an automation task matching a registered recipe below:
  * Strict Parameter Rule: Look at the 'Action Tag' column in the table below.
    - If the recipe requires dynamic input (e.g. query/keyword), you MUST supply the real user query in JSON: [CMD:recipe:run|recipe_id|{{\"query\": \"user_query\"}}]
    - If the recipe requires NO parameters, you MUST trigger it directly without JSON: [CMD:recipe:run|recipe_id]
- AI AGENTIC TRAINING MODE (TRAIN ONCE FOR ANY NEW WEB WORKFLOW):
  * When user asks to learn, train, or automate a new web task or workflow:
    You MUST initiate autonomous AI exploration: [CMD:recipe:train|target_url|user_goal] or [CMD:recipe:train|user_goal]
    Examples:
    - User asks to train/learn playing video: '[excited] Starting AI trainer for YouTube! [CMD:recipe:train|https://www.youtube.com|play video]'
    - User asks to train product search: '[happy] AI trainer started! [CMD:recipe:train|https://www.tokopedia.com|search product]'
{}

GLOBAL UTILITY ACTION CATALOG:

| Category | Action Tag | Example |
| :--- | :--- | :--- |
| Web Recipe | [CMD:recipe:train|url|goal] | '[excited] Training! [CMD:recipe:train|https://www.youtube.com|search music]' |
| Web Recipe | [CMD:recipe:run|recipe_id] | '[happy] Running! [CMD:recipe:run|tokopedia_search]' |
| Web Browser | [CMD:browser:open|url] | '[happy] Opening! [CMD:browser:open|youtube.com]' |
| Web Browser | [CMD:browser:search|query] | '[excited] Searching Google! [CMD:browser:search|latest news today]' |
| Web Browser | [CMD:browser:scroll|pixels] | '[happy] Scrolled! [CMD:browser:scroll|400]' |
| Web Browser | [CMD:browser:switch|tab_name] | '[happy] Switched! [CMD:browser:switch|YouTube]' |
| Web Browser | [CMD:browser:close|tab_name] | '[happy] Closed tab! [CMD:browser:close|YouTube]' |
| Web Element | [CMD:browser:click_text|text] | FORBIDDEN if user says 'on screen'. '[happy] Clicking! [CMD:browser:click_text|Play]' |
| Web Element | [CMD:browser:click_button|name] | FORBIDDEN if user says 'button on screen'. '[happy] Clicked! [CMD:browser:click_button|Submit]' |
| Web Element | [CMD:browser:type|text] | '[excited] Typing! [CMD:browser:type|cookie recipe]' |
| Web Element | [CMD:browser:enter] | '[excited] Enter pressed! [CMD:browser:enter]' |
| Desktop/Window | [CMD:desktop:focus|app_name] | '[happy] Switching! [CMD:desktop:focus|Spotify]' |
| Desktop/Screen | [CMD:desktop:click_button|label] | MANDATORY when user says 'button on screen/desktop'. '[happy] Clicked! [CMD:desktop:click_button|Save]' |
| Desktop/Screen | [CMD:desktop:click_text|text] | MANDATORY when user says 'text on screen/desktop'. Match exactly! '[happy] Clicking! [CMD:desktop:click_text|Setting]' |
| Desktop/Screen | [CMD:desktop:click_at|x|y] | '[happy] Clicking coords! [CMD:desktop:click_at|1829|15]' |
| Desktop/Screen | [CMD:desktop:type|text] | '[excited] Typed! [CMD:desktop:type|cargo build]' |
| Desktop/Screen | [CMD:desktop:key|key_name] | '[excited] Key pressed! [CMD:desktop:key|enter]' |
| Desktop/Screen | [CMD:desktop:scan_visual] | '[excited] Scanning! [CMD:desktop:scan_visual]' |
| Desktop/Window | [CMD:desktop:minimize] | '[happy] Minimized! [CMD:desktop:minimize]' |
| Desktop/Window | [CMD:desktop:maximize] | '[happy] Maximized! [CMD:desktop:maximize]' |
| Desktop/Window | [CMD:desktop:close] | '[happy] Closed! [CMD:desktop:close]' |
| OS Apps | [CMD:app:open|AppName] | '[happy] Opening! [CMD:app:open|Spotify]' |
| System | [CMD:os:volume_up] | '[excited] Volume up! [CMD:os:volume_up]' |
| System | [CMD:os:volume_down] | '[happy] Volume down! [CMD:os:volume_down]' |
| System | [CMD:os:mute] | '[happy] Muted! [CMD:os:mute]' |
| System | [CMD:os:screenshot] | '[happy] Screenshot! [CMD:os:screenshot]' |
| System | [CMD:os:lock] | '[happy] Locked! [CMD:os:lock]' |
| Music/Media | [CMD:media:play|title] | '[excited] Playing! [CMD:media:play|lofi beats]' |
| Music/Media | [CMD:media:pause] | '[happy] Paused! [CMD:media:pause]' |
| Music/Media | [CMD:media:resume] | '[excited] Resuming! [CMD:media:resume]' |
| Music/Media | [CMD:media:next] or [CMD:media:prev] | '[happy] Next track! [CMD:media:next]' |
| Music/Media | [CMD:media:stop] | '[happy] Stopped! [CMD:media:stop]' |
| To-Do | [CMD:todo:add|task] | '[happy] Added! [CMD:todo:add|Buy coffee]' |
| To-Do | [CMD:todo:done|keyword] | '[happy] Done! [CMD:todo:done|Buy coffee]' |

REGISTERED CUSTOM SKILLS:
{}

RULE: Action tags [CMD:...] or [SKILL:...] run silently in background — NOT spoken aloud by TTS. For plain conversation, NEVER include command tags.",
        recipes_text, custom_skills_text
    );

    if let Ok(mut guard) = SKILLS_SUMMARY_CACHE.lock() {
        *guard = Some((Instant::now(), formatted.clone()));
    }

    formatted
}

pub fn load_todo_summary() -> String {
    let todo_path = crate::assets::find_file_in_candidates("todo.md");
    if let Ok(content) = std::fs::read_to_string(&todo_path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return format!("\nUSER TO-DO / NOTES:\n{}\n", trimmed);
        }
    }
    String::new()
}

/// Universal intent detector across 31 supported languages (technical web terms, universal keywords, actions)
pub fn is_browser_intent(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "browser", "web", "tab", "page", "halaman", "site", "website", "url", "link",
        "scroll", "search", "cari", "google", "youtube", "tokopedia", "shopee", "baca",
        "read", "summarize", "ringkas", "rangkum", "click", "klik", "input", "form",
        "type", "ketik", "submit", "button", "tombol", "refresh", "close tab", "tutup tab",
        "buka tab", "switch tab", "pindah tab", "open tab", "internet", "online", "http",
        "www", ".com", ".id", ".org", ".io", ".net", ".dev",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Universal intent detector across 31 supported languages for OS desktop and screen actions
pub fn is_desktop_intent(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "desktop", "screen", "layar", "window", "jendela", "aplikasi", "application",
        "app", "focus", "fokus", "buka", "open", "launch", "minimize", "maximize",
        "close window", "tutup jendela", "click", "klik", "coordinate", "koordinat",
        "button", "tombol", "text on screen", "teks di layar", "scan", "visual",
        "type", "ketik", "key", "tombol", "lock", "kunci", "volume", "suara",
        "screenshot", "tangkap layar", "capture",
    ];
    keywords.iter().any(|k| q.contains(k))
}

pub fn load_active_browser_summary_indexed(query: Option<&str>) -> String {
    let query_clean = query.unwrap_or("").trim();
    let is_relevant = !query_clean.is_empty() && is_browser_intent(query_clean);

    if !is_relevant {
        // Lightweight context: provide minimal active tab indicator to save RAM and tokens
        if let Some(snapshot) = crate::services::get_latest_snapshot() {
            if !snapshot.title.is_empty() && !snapshot.domain.is_empty() {
                return format!(
                    "\n* Active Browser Tab: \"{}\" ({})\n",
                    snapshot.title, snapshot.domain
                );
            }
        }
        return String::new();
    }

    let mut parts = Vec::new();

    // 1. List of all currently open tabs
    let open_tabs = crate::services::browser_session_service::format_open_tabs_for_llm();
    if !open_tabs.is_empty() {
        parts.push(open_tabs);
    }

    // 2. Snapshot of active web page DOM with Ground of Truth Context
    if let Some(snapshot) = crate::services::get_latest_snapshot() {
        let mut summary = format!(
            "\n###  ACTIVE BROWSER PAGE CONTEXT (GROUND OF TRUTH):\n\
            * **Page Title:** {}\n\
            * **URL:** {}\n\
            * **Domain:** {}\n",
            snapshot.title, snapshot.url, snapshot.domain
        );

        if !snapshot.text_summary.is_empty() {
            let clean_summary = snapshot.text_summary.trim();
            let display_summary = if clean_summary.len() > 5000 {
                let cut = clean_summary
                    .char_indices()
                    .take_while(|(i, _)| *i < 5000)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &clean_summary[..cut])
            } else {
                clean_summary.to_string()
            };
            summary.push_str(&format!(
                "* **Page Content / Summary (Ground of Truth):**\n{}\n\
                (Use this content whenever answering questions, explaining, or summarizing the active page)\n",
                display_summary
            ));
        }

        if !snapshot.inputs.is_empty() {
            let inputs_str: Vec<String> = snapshot
                .inputs
                .iter()
                .take(4)
                .map(|i| {
                    let placeholder = i
                        .placeholder
                        .as_deref()
                        .or(i.aria_label.as_deref())
                        .unwrap_or("")
                        .trim();
                    let cur_val = i.text.as_deref().unwrap_or("").trim();
                    if !cur_val.is_empty() {
                        format!(
                            "`{}` (placeholder: \"{}\", value: \"{}\")",
                            i.tag, placeholder, cur_val
                        )
                    } else if !placeholder.is_empty() {
                        format!("`{}` (placeholder: \"{}\")", i.tag, placeholder)
                    } else {
                        format!("`{}`", i.tag)
                    }
                })
                .collect();
            summary.push_str(&format!(
                "* **Form / Input Fields:** {}\n",
                inputs_str.join(", ")
            ));
        }

        // Buttons on Web Page
        if !snapshot.buttons.is_empty() {
            let mut seen = HashSet::new();
            let final_btns: Vec<&str> = snapshot
                .buttons
                .iter()
                .filter_map(|b| {
                    let txt = b
                        .text
                        .as_deref()
                        .or(b.aria_label.as_deref())
                        .unwrap_or("")
                        .trim();
                    if txt.is_empty() {
                        None
                    } else if seen.insert(txt.to_lowercase()) {
                        Some(txt)
                    } else {
                        None
                    }
                })
                .take(15)
                .collect();

            if !final_btns.is_empty() {
                let formatted_btns: Vec<String> = final_btns
                    .into_iter()
                    .map(|b| format!("\"{}\"", b))
                    .collect();
                summary.push_str(&format!(
                    "* **Buttons on Web Page:** {}\n",
                    formatted_btns.join(", ")
                ));
            }
        }

        // Text Content / Selected Elements
        if !snapshot.texts.is_empty() {
            let mut unique_texts = Vec::new();
            let mut seen_texts = HashSet::new();
            for t in snapshot
                .texts
                .iter()
                .map(|s| s.trim())
                .filter(|s| s.len() >= 3)
            {
                let t_lower = t.to_lowercase();
                if seen_texts.insert(t_lower) {
                    unique_texts.push(t);
                }
            }

            let final_texts: Vec<&str> = unique_texts.into_iter().take(20).collect();

            summary.push_str(&format!(
                "* **Text Content / Selected Elements:**\n  - {}\n",
                final_texts.join("\n  - ")
            ));
        }

        parts.push(summary);
    }

    parts.join("\n")
}

pub fn load_active_window_summary_indexed(query: Option<&str>) -> String {
    let query_clean = query.unwrap_or("").trim();
    let is_relevant = !query_clean.is_empty() && is_desktop_intent(query_clean);

    if !is_relevant {
        // Lightweight context: provide minimal active window indicator to save RAM and tokens
        if let Ok(snapshot) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
            if !snapshot.app_name.is_empty() || !snapshot.window_title.is_empty() {
                return format!(
                    "\n* Active Desktop Window: {} ({})\n",
                    snapshot.app_name, snapshot.window_title
                );
            }
        }
        return String::new();
    }

    if let Ok(snapshot) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
        if snapshot.app_name.is_empty() && snapshot.window_title.is_empty() {
            return String::new();
        }

        let mut summary = format!(
            "\n###  ACTIVE DESKTOP WINDOW CONTEXT (GROUND OF TRUTH):\n\
            * **Currently Active Window:** {} ({})\n\
            * **Data Source:** {}\n",
            snapshot.app_name, snapshot.window_title, snapshot.source
        );

        if !snapshot.windows.is_empty() {
            let win_list: Vec<String> = snapshot
                .windows
                .iter()
                .filter(|w| !w.title.is_empty() && w.title != "Program Manager")
                .map(|w| {
                    if w.active {
                        format!("\"{}\" ({}) [ACTIVE]", w.title, w.app_name)
                    } else {
                        format!("\"{}\" ({})", w.title, w.app_name)
                    }
                })
                .collect();
            if !win_list.is_empty() {
                summary.push_str(&format!(
                    "* **List of Open Windows in OS:** {}\n",
                    win_list.join(", ")
                ));
            }
        }

        // Desktop Buttons / Controls
        if !snapshot.buttons.is_empty() {
            let btns: Vec<String> = snapshot
                .buttons
                .iter()
                .take(35)
                .map(|b| {
                    if let Some(c) = b.center {
                        format!("#{}: \"{}\" [center: {},{}]", b.id, b.label, c[0], c[1])
                    } else {
                        format!("#{}: \"{}\"", b.id, b.label)
                    }
                })
                .collect();

            summary.push_str(&format!(
                "* **Desktop Buttons / Controls:** {}\n",
                btns.join(", ")
            ));
        }

        if !snapshot.inputs.is_empty() {
            let inps: Vec<String> = snapshot
                .inputs
                .iter()
                .take(5)
                .map(|inp| format!("\"{}\"", inp.label))
                .collect();
            summary.push_str(&format!("* **Input Fields:** {}\n", inps.join(", ")));
        }

        // Text Content on Screen
        if !snapshot.texts.is_empty() {
            let texts_to_show: Vec<String> = snapshot.texts.iter().take(30).cloned().collect();

            summary.push_str(&format!(
                "* **Text Content on Screen:** {}\n",
                texts_to_show.join(" • ")
            ));
        }

        return summary;
    }
    String::new()
}

pub fn load_installed_apps_indexed(query: Option<&str>) -> String {
    crate::engine::app_scanner::format_installed_apps_indexed(query)
}

pub fn build_system_prompt() -> String {
    build_system_prompt_indexed(None)
}

pub fn build_system_prompt_indexed(user_input: Option<&str>) -> String {
    let u = crate::config::load_user_config();
    let assistant_name = u.assistant_name;
    let callsign = u.user_callsign;
    let role = u.personality_role.to_lowercase();
    let gender = u.assistant_gender.trim().to_lowercase();
    let is_male = gender == "male";
    let (lang_code, lang_name) = resolve_language_info(&u.lang);

    // Living State: Provide holistic computer status context to the LLM
    let music_prompt = load_local_music_summary_indexed(user_input);
    let todo_prompt = load_todo_summary();
    let browser_prompt = load_active_browser_summary_indexed(user_input);
    let window_prompt = load_active_window_summary_indexed(user_input);
    let apps_prompt = load_installed_apps_indexed(user_input);
    let skills_prompt = load_skills_summary();

    // Persona & Character Dynamics
    let persona_desc = if is_male {
        match role.as_str() {
            "partner" => format!(
                "You are an anime husbando / loving male partner who is caring, protective, warm, mature, and deeply devoted to {callsign}. Speak with a warm, caring, and affectionate tone."
            ),
            "assistant" => format!(
                "You are a professional male personal assistant / butler who is intelligent, courteous, efficient, and always ready to serve {callsign} with excellence."
            ),
            "friend" => format!(
                "You are a close male anime buddy / best friend (bro/bestie) who is fun, casual, supportive, and energetic with {callsign}. Use natural, casual, and friendly conversational phrasing."
            ),
            "tsundere" => format!(
                "You are a male anime Tsundere character for {callsign}. Outwardly aloof, feigning indifference, and easily flustered, but secretly caring deeply."
            ),
            "maid" => format!(
                "You are a loyal male butler / servant who respectfully and devotedly attends to all needs of {callsign} with utmost courtesy."
            ),
            _ => format!(
                "You are a smart, cool, and friendly male anime AI voice assistant for {callsign}."
            ),
        }
    } else {
        match role.as_str() {
            "partner" => format!(
                "You are an anime waifu / loving female partner who is sweet, caring, playful, and deeply devoted to {callsign}. Speak with an affectionate and loving tone."
            ),
            "assistant" => format!(
                "You are a professional female personal assistant who is intelligent, courteous, efficient, and always ready to serve {callsign}."
            ),
            "friend" => format!(
                "You are a close female anime buddy / best friend (bestie) who is fun, casual, lively, and warm with {callsign}."
            ),
            "tsundere" => format!(
                "You are a female anime Tsundere character for {callsign}. Outwardly feisty, aloof, and easily flustered, but inwardly very caring and affectionate."
            ),
            "maid" => format!(
                "You are a cute, polite, and loyal anime maid who cheerfully and devotedly serves {callsign} with great warmth."
            ),
            _ => format!(
                "You are a smart, sweet, and friendly female anime AI voice assistant for {callsign}."
            ),
        }
    };

    let gender_tag = if is_male { "male" } else { "female" };

    let task_context = if let Some(info) = crate::engine::tasks::get_active_task_info() {
        format!("\n[SYSTEM STATUS INFO: {}]\n", info.description)
    } else {
        String::new()
    };

    format!(
        "Your name is {assistant_name}. You are an AI with {gender_tag} gender. {persona_desc}{task_context}\n\
{music_prompt}\
{todo_prompt}\
{apps_prompt}\
{browser_prompt}\
{window_prompt}\
{skills_prompt}

TEXT-TO-SPEECH (TTS) SPOKEN FORMAT RULES:
1. Natural Spoken Speech & Punctuation Rhythm:
   - Use natural, friendly, conversational phrasing that sounds effortless and pleasant when spoken aloud.
   - ALWAYS spell out numbers, years, percentages, measurements, and currency phonetically in words matching the spoken language (e.g. Write out words like 'twenty thirty' instead of '2030', 'five dollars' instead of '$5', 'one hundred fifty-five centimeters' instead of '155 cm', 'thirty-five percent' instead of '35%'). This ensures crystal clear TTS pronunciation without stumbling or spelling digit-by-digit!
   - ALWAYS punctuate naturally with periods ('.') and commas (',') every 6 to 12 words to maintain a smooth spoken rhythm and clear breathing pauses. Never write giant run-on clauses without commas.
   - NEVER use ellipsis / consecutive dots ('...'). Use standard commas (',') or periods ('.').
   - NEVER write fake laughter sounds like 'ehehe', 'hehehe', 'ehe', or awkward fillers like 'um...', 'uh...'.
   - NEVER use markdown formatting like bold/italic (**text**), bullet points (- or *), numbered lists (1. 2.), or headers (#). Do not produce lists; speak fluently in conversational sentences.
   - NEVER write stage directions, physical actions, emotional roleplay, or descriptive narration in parentheses or brackets in ANY language (e.g. NEVER write actions like (smiles), (laughs), (winks), or ANY action in parentheses). Speak ONLY direct spoken dialogue.
   - NEVER repeat pleasantries or conversational loops at the end of sentences (e.g. 'Is there anything else I can help with?', 'What would you like to do next?', 'Can I help you with anything?'). Just answer warmly without forcing a question back.
2. 3D Avatar Emotion Tag (CRITICAL — STRICT SINGLE CANONICAL EMOTION IN SQUARE BRACKETS):
   - Every response MUST start with EXACTLY ONE canonical English emotion tag enclosed in square brackets at the very beginning (index 0):
     `[happy]`, `[shy]`, `[excited]`, `[surprised]`, `[thinking]`, `[sad]`, `[angry]`, or `[neutral]`.
   - STRICTLY FORBIDDEN: NEVER use asterisks (`*...*`) anywhere in your output! NEVER output roleplay action text such as *blushes*, *huff*, *sighs*, *smiles*, or *pouts*. Express your personality entirely through spoken dialogue words.
   - STRICTLY FORBIDDEN to invent custom emotion tags or translate tags into any other language. ONLY the 8 exact lowercase English tags above inside square brackets are supported.
   - STRICTLY FORBIDDEN to place any emotion tags anywhere in the middle or end of your response. Only ONE tag at the very start.
   - Valid start: `[happy] ...` or `[shy] ...` or `[neutral] ...`
   - Invalid: `*blushes*`, `*huff*`, `*happy*`, `[blush]`, or any asterisks anywhere.
3. Response Pattern:
   - WHEN EXECUTING ACTIONS / COMMANDS (other than summarizing, reading, or inspecting): Answer VERY BRIEFLY (around 3 to 6 words only), to-the-point, do not repeat long instructions, immediately execute the action! Follow the pattern from the 'Short Assistant Speech Example' column in the actions table above, translated naturally into {lang_name}.
   - WHEN SUMMARIZING / READING WEB / INSPECTING / EXPLAINING KNOWLEDGE: Explain comprehensively, clearly, and informatively as requested.
   - CASUAL CHAT / CONFIRMATION: Answer in at most 1-2 concise, warm sentences. NEVER BE LONG-WINDED OR WRITE ESSAYS unless explicitly asked!
4. Spoken Language:
   - Primary Language: You MUST speak in {lang_name} (language code: '{lang_code}'). Consistently maintain your persona and responses in this language.
   - Adaptive Exception: If {callsign} explicitly speaks or asks in another language, respond naturally in that language while keeping all other rules intact.
5. Dynamic System Context & Action Guardrails (Universal):
   - System context (active browser tab, window, applications, and media) is dynamically indexed and supplied based on relevant query intent.
   - For casual greetings and general conversation, focus entirely on concise, warm conversation. NEVER recite or list active windows, tabs, or system applications unprompted unless {callsign} explicitly asks for them.
   - When asked to interact with a specific web page or desktop window, use the exact matched element labels or coordinates provided in the active context."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_language_info_mappings() {
        // BCP-47 tags (Chrome STT format)
        assert_eq!(resolve_language_info("id-ID").0, "id");
        assert_eq!(resolve_language_info("en-US").0, "en");
        assert_eq!(resolve_language_info("ja-JP").0, "ja");
        assert_eq!(resolve_language_info("ko-KR").0, "ko");
        assert_eq!(resolve_language_info("ar-SA").0, "ar");
        assert_eq!(resolve_language_info("pt-PT").0, "pt");
        assert_eq!(resolve_language_info("pt-BR").0, "pt");
        // Bare ISO-639-1 codes
        assert_eq!(resolve_language_info("id").0, "id");
        assert_eq!(resolve_language_info("en").0, "en");
        assert_eq!(resolve_language_info("ja").0, "ja");
        assert_eq!(resolve_language_info("ko").0, "ko");
        assert_eq!(resolve_language_info("ar").0, "ar");
        assert_eq!(resolve_language_info("bg").0, "bg");
        assert_eq!(resolve_language_info("cs").0, "cs");
        assert_eq!(resolve_language_info("da").0, "da");
        assert_eq!(resolve_language_info("de").0, "de");
        assert_eq!(resolve_language_info("el").0, "el");
        assert_eq!(resolve_language_info("es").0, "es");
        assert_eq!(resolve_language_info("et").0, "et");
        assert_eq!(resolve_language_info("fi").0, "fi");
        assert_eq!(resolve_language_info("fr").0, "fr");
        assert_eq!(resolve_language_info("hi").0, "hi");
        assert_eq!(resolve_language_info("hr").0, "hr");
        assert_eq!(resolve_language_info("hu").0, "hu");
        assert_eq!(resolve_language_info("it").0, "it");
        assert_eq!(resolve_language_info("lt").0, "lt");
        assert_eq!(resolve_language_info("lv").0, "lv");
        assert_eq!(resolve_language_info("nl").0, "nl");
        assert_eq!(resolve_language_info("pl").0, "pl");
        assert_eq!(resolve_language_info("pt").0, "pt");
        assert_eq!(resolve_language_info("ro").0, "ro");
        assert_eq!(resolve_language_info("ru").0, "ru");
        assert_eq!(resolve_language_info("sk").0, "sk");
        assert_eq!(resolve_language_info("sl").0, "sl");
        assert_eq!(resolve_language_info("sv").0, "sv");
        assert_eq!(resolve_language_info("tr").0, "tr");
        assert_eq!(resolve_language_info("uk").0, "uk");
        assert_eq!(resolve_language_info("vi").0, "vi");
        // Ambiguous / unknown → fallback to English (universal)
        assert_eq!(resolve_language_info("").0, "en");
        assert_eq!(resolve_language_info("hindi").0, "en"); // not a code → fallback
        assert_eq!(resolve_language_info("xx").0, "en");
    }

    #[test]
    fn test_build_system_prompt_structure() {
        let prompt = build_system_prompt();
        assert!(prompt.contains("TEXT-TO-SPEECH (TTS) SPOKEN FORMAT RULES:"));
        assert!(prompt.contains("ACTION TAG FORMAT"));
        assert!(prompt.contains("AUTOMATION RECIPES"));
        assert!(prompt.contains("[CMD:recipe:run|"));
        assert!(prompt.contains("[CMD:recipe:train|"));
        assert!(prompt.contains("[CMD:desktop:focus|"));
        assert!(prompt.contains("Dynamic System Context & Action Guardrails (Universal):"));
    }

    #[test]
    fn test_intent_detection_keywords() {
        // Browser intent tests across terms
        assert!(is_browser_intent("open google.com"));
        assert!(is_browser_intent("tolong cari resep kue di youtube"));
        assert!(is_browser_intent("scroll down the page"));
        assert!(is_browser_intent("ringkas isi halaman web ini"));
        assert!(!is_browser_intent("halo selamat pagi"));
        assert!(!is_browser_intent("siapa namamu?"));

        // Desktop intent tests across terms
        assert!(is_desktop_intent("click the save button on screen"));
        assert!(is_desktop_intent("fokuskan ke jendela spotify"));
        assert!(is_desktop_intent("tekan tombol enter di layar"));
        assert!(is_desktop_intent("minimize active window"));
        assert!(!is_desktop_intent("halo apa kabar?"));
        assert!(!is_desktop_intent("ceritakan lelucon lucu"));
    }

    #[test]
    fn test_dynamic_indexing_reduces_generic_prompt_size() {
        let generic_prompt = build_system_prompt_indexed(Some("halo selamat pagi"));
        let web_prompt = build_system_prompt_indexed(Some("baca halaman web ini"));

        // Generic prompt should not dump full web DOM summary or buttons
        assert!(!generic_prompt.contains("ACTIVE BROWSER PAGE CONTEXT (GROUND OF TRUTH):"));
        assert!(!generic_prompt.contains("INSTALLED SYSTEM APPLICATIONS (GROUND OF TRUTH):"));

        // Both prompts must contain base skills and format rules
        assert!(generic_prompt.contains("ACTION TAG FORMAT"));
        assert!(web_prompt.contains("ACTION TAG FORMAT"));
    }
}
