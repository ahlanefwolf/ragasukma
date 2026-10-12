/// Google AI Mode / AI Overview tab/button label keywords across all 31 supported languages
static GOOGLE_AI_KEYWORDS: &[&str] = &[
    "ai mode",          // English / Universal
    "ai overview",      // English / Universal
    "mode ai",          // Indonesian / French / Spanish / Italian
    "ringkasan ai",     // Indonesian
    "ai による概要",    // Japanese
    "ai モード",        // Japanese
    "ai 개요",          // Korean
    "ai 모드",          // Korean
    "نظرة عامة بالذكاء الاصطناعي", // Arabic
    "وضع الذكاء الاصطناعي",        // Arabic
    "преглед с ai",     // Bulgarian
    "přehled s ai",     // Czech
    "ai-oversigt",      // Danish
    "ai-modus",         // Danish / German / Dutch
    "ai-übersicht",     // German
    "επισκόπηση ai",    // Greek
    "vista general creada por ia", // Spanish
    "modo ia",          // Spanish / Portuguese
    "visión general de ia",        // Spanish
    "ai-ülevaade",      // Estonian
    "ai-yleiskatsaus",  // Finnish
    "aperçu généré par l'ia",      // French
    "mode ia",          // French
    "ai अवलोकन",        // Hindi
    "ai मोड",           // Hindi
    "ai pregled",       // Croatian
    "ai-áttekintés",    // Hungarian
    "panoramica dell'ia",          // Italian
    "„ai“ apžvalga",    // Lithuanian
    "ai kopsavilkums",  // Latvian
    "ai-overzicht",     // Dutch
    "przegląd od ai",   // Polish
    "tryb ai",          // Polish
    "visão geral criada por ia",   // Portuguese
    "privire generală generată de ia", // Romanian
    "обзор с помощью ии",          // Russian
    "режим ии",         // Russian
    "prehľad od ai",    // Slovak
    "pregled z ai",     // Slovenian
    "ai-översikt",      // Swedish
    "yapay zeka genel bakışı",     // Turkish
    "yapay zeka modu",  // Turkish
    "огляд від ai",     // Ukrainian
    "tổng quan về ai",  // Vietnamese
    "chế độ ai",        // Vietnamese
];



/// Execute browser commands based on canonical action
pub fn execute_browser_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "open" => {
            let val = payload.trim();
            if val.is_empty() {
                return None;
            }
            let final_url = if val.starts_with("http://") || val.starts_with("https://") {
                val.to_string()
            } else if val.contains('.') {
                format!("https://{val}")
            } else {
                format!("https://www.{val}.com")
            };
            crate::services::browser_session_service::smart_open_or_switch_tab(
                &final_url,
                Some(val),
                None,
            );
            Some((format!("Open Tab: {val}"), format!("Opening tab {val}")))
        }

        "search" => {
            let query = if let Some((_site, q)) = payload.split_once('|') {
                q.trim()
            } else {
                payload.trim()
            };

            if query.is_empty() {
                return None;
            }

            // Encode query properly for standard Google Search URL
            let encoded_query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
            let google_url = format!("https://www.google.com/search?q={encoded_query}");

            crate::term_log!("[Browser AI Search] Initiating search for: '{}'", query);

            // Reset any previous search AI summary in memory to ensure fresh results
            crate::services::browser_session_service::reset_copied_text();

            crate::services::browser_session_service::smart_open_or_switch_tab(
                &google_url,
                Some(query),
                Some("google.com"),
            );

            let query_str = query.to_string();
            // Reactive event-driven worker: clicks AI Mode, waits for AI completion, copies & updates active context
            std::thread::spawn(move || {
                use std::time::Duration;

                // Step 1: Wait for snapshot containing Mode AI button or already in Mode AI
                let mode_ai_snap = crate::services::browser_session_service::wait_for_snapshot_matching(
                    Duration::from_secs(6),
                    |snap| {
                        let snap_url = snap.url.to_lowercase();
                        if !snap_url.contains("google.") {
                            return false;
                        }
                        if snap_url.contains("udm=50") {
                            return true;
                        }
                        let has_button = snap.buttons.iter().any(|b| {
                            let label = b.text.as_deref().unwrap_or("").to_lowercase();
                            let aria = b.aria_label.as_deref().unwrap_or("").to_lowercase();
                            GOOGLE_AI_KEYWORDS.iter().any(|k| label.contains(k) || aria.contains(k))
                        });
                        let has_text = snap.texts.iter().any(|t| {
                            let label = t.to_lowercase();
                            GOOGLE_AI_KEYWORDS.iter().any(|k| label.contains(k))
                        });
                        has_button || has_text
                    },
                );

                let Some(snap) = mode_ai_snap else {
                    crate::term_log!("[Browser AI Search] Mode AI button not found within timeout (normal search layout).");
                    return;
                };

                let snap_url = snap.url.to_lowercase();
                if snap_url.contains("udm=50") {
                    crate::term_log!("[Browser AI Search] Already in Mode AI tab (udm=50).");
                } else {
                    let matching_button = snap.buttons.iter().find(|b| {
                        let label = b.text.as_deref().unwrap_or("").to_lowercase();
                        let aria = b.aria_label.as_deref().unwrap_or("").to_lowercase();
                        GOOGLE_AI_KEYWORDS.iter().any(|k| label.contains(k) || aria.contains(k))
                    });

                    let matching_text = snap.texts.iter().find(|t| {
                        let label = t.to_lowercase();
                        GOOGLE_AI_KEYWORDS.iter().any(|k| label.contains(k))
                    });

                    if let Some(btn) = matching_button {
                        let sel = btn
                            .selector
                            .clone()
                            .or_else(|| btn.text.clone())
                            .unwrap_or_else(|| "Mode AI".to_string());
                        crate::term_log!("[Browser AI Search] Clicking Mode AI button with selector: '{}'", sel);
                        crate::services::browser_session_service::send_browser_command(
                            crate::services::browser_session_service::BrowserCommand {
                                action: "CLICK_BUTTON".to_string(),
                                selector: Some(sel),
                                ..Default::default()
                            },
                        );
                    } else if let Some(txt) = matching_text {
                        crate::term_log!("[Browser AI Search] Clicking Mode AI text link: '{}'", txt);
                        crate::services::browser_session_service::send_browser_command(
                            crate::services::browser_session_service::BrowserCommand {
                                action: "CLICK_TEXT".to_string(),
                                selector: Some(txt.clone()),
                                ..Default::default()
                            },
                        );
                    }

                    // Explicitly wait until the browser enters Mode AI (URL contains udm=50)
                    crate::services::browser_session_service::wait_for_snapshot_matching(
                        Duration::from_secs(8),
                        |s| s.url.to_lowercase().contains("udm=50"),
                    );
                }

                // Step 2: Reactively wait for Mode AI page to finish loading and extract the AI Overview text
                let captured_text = match crate::services::browser_session_service::wait_for_copied_text(Duration::from_secs(12)) {
                    Some(text) if !text.trim().is_empty() => Some(text),
                    _ => {
                        // Fallback: wait for snapshot that has populated text_summary from the AI container
                        let snap = crate::services::browser_session_service::wait_for_snapshot_matching(
                            Duration::from_secs(6),
                            |s| {
                                let u = s.url.to_lowercase();
                                (u.contains("udm=50") || u.contains("google.")) && !s.is_loading && !s.text_summary.trim().is_empty()
                            },
                        );
                        snap.and_then(|s| {
                            let trimmed = s.text_summary.trim().to_string();
                            if !trimmed.is_empty() {
                                Some(trimmed)
                            } else {
                                None
                            }
                        })
                    }
                };

                if let Some(text) = captured_text {
                    let trimmed = text.trim();
                    crate::term_log!(
                        "[Browser AI Mode] Captured fresh AI summary ({} characters). Updating active page context...",
                        trimmed.len()
                    );

                    // Update active browser snapshot with the rich copied markdown text
                    if let Some(mut snap) = crate::services::browser_session_service::get_latest_snapshot() {
                        snap.text_summary = trimmed.to_string();
                        let _ = crate::services::browser_session_service::save_browser_snapshot(snap);
                    }

                    // Trigger natural summarization through the main chat loop by providing the extracted content directly
                    crate::engine::remote_text::inject_remote_text(format!(
                        "Here are the search results for '{query_str}':\n\n{trimmed}\n\nPlease provide a clear and concise spoken summary of this information directly to the user. Do NOT call any browser search or open commands."
                    ));
                } else {
                    crate::term_log!("[Browser AI Search] AI summary text was not captured within timeout.");
                }
            });

            Some((
                format!("Google Search: {query}"),
                format!("Searching '{query}' on Google"),
            ))
        }

        "click_text" => {
            let sel = payload.trim();
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "CLICK_TEXT".to_string(),
                    selector: Some(sel.to_string()),
                    ..Default::default()
                },
            );
            Some((
                format!("Click Text: {sel}"),
                format!("Clicking text '{sel}' in browser"),
            ))
        }

        "click_button" => {
            let sel = payload.trim();
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "CLICK_BUTTON".to_string(),
                    selector: Some(sel.to_string()),
                    ..Default::default()
                },
            );
            Some((
                format!("Click Button: {sel}"),
                format!("Clicking button '{sel}' in browser"),
            ))
        }

        "type" => {
            let text = payload.trim();
            if text.is_empty() {
                return None;
            }

            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "TYPE_TEXT".to_string(),
                    text_to_type: Some(text.to_string()),
                    press_enter: Some(false),
                    ..Default::default()
                },
            );
            Some((
                format!("Type Text: {text}"),
                format!("Typing '{text}' into web field"),
            ))
        }

        "enter" => {
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "PRESS_ENTER".to_string(),
                    press_enter: Some(true),
                    ..Default::default()
                },
            );
            Some((
                "Press Enter".to_string(),
                "Pressing Enter key on web page".to_string(),
            ))
        }

        "scroll" => {
            let amount = payload.trim().parse::<i32>().unwrap_or(400);
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "SCROLL".to_string(),
                    scroll_y: Some(amount),
                    ..Default::default()
                },
            );
            Some((
                "Scroll Web Page".to_string(),
                format!("Scrolling page by {amount}px"),
            ))
        }

        "switch" => {
            let q = payload.trim();
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "SWITCH_TAB".to_string(),
                    query: Some(q.to_string()),
                    ..Default::default()
                },
            );
            Some((
                format!("Switch Tab: {q}"),
                format!("Switching to tab '{q}'"),
            ))
        }

        "close" => {
            let q = payload.trim();
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "CLOSE_TAB".to_string(),
                    query: Some(q.to_string()),
                    ..Default::default()
                },
            );
            Some((format!("Close Tab: {q}"), format!("Closing tab '{q}'")))
        }

        "install_ext" | "open_ext_folder" => {
            let zip_file = crate::assets::find_file_in_candidates("ragasukma-companion.zip");
            let target_dir = if zip_file.exists() {
                zip_file
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(crate::assets::get_assets_dir)
            } else {
                crate::assets::get_assets_dir()
            };
            let abs_dir = target_dir.canonicalize().unwrap_or(target_dir);

            #[cfg(target_os = "windows")]
            let abs_str = abs_dir.to_string_lossy().trim_start_matches(r"\\?\").to_string();
            #[cfg(not(target_os = "windows"))]
            let abs_str = abs_dir.to_string_lossy().to_string();

            #[cfg(target_os = "windows")]
            {
                use windows::core::HSTRING;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

                let h_op = HSTRING::from("open");
                let h_dir = HSTRING::from(&abs_str);

                unsafe {
                    let _ = ShellExecuteW(
                        None,
                        windows::core::PCWSTR(h_op.as_wide().as_ptr()),
                        windows::core::PCWSTR(h_dir.as_wide().as_ptr()),
                        None,
                        None,
                        SW_SHOWNORMAL,
                    );
                }

                if action == "install_ext" {
                    crate::assets::open_browser("chrome://extensions");
                }
            }

            #[cfg(target_os = "macos")]
            {
                let mut cmd = std::process::Command::new("open");
                cmd.arg(&abs_str);
                crate::engine::skills::configure_subprocess(&mut cmd);
                let _ = cmd.spawn();

                if action == "install_ext" {
                    let mut b_cmd = std::process::Command::new("open");
                    b_cmd.arg("chrome://extensions");
                    crate::engine::skills::configure_subprocess(&mut b_cmd);
                    let _ = b_cmd.spawn();
                }
            }

            #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
            {
                let mut cmd = std::process::Command::new("xdg-open");
                cmd.arg(&abs_str);
                crate::engine::skills::configure_subprocess(&mut cmd);
                let _ = cmd.spawn();
            }

            Some((
                "Open Extension Folder".to_string(),
                format!("Opening extension folder ({abs_str})"),
            ))
        }

        _ => None,
    }
}
