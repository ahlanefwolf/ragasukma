#[cfg(target_os = "linux")]
use super::{current_epoch_ms, ActiveWindowSnapshot, DesktopElement, WindowItem};

#[cfg(target_os = "linux")]
static ATSPI_CONN: std::sync::Mutex<Option<atspi::connection::AccessibilityConnection>> =
    std::sync::Mutex::new(None);

#[cfg(target_os = "linux")]
async fn get_or_init_atspi_conn() -> Option<atspi::connection::AccessibilityConnection> {
    if let Ok(lock) = ATSPI_CONN.lock() {
        if let Some(ref conn) = *lock {
            return Some(conn.clone());
        }
    }

    match atspi::connection::AccessibilityConnection::new().await {
        Ok(conn) => {
            if let Ok(mut lock) = ATSPI_CONN.lock() {
                *lock = Some(conn.clone());
            }
            Some(conn)
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            eprintln!("⚠️  [Desktop Scanner] AT-SPI D-Bus init error: {_e}");
            None
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct CompositorWindow {
    pub id: i64,
    pub app: String,
    pub title: String,
    pub focused: bool,
}

#[cfg(target_os = "linux")]
fn get_sway_windows() -> Option<Vec<CompositorWindow>> {
    if std::env::var("SWAYSOCK").is_err() {
        return None;
    }

    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let val = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;

    fn collect_nodes(node: &serde_json::Value, out: &mut Vec<CompositorWindow>) {
        let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let app = node
            .get("app_id")
            .and_then(|a| a.as_str())
            .or_else(|| {
                node.get("window_properties")
                    .and_then(|wp| wp.get("class"))
                    .and_then(|c| c.as_str())
            })
            .unwrap_or("");

        if !name.is_empty() && !app.is_empty() && !app.eq_ignore_ascii_case("waybar") {
            let id = node.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
            let focused = node.get("focused").and_then(|f| f.as_bool()).unwrap_or(false);
            out.push(CompositorWindow {
                id,
                app: app.to_string(),
                title: name.to_string(),
                focused,
            });
        }

        let empty_vec = Vec::new();
        let nodes = node.get("nodes").and_then(|n| n.as_array()).unwrap_or(&empty_vec);
        for child in nodes {
            collect_nodes(child, out);
        }

        let floating = node.get("floating_nodes").and_then(|n| n.as_array()).unwrap_or(&empty_vec);
        for child in floating {
            collect_nodes(child, out);
        }
    }

    let mut wins = Vec::new();
    collect_nodes(&val, &mut wins);
    if wins.is_empty() {
        None
    } else {
        Some(wins)
    }
}

#[cfg(target_os = "linux")]
pub fn get_focused_window_quick_info_linux() -> Option<(String, String)> {
    if std::env::var("SWAYSOCK").is_ok() {
        if let Ok(output) = std::process::Command::new("swaymsg")
            .args(["-t", "get_tree"])
            .output()
        {
            if output.status.success() {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    fn find_focused(node: &serde_json::Value) -> Option<(String, String)> {
                        if node.get("focused").and_then(|f| f.as_bool()).unwrap_or(false) {
                            let title = node.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                            let app = node
                                .get("app_id")
                                .and_then(|a| a.as_str())
                                .or_else(|| {
                                    node.get("window_properties")
                                        .and_then(|wp| wp.get("class"))
                                        .and_then(|c| c.as_str())
                                })
                                .unwrap_or("")
                                .to_string();
                            return Some((app, title));
                        }
                        if let Some(nodes) = node.get("nodes").and_then(|n| n.as_array()) {
                            for child in nodes {
                                if let Some(res) = find_focused(child) {
                                    return Some(res);
                                }
                            }
                        }
                        if let Some(floating) = node.get("floating_nodes").and_then(|n| n.as_array()) {
                            for child in floating {
                                if let Some(res) = find_focused(child) {
                                    return Some(res);
                                }
                            }
                        }
                        None
                    }
                    if let Some(res) = find_focused(&val) {
                        return Some(res);
                    }
                }
            }
        }
    }

    // Fallback: X11 xdotool
    if let Ok(output) = std::process::Command::new("xdotool")
        .args(["getwindowfocus", "getwindowname"])
        .output()
    {
        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !title.is_empty() {
                return Some((String::new(), title));
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub async fn scan_active_window_linux() -> Result<ActiveWindowSnapshot, String> {
    use atspi::proxy::accessible::ObjectRefExt;
    use atspi::{Role, State};

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        ..Default::default()
    };

    let sway_windows = get_sway_windows();

    let mut all_windows: Vec<WindowItem> = Vec::new();
    let mut window_counter = 1;

    let mut focused_app = String::new();
    let mut focused_title = String::new();
    let mut focused_win_id = String::new();

    if let Some(ref wins) = sway_windows {
        for w in wins {
            all_windows.push(WindowItem {
                id: window_counter,
                app_name: w.app.clone(),
                title: w.title.clone(),
                active: w.focused,
                window_id: format!("sway:{}", w.id),
            });
            window_counter += 1;

            if w.focused {
                focused_app = w.app.clone();
                focused_title = w.title.clone();
                focused_win_id = format!("sway:{}", w.id);
            }
        }
    }

    if !focused_app.is_empty() {
        snapshot.app_name = focused_app.clone();
        snapshot.window_title = focused_title.clone();
        snapshot.window_id = focused_win_id.clone();
    }

    if let Some(conn) = get_or_init_atspi_conn().await {
        let zbus_conn = conn.connection();
        if let Ok(desktop) = conn.root_accessible_on_registry().await {
            if let Ok(children) = desktop.get_children().await {
                let mut target_win_ref: Option<atspi::ObjectRefOwned> = None;

                for app_ref in children {
                    let Ok(app_proxy) = app_ref.as_accessible_proxy(zbus_conn).await else {
                        continue;
                    };

                    let app_name = app_proxy.name().await.unwrap_or_default();
                    if app_name.is_empty() {
                        continue;
                    }

                    if let Ok(windows) = app_proxy.get_children().await {
                        for win_ref in windows {
                            let Ok(win_proxy) = win_ref.as_accessible_proxy(zbus_conn).await else {
                                continue;
                            };

                            let role = win_proxy.get_role().await.unwrap_or(Role::Invalid);
                            let win_name = win_proxy.name().await.unwrap_or_default();
                            let state = win_proxy.get_state().await.unwrap_or_default();
                            let is_active = state.contains(State::Active) || state.contains(State::Focused);

                            if !matches!(role, Role::Window | Role::Frame | Role::Dialog | Role::Application)
                                || win_name.is_empty()
                            {
                                continue;
                            }

                            if sway_windows.is_none() {
                                let is_panel = app_name.eq_ignore_ascii_case("waybar")
                                    || app_name.eq_ignore_ascii_case("desktop");
                                if !is_panel {
                                    all_windows.push(WindowItem {
                                        id: window_counter,
                                        app_name: app_name.clone(),
                                        title: win_name.clone(),
                                        active: is_active,
                                        window_id: win_ref.path_as_str().to_string(),
                                    });
                                    window_counter += 1;

                                    if is_active && snapshot.app_name.is_empty() {
                                        snapshot.app_name = app_name.clone();
                                        snapshot.window_title = win_name.clone();
                                        snapshot.window_id = win_ref.path_as_str().to_string();
                                        target_win_ref = Some(win_ref.clone());
                                    }
                                }
                            } else {
                                let app_lower = app_name.to_lowercase();
                                let f_app_lower = focused_app.to_lowercase();
                                let app_matches = app_lower.contains(&f_app_lower)
                                    || f_app_lower.contains(&app_lower);
                                let title_matches = !focused_title.is_empty()
                                    && (win_name.contains(&focused_title)
                                        || focused_title.contains(&win_name));

                                if app_matches || title_matches {
                                    target_win_ref = Some(win_ref.clone());
                                    break;
                                }
                            }
                        }
                    }

                    if target_win_ref.is_some() {
                        break;
                    }
                }

                if let Some(win_ref) = target_win_ref {
                    if let Ok(win_proxy) = win_ref.as_accessible_proxy(zbus_conn).await {
                        let mut element_id_counter = 1;
                        let mut queue: Vec<(atspi::ObjectRefOwned, usize)> = Vec::new();
                        if let Ok(win_children) = win_proxy.get_children().await {
                            for c in win_children {
                                queue.push((c, 1));
                            }
                        }

                        while let Some((node_ref, depth)) = queue.pop() {
                            let Ok(node_proxy) = node_ref.as_accessible_proxy(zbus_conn).await else {
                                continue;
                            };

                            let role = node_proxy.get_role().await.unwrap_or(Role::Invalid);
                            let name = node_proxy.name().await.unwrap_or_default().trim().to_string();

                            if !name.is_empty() {
                                match role {
                                    Role::Button
                                    | Role::ToggleButton
                                    | Role::RadioButton
                                    | Role::Menu
                                    | Role::MenuItem
                                    | Role::CheckMenuItem
                                    | Role::RadioMenuItem
                                    | Role::PageTab
                                    | Role::ListItem
                                    | Role::TableCell => {
                                        snapshot.buttons.push(DesktopElement {
                                            id: element_id_counter,
                                            label: name.clone(),
                                            role: format!("{:?}", role).to_lowercase(),
                                            ..Default::default()
                                        });
                                        element_id_counter += 1;
                                    }
                                    Role::Text | Role::Entry | Role::PasswordText => {
                                        snapshot.inputs.push(DesktopElement {
                                            id: element_id_counter,
                                            label: name.clone(),
                                            role: "input".to_string(),
                                            ..Default::default()
                                        });
                                        element_id_counter += 1;
                                    }
                                    Role::Label | Role::Static | Role::Heading
                                        if !snapshot.texts.contains(&name) =>
                                    {
                                        snapshot.texts.push(name.clone());
                                    }
                                    _ => {}
                                }
                            }

                            if depth < 6 {
                                if let Ok(children) = node_proxy.get_children().await {
                                    for child in children {
                                        queue.push((child, depth + 1));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    snapshot.windows = all_windows;
    if snapshot.app_name.is_empty() {
        snapshot.text_summary = "Tidak ada jendela aplikasi yang terfokus".to_string();
    }

    Ok(snapshot)
}
