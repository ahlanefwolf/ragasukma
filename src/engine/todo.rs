use std::path::PathBuf;

pub fn get_todo_path() -> PathBuf {
    let candidate = crate::assets::find_file_in_candidates("todo.md");
    if candidate.exists() {
        candidate
    } else {
        crate::assets::get_assets_dir().join("todo.md")
    }
}

pub fn add_task_to_content(content: &str, task: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let target_item = format!("- [ ] {}", task);

    let mut insert_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let l = line.to_lowercase();
        if l.starts_with("## ") && (l.contains("sedang berjalan") || l.contains("to-do") || l.contains("todo")) {
            insert_idx = Some(i + 1);
            break;
        }
    }

    if let Some(idx) = insert_idx {
        lines.insert(idx, target_item);
    } else {
        lines.push(String::new());
        lines.push("## ⏳ Sedang Berjalan / To-Do".to_string());
        lines.push(target_item);
    }

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content
}

pub fn complete_task_in_content(content: &str, query: &str) -> Option<(String, String)> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found_idx = None;
    let mut matched_title = String::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") {
            let task_text = trimmed.trim_start_matches("- [ ]").trim();
            if task_text.to_lowercase().contains(&q) {
                found_idx = Some(i);
                matched_title = task_text.to_string();
                break;
            }
        }
    }

    let found_idx = found_idx?;
    lines.remove(found_idx);

    let completed_item = format!("- [x] {}", matched_title);
    let mut done_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let l = line.to_lowercase();
        if l.starts_with("## ") && l.contains("selesai") {
            done_idx = Some(i + 1);
            break;
        }
    }

    if let Some(idx) = done_idx {
        lines.insert(idx, completed_item);
    } else {
        lines.push(String::new());
        lines.push("## ✅ Selesai".to_string());
        lines.push(completed_item);
    }

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    Some((new_content, matched_title))
}

pub fn add_task(task_text: &str) -> std::io::Result<String> {
    let task = task_text.trim();
    if task.is_empty() {
        return Ok("Teks tugas tidak boleh kosong".to_string());
    }

    let path = get_todo_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let default_content = "# 📋 Daftar Tugas & Catatan\n\n## ⏳ Sedang Berjalan / To-Do\n\n## ✅ Selesai\n";
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| default_content.to_string());

    let new_content = add_task_to_content(&content, task);
    std::fs::write(&path, new_content)?;
    Ok(format!("Tugas '{}' berhasil dicatat ke todo.md", task))
}

pub fn complete_task(query: &str) -> std::io::Result<String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok("Kata kunci tugas tidak boleh kosong".to_string());
    }

    let path = get_todo_path();
    if !path.exists() {
        return Ok("Daftar to-do belum dibuat".to_string());
    }

    let content = std::fs::read_to_string(&path)?;
    match complete_task_in_content(&content, q) {
        Some((new_content, matched)) => {
            std::fs::write(&path, new_content)?;
            Ok(format!("Tugas '{}' berhasil ditandai selesai di todo.md", matched))
        }
        None => Ok(format!("Tugas dengan kata kunci '{}' tidak ditemukan di daftar aktif", q)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_complete_todo() {
        let initial = "# 📋 Daftar Tugas\n\n## ⏳ Sedang Berjalan / To-Do\n- [ ] Beli susu\n\n## ✅ Selesai\n";
        let updated = add_task_to_content(initial, "Undangan nikahan Yusuf");
        assert!(updated.contains("- [ ] Undangan nikahan Yusuf"));
        assert!(updated.contains("- [ ] Beli susu"));

        let (done, matched) = complete_task_in_content(&updated, "nikahan").expect("should find task");
        assert_eq!(matched, "Undangan nikahan Yusuf");
        assert!(!done.contains("- [ ] Undangan nikahan Yusuf"));
        assert!(done.contains("- [x] Undangan nikahan Yusuf"));
        assert!(done.contains("- [ ] Beli susu"));
    }
}
