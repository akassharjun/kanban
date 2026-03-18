use crate::state::AppState;
use crate::models::Issue;
use crate::commands::triage;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIssue {
    pub title: String,
    pub description: String,
    pub suggested_priority: Option<String>,
    pub suggested_label_ids: Vec<i64>,
    pub suggested_assignee_id: Option<i64>,
}

/// Parse natural language text into a structured issue.
/// First sentence or first line becomes the title; rest becomes description.
/// Then run triage logic for priority/labels/assignee.
pub fn parse_nl_text(text: &str) -> (String, String) {
    let trimmed = text.trim();

    // Try to split on first sentence boundary (. or ! or ?)
    // But only if there's more text after it
    if let Some(idx) = find_sentence_end(trimmed) {
        let title_part = &trimmed[..=idx];
        let desc_part = trimmed[idx + 1..].trim();
        let title = clean_title(title_part);
        let description = if desc_part.is_empty() {
            trimmed.to_string()
        } else {
            trimmed.to_string()
        };
        return (title, description);
    }

    // Try to split on first newline
    if let Some(idx) = trimmed.find('\n') {
        let title = clean_title(&trimmed[..idx]);
        return (title, trimmed.to_string());
    }

    // Single line: use entire text as title
    let title = clean_title(trimmed);
    (title, trimmed.to_string())
}

fn find_sentence_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if (b == b'.' || b == b'!' || b == b'?') && i > 10 {
            // Make sure there's more text after
            if i + 1 < bytes.len() {
                return Some(i);
            }
        }
    }
    None
}

/// Clean up and capitalize a title extracted from natural language.
fn clean_title(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Capitalize first letter
    let mut chars = trimmed.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            format!("{}{}", upper, chars.as_str())
        }
    }
}

#[tauri::command]
pub fn parse_natural_language(
    state: State<'_, AppState>,
    project_id: i64,
    text: String,
) -> Result<ParsedIssue, String> {
    state.rt.block_on(async {
        let (title, description) = parse_nl_text(&text);
        if title.is_empty() {
            return Err("Could not extract a title from the text".to_string());
        }

        let suggestion = triage::triage_logic(
            &state.pool,
            project_id,
            &title,
            Some(&description),
        )
        .await?;

        Ok(ParsedIssue {
            title,
            description,
            suggested_priority: suggestion.suggested_priority,
            suggested_label_ids: suggestion.suggested_label_ids,
            suggested_assignee_id: suggestion.suggested_assignee_id,
        })
    })
}

#[tauri::command]
pub fn create_from_natural_language(
    state: State<'_, AppState>,
    project_id: i64,
    text: String,
    status_id: i64,
) -> Result<Issue, String> {
    state.rt.block_on(async {
        let (title, description) = parse_nl_text(&text);
        if title.is_empty() {
            return Err("Could not extract a title from the text".to_string());
        }

        let suggestion = triage::triage_logic(
            &state.pool,
            project_id,
            &title,
            Some(&description),
        )
        .await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let priority = suggestion.suggested_priority.as_deref().unwrap_or("none");

        let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;

        let (counter, prefix): (i64, String) = sqlx::query_as(
            "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
        )
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        let identifier = format!("{}-{}", prefix, counter);

        let max_pos: Option<f64> = sqlx::query_scalar(
            "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
        )
        .bind(project_id)
        .bind(status_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        let position = max_pos.unwrap_or(-1.0) + 1.0;

        let issue_id: i64 = sqlx::query_scalar(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
        )
        .bind(project_id)
        .bind(&identifier)
        .bind(&title)
        .bind(&description)
        .bind(status_id)
        .bind(priority)
        .bind(suggestion.suggested_assignee_id)
        .bind(None::<i64>) // parent_id
        .bind(position)
        .bind(&now)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Apply labels
        for lid in &suggestion.suggested_label_ids {
            let _ = sqlx::query(
                "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING"
            )
            .bind(issue_id)
            .bind(*lid)
            .execute(&mut *tx)
            .await;
        }

        let issue: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(issue)
    })
}
