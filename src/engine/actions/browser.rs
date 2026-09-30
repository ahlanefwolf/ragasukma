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
            let parts: Vec<&str> = payload.split('|').collect();
            let (site, query) = if parts.len() >= 2 {
                (parts[0].trim(), parts[1].trim())
            } else {
                ("", payload.trim())
            };

            if !site.is_empty() {
                let domain = if site.contains('.') {
                    site.to_string()
                } else {
                    format!("{site}.com")
                };
                let site_url = if site.starts_with("http://") || site.starts_with("https://") {
                    site.to_string()
                } else {
                    format!("https://www.{domain}")
                };

                let tabs = crate::services::browser_session_service::get_browser_tabs();
                let matching_tab = tabs.iter().find(|t| {
                    let t_url = t.url.to_lowercase();
                    let t_title = t.title.to_lowercase();
                    t_url.contains(&site.to_lowercase()) || t_title.contains(&site.to_lowercase())
                });

                if let Some(tab) = matching_tab {
                    println!(
                        "🔄 [Universal In-Page Search] Switching to active tab '{}' (ID: {}) and searching '{}'",
                        tab.title, tab.id, query
                    );
                    crate::services::browser_session_service::send_browser_command(
                        crate::services::browser_session_service::BrowserCommand {
                            action: "SWITCH_OR_NAVIGATE".to_string(),
                            query: Some(query.to_string()),
                            domain: Some(domain),
                            ..Default::default()
                        },
                    );
                } else {
                    crate::services::browser_session_service::smart_open_or_switch_tab(
                        &site_url,
                        Some(query),
                        Some(&domain),
                    );
                }

                Some((
                    format!("Search on {site}: {query}"),
                    format!("Searching '{query}' on {site}"),
                ))
            } else {
                crate::services::browser_session_service::send_browser_command(
                    crate::services::browser_session_service::BrowserCommand {
                        action: "IN_PAGE_SEARCH".to_string(),
                        query: Some(query.to_string()),
                        ..Default::default()
                    },
                );
                Some((
                    format!("Search on Web Page: {query}"),
                    format!("Searching '{query}' on active page"),
                ))
            }
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
            let assets_dir = crate::assets::get_assets_dir();
            let abs_str = assets_dir.to_string_lossy().to_string();

            #[cfg(target_os = "windows")]
            {
                let mut cmd = std::process::Command::new("explorer");
                cmd.arg(&abs_str);
                crate::engine::skills::configure_subprocess(&mut cmd);
                let _ = cmd.spawn();

                if action == "install_ext" {
                    let mut b_cmd = std::process::Command::new("cmd");
                    b_cmd.args(["/C", "start", "chrome://extensions"]);
                    crate::engine::skills::configure_subprocess(&mut b_cmd);
                    let _ = b_cmd.spawn();
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
