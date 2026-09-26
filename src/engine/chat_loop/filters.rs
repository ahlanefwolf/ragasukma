/// Filter halusinasi teks Whisper/STT hening bawaan dataset
pub fn is_hallucination_text(raw_text: &str) -> bool {
    let lower = raw_text.to_lowercase();
    let stripped_punctuation: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let clean_trimmed = stripped_punctuation.trim();

    clean_trimmed == "terima kasih"
        || clean_trimmed == "makasih"
        || clean_trimmed == "terimakasih"
        || clean_trimmed == "terima kasih banyak"
        || clean_trimmed == "thank you"
        || clean_trimmed == "thanks"
        || clean_trimmed == "terima kasih telah menonton"
        || lower.contains("menonton")
        || lower.contains("subtitles by")
        || lower.contains("subscribe")
        || lower.contains("thank you for watching")
        || lower.contains("terima kasih kerana")
        || lower.contains("terima kasih sudah")
        || (lower.contains("terima kasih") && lower.matches("terima kasih").count() > 1)
}
