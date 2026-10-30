//! Evidence relevance ranking.
//!
//! `goal_evidence.relevance_score` decides which cached files survive the prompt cap, so the score
//! has to reflect how much a file actually mattered to the goal.
//!
//! The weights describe *evidence of work*, not file kinds or languages:
//! - a file the agent **modified or created** is part of the solution, so it ranks highest;
//! - a file the agent **read** is context: useful, but not proof that it matters.
//!
//! # Recency
//!
//! A long session reads and edits many files. Without a time component, a file edited fifty turns
//! ago and one edited just now rank identically, so the prompt cap keeps stale context alive.
//!
//! Recency is applied as a **floored decay**: the score sinks with age but never below a share of
//! its origin weight. A naive newest-wins ranking would evict project configuration that was read
//! once at the start of the session and never touched again, which is exactly the context the
//! agent still needs. The floor keeps old-but-load-bearing files in the running while letting
//! recently touched files win the scarce prompt slots.
//!
//! Staleness is not scored here. Stale rows are filtered out of the prompt entirely by
//! `get_active_evidences_for_goal` (`is_stale = 0`).

/// An evidence file the agent edited, created, or generated via a command.
pub const SCORE_MODIFIED: f64 = 1.0;

/// An evidence file the agent only inspected.
pub const SCORE_READ: f64 = 0.4;

/// How long, in seconds, it takes for recency to wear off by one full decay step.
///
/// One hour is long enough that a working session keeps its context intact, and short enough that
/// yesterday's exploration stops competing with what the agent is doing now.
pub const RECENCY_HALF_LIFE_SECS: f64 = 3_600.0;

/// The fraction of the origin weight that survives no matter how old an entry is.
///
/// This is the floor that protects early-session context. At `0.5`, an ancient edited file still
/// outranks a freshly read one (0.5 vs 0.4), and an ancient read file (0.2) still stays above zero
/// so it remains a candidate when the prompt has room.
pub const RECENCY_FLOOR_SHARE: f64 = 0.5;

/// What the agent did with a file inside the current goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceOrigin {
    /// Read or inspected only.
    Read,
    /// Written: edited, created, or produced by a command.
    Modified,
}

/// Base weight for an origin, before any recency is applied.
pub fn base_score(origin: EvidenceOrigin) -> f64 {
    match origin {
        EvidenceOrigin::Modified => SCORE_MODIFIED,
        EvidenceOrigin::Read => SCORE_READ,
    }
}

/// Recency multiplier in `[RECENCY_FLOOR_SHARE, 1.0]`.
///
/// The decay is linear rather than exponential so the ordering stays easy to reason about and the
/// floor is reached predictably instead of asymptotically.
pub fn recency_factor(age_secs: i64) -> f64 {
    if age_secs <= 0 {
        return 1.0;
    }
    let age = age_secs as f64;
    let decayed = 1.0 - (1.0 - RECENCY_FLOOR_SHARE) * (age / RECENCY_HALF_LIFE_SECS).min(1.0);
    decayed.max(RECENCY_FLOOR_SHARE)
}

/// Ranks one evidence entry for a goal, combining work type and age.
///
/// `age_secs` is how long ago the entry was observed; pass `0` when freshness is unknown, which
/// returns the full base score and never demotes an entry on missing information.
///
/// Kept pure (no database access) so the ordering policy is unit-testable on its own.
pub fn rank_evidence(origin: EvidenceOrigin, age_secs: i64) -> f64 {
    base_score(origin) * recency_factor(age_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_files_outrank_read_files_at_equal_age() {
        assert!(rank_evidence(EvidenceOrigin::Modified, 0) > rank_evidence(EvidenceOrigin::Read, 0));
        assert!(
            rank_evidence(EvidenceOrigin::Modified, i64::MAX / 2)
                > rank_evidence(EvidenceOrigin::Read, i64::MAX / 2),
            "an edited file must stay ahead of a read file however old both are"
        );
    }

    #[test]
    fn recent_entries_outrank_old_entries_of_the_same_origin() {
        for origin in [EvidenceOrigin::Read, EvidenceOrigin::Modified] {
            assert!(rank_evidence(origin, 0) > rank_evidence(origin, 600));
            assert!(rank_evidence(origin, 600) > rank_evidence(origin, 10_000));
        }
    }

    #[test]
    fn recency_never_evicts_old_context_completely() {
        // A very old edited file must still outrank a freshly read file: early configuration must
        // not be pushed out by whatever the agent happened to open most recently.
        let ancient_edit = rank_evidence(EvidenceOrigin::Modified, i64::MAX / 2);
        let fresh_read = rank_evidence(EvidenceOrigin::Read, 0);
        assert!(
            ancient_edit > fresh_read,
            "ancient edit {ancient_edit} must still beat fresh read {fresh_read}"
        );
        assert!(ancient_edit > 0.0, "floor must keep old evidence non-zero");
    }

    #[test]
    fn unknown_age_does_not_demote() {
        for origin in [EvidenceOrigin::Read, EvidenceOrigin::Modified] {
            assert_eq!(rank_evidence(origin, 0), base_score(origin));
        }
        // Negative ages (clock skew) are treated as fresh rather than inflating the score.
        assert_eq!(rank_evidence(EvidenceOrigin::Modified, -500), SCORE_MODIFIED);
    }

    #[test]
    fn scores_stay_within_the_shared_range() {
        // `reuse_evidence_from_parent` also writes into this column using a keyword ratio in
        // `[0,1]`, so ranking must never exceed that range.
        for origin in [EvidenceOrigin::Read, EvidenceOrigin::Modified] {
            for age in [0i64, 1, 600, 3_600, 86_400, i64::MAX / 2] {
                let score = rank_evidence(origin, age);
                assert!((0.0..=1.0).contains(&score), "{origin:?}/{age} produced {score}");
                assert!(!score.is_nan(), "{origin:?}/{age} must never yield NaN");
            }
        }
    }

    #[test]
    fn ordering_is_total_and_stable() {
        let mut scores: Vec<f64> = [EvidenceOrigin::Read, EvidenceOrigin::Modified]
            .into_iter()
            .flat_map(|o| [0i64, 600, 86_400].map(move |a| rank_evidence(o, a)))
            .collect();
        scores.sort_by(|a, b| b.total_cmp(a));
        assert_eq!(scores.first().copied(), Some(SCORE_MODIFIED));
        assert!(scores.iter().all(|s| s.is_finite()));
    }
}
