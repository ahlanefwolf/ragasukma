use regex::Regex;
use std::sync::LazyLock;

static RE_SKILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(?:SKILL|CMD):[^\]]+\]").expect("valid regex"));

// Matches Claude-style XML tool-call blocks the model may emit instead of [CMD:...]:
//   <dots_function_call>...</dots_function_call>
//   <invoke ...>...</invoke>
// NOTE: <parameter> is intentionally NOT matched standalone — it is always nested
// inside <invoke> or <dots_function_call> which are already stripped by the patterns above.
static RE_XML_TOOL_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)<dots_function_call[^>]*>.*?</dots_function_call>|<invoke[^>]*>.*?</invoke>",
    )
    .expect("valid xml tool call regex")
});

/// Clean AI text: strip action tags ([CMD:...], [SKILL:...]), XML tool-call blocks
/// emitted by Claude models, and asterisk emotion markers — without cutting real content.
pub fn clean_ai_text(text: &str) -> String {
    // First strip any XML tool-call blocks Claude may have emitted
    let text = RE_XML_TOOL_CALL.replace_all(text, "");

    // Then strip [CMD:...] / [SKILL:...] tags
    let text_clean = RE_SKILL.replace_all(&text, "");

    let mut result = String::new();
    let mut chars = text_clean.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut inside = String::new();
            inside.push('[');
            while let Some(&next_c) = chars.peek() {
                chars.next();
                inside.push(next_c);
                if next_c == ']' {
                    break;
                }
            }
            // Action tags must not be modified/deleted
            result.push_str(&inside);
            continue;
        }

        if c == '*' {
            let mut inside = String::new();
            let mut closed = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '*' {
                    closed = true;
                    if chars.peek() == Some(&'*') {
                        chars.next();
                    }
                    break;
                }
                inside.push(next_c);
            }

            if !closed {
                result.push('*');
                result.push_str(&inside);
            }
            continue;
        }

        result.push(c);
    }

    result.trim().to_string()
}
