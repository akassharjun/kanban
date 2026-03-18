use crate::state::AppState;
use crate::models::{Issue, Label, Member};
use tauri::State;
use serde::{Deserialize, Serialize};

/// Keywords mapped to priority levels for rule-based triage.
const URGENT_KEYWORDS: &[&str] = &[
    "crash", "security", "vulnerability", "data loss", "broken", "emergency",
    "critical", "exploit", "outage", "corruption", "blocked", "blocker",
    "production down", "p0", "hotfix", "regression",
];

const HIGH_KEYWORDS: &[&str] = &[
    "bug", "error", "fail", "failure", "wrong", "incorrect", "fix",
    "degraded", "slow", "timeout", "leak", "memory", "performance",
    "breaking", "p1", "important", "severe",
];

const MEDIUM_KEYWORDS: &[&str] = &[
    "improve", "enhancement", "update", "refactor", "add", "implement",
    "feature", "support", "integrate", "extend", "missing", "needed",
    "request", "p2",
];

const LOW_KEYWORDS: &[&str] = &[
    "nice to have", "cosmetic", "minor", "typo", "documentation", "docs",
    "cleanup", "style", "formatting", "chore", "polish", "trivial",
    "suggestion", "p3", "p4",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageSuggestion {
    pub suggested_priority: Option<String>,
    pub suggested_label_ids: Vec<i64>,
    pub suggested_assignee_id: Option<i64>,
    pub suggested_epic_id: Option<i64>,
    pub confidence: f64,
    pub reasoning: String,
}

/// Compute priority from text using keyword matching. Returns (priority, matched_keywords).
fn suggest_priority(text: &str) -> (Option<String>, Vec<String>) {
    let lower = text.to_lowercase();
    let mut matched = Vec::new();

    for kw in URGENT_KEYWORDS {
        if lower.contains(kw) {
            matched.push(kw.to_string());
        }
    }
    if !matched.is_empty() {
        return (Some("urgent".to_string()), matched);
    }

    for kw in HIGH_KEYWORDS {
        if lower.contains(kw) {
            matched.push(kw.to_string());
        }
    }
    if !matched.is_empty() {
        return (Some("high".to_string()), matched);
    }

    for kw in MEDIUM_KEYWORDS {
        if lower.contains(kw) {
            matched.push(kw.to_string());
        }
    }
    if !matched.is_empty() {
        return (Some("medium".to_string()), matched);
    }

    for kw in LOW_KEYWORDS {
        if lower.contains(kw) {
            matched.push(kw.to_string());
        }
    }
    if !matched.is_empty() {
        return (Some("low".to_string()), matched);
    }

    (None, vec![])
}

/// Match labels whose names appear in the given text.
fn match_labels(text: &str, labels: &[Label]) -> Vec<i64> {
    let lower = text.to_lowercase();
    labels
        .iter()
        .filter(|l| lower.contains(&l.name.to_lowercase()))
        .map(|l| l.id)
        .collect()
}

/// Find the best assignee: the member who has completed the most issues with matching labels.
/// Falls back to the member with lowest current workload (fewest non-completed issues).
async fn suggest_assignee(
    pool: &sqlx::AnyPool,
    project_id: i64,
    label_ids: &[i64],
) -> Result<Option<i64>, String> {
    // Strategy 1: If we have matching labels, find who completed most issues with those labels
    if !label_ids.is_empty() {
        let placeholders: Vec<String> = label_ids.iter().enumerate()
            .map(|(i, _)| format!("${}", i + 2))
            .collect();
        let query = format!(
            "SELECT i.assignee_id FROM issues i \
             JOIN issue_labels il ON i.id = il.issue_id \
             JOIN statuses s ON i.status_id = s.id \
             WHERE i.project_id = $1 AND s.category = 'completed' \
             AND il.label_id IN ({}) AND i.assignee_id IS NOT NULL \
             GROUP BY i.assignee_id ORDER BY COUNT(*) DESC LIMIT 1",
            placeholders.join(", ")
        );
        let mut q = sqlx::query_scalar::<_, i64>(&query).bind(project_id);
        for lid in label_ids {
            q = q.bind(*lid);
        }
        if let Ok(Some(aid)) = q.fetch_optional(pool).await.map_err(|e| e.to_string()) {
            return Ok(Some(aid));
        }
    }

    // Strategy 2: Lowest workload (fewest in-progress issues)
    let result: Option<i64> = sqlx::query_scalar(
        "SELECT m.id FROM members m \
         LEFT JOIN issues i ON i.assignee_id = m.id AND i.project_id = $1 \
         LEFT JOIN statuses s ON i.status_id = s.id AND s.category IN ('unstarted', 'started') \
         GROUP BY m.id ORDER BY COUNT(i.id) ASC LIMIT 1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Find an epic (parent issue) whose title/description shares words with the issue text.
async fn suggest_epic(
    pool: &sqlx::AnyPool,
    project_id: i64,
    text: &str,
) -> Result<Option<i64>, String> {
    // Epics are issues that have children (i.e., are parent_id of other issues)
    // or issues with no parent themselves that are large (heuristic: in backlog/todo with description)
    let epics = sqlx::query_as::<_, Issue>(
        "SELECT DISTINCT p.* FROM issues p \
         WHERE p.project_id = $1 AND p.parent_id IS NULL \
         AND (EXISTS (SELECT 1 FROM issues c WHERE c.parent_id = p.id) \
              OR p.description IS NOT NULL) \
         ORDER BY p.updated_at DESC LIMIT 50"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if epics.is_empty() {
        return Ok(None);
    }

    let text_lower = text.to_lowercase();
    let text_words: std::collections::HashSet<String> = text_lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_string())
        .collect();

    let mut best_id = None;
    let mut best_score = 0usize;

    for epic in &epics {
        let epic_text = format!(
            "{} {}",
            epic.title,
            epic.description.as_deref().unwrap_or("")
        )
        .to_lowercase();
        let epic_words: std::collections::HashSet<String> = epic_text
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect();

        let overlap = text_words.intersection(&epic_words).count();
        if overlap > best_score && overlap >= 2 {
            best_score = overlap;
            best_id = Some(epic.id);
        }
    }

    Ok(best_id)
}

/// Compute a confidence score based on how many fields we could suggest.
fn compute_confidence(suggestion: &TriageSuggestion) -> f64 {
    let mut score = 0.0;
    let total = 4.0;

    if suggestion.suggested_priority.is_some() {
        score += 1.0;
    }
    if !suggestion.suggested_label_ids.is_empty() {
        score += 1.0;
    }
    if suggestion.suggested_assignee_id.is_some() {
        score += 1.0;
    }
    if suggestion.suggested_epic_id.is_some() {
        score += 1.0;
    }

    score / total
}

/// Core triage logic: given project, title, and description, produce suggestions.
pub async fn triage_logic(
    pool: &sqlx::AnyPool,
    project_id: i64,
    title: &str,
    description: Option<&str>,
) -> Result<TriageSuggestion, String> {
    let full_text = format!("{} {}", title, description.unwrap_or(""));
    let mut reasons = Vec::new();

    // 1. Priority
    let (priority, matched_kw) = suggest_priority(&full_text);
    if let Some(ref p) = priority {
        reasons.push(format!(
            "Priority '{}' suggested based on keywords: {}",
            p,
            matched_kw.join(", ")
        ));
    }

    // 2. Labels
    let labels = sqlx::query_as::<_, Label>(
        "SELECT * FROM labels WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let label_ids = match_labels(&full_text, &labels);
    if !label_ids.is_empty() {
        let names: Vec<String> = labels
            .iter()
            .filter(|l| label_ids.contains(&l.id))
            .map(|l| l.name.clone())
            .collect();
        reasons.push(format!("Labels matched: {}", names.join(", ")));
    }

    // 3. Assignee
    let assignee_id = suggest_assignee(pool, project_id, &label_ids).await?;
    if let Some(aid) = assignee_id {
        let member: Option<Member> = sqlx::query_as(
            "SELECT * FROM members WHERE id = $1"
        )
        .bind(aid)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        if let Some(m) = member {
            reasons.push(format!(
                "Assignee '{}' suggested based on workload/expertise",
                m.display_name.as_deref().unwrap_or(&m.name)
            ));
        }
    }

    // 4. Epic
    let epic_id = suggest_epic(pool, project_id, &full_text).await?;
    if let Some(eid) = epic_id {
        let epic: Option<Issue> = sqlx::query_as(
            "SELECT * FROM issues WHERE id = $1"
        )
        .bind(eid)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        if let Some(e) = epic {
            reasons.push(format!(
                "Epic '{}' ({}) suggested based on keyword similarity",
                e.title, e.identifier
            ));
        }
    }

    if reasons.is_empty() {
        reasons.push("No strong signals found in title/description".to_string());
    }

    let mut suggestion = TriageSuggestion {
        suggested_priority: priority,
        suggested_label_ids: label_ids,
        suggested_assignee_id: assignee_id,
        suggested_epic_id: epic_id,
        confidence: 0.0,
        reasoning: reasons.join(". "),
    };
    suggestion.confidence = compute_confidence(&suggestion);

    Ok(suggestion)
}

#[tauri::command]
pub fn triage_issue(
    state: State<'_, AppState>,
    project_id: i64,
    title: String,
    description: Option<String>,
) -> Result<TriageSuggestion, String> {
    state.rt.block_on(async {
        triage_logic(&state.pool, project_id, &title, description.as_deref()).await
    })
}

#[tauri::command]
pub fn auto_triage_and_apply(
    state: State<'_, AppState>,
    issue_id: i64,
) -> Result<TriageSuggestion, String> {
    state.rt.block_on(async {
        let issue: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let suggestion = triage_logic(
            &state.pool,
            issue.project_id,
            &issue.title,
            issue.description.as_deref(),
        )
        .await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Apply suggestions with confidence > 0.25 (at least one field matched)
        if suggestion.confidence > 0.0 {
            if let Some(ref p) = suggestion.suggested_priority {
                if issue.priority == "none" {
                    sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                        .bind(p)
                        .bind(&now)
                        .bind(issue_id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }

            if let Some(aid) = suggestion.suggested_assignee_id {
                if issue.assignee_id.is_none() {
                    sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                        .bind(aid)
                        .bind(&now)
                        .bind(issue_id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }

            if let Some(eid) = suggestion.suggested_epic_id {
                if issue.parent_id.is_none() {
                    sqlx::query("UPDATE issues SET parent_id = $1, updated_at = $2 WHERE id = $3")
                        .bind(eid)
                        .bind(&now)
                        .bind(issue_id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }

            if !suggestion.suggested_label_ids.is_empty() {
                for lid in &suggestion.suggested_label_ids {
                    let _ = sqlx::query(
                        "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING"
                    )
                    .bind(issue_id)
                    .bind(*lid)
                    .execute(&state.pool)
                    .await;
                }
            }

            // Log activity
            sqlx::query(
                "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(issue_id)
            .bind("auto_triage")
            .bind(None::<String>)
            .bind(&suggestion.reasoning)
            .bind(&now)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(suggestion)
    })
}
