use regex::Regex;
use std::sync::LazyLock;

static RE_SKILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(?:SKILL|CMD):[^\]]+\]").expect("valid regex"));

// Matches Claude-style and tool-call XML blocks the model may emit instead of [CMD:...]:
//   <dots_function_call>...</dots_function_call>
//   <tool_call>...</tool_call>
//   <invoke ...>...</invoke>
static RE_XML_TOOL_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)<(?:dots_function_call|tool_call)[^>]*>.*?</(?:dots_function_call|tool_call)>|<invoke[^>]*>.*?</invoke>",
    )
    .expect("valid xml tool call regex")
});

static RE_INVOKE_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<invoke\s+name=["']([^"']+)["'][^>]*>(.*?)</invoke>"#)
        .expect("valid invoke block regex")
});

static RE_PARAM_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<parameter\s+name=["']([^"']+)["'][^>]*>(.*?)</parameter>"#)
        .expect("valid param block regex")
});

/// Translates tool-call XML blocks (e.g. from Dots Studio, Claude, or tool-fine-tuned LLMs)
/// into canonical Ragasukma `[CMD:...]` action tags so they execute seamlessly.
#[must_use]
pub fn normalize_tool_calls_to_commands(text: &str) -> String {
    let mut normalized = text.to_string();

    for cap in RE_INVOKE_BLOCK.captures_iter(text) {
        let full_match = cap.get(0).map_or("", |m| m.as_str());
        let raw_name = cap.get(1).map_or("", |m| m.as_str()).trim();
        let body = cap.get(2).map_or("", |m| m.as_str());

        // Extract parameters into a map
        let mut params = std::collections::HashMap::new();
        for pcap in RE_PARAM_BLOCK.captures_iter(body) {
            let pname = pcap.get(1).map_or("", |m| m.as_str()).trim().to_lowercase();
            let pval = pcap.get(2).map_or("", |m| m.as_str()).trim().to_string();
            params.insert(pname, pval);
        }

        let cmd_tag = map_tool_call_to_cmd(raw_name, &params);
        if let Some(cmd) = cmd_tag {
            normalized = normalized.replace(full_match, &format!(" {cmd} "));
        }
    }

    normalized
}

fn map_tool_call_to_cmd(
    name: &str,
    params: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let name_lower = name.to_lowercase();

    // 1. Direct canonical tag: if name is already formatted like "desktop:click_text" or "cmd:desktop:click_text"
    let clean_name = name_lower.strip_prefix("cmd:").unwrap_or(&name_lower);

    // Get primary text parameter if available
    let primary_param = params
        .get("text")
        .or_else(|| params.get("query"))
        .or_else(|| params.get("target"))
        .or_else(|| params.get("value"))
        .or_else(|| params.get("url"))
        .or_else(|| params.get("label"))
        .or_else(|| params.get("title"))
        .or_else(|| params.get("name"))
        .cloned();

    // 2. Dots / generic tool mapping
    match clean_name {
        "command" | "cmd" => {
            // E.g. <invoke name="command"><parameter name="text">Providers</parameter></invoke>
            // or <parameter name="text">[CMD:desktop:click_text|Providers]</parameter>
            if let Some(val) = primary_param {
                let trimmed = val.trim();
                if trimmed.starts_with("[CMD:") || trimmed.starts_with("[SKILL:") {
                    return Some(trimmed.to_string());
                }
                // If it's a bare word/target without namespace, default to desktop text click
                return Some(format!("[CMD:desktop:click_text|{trimmed}]"));
            }
            None
        }
        "desktop_scan_visual" | "desktop:scan_visual" | "scan_visual" => {
            Some("[CMD:desktop:scan_visual]".to_string())
        }
        "desktop_click_text" | "desktop:click_text" | "click_text" => {
            primary_param.map(|val| format!("[CMD:desktop:click_text|{val}]"))
        }
        "desktop_click_button" | "desktop:click_button" | "click_button" => {
            primary_param.map(|val| format!("[CMD:desktop:click_button|{val}]"))
        }
        "desktop_click_at" | "desktop:click_at" | "click_at" => {
            let x = params.get("x").map_or("0", |s| s.as_str());
            let y = params.get("y").map_or("0", |s| s.as_str());
            Some(format!("[CMD:desktop:click_at|{x}|{y}]"))
        }
        "desktop_type" | "desktop:type" => {
            primary_param.map(|val| format!("[CMD:desktop:type|{val}]"))
        }
        "desktop_key" | "desktop:key" => {
            primary_param.map(|val| format!("[CMD:desktop:key|{val}]"))
        }
        "desktop_focus" | "desktop:focus" => {
            primary_param.map(|val| format!("[CMD:desktop:focus|{val}]"))
        }
        "desktop_minimize" | "desktop:minimize" => Some("[CMD:desktop:minimize]".to_string()),
        "desktop_maximize" | "desktop:maximize" => Some("[CMD:desktop:maximize]".to_string()),
        "desktop_close" | "desktop:close" => Some("[CMD:desktop:close]".to_string()),
        "browser_open" | "browser:open" => {
            primary_param.map(|val| format!("[CMD:browser:open|{val}]"))
        }
        "browser_search" | "browser:search" => {
            primary_param.map(|val| format!("[CMD:browser:search|{val}]"))
        }
        "browser_switch" | "browser:switch" => {
            primary_param.map(|val| format!("[CMD:browser:switch|{val}]"))
        }
        "browser_close" | "browser:close" => {
            primary_param.map(|val| format!("[CMD:browser:close|{val}]"))
        }
        "browser_scroll" | "browser:scroll" => {
            let dist = primary_param.unwrap_or_else(|| "400".to_string());
            Some(format!("[CMD:browser:scroll|{dist}]"))
        }
        "browser_type" | "browser:type" => {
            primary_param.map(|val| format!("[CMD:browser:type|{val}]"))
        }
        "browser_enter" | "browser:enter" => Some("[CMD:browser:enter]".to_string()),
        "app_open" | "app:open" => primary_param.map(|val| format!("[CMD:app:open|{val}]")),
        "media_play" | "media:play" => primary_param.map(|val| format!("[CMD:media:play|{val}]")),
        "media_pause" | "media:pause" => Some("[CMD:media:pause]".to_string()),
        "media_resume" | "media:resume" => Some("[CMD:media:resume]".to_string()),
        "media_stop" | "media:stop" => Some("[CMD:media:stop]".to_string()),
        "os_volume_up" | "os:volume_up" => Some("[CMD:os:volume_up]".to_string()),
        "os_volume_down" | "os:volume_down" => Some("[CMD:os:volume_down]".to_string()),
        "os_mute" | "os:mute" => Some("[CMD:os:mute]".to_string()),
        "os_screenshot" | "os:screenshot" => Some("[CMD:os:screenshot]".to_string()),
        "os_lock" | "os:lock" => Some("[CMD:os:lock]".to_string()),
        "todo_add" | "todo:add" => primary_param.map(|val| format!("[CMD:todo:add|{val}]")),
        "todo_done" | "todo:done" => primary_param.map(|val| format!("[CMD:todo:done|{val}]")),
        _ => {
            // Check if name has namespace:action format directly
            if let Some((ns, act)) = clean_name.split_once(':') {
                if let Some(param) = primary_param {
                    Some(format!("[CMD:{ns}:{act}|{param}]"))
                } else {
                    Some(format!("[CMD:{ns}:{act}]"))
                }
            } else if let Some(param) = primary_param {
                Some(format!("[CMD:{clean_name}|{param}]"))
            } else {
                Some(format!("[SKILL:{clean_name}]"))
            }
        }
    }
}

static RE_BOLD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\*\*([^\*\n]+?)\*\*").expect("valid bold regex")
});

static RE_SINGLE_STAR_EMOTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\*[^\*\n]{1,60}\*").expect("valid single star emote regex")
});

static RE_ROLEPLAY_PARENS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\((?:tersenyum|tertawa|mengangguk|menatap|tersipu|melirik|sambil [^\)\n]{1,40}|giggle|chuckle|sigh|whisper|smiles?|laughs?|nodding?|blushing?)\)").expect("valid roleplay parens regex")
});

/// Clean AI text: strip action tags ([CMD:...], [SKILL:...]), XML tool-call blocks,
/// bold markdown asterisks (**bold** -> bold), single-asterisk emotion/roleplay tags
/// (*malu-malu*, *happy*, *tersenyum* -> removed), roleplay parentheses, and backticks.
pub fn clean_ai_text(text: &str) -> String {
    // 1. Strip any XML tool-call blocks Claude may have emitted
    let text = RE_XML_TOOL_CALL.replace_all(text, "");

    // 2. Strip [CMD:...] / [SKILL:...] tags
    let text = RE_SKILL.replace_all(&text, "");

    // 3. Normalisasi bold markdown: `**teks**` -> `teks`
    let text = RE_BOLD.replace_all(&text, "$1");

    // 4. Buang tag bintang tunggal emosi/roleplay (*malu-malu*, *happy*, *tersenyum*, dsb.)
    let text = RE_SINGLE_STAR_EMOTE.replace_all(&text, " ");

    // 5. Buang kurung aksi roleplay umum: (tersenyum), (sambil tertawa), dll.
    let text = RE_ROLEPLAY_PARENS.replace_all(&text, " ");

    // 6. Buang backtick (`` ` ``) dan header markdown (`#`)
    let mut cleaned = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '`' | '#' => cleaned.push(' '),
            _ => cleaned.push(c),
        }
    }

    // 7. Rapikan multiple spaces
    let mut result = String::with_capacity(cleaned.len());
    let mut last_was_space = false;
    for c in cleaned.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_dots_command_tool_call() {
        let raw = "<dots_function_call>\n<invoke name=\"command\">\n<parameter name=\"text\">Providers</parameter>\n</invoke>\n</dots_function_call>";
        let norm = normalize_tool_calls_to_commands(raw);
        assert!(norm.contains("[CMD:desktop:click_text|Providers]"));
    }

    #[test]
    fn test_normalize_dots_scan_visual() {
        let raw = "<dots_function_call>\n<invoke name=\"desktop_scan_visual\">\n</invoke>\n</dots_function_call>";
        let norm = normalize_tool_calls_to_commands(raw);
        assert!(norm.contains("[CMD:desktop:scan_visual]"));
    }

    #[test]
    fn test_normalize_tool_call_generic() {
        let raw = "*happy* Oke! <tool_call><invoke name=\"browser:open\"><parameter name=\"url\">youtube.com</parameter></invoke></tool_call>";
        let norm = normalize_tool_calls_to_commands(raw);
        assert!(norm.contains("[CMD:browser:open|youtube.com]"));
    }

    #[test]
    fn test_clean_ai_text_removes_roleplay_and_bold() {
        let raw = "*malu-malu* Halo Kak! Ini **sangat penting** lho! (tersenyum) `kode_rahasia`";
        let cleaned = clean_ai_text(raw);
        assert_eq!(cleaned, "Halo Kak! Ini sangat penting lho! kode_rahasia");
    }

    #[test]
    fn test_clean_ai_text_strips_cmd_and_emotes() {
        let raw = "*happy* Siap! [CMD:desktop:click_text|Save] *tersenyum manis*";
        let cleaned = clean_ai_text(raw);
        assert_eq!(cleaned, "Siap!");
    }
}
