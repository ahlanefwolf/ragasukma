-- Read-only audit of historical agent data.
--
-- Purpose: check, against what actually accumulated, whether the two defects that were fixed in
-- code ever took effect. Nothing here writes.
--
-- Run:  sqlite3 -header -column assets/ragasukma.db < assets/audit_history.sql
--
-- Every section states what a notable result would mean, so the numbers can be read without
-- re-deriving the reasoning later.

.headers on
.mode column

-- ============================================================================
-- 1. GOALS MARKED COMPLETED THAT NO VERIFICATION BACKS
--
-- These are goals the old conclusion path closed on assertion alone. A non-zero count here means
-- a turn that only talked (e.g. asked a clarifying question) was recorded as finished work.
-- ============================================================================
SELECT '--- 1. completed goals without a passing verification ---' AS section;

SELECT
    g.status,
    COUNT(*) AS goals,
    SUM(CASE WHEN v.id IS NULL THEN 1 ELSE 0 END) AS without_passing_verification
FROM goals g
LEFT JOIN goal_verifications v ON v.goal_id = g.id AND v.is_passed = 1
WHERE g.status IN ('completed', 'completed_unverified', 'failed')
GROUP BY g.status
ORDER BY g.status;

-- ============================================================================
-- 2. CONCLUSION TEXT THAT LOOKS LIKE A QUESTION RATHER THAN A RESULT
--
-- The old gate treated formatted text as a conclusion. A clarifying question is formatted too.
-- "Ends with ?" is a coarse signal, but it is enough to show whether the pattern occurred.
-- ============================================================================
SELECT '--- 2. conclusions ending in a question mark ---' AS section;

SELECT
    COUNT(*) AS conclusions_total,
    SUM(CASE WHEN TRIM(conclusion) LIKE '%?' THEN 1 ELSE 0 END) AS ending_in_question
FROM goals
WHERE conclusion IS NOT NULL AND TRIM(conclusion) != '';

SELECT 'goals whose conclusion is a question (id, status, first 80 chars):' AS detail;

SELECT
    id,
    status,
    SUBSTR(REPLACE(conclusion, CHAR(10), ' '), 1, 80) AS conclusion_preview
FROM goals
WHERE conclusion IS NOT NULL
  AND TRIM(conclusion) LIKE '%?'
ORDER BY created_at DESC
LIMIT 15;

-- ============================================================================
-- 3. SUCCESS CRITERIA THAT MERELY COPY THE DEMAND
--
-- The old code duplicated the demand into success_criteria, printing the same specification twice
-- on every turn. Counts how many stored goals still carry that copy.
-- ============================================================================
SELECT '--- 3. success_criteria duplicating the demand ---' AS section;

SELECT
    COUNT(*) AS goals_total,
    SUM(CASE WHEN success_criteria IS NOT NULL AND success_criteria != '[]' THEN 1 ELSE 0 END) AS have_criteria,
    SUM(CASE
            WHEN success_criteria IS NULL OR success_criteria = '[]' THEN 0
            WHEN TRIM(success_criteria) = '["' || REPLACE(TRIM(demand), '"', '\"') || '"]' THEN 1
            ELSE 0
        END) AS criteria_equal_demand
FROM goals;

-- ============================================================================
-- 4. EVIDENCE SCORE DISTRIBUTION
--
-- Scores below the plain-read weight (0.4) mean the old keyword rule dragged inheritance down and
-- those files lost the prompt cap to whatever was read last. Also shows how much evidence is
-- orphaned (linked to no goal at all).
-- ============================================================================
SELECT '--- 4. goal_evidence relevance_score distribution ---' AS section;

SELECT
    COUNT(*) AS links,
    ROUND(MIN(relevance_score), 5) AS min_score,
    ROUND(MAX(relevance_score), 5) AS max_score,
    ROUND(AVG(relevance_score), 5) AS avg_score,
    SUM(CASE WHEN relevance_score < 0.4 THEN 1 ELSE 0 END) AS below_plain_read,
    SUM(CASE WHEN relevance_score < 0.01 THEN 1 ELSE 0 END) AS near_zero
FROM goal_evidence;

SELECT 'score buckets:' AS detail;

SELECT
    CASE
        WHEN relevance_score < 0.01 THEN 'a. < 0.01 (effectively zero)'
        WHEN relevance_score < 0.4 THEN 'b. 0.01 - 0.39 (below a plain read)'
        WHEN relevance_score < 0.7 THEN 'c. 0.40 - 0.69 (read-level)'
        WHEN relevance_score < 1.0 THEN 'd. 0.70 - 0.99 (inherited continuation / edit)'
        ELSE 'e. 1.00 (fully relevant)'
    END AS bucket,
    COUNT(*) AS rows
FROM goal_evidence
GROUP BY bucket
ORDER BY bucket;

SELECT '--- 5. evidence linked to no goal (orphans) ---' AS section;

SELECT
    (SELECT COUNT(*) FROM evidence_cache) AS evidence_rows,
    (SELECT COUNT(DISTINCT evidence_id) FROM goal_evidence) AS linked_evidence,
    (SELECT COUNT(*) FROM evidence_cache e
       WHERE NOT EXISTS (SELECT 1 FROM goal_evidence ge WHERE ge.evidence_id = e.id)) AS orphan_evidence,
    (SELECT COUNT(*) FROM evidence_cache WHERE is_stale = 1) AS stale_evidence;
