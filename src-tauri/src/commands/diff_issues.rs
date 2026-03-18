use crate::models::Issue;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct DiffIssueInput {
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub line_range: Option<String>,
    pub severity: String,
    pub status_id: Option<i64>,
    pub assignee_id: Option<i64>,
}

#[tauri::command]
pub fn create_issue_from_diff(state: State<AppState>, input: DiffIssueInput) -> Result<Issue, String> {
    state.rt.block_on(async {
        create_issue_from_diff_async(&state.pool, input).await
    })
}

pub async fn create_issue_from_diff_async(
    pool: &sqlx::AnyPool,
    input: DiffIssueInput,
) -> Result<Issue, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Atomically increment counter and get new value + prefix
    let (counter, prefix): (i64, String) = sqlx::query_as(
        "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
    )
    .bind(input.project_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let identifier = format!("{}-{}", prefix, counter);

    // Determine status_id: use provided or find first unstarted status
    let status_id = if let Some(sid) = input.status_id {
        sid
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM statuses WHERE project_id = $1 AND category = 'unstarted' ORDER BY position ASC LIMIT 1",
        )
        .bind(input.project_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
    };

    // Map severity to priority
    let priority = match input.severity.as_str() {
        "bug" => "high",
        "improvement" => "medium",
        "todo" => "low",
        _ => "none",
    };

    // Build description with file context
    let mut desc = String::new();
    desc.push_str(&format!("**File:** `{}`\n", input.file_path));
    if let Some(ref range) = input.line_range {
        desc.push_str(&format!("**Lines:** {}\n", range));
    }
    desc.push_str(&format!("**Severity:** {}\n", input.severity));
    if let Some(ref d) = input.description {
        desc.push_str(&format!("\n---\n\n{}", d));
    }

    // Get max position
    let max_pos: Option<f64> = sqlx::query_scalar(
        "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
    )
    .bind(input.project_id)
    .bind(status_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let position = max_pos.unwrap_or(-1.0) + 1.0;

    let issue_id: i64 = sqlx::query_scalar(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, position, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
    )
    .bind(input.project_id)
    .bind(&identifier)
    .bind(&input.title)
    .bind(&desc)
    .bind(status_id)
    .bind(priority)
    .bind(input.assignee_id)
    .bind(position)
    .bind(&now)
    .bind(&now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Link the file to the issue automatically
    let link_type = match input.severity.as_str() {
        "bug" => "cause",
        "improvement" | "todo" => "related",
        _ => "related",
    };
    sqlx::query(
        "INSERT INTO issue_file_links (issue_id, file_path, link_type) VALUES ($1, $2, $3)",
    )
    .bind(issue_id)
    .bind(&input.file_path)
    .bind(link_type)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // If severity is "bug", try to add the "bug" label
    let bug_label: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM labels WHERE project_id = $1 AND name = 'bug'",
    )
    .bind(input.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    if input.severity == "bug" {
        if let Some(label_id) = bug_label {
            let _ = sqlx::query(
                "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)",
            )
            .bind(issue_id)
            .bind(label_id)
            .execute(&mut *tx)
            .await;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(issue)
}
