use anyhow::{bail, Result};

/// Applies surgical edits to file content with a multi-tier resilient matcher:
/// 1. Literal exact match
/// 2. Trimmed leading/trailing newline match
/// 3. Whitespace & indentation tolerant line-by-line sliding window
/// 4. Quote & semicolon relaxed matching (single quotes vs double quotes, trailing semicolons/commas)
/// 5. Anchor matching (first and last non-empty line match uniquely)
/// 6. High-similarity fuzzy window matching (similarity >= 0.85)
/// 7. Detailed contextual diagnostic reporting exact file lines if all matchers fail.
pub fn apply_resilient_edit(
    content: &str,
    old_text: &str,
    new_text: &str,
    file_path: &str,
) -> Result<String> {
    let normalized_content = content.replace("\r\n", "\n");
    let normalized_old = old_text.replace("\r\n", "\n");
    let normalized_new = new_text.replace("\r\n", "\n");

    // Tier 1: Exact literal match
    if normalized_content.contains(&normalized_old) {
        let count = normalized_content.matches(&normalized_old).count();
        if count == 1 {
            return Ok(normalized_content.replacen(&normalized_old, &normalized_new, 1));
        } else if count > 1 {
            bail!(
                "'old_text' matches {} times in file '{}'. Provide 2-3 additional surrounding lines to make it unique.",
                count, file_path
            );
        }
    }

    // Tier 2: Trimmed leading/trailing blank lines
    let trimmed_old = normalized_old.trim_matches('\n');
    let trimmed_new = normalized_new.trim_matches('\n');
    if !trimmed_old.is_empty() && trimmed_old != normalized_old && normalized_content.contains(trimmed_old) {
        let count = normalized_content.matches(trimmed_old).count();
        if count == 1 {
            return Ok(normalized_content.replacen(trimmed_old, trimmed_new, 1));
        }
    }

    let content_lines: Vec<&str> = normalized_content.lines().collect();
    let raw_old_lines: Vec<&str> = normalized_old.lines().collect();

    // Strip leading & trailing empty lines from old_lines
    let start_empty = raw_old_lines.iter().take_while(|l| l.trim().is_empty()).count();
    let end_empty = raw_old_lines.iter().rev().take_while(|l| l.trim().is_empty()).count();
    let old_lines: Vec<&str> = if start_empty + end_empty < raw_old_lines.len() {
        raw_old_lines[start_empty..raw_old_lines.len() - end_empty].to_vec()
    } else {
        raw_old_lines
    };

    if old_lines.is_empty() {
        bail!("'old_text' cannot be empty or only whitespace");
    }

    // Tier 3: Whitespace-tolerant line-by-line matching
    if content_lines.len() >= old_lines.len() {
        let mut match_indices = Vec::new();
        for i in 0..=(content_lines.len() - old_lines.len()) {
            let window = &content_lines[i..i + old_lines.len()];
            if window.iter().zip(old_lines.iter()).all(|(c, o)| c.trim() == o.trim()) {
                match_indices.push(i);
            }
        }

        if match_indices.len() == 1 {
            let start_idx = match_indices[0];
            let mut result_lines = Vec::new();
            result_lines.extend_from_slice(&content_lines[..start_idx]);
            result_lines.push(&normalized_new);
            result_lines.extend_from_slice(&content_lines[start_idx + old_lines.len()..]);
            return Ok(result_lines.join("\n"));
        } else if match_indices.len() > 1 {
            bail!(
                "'old_text' matched {} different sections with whitespace tolerance in '{}'. Provide more surrounding lines.",
                match_indices.len(), file_path
            );
        }
    }

    // Tier 4: Punctuation & quote relaxed matching (single quotes vs double quotes, trailing semicolons/commas)
    fn relax_line(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && *c != ';' && *c != ',')
            .map(|c| if c == '\'' || c == '`' { '"' } else { c })
            .collect()
    }

    if content_lines.len() >= old_lines.len() {
        let relaxed_old: Vec<String> = old_lines.iter().map(|l| relax_line(l)).collect();
        let mut relaxed_matches = Vec::new();
        for i in 0..=(content_lines.len() - old_lines.len()) {
            let window = &content_lines[i..i + old_lines.len()];
            let matches = window
                .iter()
                .zip(relaxed_old.iter())
                .all(|(c, o)| relax_line(c) == *o);
            if matches {
                relaxed_matches.push(i);
            }
        }

        if relaxed_matches.len() == 1 {
            let start_idx = relaxed_matches[0];
            let mut result_lines = Vec::new();
            result_lines.extend_from_slice(&content_lines[..start_idx]);
            result_lines.push(&normalized_new);
            result_lines.extend_from_slice(&content_lines[start_idx + old_lines.len()..]);
            return Ok(result_lines.join("\n"));
        }
    }

    // Tier 5: Anchor matching with meaningful non-trivial anchors
    if old_lines.len() >= 3 {
        // Find first non-trivial line (not just a solitary brace or tag like `</div>` or `}`)
        let first_meaningful = old_lines
            .iter()
            .find(|l| {
                let t = l.trim();
                t.len() > 3 && t != "</div>" && t != "}" && t != "};" && t != "</button>" && t != "</div>;"
            })
            .map(|l| l.trim());

        let last_meaningful = old_lines
            .iter()
            .rev()
            .find(|l| {
                let t = l.trim();
                t.len() > 3 && t != "</div>" && t != "}" && t != "};" && t != "</button>" && t != "</div>;"
            })
            .map(|l| l.trim());

        if let (Some(first_anchor), Some(last_anchor)) = (first_meaningful, last_meaningful) {
            let mut anchor_matches = Vec::new();
            for i in 0..content_lines.len() {
                if content_lines[i].trim() == first_anchor {
                    let min_end = i + 1;
                    let max_end = (i + old_lines.len() + 15).min(content_lines.len());
                    for (j, line) in content_lines.iter().enumerate().take(max_end).skip(min_end) {
                        if line.trim() == last_anchor {
                            anchor_matches.push((i, j));
                        }
                    }
                }
            }

            if !anchor_matches.is_empty() {
                let target_span = old_lines.len();
                let mut best_anchor = anchor_matches[0];
                let mut min_diff = (anchor_matches[0].1 - anchor_matches[0].0 + 1).abs_diff(target_span);
                let mut second_diff = usize::MAX;

                for &(start, end) in &anchor_matches[1..] {
                    let diff = (end - start + 1).abs_diff(target_span);
                    if diff < min_diff {
                        second_diff = min_diff;
                        min_diff = diff;
                        best_anchor = (start, end);
                    } else if diff < second_diff {
                        second_diff = diff;
                    }
                }

                if anchor_matches.len() == 1 || min_diff < second_diff {
                    let (start_idx, end_idx) = best_anchor;
                    let mut result_lines = Vec::new();
                    result_lines.extend_from_slice(&content_lines[..start_idx]);
                    result_lines.push(&normalized_new);
                    result_lines.extend_from_slice(&content_lines[end_idx + 1..]);
                    return Ok(result_lines.join("\n"));
                }
            }
        }
    }

    // Tier 6: Elastic similarity window matching (handles line count variations +/- 3 lines)
    if !content_lines.is_empty() && !old_lines.is_empty() {
        let min_len = old_lines.len().saturating_sub(3).max(1);
        let max_len = (old_lines.len() + 3).min(content_lines.len());

        let mut best_score = 0.0f64;
        let mut second_best_score = 0.0f64;
        let mut best_range = None;

        for window_len in min_len..=max_len {
            if content_lines.len() < window_len {
                continue;
            }
            for i in 0..=(content_lines.len() - window_len) {
                let window = &content_lines[i..i + window_len];
                let mut score_sum = 0.0;
                let pair_count = window.len().min(old_lines.len());
                for (c, o) in window.iter().take(pair_count).zip(old_lines.iter().take(pair_count)) {
                    let c_trim = c.trim();
                    let o_trim = o.trim();
                    if c_trim == o_trim {
                        score_sum += 1.0;
                    } else if c_trim.to_lowercase() == o_trim.to_lowercase() {
                        score_sum += 0.95;
                    } else {
                        let c_chars: std::collections::HashSet<char> = c_trim.chars().filter(|ch| !ch.is_whitespace()).collect();
                        let o_chars: std::collections::HashSet<char> = o_trim.chars().filter(|ch| !ch.is_whitespace()).collect();
                        let intersect = c_chars.intersection(&o_chars).count();
                        let union = c_chars.union(&o_chars).count();
                        if union > 0 {
                            score_sum += (intersect as f64) / (union as f64);
                        }
                    }
                }
                let avg_score = score_sum / (old_lines.len().max(window_len) as f64);
                if avg_score > best_score {
                    second_best_score = best_score;
                    best_score = avg_score;
                    best_range = Some((i, i + window_len));
                } else if avg_score > second_best_score {
                    second_best_score = avg_score;
                }
            }
        }

        if best_score >= 0.65 && (best_score - second_best_score) >= 0.04 {
            if let Some((start_idx, end_idx)) = best_range {
                let mut result_lines = Vec::new();
                result_lines.extend_from_slice(&content_lines[..start_idx]);
                result_lines.push(&normalized_new);
                result_lines.extend_from_slice(&content_lines[end_idx..]);
                return Ok(result_lines.join("\n"));
            }
        }
    }

    // Tier 7: Diagnostics reporting exact file lines
    let mut best_hint_idx = None;
    let mut highest_keyword_match = 0;

    for (idx, line) in content_lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        let matches = old_lines
            .iter()
            .filter(|o| o.trim().len() >= 4 && line_lower.contains(&o.trim().to_lowercase()))
            .count();
        if matches > highest_keyword_match {
            highest_keyword_match = matches;
            best_hint_idx = Some(idx);
        }
    }

    let hint_block = if let Some(center_idx) = best_hint_idx {
        let start = center_idx.saturating_sub(4);
        let end = (center_idx + 6).min(content_lines.len());
        let snippet = content_lines[start..end]
            .iter()
            .enumerate()
            .map(|(offset, l)| format!("{:>4} | {}", start + offset + 1, l))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\nClosest actual content in '{}' (lines {}-{}):\n```\n{}\n```\nAction: Copy the exact lines above to form `old_text` or use `workspace_write_file` if replacing a large block.",
            file_path, start + 1, end, snippet
        )
    } else {
        format!(
            "\nUse `workspace_read_file` to inspect the exact lines of '{}' before editing.",
            file_path
        )
    };

    bail!("'old_text' not found in file '{}'.{}", file_path, hint_block)
}
