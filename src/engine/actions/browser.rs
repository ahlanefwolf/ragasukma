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

            crate::services::browser_session_service::smart_open_or_switch_tab(
                &google_url,
                Some(query),
                Some("google.com"),
            );

            // Reactive fast-polling worker: clicks AI Mode the instant the element appears in DOM snapshot
            std::thread::spawn(|| {
                use std::time::{Duration, Instant};

                let start = Instant::now();
                let max_timeout = Duration::from_secs(4);

                while start.elapsed() < max_timeout {
                    if let Some(snap) = crate::services::browser_session_service::get_latest_snapshot() {
                        let snap_url = snap.url.to_lowercase();
                        // Verify we are on Google search results page
                        if snap_url.contains("google.") && snap_url.contains("search") {
                            // 1. Search in buttons
                            let matching_button = snap.buttons.iter().find(|b| {
                                let label = b.text.as_deref().unwrap_or("").to_lowercase();
                                let aria = b.aria_label.as_deref().unwrap_or("").to_lowercase();
                                GOOGLE_AI_KEYWORDS.iter().any(|k| label.contains(k) || aria.contains(k))
                            });

                            // 2. Search in interactive texts / links
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
                                crate::services::browser_session_service::send_browser_command(
                                    crate::services::browser_session_service::BrowserCommand {
                                        action: "CLICK_BUTTON".to_string(),
                                        selector: Some(sel),
                                        ..Default::default()
                                    },
                                );
                                return;
                            } else if let Some(txt) = matching_text {
                                crate::services::browser_session_service::send_browser_command(
                                    crate::services::browser_session_service::BrowserCommand {
                                        action: "CLICK_TEXT".to_string(),
                                        selector: Some(txt.clone()),
                                        ..Default::default()
                                    },
                                );
                                return;
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_millis(50));
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
