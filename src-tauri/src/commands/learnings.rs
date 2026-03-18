use crate::models::agent::TaskLearning;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Deserialize)]
pub struct RecordLearningInput {
    pub task_identifier: String,
    pub agent_id: String,
    pub outcome: String,
    pub approach_summary: String,
    pub key_insight: Option<String>,
    pub pitfalls: Option<Vec<String>>,
    pub effective_patterns: Option<Vec<String>>,
    pub relevant_files: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimilarTaskResult {
    pub learning: TaskLearning,
    pub similarity_score: f64,
    pub issue_title: String,
    pub issue_identifier: String,
}

#[tauri::command]
pub fn record_learning(state: State<AppState>, input: RecordLearningInput) -> Result<TaskLearning, String> {
    if input.approach_summary.trim().is_empty() {
        return Err("approach_summary cannot be empty".to_string());
    }
    let valid_outcomes = ["success", "failure", "partial"];
    if !valid_outcomes.contains(&input.outcome.as_str()) {
        return Err(format!("outcome must be one of: {}", valid_outcomes.join(", ")));
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let pitfalls = serde_json::to_string(&input.pitfalls.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());
        let effective_patterns = serde_json::to_string(&input.effective_patterns.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());
        let relevant_files = serde_json::to_string(&input.relevant_files.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());
        let tags = serde_json::to_string(&input.tags.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO task_learnings (task_identifier, agent_id, outcome, approach_summary, key_insight, pitfalls, effective_patterns, relevant_files, tags, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id"
        )
        .bind(&input.task_identifier)
        .bind(&input.agent_id)
        .bind(&input.outcome)
        .bind(&input.approach_summary)
        .bind(&input.key_insight)
        .bind(&pitfalls)
        .bind(&effective_patterns)
        .bind(&relevant_files)
        .bind(&tags)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, TaskLearning>("SELECT * FROM task_learnings WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

/// Compute Jaccard similarity between two sets of lowercase strings
fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let set_a: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: std::collections::HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;
    if union == 0.0 { 0.0 } else { intersection / union }
}

/// Extract words from a string for similarity matching
fn extract_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

#[tauri::command]
pub fn find_similar_learnings(
    state: State<AppState>,
    project_id: i64,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    limit: Option<i32>,
) -> Result<Vec<SimilarTaskResult>, String> {
    let limit = limit.unwrap_or(5);
    state.rt.block_on(async {
        // Fetch all learnings for the project by joining with issues
        let learnings = sqlx::query_as::<_, TaskLearning>(
            "SELECT tl.* FROM task_learnings tl JOIN issues i ON tl.task_identifier = i.identifier WHERE i.project_id = $1 ORDER BY tl.created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Prepare search terms
        let search_tags: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        let title_words = extract_words(&title);
        let desc_words = description.as_deref().map(|d| extract_words(d)).unwrap_or_default();
        let all_search_words: Vec<String> = title_words.iter().chain(desc_words.iter()).cloned().collect();

        let mut results: Vec<SimilarTaskResult> = Vec::new();

        for learning in learnings {
            // Parse learning tags
            let learning_tags: Vec<String> = serde_json::from_str(&learning.tags)
                .unwrap_or_else(|_| Vec::<String>::new())
                .iter()
                .map(|t: &String| t.to_lowercase())
                .collect();

            // Tag similarity (weight: 0.6)
            let tag_sim = jaccard_similarity(&search_tags, &learning_tags);

            // Title/description word overlap (weight: 0.4)
            let learning_words = extract_words(&learning.approach_summary);
            let word_sim = jaccard_similarity(&all_search_words, &learning_words);

            let similarity_score = tag_sim * 0.6 + word_sim * 0.4;

            if similarity_score > 0.0 {
                // Fetch issue title and identifier
                let issue_info: Option<(String, String)> = sqlx::query_as(
                    "SELECT title, identifier FROM issues WHERE identifier = $1"
                )
                .bind(&learning.task_identifier)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| e.to_string())?;

                let (issue_title, issue_identifier) = issue_info
                    .unwrap_or_else(|| (String::new(), learning.task_identifier.clone()));

                results.push(SimilarTaskResult {
                    learning,
                    similarity_score,
                    issue_title,
                    issue_identifier,
                });
            }
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit as usize);

        Ok(results)
    })
}

#[tauri::command]
pub fn list_learnings(
    state: State<AppState>,
    project_id: i64,
    outcome: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<TaskLearning>, String> {
    let limit = limit.unwrap_or(50);
    state.rt.block_on(async {
        if let Some(ref outcome) = outcome {
            sqlx::query_as::<_, TaskLearning>(
                "SELECT tl.* FROM task_learnings tl JOIN issues i ON tl.task_identifier = i.identifier WHERE i.project_id = $1 AND tl.outcome = $2 ORDER BY tl.created_at DESC LIMIT $3"
            )
            .bind(project_id)
            .bind(outcome)
            .bind(limit)
            .fetch_all(&state.pool)
            .await
        } else {
            sqlx::query_as::<_, TaskLearning>(
                "SELECT tl.* FROM task_learnings tl JOIN issues i ON tl.task_identifier = i.identifier WHERE i.project_id = $1 ORDER BY tl.created_at DESC LIMIT $2"
            )
            .bind(project_id)
            .bind(limit)
            .fetch_all(&state.pool)
            .await
        }
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_learnings_for_task(state: State<AppState>, task_identifier: String) -> Result<Vec<TaskLearning>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, TaskLearning>(
            "SELECT * FROM task_learnings WHERE task_identifier = $1 ORDER BY created_at DESC"
        )
        .bind(&task_identifier)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}
