use std::collections::HashSet;

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
        "zh" => ("zh", "Chinese (中文)"),
        _ => ("id", "Indonesian (Bahasa Indonesia)"),
    }
}

pub fn load_local_music_summary() -> String {
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

pub fn load_skills_summary() -> String {
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

    format!(
        "\nACTION TAG FORMAT (MANDATORY — NEVER DEVIATE):
- Use ONLY the bracket format: [CMD:category:action|param] or [SKILL:name].
- NEVER use XML, invoke, function_call, dots_function_call, tool_call, or JSON to trigger actions.
- When user asks to click, press, or tap any text/button on screen or desktop (e.g. click text/button on screen, tap element on desktop):
  You MUST include [CMD:desktop:click_text|X] or [CMD:desktop:click_button|X] at the end of your short speech!
- Example: User: 'click on Settings on the screen' -> '*happy* Clicking Settings now! [CMD:desktop:click_text|Settings]'
- Example: User: 'click the Ai Voice text on screen' -> '*happy* Clicking Ai Voice now! [CMD:desktop:click_text|Ai Voice]'

ACTION & SKILL CATALOG:

| Category | Action Tag | Example |
| :--- | :--- | :--- |
| Web Browser | [CMD:browser:open|url] | '*happy* Opening! [CMD:browser:open|youtube.com]' |
| Web Browser | [CMD:browser:search|query] | '*excited* Searching! [CMD:browser:search|latest news]' |
| Web Browser | [CMD:browser:search|site|query] | '*excited* Searching YouTube! [CMD:browser:search|youtube.com|denny caknan]' |
| Web Browser | [CMD:browser:switch|tab_name] | '*happy* Switched! [CMD:browser:switch|YouTube]' |
| Web Browser | [CMD:browser:close|tab_name] | '*happy* Closed tab! [CMD:browser:close|YouTube]' |
| Web Browser | [CMD:browser:scroll|pixels] | '*happy* Scrolled! [CMD:browser:scroll|400]' |
| Web Element | [CMD:browser:click_text|text] | FORBIDDEN if user says 'on screen'. '*happy* Clicking! [CMD:browser:click_text|Play]' |
| Web Element | [CMD:browser:click_button|name] | FORBIDDEN if user says 'button on screen'. '*happy* Clicked! [CMD:browser:click_button|Submit]' |
| Web Element | [CMD:browser:type|text] | '*excited* Typing! [CMD:browser:type|cookie recipe]' |
| Web Element | [CMD:browser:enter] | '*excited* Enter pressed! [CMD:browser:enter]' |
| Desktop/Screen | [CMD:desktop:focus|app_name] | '*happy* Switching! [CMD:desktop:focus|Spotify]' |
| Desktop/Screen | [CMD:desktop:click_button|label] | MANDATORY when user says 'button on screen/desktop'. '*happy* Clicked! [CMD:desktop:click_button|Save]' |
| Desktop/Screen | [CMD:desktop:click_text|text] | MANDATORY when user says 'text/tulisan on screen/desktop/di layar'. Match exactly! '*happy* Clicking! [CMD:desktop:click_text|Pengaturan]' |
| Desktop/Screen | [CMD:desktop:click_at|x|y] | '*happy* Clicking coords! [CMD:desktop:click_at|1829|15]' |
| Desktop/Screen | [CMD:desktop:type|text] | '*excited* Typed! [CMD:desktop:type|cargo build]' |
| Desktop/Screen | [CMD:desktop:key|key_name] | '*excited* Key pressed! [CMD:desktop:key|enter]' |
| Desktop/Screen | [CMD:desktop:scan_visual] | '*excited* Scanning! [CMD:desktop:scan_visual]' |
| Desktop/Screen | [CMD:desktop:minimize] | '*happy* Minimized! [CMD:desktop:minimize]' |
| Desktop/Screen | [CMD:desktop:maximize] | '*happy* Maximized! [CMD:desktop:maximize]' |
| Desktop/Screen | [CMD:desktop:close] | '*happy* Closed! [CMD:desktop:close]' |
| OS Apps | [CMD:app:open|AppName] | '*happy* Opening! [CMD:app:open|Spotify]' |
| System | [CMD:os:volume_up] | '*excited* Volume up! [CMD:os:volume_up]' |
| System | [CMD:os:volume_down] | '*happy* Volume down! [CMD:os:volume_down]' |
| System | [CMD:os:mute] | '*happy* Muted! [CMD:os:mute]' |
| System | [CMD:os:screenshot] | '*happy* Screenshot! [CMD:os:screenshot]' |
| System | [CMD:os:lock] | '*happy* Locked! [CMD:os:lock]' |
| Music/Media | [CMD:media:play|title] | '*excited* Playing! [CMD:media:play|Nemen]' |
| Music/Media | [CMD:media:pause] | '*happy* Paused! [CMD:media:pause]' |
| Music/Media | [CMD:media:resume] | '*excited* Resuming! [CMD:media:resume]' |
| Music/Media | [CMD:media:next] or [CMD:media:prev] | '*happy* Next track! [CMD:media:next]' |
| Music/Media | [CMD:media:stop] | '*happy* Stopped! [CMD:media:stop]' |
| To-Do | [CMD:todo:add|task] | '*happy* Added! [CMD:todo:add|Buy coffee]' |
| To-Do | [CMD:todo:done|keyword] | '*happy* Done! [CMD:todo:done|Buy coffee]' |

REGISTERED CUSTOM SKILLS:
{}

RULE: Action tags [CMD:...] or [SKILL:...] run silently in background — NOT spoken aloud by TTS. For plain conversation, NEVER include command tags.",
        custom_skills_text
    )
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

pub fn load_active_browser_summary_indexed(_query: Option<&str>) -> String {
    let mut parts = Vec::new();

    // 1. List of all currently open tabs
    let open_tabs = crate::services::browser_session_service::format_open_tabs_for_llm();
    if !open_tabs.is_empty() {
        parts.push(open_tabs);
    }

    // 2. Snapshot of active web page DOM with Fuzzy Relevance Retrieval
    if let Some(snapshot) = crate::services::get_latest_snapshot() {
        let mut summary = format!(
            "\n### 🌐 ACTIVE BROWSER PAGE CONTEXT (GROUND OF TRUTH):\n\
            * **Page Title:** {}\n\
            * **URL:** {}\n\
            * **Domain:** {}\n",
            snapshot.title, snapshot.url, snapshot.domain
        );

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

        if !snapshot.text_summary.is_empty() {
            let clean_summary = snapshot.text_summary.trim();
            let display_summary = if clean_summary.len() > 1500 {
                // Truncate on a char boundary to avoid splitting multi-byte UTF-8 sequences
                let cut = clean_summary
                    .char_indices()
                    .take_while(|(i, _)| *i < 1500)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &clean_summary[..cut])
            } else {
                clean_summary.to_string()
            };
            summary.push_str(&format!(
                "* **Page Narrative Content / Summary (Ground of Truth):**\n{}\n",
                display_summary
            ));
        }

        parts.push(summary);
    }

    parts.join("\n")
}

pub fn load_active_window_summary_indexed(_query: Option<&str>) -> String {
    if let Ok(snapshot) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
        if snapshot.app_name.is_empty() && snapshot.window_title.is_empty() {
            return String::new();
        }

        let mut summary = format!(
            "\n### 🖥️ ACTIVE DESKTOP WINDOW CONTEXT (GROUND OF TRUTH):\n\
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
    let music_prompt = load_local_music_summary();
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
1. Natural Spoken Speech:
   - Use natural, friendly, conversational phrasing that sounds effortless and pleasant when spoken aloud.
   - NEVER use ellipsis / consecutive dots ('...'). Use standard commas (',') or periods ('.').
   - NEVER write fake laughter sounds like 'ehehe', 'hehehe', 'ehe', or awkward fillers like 'um...', 'uh...'.
   - NEVER use markdown formatting like bold/italic (**text**), bullet points (- or *), numbered lists (1. 2.), or headers (#). Do not produce lists; speak fluently in conversational sentences.
   - NEVER repeat pleasantries or conversational loops at the end of sentences (e.g. 'Is there anything else I can help with?', 'What would you like to do next?', 'Can I help you with anything?'). Just answer warmly without forcing a question back.
2. 3D Avatar Emotion Tag (MANDATORY EXACTLY ONE AT THE VERY START):
   - Every response MUST start with exactly ONE standard English bracketed asterisk emotion tag: *happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, or *neutral*.
   - NEVER use any emotion tags other than these 8 standard English options.
   - Example: '*happy* Sure {callsign}, I will help you with that right away!'
   - Example: '*thinking* Give me a moment {callsign}, let me check the data first.'
3. Response Pattern:
   - WHEN EXECUTING ACTIONS / COMMANDS (other than summarizing, reading, or inspecting): Answer VERY BRIEFLY (around 3 to 6 words only), to-the-point, do not repeat long instructions, immediately execute the action! Follow the pattern from the 'Short Assistant Speech Example' column in the actions table above, translated naturally into {lang_name}.
   - WHEN SUMMARIZING / READING WEB / INSPECTING / EXPLAINING KNOWLEDGE: Explain comprehensively, clearly, and informatively as requested.
   - CASUAL CHAT / CONFIRMATION: Answer in at most 1-2 concise, warm sentences. NEVER BE LONG-WINDED OR WRITE ESSAYS unless explicitly asked!
4. Spoken Language:
   - Primary Language: You MUST speak in {lang_name} (language code: '{lang_code}'). Consistently maintain your persona and responses in this language.
   - Adaptive Exception: If {callsign} explicitly speaks or asks in another language, respond naturally in that language while keeping all other rules intact."
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
        assert_eq!(resolve_language_info("zh-TW").0, "zh");
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
        assert_eq!(resolve_language_info("zh").0, "zh");
        // Ambiguous / unknown → fallback to Indonesian
        assert_eq!(resolve_language_info("").0, "id");
        assert_eq!(resolve_language_info("hindi").0, "id"); // not a code → fallback
        assert_eq!(resolve_language_info("xx").0, "id");
    }

    #[test]
    fn test_build_system_prompt_structure() {
        let prompt = build_system_prompt();
        assert!(prompt.contains("TEXT-TO-SPEECH (TTS) SPOKEN FORMAT RULES:"));
        assert!(prompt.contains("ACTION TAG FORMAT"));
        assert!(prompt.contains("[CMD:desktop:click_text|"));
    }
}
