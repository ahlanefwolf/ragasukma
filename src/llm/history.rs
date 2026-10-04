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
    #[repr(C)]
    struct TimeZoneInformation {
        bias: i32,
        standard_name: [u16; 32],
        standard_date: SystemTime,
        standard_bias: i32,
        daylight_name: [u16; 32],
        daylight_date: SystemTime,
        daylight_bias: i32,
    }

    unsafe extern "system" {
        fn GetLocalTime(lp_system_time: *mut SystemTime);
        fn GetTimeZoneInformation(lp_time_zone_information: *mut TimeZoneInformation) -> u32;
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
    let mut tzi = std::mem::MaybeUninit::<TimeZoneInformation>::zeroed();

    unsafe {
        GetLocalTime(&mut st);
        GetTimeZoneInformation(tzi.as_mut_ptr());
    };

    let tzi = unsafe { tzi.assume_init() };
    // Bias in minutes from UTC: UTC = Local + Bias => Offset = -Bias
    let offset_mins = -tzi.bias;
    let sign = if offset_mins >= 0 { '+' } else { '-' };
    let abs_mins = offset_mins.abs();
    let off_hours = abs_mins / 60;
    let off_minutes = abs_mins % 60;

    (
        format!("{:04}-{:02}-{:02}", st.w_year, st.w_month, st.w_day),
        format!(
            "{:02}:{:02}:{:02}{}{:02}:{:02}",
            st.w_hour, st.w_minute, st.w_second, sign, off_hours, off_minutes
        ),
    )
}

#[cfg(not(windows))]
fn get_local_date_and_time() -> (String, String) {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    #[repr(C)]
    struct Tm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        tm_zone: *const std::os::raw::c_char,
    }

    unsafe extern "C" {
        fn localtime_r(time_p: *const i64, result: *mut Tm) -> *mut Tm;
    }

    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null(),
    };

    unsafe {
        localtime_r(&secs, &mut tm);
    }

    let year = tm.tm_year + 1900;
    let month = tm.tm_mon + 1;
    let day = tm.tm_mday;
    let hour = tm.tm_hour;
    let minute = tm.tm_min;
    let second = tm.tm_sec;

    // tm_gmtoff is seconds east of UTC (e.g. +25200 for +07:00, +28800 for +08:00)
    let off_secs = tm.tm_gmtoff;
    let sign = if off_secs >= 0 { '+' } else { '-' };
    let abs_secs = off_secs.abs();
    let off_hours = abs_secs / 3600;
    let off_minutes = (abs_secs % 3600) / 60;

    (
        format!("{:04}-{:02}-{:02}", year, month, day),
        format!(
            "{:02}:{:02}:{:02}{}{:02}:{:02}",
            hour, minute, second, sign, off_hours, off_minutes
        ),
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
        crate::term_err!("[History] Failed to create histories directory: {e}");
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
                    "# Conversation History — {date_str}\n> Assistant: {assistant_name} ({role}) | User: {callsign}\n\n---\n\n"
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
                "[History] Failed to write chat history to {:?}: {e}",
                file_path
            );
        }
    }
}
