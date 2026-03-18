use crate::state::AppState;
use crate::models::Issue;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedTask {
    pub title: String,
    pub description: Option<String>,
    pub suggested_priority: Option<String>,
    pub suggested_labels: Vec<String>,
}

/// Parse markdown checklists: `- [ ] item` or `- [x] item`
fn parse_checklists(text: &str) -> Vec<DecomposedTask> {
    let mut tasks = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // Match `- [ ] text` or `- [x] text` or `* [ ] text`
        let item = if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            Some(&trimmed[6..])
        } else if trimmed.starts_with("* [ ] ") || trimmed.starts_with("* [x] ") || trimmed.starts_with("* [X] ") {
            Some(&trimmed[6..])
        } else {
            None
        };

        if let Some(item_text) = item {
            let title = item_text.trim().to_string();
            if !title.is_empty() {
                tasks.push(DecomposedTask {
                    title,
                    description: None,
                    suggested_priority: None,
                    suggested_labels: vec![],
                });
            }
        }
    }
    tasks
}

/// Parse numbered lists: `1. item`, `2. item`, etc.
fn parse_numbered_lists(text: &str) -> Vec<DecomposedTask> {
    let mut tasks = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // Match `N. text` or `N) text`
        if let Some(rest) = try_parse_numbered_line(trimmed) {
            let title = rest.trim().to_string();
            if !title.is_empty() {
                tasks.push(DecomposedTask {
                    title,
                    description: None,
                    suggested_priority: None,
                    suggested_labels: vec![],
                });
            }
        }
    }
    tasks
}

fn try_parse_numbered_line(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut i = 0;
    // Skip leading digits
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i >= bytes.len() {
        return None;
    }
    // Expect `.` or `)` followed by space
    if (bytes[i] == b'.' || bytes[i] == b')') && i + 1 < bytes.len() && bytes[i + 1] == b' ' {
        Some(&line[i + 2..])
    } else {
        None
    }
}

/// Parse markdown headings: `## Section` → sub-issue per section.
/// Content under each heading becomes the description.
fn parse_headings(text: &str) -> Vec<DecomposedTask> {
    let mut tasks = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_desc = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            // Save previous section
            if let Some(title) = current_title.take() {
                tasks.push(DecomposedTask {
                    title,
                    description: if current_desc.trim().is_empty() {
                        None
                    } else {
                        Some(current_desc.trim().to_string())
                    },
                    suggested_priority: None,
                    suggested_labels: vec![],
                });
            }
            current_desc.clear();
            let heading_text = trimmed
                .trim_start_matches('#')
                .trim()
                .to_string();
            if !heading_text.is_empty() {
                current_title = Some(heading_text);
            }
        } else if current_title.is_some() {
            current_desc.push_str(line);
            current_desc.push('\n');
        }
    }

    // Save last section
    if let Some(title) = current_title {
        tasks.push(DecomposedTask {
            title,
            description: if current_desc.trim().is_empty() {
                None
            } else {
                Some(current_desc.trim().to_string())
            },
            suggested_priority: None,
            suggested_labels: vec![],
        });
    }

    tasks
}

/// Core decomposition logic: try checklists first, then numbered lists, then headings.
pub fn decompose_text(text: &str) -> Vec<DecomposedTask> {
    let tasks = parse_checklists(text);
    if !tasks.is_empty() {
        return tasks;
    }

    let tasks = parse_numbered_lists(text);
    if !tasks.is_empty() {
        return tasks;
    }

    parse_headings(text)
}

#[tauri::command]
pub fn decompose_issue(
    state: State<'_, AppState>,
    issue_id: i64,
) -> Result<Vec<DecomposedTask>, String> {
    state.rt.block_on(async {
        let issue: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let text = issue.description.as_deref().unwrap_or("");
        if text.is_empty() {
            return Err("Issue has no description to decompose".to_string());
        }

        let tasks = decompose_text(text);
        if tasks.is_empty() {
            return Err("No decomposable structure found (checklists, numbered lists, or headings)".to_string());
        }

        Ok(tasks)
    })
}

#[tauri::command]
pub fn apply_decomposition(
    state: State<'_, AppState>,
    issue_id: i64,
) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let issue: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let text = issue.description.as_deref().unwrap_or("");
        if text.is_empty() {
            return Err("Issue has no description to decompose".to_string());
        }

        let tasks = decompose_text(text);
        if tasks.is_empty() {
            return Err("No decomposable structure found".to_string());
        }

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let mut created_issues = Vec::new();

        for (idx, task) in tasks.iter().enumerate() {
            let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;

            // Increment project counter
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            )
            .bind(issue.project_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
            )
            .bind(issue.project_id)
            .bind(issue.status_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0 + idx as f64;

            let priority = task.suggested_priority.as_deref().unwrap_or(&issue.priority);

            let sub_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
            )
            .bind(issue.project_id)
            .bind(&identifier)
            .bind(&task.title)
            .bind(&task.description)
            .bind(issue.status_id)
            .bind(priority)
            .bind(issue.assignee_id)
            .bind(issue_id) // parent = the decomposed issue
            .bind(position)
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let sub_issue: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
                .bind(sub_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;
            created_issues.push(sub_issue);
        }

        // Log activity on parent
        sqlx::query(
            "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(issue_id)
        .bind("decomposition")
        .bind(None::<String>)
        .bind(format!("Decomposed into {} sub-issues", created_issues.len()))
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(created_issues)
    })
}
