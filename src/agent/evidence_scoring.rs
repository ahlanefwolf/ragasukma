//! Relevance of inherited evidence, measured without a word list.
//!
//! When a goal continues a parent goal, the evidence the parent gathered is offered to the child.
//! How strongly each file is carried over decides which files survive the prompt cap, because
//! `get_active_evidences_for_goal` sorts on this score.
//!
//! The previous rule counted how many request words appeared in a file's path, symbol, or summary,
//! then divided by the total number of words. That division is what broke it: a request is mostly
//! words that appear in every file (`the`, `a`, `to`), so the denominator grew with the length of
//! the sentence while the numerator did not. Relevance sank toward zero, and every inherited file
//! lost to a freshly read one — even the files the task was actually about.
//!
//! # What replaces it
//!
//! A word is useful for telling files apart only if it is *rare*. That is measurable from the
//! evidence itself, so no stopword list is needed and nothing is tuned to a language:
//!
//! - a word present in every candidate file carries no information, and is ignored;
//! - a word present in one file identifies that file, and is worth the most.
//!
//! That is the same idea as inverse document frequency, restricted to the handful of files under
//! comparison. Adding a new language, or a request phrased in a mix of languages, requires no
//! change here.
//!
//! The score also has to be comparable with what the ranking module produces, since both write into
//! the same column. `MAX_SCORE` is the ceiling so a perfect match lands exactly at a fully relevant
//! edit, and the caller never lets inheritance lower a score that is already higher.

use std::collections::HashMap;

/// Ceiling of the inherited-relevance score.
///
/// Matches the top weight in `evidence_ranking`, so an inherited file that matches the continuation
/// request perfectly is indistinguishable from one the agent just edited, and never outranks it.
pub const MAX_SCORE: f64 = 1.0;

/// Everything about one candidate file that a request word may match against.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    pub file_path: &'a str,
    pub symbol_name: Option<&'a str>,
    pub summary: &'a str,
}

/// Splits text into comparable words.
///
/// Only characters that cannot belong to a word are used as separators; no vocabulary is assumed.
/// Single characters are dropped because they cannot distinguish one file from another and would
/// otherwise make every request match everything.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() > 1)
        .map(|w| w.to_lowercase())
        .collect()
}

/// How many candidate files each request word appears in.
///
/// A word absent from every candidate is still recorded with a count of zero: it is compared
/// against the candidate space, and a request word found in no file simply contributes nothing.
fn document_frequencies(
    request_words: &[String],
    candidates: &[Candidate<'_>],
) -> HashMap<String, usize> {
    // A word counts once per file, however often it repeats inside that file. Repeating a word
    // does not make it a better identifier.
    let candidate_words: Vec<Vec<String>> = candidates
        .iter()
        .map(|c| {
            let mut words = tokenize(c.file_path);
            if let Some(sym) = c.symbol_name {
                words.extend(tokenize(sym));
            }
            words.extend(tokenize(c.summary));
            words
        })
        .collect();

    let mut frequencies: HashMap<String, usize> = HashMap::new();
    for word in request_words {
        if frequencies.contains_key(word) {
            continue;
        }
        let count = candidate_words
            .iter()
            .filter(|words| words.iter().any(|w| w == word))
            .count();
        frequencies.insert(word.clone(), count);
    }
    frequencies
}

/// Scores how specifically a candidate file answers the continuation request, in `[0, MAX_SCORE]`.
///
/// `total_candidates` is the number of files being compared, which decides the weight of a word:
/// the rarer it is among them, the more it says about this particular file.
///
/// `request_words` may be anything the caller considers the ongoing intent — a sentence, a title,
/// or nothing at all. Empty input yields `0.0`, meaning "no evidence that this file is still
/// relevant", which is the honest answer when there is nothing to compare against.
pub fn score_candidate(
    request_words: &[String],
    candidates: &[Candidate<'_>],
    total_candidates: usize,
    file_index: usize,
) -> f64 {
    if request_words.is_empty() || total_candidates == 0 {
        return 0.0;
    }

    let unique_request_words: Vec<String> = {
        let mut seen: Vec<String> = Vec::with_capacity(request_words.len());
        for w in request_words {
            if !seen.contains(w) {
                seen.push(w.clone());
            }
        }
        seen
    };

    let frequencies = document_frequencies(&unique_request_words, candidates);
    let candidate = match candidates.get(file_index) {
        Some(c) => *c,
        None => return 0.0,
    };

    let mut file_words = tokenize(candidate.file_path);
    if let Some(sym) = candidate.symbol_name {
        file_words.extend(tokenize(sym));
    }
    file_words.extend(tokenize(candidate.summary));

    let total = total_candidates as f64;
    let mut matched_weight = 0.0;
    let mut available_weight = 0.0;

    for word in &unique_request_words {
        // A word in every file separates nothing; a word in no file cannot match. Both are
        // excluded from the denominator so they cannot drag the score down the way the old
        // fixed divisor did.
        let frequency = frequencies.get(word).copied().unwrap_or(0);
        if frequency == 0 || frequency == total_candidates {
            continue;
        }
        let specificity = (total / frequency as f64).ln() + 1.0;
        available_weight += specificity;
        if file_words.iter().any(|w| w == word) {
            matched_weight += specificity;
        }
    }

    if available_weight <= 0.0 {
        // Every request word either appears everywhere or nowhere: nothing in this request
        // distinguishes one file from another, so no file can claim relevance. Falling back to a
        // neutral value here would invent a signal that does not exist.
        return 0.0;
    }

    (matched_weight / available_weight).clamp(0.0, MAX_SCORE)
}

/// Splits a free-form request into the words used for scoring.
///
/// Exposed so every caller tokenizes the same way instead of each inventing its own filter.
pub fn request_words(text: &str) -> Vec<String> {
    tokenize(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates(paths: &[(&str, &str)]) -> Vec<Candidate<'static>> {
        // Leaked for test simplicity: the fixtures are static and the test process is short-lived.
        paths
            .iter()
            .map(|(path, summary)| Candidate {
                file_path: Box::leak(path.to_string().into_boxed_str()),
                symbol_name: None,
                summary: Box::leak(summary.to_string().into_boxed_str()),
            })
            .collect()
    }

    /// The defect this module replaces: a long request made every file look irrelevant because the
    /// divisor grew with the sentence. A file that plainly answers the request must score high no
    /// matter how many filler words surround it.
    #[test]
    fn a_long_request_does_not_dilute_the_score() {
        let cands = candidates(&[
            ("src/auth/login.rs", "handles login flow"),
            ("src/ui/button.rs", "renders a button"),
        ]);
        let request = request_words(
            "tolong perbaiki login yang error itu ya bro please fix the login thing because it keeps failing on me",
        );
        let login = score_candidate(&request, &cands, cands.len(), 0);
        let button = score_candidate(&request, &cands, cands.len(), 1);
        assert!(login > 0.8, "the login file answers the request, got {login}");
        assert_eq!(button, 0.0, "an unrelated file must score zero, got {button}");
    }

    /// A word present in every candidate identifies nothing, so it must not count as a match even
    /// though it genuinely appears in the request and in the file.
    #[test]
    fn a_word_shared_by_every_file_carries_no_weight() {
        let cands = candidates(&[
            ("src/parser.rs", "parse input"),
            ("src/writer.rs", "write output"),
        ]);
        // "rs" and "src" appear in both paths.
        let request = request_words("src writer");
        let writer = score_candidate(&request, &cands, cands.len(), 1);
        let parser = score_candidate(&request, &cands, cands.len(), 0);
        assert!(writer > parser, "the distinguishing word must decide: {writer} vs {parser}");
        assert_eq!(parser, 0.0);
    }

    /// Repeated request words must not inflate a match; saying "login login login" is still one
    /// topic.
    #[test]
    fn repetition_inside_the_request_changes_nothing() {
        let cands = candidates(&[
            ("src/auth/login.rs", "login flow"),
            ("src/ui/button.rs", "renders a button"),
        ]);
        let once = score_candidate(&request_words("login"), &cands, cands.len(), 0);
        let thrice = score_candidate(&request_words("login login login"), &cands, cands.len(), 0);
        assert_eq!(once, thrice);
    }

    /// Nothing to compare against means nothing to claim.
    #[test]
    fn an_empty_request_scores_zero() {
        let cands = candidates(&[("src/auth/login.rs", "login flow")]);
        assert_eq!(score_candidate(&[], &cands, cands.len(), 0), 0.0);
        assert_eq!(score_candidate(&request_words("   "), &cands, cands.len(), 0), 0.0);
    }

    /// Rarer words must count for more than common ones.
    ///
    /// Without this, any weighting scheme passes: a fixture where every request word appears in the
    /// same number of files cannot tell uniform weights from inverse frequency. Here one word is in
    /// two of three files and the other in one of three, so a file matching only the rare word must
    /// still outrank a file matching only the common one.
    #[test]
    fn a_rare_word_outweighs_a_common_one() {
        let cands = candidates(&[
            ("src/shared.rs", "shared helper"),
            ("src/other.rs", "shared helper too"),
            ("src/specific.rs", "unique handler"),
        ]);
        // "shared" is in 2 of 3 files; "unique" is in 1 of 3.
        let common_only = score_candidate(&request_words("shared"), &cands, cands.len(), 0);
        let rare_only = score_candidate(&request_words("unique"), &cands, cands.len(), 2);
        assert_eq!(common_only, 1.0, "a lone word that matches is a full match for that request");
        assert_eq!(rare_only, 1.0);

        // With both words requested, only the rare one distinguishes: the scores must differ.
        let request = request_words("shared unique");
        let common_file = score_candidate(&request, &cands, cands.len(), 0);
        let rare_file = score_candidate(&request, &cands, cands.len(), 2);
        assert!(
            rare_file > common_file,
            "matching only the rare word must beat matching only the common word: {rare_file} vs {common_file}"
        );
    }

    /// Scores share a column with `evidence_ranking`, so they must stay in the same range.
    #[test]
    fn scores_never_exceed_the_shared_ceiling() {
        let cands = candidates(&[("src/a.rs", "alpha"), ("src/b.rs", "beta"), ("src/c.rs", "gamma")]);
        let request = request_words("alpha beta gamma src");
        for index in 0..cands.len() {
            let score = score_candidate(&request, &cands, cands.len(), index);
            assert!((0.0..=MAX_SCORE).contains(&score), "index {index} produced {score}");
            assert!(!score.is_nan());
        }
    }
}
