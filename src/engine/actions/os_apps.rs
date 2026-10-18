use super::catalog::get_builtin_system_skills;
use crate::engine::app_scanner::get_installed_applications;
use crate::engine::skills::execute_command_by_os;

/// Combined handler for OS system control and application launching
pub fn execute_os_or_app_action(
    namespace: &str,
    action: &str,
    payload: &str,
) -> Option<(String, String)> {
    let act = action.trim().to_lowercase();
    let query = payload.trim();

    // 1. Pure OS utility action
    match act.as_str() {
        "volume_up" => return execute_by_catalog_name("Volume Up"),
        "volume_down" => return execute_by_catalog_name("Volume Down"),
        "mute" => return execute_by_catalog_name("Mute Audio"),
        "lock" => return execute_by_catalog_name("Lock Screen"),
        "screenshot" => return execute_by_catalog_name("Screenshot"),
        _ => {}
    }

    // 2. All application requests (both payload and action) are routed to dynamic app scanner
    let target = if !query.is_empty() {
        query
    } else if namespace == "app" || namespace == "os" {
        action
    } else {
        ""
    };

    if !target.is_empty() {
        return execute_app_action(target);
    }

    None
}

/// Executes native OS system action by action name
pub fn execute_os_action(action: &str) -> Option<(String, String)> {
    execute_os_or_app_action("os", action, "")
}

fn execute_by_catalog_name(catalog_name: &str) -> Option<(String, String)> {
    for item in get_builtin_system_skills() {
        if item.name.eq_ignore_ascii_case(catalog_name) {
            match execute_command_by_os(&item.command) {
                Ok(_) => return Some((item.name, "Executed successfully".to_string())),
                Err(e) => return Some((item.name, format!("Error: {e}"))),
            }
        }
    }
    None
}

/// Application launcher for installed OS applications
/// Format: `app:open|<name>`
pub fn execute_app_action(payload: &str) -> Option<(String, String)> {
    let app_query = payload.trim();
    if app_query.is_empty() {
        return None;
    }

    let installed_apps = get_installed_applications();
    let target_app = find_target_app(app_query, &installed_apps);

    // Context-Aware App Switcher: If a matching window is already running, switch focus to it
    // Always scan fresh active windows so we don't rely on stale cached snapshots
    let snapshot = crate::engine::window_scanner::scan_active_window_sync()
        .ok()
        .or_else(crate::engine::actions::desktop::load_snapshot);

    if let Some(snap) = snapshot {
        let mut all_windows = snap.windows.clone();
        if all_windows.is_empty() && (!snap.app_name.is_empty() || !snap.window_title.is_empty()) {
            all_windows.push(crate::engine::window_scanner::WindowItem {
                id: 0,
                app_name: snap.app_name.clone(),
                title: snap.window_title.clone(),
                active: true,
                window_id: snap.window_id.clone(),
            });
        }

        if let Some(matching_win) = find_matching_window(app_query, target_app, &all_windows) {
            let display_name = target_app.map(|a| a.name.as_str()).unwrap_or(app_query);
            let _ = try_focus_window(matching_win);
            crate::term_log!(
                "[App Switcher] Application already running: switched focus to existing window '{}' (app: '{}')",
                matching_win.title,
                matching_win.app_name
            );
            return Some((
                format!("Focus {display_name}"),
                format!("Application '{display_name}' is already running; focused window '{}'", matching_win.title),
            ));
        }
    }

    // Helper closure: after launching an app for the first time, poll asynchronously
    // or wait briefly and focus the window as soon as it appears
    let focus_launched_app = |query: &str, app: Option<&crate::engine::app_scanner::InstalledApp>| {
        let q = query.to_string();
        let app_clone = app.cloned();
        std::thread::spawn(move || {
            // Poll for up to 3 seconds (30 * 100ms) for the window to appear
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                    if let Some(w) = find_matching_window(&q, app_clone.as_ref(), &snap.windows) {
                        let _ = try_focus_window(w);
                        let _ = snap.save_to_file();
                        break;
                    }
                }
            }
        });
    };

    // 1. Search application based on matching hierarchy (Exact -> Contains -> Aliases -> Fuzzy)
    if let Some(target_app) = target_app {
        if launch_app(target_app).is_ok() {
            focus_launched_app(app_query, Some(target_app));
            return Some((
                format!("Open {}", target_app.name),
                format!("Successfully opened application '{}'", target_app.name),
            ));
        }
    }

    // 2. Fallback: Directly execute binary name in system PATH if available
    let q_clean = app_query.split('(').next().unwrap_or(app_query).trim();

    #[cfg(target_os = "windows")]
    {
        if launch_windows_exec(q_clean).is_ok() {
            focus_launched_app(q_clean, None);
            return Some((
                format!("Open {q_clean}"),
                "Successfully launched application from system".to_string(),
            ));
        }
        let explorer_cmd = format!("explorer.exe \"{}\"", q_clean);
        if launch_windows_exec(&explorer_cmd).is_ok() {
            focus_launched_app(q_clean, None);
            return Some((
                format!("Open {q_clean}"),
                "Successfully launched application from system".to_string(),
            ));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mac_cmd = format!("open -a \"{q_clean}\" 2>/dev/null || ({q_clean} &)");
        let mut command = std::process::Command::new("sh");
        command.args(["-c", &mac_cmd]);
        crate::engine::skills::configure_subprocess(&mut command);
        if command.spawn().is_ok() {
            return Some((
                format!("Open {q_clean}"),
                "Successfully launched application from system".to_string(),
            ));
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Try multiple binary candidates: "g4-music", "g4music", or literal
        let cand_dash = q_clean.to_lowercase().replace(' ', "-");
        let cand_nospace = q_clean.to_lowercase().replace([' ', '-', '_'], "");
        let cand_literal = q_clean.to_lowercase();

        let mut checked = std::collections::HashSet::new();
        for binary_candidate in [cand_dash, cand_nospace, cand_literal] {
            if !binary_candidate.is_empty() && checked.insert(binary_candidate.clone()) {
                let cmd_test = format!("command -v {binary_candidate} >/dev/null 2>&1");
                if std::process::Command::new("sh")
                    .args(["-c", &cmd_test])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    if launch_linux_command(&binary_candidate).is_ok() {
                        return Some((
                            format!("Open {q_clean}"),
                            "Successfully launched application from system".to_string(),
                        ));
                    }
                }
            }
        }
    }

    None
}

fn normalize_alnum(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn extract_exec_binary(exec: &str) -> &str {
    let trimmed = exec.trim();
    let first = trimmed.split_whitespace().next().unwrap_or(trimmed);
    let bin = first.rsplit('/').next().unwrap_or(first);
    bin.trim_matches(|c: char| c == '"' || c == '\'')
}

fn extract_desktop_id_base(id: &str) -> &str {
    let clean = id.trim_end_matches(".desktop");
    clean.rsplit('.').next().unwrap_or(clean)
}

/// Search installed applications using a 4-tier hierarchy:
/// 1. Exact match (name / desktop_id / generic_name / exec binary / normalized alphanumeric)
/// 2. Substring & StartsWith match
/// 3. Generic category alias (terminal -> foot, browser -> chrome, editor -> zed)
/// 4. Fuzzy Levenshtein similarity (>= 70%)
pub fn find_target_app<'a>(
    query: &str,
    apps: &'a [crate::engine::app_scanner::InstalledApp],
) -> Option<&'a crate::engine::app_scanner::InstalledApp> {
    let raw = query.trim();
    if raw.is_empty() {
        return None;
    }

    // Clean label formats like "Foot (Terminal)" -> ["Foot", "Terminal"]
    let candidates: Vec<&str> = if let Some((base, role)) = raw.split_once('(') {
        let clean_role = role.trim_end_matches(')').trim();
        vec![base.trim(), clean_role, raw]
    } else {
        vec![raw]
    };

    // Priority 1: Exact match & Normalized alphanumeric match
    for q in &candidates {
        let q_norm = normalize_alnum(q);
        for app in apps {
            let exec_bin = extract_exec_binary(&app.exec);
            let id_base = app.desktop_id.as_deref().map(extract_desktop_id_base);

            // Exact literal check (case-insensitive)
            if app.name.eq_ignore_ascii_case(q)
                || app
                    .desktop_id
                    .as_deref()
                    .is_some_and(|id| id.eq_ignore_ascii_case(q))
                || id_base.is_some_and(|b| b.eq_ignore_ascii_case(q))
                || exec_bin.eq_ignore_ascii_case(q)
                || app
                    .generic_name
                    .as_deref()
                    .is_some_and(|g| g.eq_ignore_ascii_case(q))
            {
                return Some(app);
            }

            // Normalized alphanumeric match (e.g. "G4 music" -> "g4music" matches exec "g4music" or desktop_id "com.github.neithern.g4music.desktop")
            if !q_norm.is_empty() {
                if normalize_alnum(&app.name) == q_norm
                    || normalize_alnum(exec_bin) == q_norm
                    || id_base.map(normalize_alnum).as_deref() == Some(&q_norm)
                    || app.desktop_id.as_deref().map(normalize_alnum).as_deref() == Some(&q_norm)
                {
                    return Some(app);
                }
            }
        }
    }

    // Priority 2: Substring & StartsWith match
    for q in &candidates {
        let q_lower = q.to_lowercase();
        let q_norm = normalize_alnum(q);

        for app in apps {
            let name_lower = app.name.to_lowercase();
            let gen_lower = app.generic_name.as_deref().unwrap_or("").to_lowercase();
            let id_lower = app.desktop_id.as_deref().unwrap_or("").to_lowercase();
            let exec_bin_lower = extract_exec_binary(&app.exec).to_lowercase();
            let id_base_lower = app.desktop_id.as_deref().map(extract_desktop_id_base).unwrap_or("").to_lowercase();

            if name_lower.starts_with(&q_lower)
                || name_lower.contains(&q_lower)
                || (!gen_lower.is_empty() && gen_lower.contains(&q_lower))
                || (!id_lower.is_empty() && id_lower.contains(&q_lower))
                || (!exec_bin_lower.is_empty() && exec_bin_lower.contains(&q_lower))
                || (!id_base_lower.is_empty() && id_base_lower.contains(&q_lower))
                || q_lower.contains(&name_lower)
                || (!exec_bin_lower.is_empty() && q_lower.contains(&exec_bin_lower))
                || (!id_base_lower.is_empty() && q_lower.contains(&id_base_lower))
            {
                return Some(app);
            }

            // Also check normalized alnum contains (e.g. "g4music" within "comgithubneitherng4musicdesktop")
            if q_norm.len() >= 3 {
                let id_norm = normalize_alnum(&id_lower);
                let exec_norm = normalize_alnum(&exec_bin_lower);
                let name_norm = normalize_alnum(&name_lower);

                if id_norm.contains(&q_norm)
                    || exec_norm.contains(&q_norm)
                    || name_norm.contains(&q_norm)
                    || q_norm.contains(&exec_norm) && !exec_norm.is_empty()
                {
                    return Some(app);
                }
            }
        }
    }

    // Priority 3: Generic category alias matching (terminal -> foot, browser -> chrome, etc.)
    for q in &candidates {
        let q_lower = q.to_lowercase();
        for app in apps {
            if crate::engine::app_scanner::is_app_matching_keywords(app, &[&q_lower]) {
                return Some(app);
            }
        }
    }

    // Priority 4: Fuzzy Levenshtein Similarity (threshold >= 0.70)
    let mut best_app = None;
    let mut best_sim = 0.0f32;

    for q in &candidates {
        for app in apps {
            let exec_bin = extract_exec_binary(&app.exec);
            let sim_name = crate::engine::perception::string_similarity(q, &app.name);
            let sim_exec = crate::engine::perception::string_similarity(q, exec_bin);
            let sim_gen = app
                .generic_name
                .as_deref()
                .map_or(0.0, |g| crate::engine::perception::string_similarity(q, g));
            let sim_id = app.desktop_id.as_deref().map_or(0.0, |id| {
                let base = extract_desktop_id_base(id);
                crate::engine::perception::string_similarity(q, base)
            });

            let max_score = sim_name.max(sim_exec).max(sim_gen).max(sim_id);
            if max_score >= 0.70 && max_score > best_sim {
                best_sim = max_score;
                best_app = Some(app);
            }
        }
    }

    if let Some(app) = best_app {
        crate::term_log!(
            "[Fuzzy Match App] Found similar application '{}' -> '{}' (score: {:.1}%)",
            raw,
            app.name,
            best_sim * 100.0
        );
        return Some(app);
    }

    None
}

/// Find an already opened window that corresponds to the requested application query or target app.
pub fn find_matching_window<'a>(
    app_query: &str,
    target_app: Option<&crate::engine::app_scanner::InstalledApp>,
    windows: &'a [crate::engine::window_scanner::WindowItem],
) -> Option<&'a crate::engine::window_scanner::WindowItem> {
    if windows.is_empty() {
        return None;
    }

    let is_system_shell_window = |w: &crate::engine::window_scanner::WindowItem| -> bool {
        let title_clean = w.title.trim();
        let app_clean = w.app_name.to_lowercase();
        // Ignore desktop background, taskbar, shell tray, or empty windows in Windows/Linux
        title_clean == "Program Manager"
            || title_clean == "Windows Shell Experience Host"
            || (app_clean == "explorer.exe" && (title_clean.is_empty() || title_clean == "Program Manager"))
    };

    let mut candidates: Vec<&str> = Vec::new();
    if let Some(app) = target_app {
        candidates.push(&app.name);
        let exec_bin = extract_exec_binary(&app.exec);
        // Do not use generic launcher binaries as candidates to prevent matching shell windows
        if !exec_bin.is_empty()
            && !exec_bin.eq_ignore_ascii_case("explorer.exe")
            && !exec_bin.eq_ignore_ascii_case("explorer")
            && !exec_bin.eq_ignore_ascii_case("open")
            && !exec_bin.eq_ignore_ascii_case("sh")
        {
            candidates.push(exec_bin);
        }
        if let Some(ref id) = app.desktop_id {
            candidates.push(id);
            let base = extract_desktop_id_base(id);
            if !base.is_empty() {
                candidates.push(base);
            }
        }
        if let Some(ref generic_name) = app.generic_name {
            candidates.push(generic_name);
        }
    }

    let clean_query = app_query.split('(').next().unwrap_or(app_query).trim();
    if !clean_query.is_empty() && !candidates.contains(&clean_query) {
        candidates.push(clean_query);
    }

    // Priority 1: Exact or Normalized alphanumeric match on app_name or title
    for cand in &candidates {
        let cand_trim = cand.trim();
        if cand_trim.is_empty() {
            continue;
        }
        let cand_norm = normalize_alnum(cand_trim);

        for w in windows {
            if is_system_shell_window(w) {
                continue;
            }
            let app_clean = w.app_name.trim_end_matches(".exe");
            let app_norm = normalize_alnum(app_clean);
            let title_norm = normalize_alnum(&w.title);

            if app_clean.eq_ignore_ascii_case(cand_trim)
                || w.title.eq_ignore_ascii_case(cand_trim)
                || (!cand_norm.is_empty() && app_norm == cand_norm)
                || (!cand_norm.is_empty() && title_norm == cand_norm)
            {
                return Some(w);
            }
        }
    }

    // Priority 2: Substring / Contains match
    for cand in &candidates {
        let cand_trim = cand.trim();
        if cand_trim.is_empty() {
            continue;
        }
        let cand_lower = cand_trim.to_lowercase();
        let cand_norm = normalize_alnum(cand_trim);

        for w in windows {
            if is_system_shell_window(w) {
                continue;
            }
            let app_clean = w.app_name.trim_end_matches(".exe");
            let app_lower = app_clean.to_lowercase();
            let title_lower = w.title.to_lowercase();
            let app_norm = normalize_alnum(app_clean);
            let title_norm = normalize_alnum(&w.title);

            if app_lower.contains(&cand_lower)
                || title_lower.contains(&cand_lower)
                || (!app_lower.is_empty() && cand_lower.contains(&app_lower))
            {
                return Some(w);
            }

            if cand_norm.len() >= 3 {
                if (!app_norm.is_empty() && (app_norm.contains(&cand_norm) || cand_norm.contains(&app_norm)))
                    || (!title_norm.is_empty() && title_norm.contains(&cand_norm))
                {
                    return Some(w);
                }
            }
        }
    }

    // Priority 3: Fuzzy similarity match (threshold >= 0.70)
    let mut best_win = None;
    let mut best_sim = 0.0f32;
    for cand in &candidates {
        let cand_trim = cand.trim();
        if cand_trim.is_empty() {
            continue;
        }
        for w in windows {
            if is_system_shell_window(w) {
                continue;
            }
            let app_clean = w.app_name.trim_end_matches(".exe");
            let sim_app = crate::engine::perception::string_similarity(cand_trim, app_clean);
            let sim_title = crate::engine::perception::string_similarity(cand_trim, &w.title);
            let max_score = sim_app.max(sim_title);
            if max_score >= 0.70 && max_score > best_sim {
                best_sim = max_score;
                best_win = Some(w);
            }
        }
    }

    best_win
}

fn try_focus_window(w: &crate::engine::window_scanner::WindowItem) -> bool {
    // 1. Try native platform window handle / container identifier
    if !w.window_id.is_empty() && (w.window_id.starts_with("sway:") || w.window_id.starts_with("win:")) {
        if crate::engine::actions::desktop::focus_window(&w.window_id) {
            return true;
        }
    }
    // 2. Try window title
    if !w.title.is_empty() && crate::engine::actions::desktop::focus_window(&w.title) {
        return true;
    }
    // 3. Try app name
    let app_clean = w.app_name.trim_end_matches(".exe");
    if !app_clean.is_empty() && crate::engine::actions::desktop::focus_window(app_clean) {
        return true;
    }
    // 4. Try window_id as general target
    if !w.window_id.is_empty() && crate::engine::actions::desktop::focus_window(&w.window_id) {
        return true;
    }
    false
}

/// Multiplatform application launcher (multi-distro, multi-DE, and multi-WM)
pub fn launch_app(app: &crate::engine::app_scanner::InstalledApp) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        launch_windows_exec(&app.exec)
    }

    #[cfg(target_os = "macos")]
    {
        let mac_cmd = format!("open -a \"{}\" 2>/dev/null || ({})", app.name, app.exec);
        let mut command = std::process::Command::new("sh");
        command.args(["-c", &mac_cmd]);
        crate::engine::skills::configure_subprocess(&mut command);
        command.spawn()?;
        Ok(())
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        launch_linux_app(app)
    }
}

#[cfg(target_os = "windows")]
fn launch_windows_exec(exec: &str) -> std::io::Result<()> {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let trimmed = exec.trim();
    if trimmed.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Empty execution target",
        ));
    }

    // Try ShellExecuteW directly without cmd or consoles
    // 1. Separate file and args if applicable
    let (file_part, arg_part) = if let Some(stripped) = trimmed.strip_prefix('"') {
        if let Some(end_idx) = stripped.find('"') {
            let f = &stripped[..end_idx];
            let a = stripped[end_idx + 1..].trim();
            (f, if a.is_empty() { None } else { Some(a) })
        } else {
            (trimmed.trim_matches('"'), None)
        }
    } else if let Some((f, a)) = trimmed.split_once(' ') {
        (f.trim(), Some(a.trim()))
    } else {
        (trimmed, None)
    };

    let h_op = HSTRING::from("open");
    let h_file = HSTRING::from(file_part);
    let h_args = arg_part.map(HSTRING::from);

    unsafe {
        let instance = ShellExecuteW(
            None,
            windows::core::PCWSTR(h_op.as_wide().as_ptr()),
            windows::core::PCWSTR(h_file.as_wide().as_ptr()),
            h_args
                .as_ref()
                .map(|a| windows::core::PCWSTR(a.as_wide().as_ptr()))
                .unwrap_or(windows::core::PCWSTR::null()),
            None,
            SW_SHOWNORMAL,
        );

        // Win32: HINSTANCE > 32 indicates success
        if instance.0 as usize > 32 {
            return Ok(());
        }
    }

    // Fallback: silently execute command with CREATE_NO_WINDOW (0x08000000)
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = std::process::Command::new("explorer.exe");
        cmd.arg(trimmed);
        cmd.creation_flags(CREATE_NO_WINDOW);
        crate::engine::skills::configure_subprocess(&mut cmd);
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    Err(std::io::Error::other(format!(
        "Failed to launch '{trimmed}' via ShellExecuteW"
    )))
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn launch_linux_app(app: &crate::engine::app_scanner::InstalledApp) -> std::io::Result<()> {
    // 1. Check Compositor Wayland: Sway (Sway IPC seat spawn)
    if std::env::var("SWAYSOCK").is_ok() {
        if let Ok(status) = std::process::Command::new("swaymsg")
            .args(["exec", "--", &app.exec])
            .status()
        {
            if status.success() {
                crate::term_log!(
                    "[Linux Launcher] Opening '{}' via Sway IPC (swaymsg exec)",
                    app.name
                );
                return Ok(());
            }
        }
    }

    // 2. Check Compositor Wayland: Hyprland (Hyprland IPC dispatcher)
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        if let Ok(status) = std::process::Command::new("hyprctl")
            .args(["dispatch", "exec", "--", &app.exec])
            .status()
        {
            if status.success() {
                crate::term_log!(
                    "[Linux Launcher] Opening '{}' via Hyprland IPC (hyprctl dispatch exec)",
                    app.name
                );
                return Ok(());
            }
        }
    }

    // 3. Check Window Manager X11: i3
    if std::env::var("I3SOCK").is_ok() {
        if let Ok(status) = std::process::Command::new("i3-msg")
            .args(["exec", &app.exec])
            .status()
        {
            if status.success() {
                crate::term_log!(
                    "[Linux Launcher] Opening '{}' via i3 IPC (i3-msg exec)",
                    app.name
                );
                return Ok(());
            }
        }
    }

    // 4. Check XDG Desktop Environment standard launcher (GNOME / KDE / XFCE / MATE)
    if let Some(ref desktop_id) = app.desktop_id {
        if let Ok(status) = std::process::Command::new("gtk-launch")
            .arg(desktop_id)
            .status()
        {
            if status.success() {
                crate::term_log!(
                    "[Linux Launcher] Opening '{}' via gtk-launch ({})",
                    app.name,
                    desktop_id
                );
                return Ok(());
            }
        }
    }

    if let Some(ref desktop_path) = app.desktop_path {
        if let Ok(status) = std::process::Command::new("gio")
            .args(["launch", desktop_path])
            .status()
        {
            if status.success() {
                crate::term_log!("[Linux Launcher] Opening '{}' via gio launch", app.name);
                return Ok(());
            }
        }
    }

    // 5. Universal Detached Fallback (bekerja in all distro & lingkungan desktop)
    launch_linux_command(&app.exec)?;
    crate::term_log!(
        "[Linux Launcher] Opening '{}' via background detached process",
        app.name
    );
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn launch_linux_command(cmd_str: &str) -> std::io::Result<()> {
    // Check Sway
    if std::env::var("SWAYSOCK").is_ok() {
        if let Ok(status) = std::process::Command::new("swaymsg")
            .args(["exec", "--", cmd_str])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }
    }

    // Check Hyprland
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        if let Ok(status) = std::process::Command::new("hyprctl")
            .args(["dispatch", "exec", "--", cmd_str])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }
    }

    // Check i3
    if std::env::var("I3SOCK").is_ok() {
        if let Ok(status) = std::process::Command::new("i3-msg")
            .args(["exec", cmd_str])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }
    }

    // Fallback universal detached spawn
    let detached = format!("nohup {cmd_str} >/dev/null 2>&1 &");
    let mut command = std::process::Command::new("sh");
    command.args(["-c", &detached]);
    crate::engine::skills::configure_subprocess(&mut command);
    command.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::app_scanner::InstalledApp;

    #[test]
    fn test_find_target_app_exact_generic_and_fuzzy() {
        let apps = vec![
            InstalledApp {
                name: "Foot".to_string(),
                exec: "foot".to_string(),
                icon: None,
                comment: Some("A wayland native terminal emulator".to_string()),
                generic_name: Some("Terminal".to_string()),
                keywords: vec![
                    "shell".to_string(),
                    "command".to_string(),
                    "terminalemulator".to_string(),
                ],
                desktop_id: Some("foot".to_string()),
                desktop_path: None,
            },
            InstalledApp {
                name: "Google Chrome".to_string(),
                exec: "google-chrome".to_string(),
                icon: None,
                comment: Some("Access the internet".to_string()),
                generic_name: Some("Web Browser".to_string()),
                keywords: vec!["internet".to_string(), "webbrowser".to_string()],
                desktop_id: Some("google-chrome".to_string()),
                desktop_path: None,
            },
            InstalledApp {
                name: "Zed".to_string(),
                exec: "zed".to_string(),
                icon: None,
                comment: None,
                generic_name: Some("Text Editor".to_string()),
                keywords: vec!["code".to_string(), "texteditor".to_string()],
                desktop_id: Some("dev.zed.Zed".to_string()),
                desktop_path: None,
            },
            InstalledApp {
                name: "Gapless".to_string(),
                exec: "g4music %U".to_string(),
                icon: Some("com.github.neithern.g4music".to_string()),
                comment: Some("Play your music elegantly".to_string()),
                generic_name: Some("Music Player".to_string()),
                keywords: vec!["music".to_string(), "player".to_string()],
                desktop_id: Some("com.github.neithern.g4music.desktop".to_string()),
                desktop_path: None,
            },
        ];

        // 1. Exact Name
        assert_eq!(find_target_app("Foot", &apps).unwrap().name, "Foot");

        // 2. Generic Name / Alias
        assert_eq!(find_target_app("terminal", &apps).unwrap().name, "Foot");

        // 3. Label with Parenthesis from prompt ground of truth
        assert_eq!(
            find_target_app("Foot (Terminal)", &apps).unwrap().name,
            "Foot"
        );

        // 4. Substring & Generic
        assert_eq!(
            find_target_app("chrome", &apps).unwrap().name,
            "Google Chrome"
        );
        assert_eq!(
            find_target_app("browser", &apps).unwrap().name,
            "Google Chrome"
        );

        // 5. Gapless / G4Music matching across space and binary
        assert_eq!(
            find_target_app("G4 music", &apps).unwrap().name,
            "Gapless"
        );
        assert_eq!(
            find_target_app("g4music", &apps).unwrap().name,
            "Gapless"
        );
        assert_eq!(
            find_target_app("g4-music", &apps).unwrap().name,
            "Gapless"
        );

        // 6. Fuzzy typo
        assert_eq!(find_target_app("termnl", &apps).unwrap().name, "Foot");
        assert_eq!(find_target_app("zedd", &apps).unwrap().name, "Zed");
    }

    #[test]
    fn test_find_matching_window_open_vs_focus() {
        use crate::engine::window_scanner::WindowItem;

        let windows = vec![
            WindowItem {
                id: 1,
                app_name: "dev.zed.Zed".to_string(),
                title: "ai-voice — src/main.rs".to_string(),
                active: true,
                window_id: "sway:101".to_string(),
            },
            WindowItem {
                id: 2,
                app_name: "google-chrome".to_string(),
                title: "GitHub - Mozilla Firefox / Google Chrome".to_string(),
                active: false,
                window_id: "sway:102".to_string(),
            },
            WindowItem {
                id: 3,
                app_name: "com.github.neithern.g4music".to_string(),
                title: "Gapless Music Player".to_string(),
                active: false,
                window_id: "sway:103".to_string(),
            },
        ];

        // 1. Direct query matching open window app_name / title
        let match_zed = find_matching_window("zed", None, &windows);
        assert!(match_zed.is_some());
        assert_eq!(match_zed.unwrap().window_id, "sway:101");

        // 2. Matching with resolved InstalledApp (e.g. G4 music -> Gapless -> open window)
        let g4_app = InstalledApp {
            name: "Gapless".to_string(),
            exec: "g4music %U".to_string(),
            icon: None,
            comment: None,
            generic_name: Some("Music Player".to_string()),
            keywords: vec!["music".to_string()],
            desktop_id: Some("com.github.neithern.g4music.desktop".to_string()),
            desktop_path: None,
        };
        let match_g4 = find_matching_window("G4 music", Some(&g4_app), &windows);
        assert!(match_g4.is_some());
        assert_eq!(match_g4.unwrap().window_id, "sway:103");

        // 3. Query for an app that has no open window returns None
        let match_none = find_matching_window("calculator", None, &windows);
        assert!(match_none.is_none());
    }
}
