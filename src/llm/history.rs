use std::fs::OpenOptions;
use std::io::Write;

#[cfg(windows)]
fn get_local_date_and_time() -> (String, String) {
    #[repr(C)]
    struct SystemTime {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }
    unsafe extern "system" {
        fn GetLocalTime(lp_system_time: *mut SystemTime);
    }
    let mut st = SystemTime {
        w_year: 0,
        w_month: 0,
        w_day_of_week: 0,
        w_day: 0,
        w_hour: 0,
        w_minute: 0,
        w_second: 0,
        w_milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut st) };
    (
        format!("{:04}-{:02}-{:02}", st.w_year, st.w_month, st.w_day),
        format!("{:02}:{:02}:{:02}", st.w_hour, st.w_minute, st.w_second),
    )
}

#[cfg(not(windows))]
fn get_local_date_and_time() -> (String, String) {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (
        format!("{:04}-{:02}-{:02}", year, m, d),
        format!("{:02}:{:02}:{:02}", hour, min, sec),
    )
}

pub fn log_conversation_turn(
    callsign: &str,
    assistant_name: &str,
    role: &str,
    user_msg: &str,
    assistant_msg: &str,
) {
    let (date_str, time_str) = get_local_date_and_time();
    let histories_dir = crate::assets::get_assets_dir().join("histories");
    if let Err(e) = std::fs::create_dir_all(&histories_dir) {
        crate::term_err!("⚠️ [History] Failed to create histories directory: {e}");
        return;
    }

    let file_path = histories_dir.join(format!("{date_str}.md"));
    let is_new = !file_path.exists();

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
    {
        Ok(mut file) => {
            if is_new {
                let header = format!(
                    "# Riwayat Percakapan — {date_str}\n> Asisten: {assistant_name} ({role}) | Pengguna: {callsign}\n\n---\n\n"
                );
                let _ = file.write_all(header.as_bytes());
            }

            let entry = format!(
                "### [{time_str}] {callsign}\n{user_msg}\n\n### [{time_str}] {assistant_name}\n{assistant_msg}\n\n---\n\n"
            );
            let _ = file.write_all(entry.as_bytes());
        }
        Err(e) => {
            crate::term_err!(
                "⚠️ [History] Failed to write chat history to {:?}: {e}",
                file_path
            );
        }
    }
}
