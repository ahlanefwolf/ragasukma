use serde::Deserialize;
use std::net::TcpStream;

use crate::engine::{
    SkillItem, execute_command_by_os, get_installed_applications, parse_skills_markdown,
    save_skills_markdown,
};
use crate::server::send_json_response;

pub fn handle_get_apps(stream: &mut TcpStream) {
    let apps = get_installed_applications();
    let json = serde_json::to_string(&apps).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_get_skills(stream: &mut TcpStream) {
    let skills = parse_skills_markdown();
    let json = serde_json::to_string(&skills).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_post_skills(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct AddSkillPayload {
        name: String,
        description: String,
        command: String,
    }

    if let Ok(item) = serde_json::from_str::<AddSkillPayload>(body_str) {
        let mut skills = parse_skills_markdown();
        if let Some(pos) = skills
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(&item.name))
        {
            skills[pos] = SkillItem {
                name: item.name,
                description: item.description,
                command: item.command,
            };
        } else {
            skills.push(SkillItem {
                name: item.name,
                description: item.description,
                command: item.command,
            });
        }
        if let Err(e) = save_skills_markdown(&skills) {
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        } else {
            send_json_response(stream, 200, "{\"success\":true}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid skill format\"}");
    }
}

pub fn handle_delete_skill(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct DeleteSkillPayload {
        name: String,
    }

    if let Ok(item) = serde_json::from_str::<DeleteSkillPayload>(body_str) {
        let mut skills = parse_skills_markdown();
        skills.retain(|s| !s.name.eq_ignore_ascii_case(&item.name));
        if let Err(e) = save_skills_markdown(&skills) {
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        } else {
            send_json_response(stream, 200, "{\"success\":true}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid delete payload\"}");
    }
}

pub fn handle_run_skill(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct RunSkillPayload {
        command: String,
    }

    if let Ok(payload) = serde_json::from_str::<RunSkillPayload>(body_str) {
        if let Err(e) = execute_command_by_os(&payload.command) {
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        } else {
            send_json_response(stream, 200, "{\"success\":true}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid run payload\"}");
    }
}
