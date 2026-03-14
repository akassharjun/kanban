use crate::state::AppState;
use crate::models::{Issue, ActivityLogEntry};
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateIssueInput {
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status_id: i64,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub estimate: Option<f64>,
    pub due_date: Option<String>,
    pub label_ids: Option<Vec<i64>>,
}

#[derive(Deserialize)]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_id: Option<i64>,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub position: Option<f64>,
    pub estimate: Option<f64>,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct ListIssuesFilter {
    pub project_id: i64,
    pub status_id: Option<i64>,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub label_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkUpdateInput {
    pub issue_ids: Vec<i64>,
    pub status_id: Option<i64>,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
}

#[derive(Serialize)]
pub struct IssueWithLabels {
    #[serde(flatten)]
    pub issue: Issue,
    pub labels: Vec<crate::models::Label>,
}

// Helper to log activity
async fn log_activity(pool: &sqlx::SqlitePool, issue_id: i64, field: &str, old_val: Option<String>, new_val: Option<String>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES (?, ?, ?, ?, ?)")
        .bind(issue_id).bind(field).bind(old_val).bind(new_val).bind(&now)
        .execute(pool).await?;
    Ok(())
}

// Helper to log undo
async fn log_undo(pool: &sqlx::SqlitePool, op_type: &str, entity_type: &str, entity_id: i64, before: Option<String>, after: Option<String>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // Clear any undone entries after the current position (new operation invalidates redo stack)
    sqlx::query("DELETE FROM undo_log WHERE undone = 1")
        .execute(pool).await?;
    sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(op_type).bind(entity_type).bind(entity_id).bind(before).bind(after).bind(&now)
        .execute(pool).await?;
    Ok(())
}

#[tauri::command]
pub fn create_issue(state: State<AppState>, input: CreateIssueInput) -> Result<Issue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let priority = input.priority.unwrap_or_else(|| "none".to_string());

        // Get next identifier
        let project = sqlx::query_as::<_, crate::models::Project>("SELECT * FROM projects WHERE id = ?")
            .bind(input.project_id).fetch_one(&state.pool).await?;
        let counter = project.issue_counter + 1;
        let identifier = format!("{}-{}", project.prefix, counter);

        // Update counter
        sqlx::query("UPDATE projects SET issue_counter = ? WHERE id = ?")
            .bind(counter).bind(input.project_id).execute(&state.pool).await?;

        // Get max position for this status
        let max_pos: Option<f64> = sqlx::query_scalar("SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?")
            .bind(input.project_id).bind(input.status_id)
            .fetch_one(&state.pool).await?;
        let position = max_pos.unwrap_or(-1.0) + 1.0;

        let result = sqlx::query(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(input.project_id)
        .bind(&identifier)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.status_id)
        .bind(&priority)
        .bind(input.assignee_id)
        .bind(input.parent_id)
        .bind(position)
        .bind(input.estimate)
        .bind(&input.due_date)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await?;

        let issue_id = result.last_insert_rowid();

        // Add labels
        if let Some(label_ids) = &input.label_ids {
            for label_id in label_ids {
                sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES (?, ?)")
                    .bind(issue_id).bind(label_id).execute(&state.pool).await?;
            }
        }

        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(issue_id).fetch_one(&state.pool).await?;

        let snapshot = serde_json::to_string(&issue).unwrap_or_default();
        log_undo(&state.pool, "create", "issue", issue_id, None, Some(snapshot)).await?;

        Ok(issue)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_issue(state: State<AppState>, id: i64) -> Result<IssueWithLabels, String> {
    state.rt.block_on(async {
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;
        let labels = sqlx::query_as::<_, crate::models::Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = ?"
        ).bind(id).fetch_all(&state.pool).await?;
        Ok(IssueWithLabels { issue, labels })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_issue_by_identifier(state: State<AppState>, identifier: String) -> Result<IssueWithLabels, String> {
    state.rt.block_on(async {
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await?;
        let labels = sqlx::query_as::<_, crate::models::Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = ?"
        ).bind(issue.id).fetch_all(&state.pool).await?;
        Ok(IssueWithLabels { issue, labels })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_issues(state: State<AppState>, filter: ListIssuesFilter) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("SELECT i.* FROM issues i");

        if filter.label_id.is_some() {
            qb.push(" JOIN issue_labels il ON i.id = il.issue_id");
        }

        qb.push(" WHERE i.project_id = ");
        qb.push_bind(filter.project_id);

        if let Some(status_id) = filter.status_id {
            qb.push(" AND i.status_id = ");
            qb.push_bind(status_id);
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND i.priority = ");
            qb.push_bind(priority.clone());
        }
        if let Some(assignee_id) = filter.assignee_id {
            qb.push(" AND i.assignee_id = ");
            qb.push_bind(assignee_id);
        }
        if let Some(label_id) = filter.label_id {
            qb.push(" AND il.label_id = ");
            qb.push_bind(label_id);
        }
        if let Some(parent_id) = filter.parent_id {
            qb.push(" AND i.parent_id = ");
            qb.push_bind(parent_id);
        }
        if let Some(ref search) = filter.search {
            let pattern = format!("%{}%", search);
            qb.push(" AND (i.title LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR i.description LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        qb.push(" ORDER BY i.position");

        qb.build_query_as::<Issue>()
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_issue(state: State<AppState>, id: i64, input: UpdateIssueInput) -> Result<Issue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let old_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;
        let old_snapshot = serde_json::to_string(&old_issue).unwrap_or_default();

        if let Some(ref title) = input.title {
            if title != &old_issue.title {
                log_activity(&state.pool, id, "title", Some(old_issue.title.clone()), Some(title.clone())).await?;
            }
            sqlx::query("UPDATE issues SET title = ?, updated_at = ? WHERE id = ?")
                .bind(title).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref desc) = input.description {
            log_activity(&state.pool, id, "description", old_issue.description.clone(), Some(desc.clone())).await?;
            sqlx::query("UPDATE issues SET description = ?, updated_at = ? WHERE id = ?")
                .bind(desc).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(status_id) = input.status_id {
            if status_id != old_issue.status_id {
                log_activity(&state.pool, id, "status_id", Some(old_issue.status_id.to_string()), Some(status_id.to_string())).await?;
            }
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(status_id).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref priority) = input.priority {
            if priority != &old_issue.priority {
                log_activity(&state.pool, id, "priority", Some(old_issue.priority.clone()), Some(priority.clone())).await?;
            }
            sqlx::query("UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?")
                .bind(priority).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(assignee_id) = input.assignee_id {
            // Use 0 or -1 to unassign
            let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
            log_activity(&state.pool, id, "assignee_id", old_issue.assignee_id.map(|v| v.to_string()), val.map(|v| v.to_string())).await?;
            if let Some(v) = val {
                sqlx::query("UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET assignee_id = NULL, updated_at = ? WHERE id = ?")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(parent_id) = input.parent_id {
            let val = if parent_id <= 0 { None } else { Some(parent_id) };
            if let Some(v) = val {
                sqlx::query("UPDATE issues SET parent_id = ?, updated_at = ? WHERE id = ?")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET parent_id = NULL, updated_at = ? WHERE id = ?")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(position) = input.position {
            sqlx::query("UPDATE issues SET position = ?, updated_at = ? WHERE id = ?")
                .bind(position).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(estimate) = input.estimate {
            let val = if estimate < 0.0 { None } else { Some(estimate) };
            if let Some(v) = val {
                sqlx::query("UPDATE issues SET estimate = ?, updated_at = ? WHERE id = ?")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET estimate = NULL, updated_at = ? WHERE id = ?")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(ref due_date) = input.due_date {
            let val = if due_date.is_empty() { None } else { Some(due_date.clone()) };
            if let Some(ref v) = val {
                sqlx::query("UPDATE issues SET due_date = ?, updated_at = ? WHERE id = ?")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET due_date = NULL, updated_at = ? WHERE id = ?")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }

        let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;
        let new_snapshot = serde_json::to_string(&updated).unwrap_or_default();

        log_undo(&state.pool, "update", "issue", id, Some(old_snapshot), Some(new_snapshot)).await?;

        // Check if parent should auto-complete (all children done/discarded)
        if input.status_id.is_some() {
            if let Some(parent_id) = updated.parent_id {
                let status = sqlx::query_as::<_, crate::models::Status>("SELECT * FROM statuses WHERE id = ?")
                    .bind(updated.status_id).fetch_one(&state.pool).await?;
                if status.category == "completed" || status.category == "discarded" {
                    // Check all siblings
                    let incomplete: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.parent_id = ? AND s.category NOT IN ('completed', 'discarded')"
                    ).bind(parent_id).fetch_one(&state.pool).await?;

                    if incomplete == 0 {
                        // Auto-close parent - find a 'completed' status for the project
                        let done_status: Option<crate::models::Status> = sqlx::query_as(
                            "SELECT * FROM statuses WHERE project_id = (SELECT project_id FROM issues WHERE id = ?) AND category = 'completed' LIMIT 1"
                        ).bind(parent_id).fetch_optional(&state.pool).await?;

                        if let Some(done) = done_status {
                            let parent_before = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                                .bind(parent_id).fetch_one(&state.pool).await?;
                            let parent_old_snapshot = serde_json::to_string(&parent_before).unwrap_or_default();
                            let parent_old_status_id = parent_before.status_id;

                            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                                .bind(done.id).bind(&now).bind(parent_id).execute(&state.pool).await?;

                            let parent_after = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                                .bind(parent_id).fetch_one(&state.pool).await?;
                            let parent_new_snapshot = serde_json::to_string(&parent_after).unwrap_or_default();

                            log_activity(&state.pool, parent_id, "status_id", Some(parent_old_status_id.to_string()), Some(done.id.to_string())).await?;
                            log_undo(&state.pool, "update", "issue", parent_id, Some(parent_old_snapshot), Some(parent_new_snapshot)).await?;
                        }
                    }
                }
            }
        }

        Ok(updated)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_issue(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let old_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;

        // Also snapshot label associations so undo can restore them
        let labels: Vec<i64> = sqlx::query_scalar("SELECT label_id FROM issue_labels WHERE issue_id = ?")
            .bind(id).fetch_all(&state.pool).await?;
        let mut snapshot_val = serde_json::to_value(&old_issue).unwrap();
        snapshot_val.as_object_mut().unwrap().insert("label_ids".to_string(), serde_json::to_value(&labels).unwrap());
        let old_snapshot = serde_json::to_string(&snapshot_val).unwrap_or_default();

        sqlx::query("DELETE FROM issues WHERE id = ?").bind(id).execute(&state.pool).await?;

        log_undo(&state.pool, "delete", "issue", id, Some(old_snapshot), None).await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn duplicate_issue(state: State<AppState>, id: i64) -> Result<Issue, String> {
    state.rt.block_on(async {
        let original = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Get next identifier
        let project = sqlx::query_as::<_, crate::models::Project>("SELECT * FROM projects WHERE id = ?")
            .bind(original.project_id).fetch_one(&state.pool).await?;
        let counter = project.issue_counter + 1;
        let identifier = format!("{}-{}", project.prefix, counter);
        sqlx::query("UPDATE projects SET issue_counter = ? WHERE id = ?")
            .bind(counter).bind(original.project_id).execute(&state.pool).await?;

        let result = sqlx::query(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(original.project_id)
        .bind(&identifier)
        .bind(format!("{} (copy)", original.title))
        .bind(&original.description)
        .bind(original.status_id)
        .bind(&original.priority)
        .bind(original.assignee_id)
        .bind(original.parent_id)
        .bind(original.position + 0.5)
        .bind(original.estimate)
        .bind(&original.due_date)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await?;

        let new_id = result.last_insert_rowid();

        // Copy labels
        let labels = sqlx::query_scalar::<_, i64>("SELECT label_id FROM issue_labels WHERE issue_id = ?")
            .bind(id).fetch_all(&state.pool).await?;
        for label_id in labels {
            sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES (?, ?)")
                .bind(new_id).bind(label_id).execute(&state.pool).await?;
        }

        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(new_id).fetch_one(&state.pool).await?;

        Ok(issue)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn bulk_update_issues(state: State<AppState>, input: BulkUpdateInput) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for issue_id in &input.issue_ids {
            if let Some(status_id) = input.status_id {
                sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                    .bind(status_id).bind(&now).bind(issue_id).execute(&state.pool).await?;
            }
            if let Some(ref priority) = input.priority {
                sqlx::query("UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?")
                    .bind(priority).bind(&now).bind(issue_id).execute(&state.pool).await?;
            }
            if let Some(assignee_id) = input.assignee_id {
                let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
                if let Some(v) = val {
                    sqlx::query("UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?")
                        .bind(v).bind(&now).bind(issue_id).execute(&state.pool).await?;
                } else {
                    sqlx::query("UPDATE issues SET assignee_id = NULL, updated_at = ? WHERE id = ?")
                        .bind(&now).bind(issue_id).execute(&state.pool).await?;
                }
            }
        }

        // Fetch updated issues
        let placeholders: Vec<String> = input.issue_ids.iter().map(|_| "?".to_string()).collect();
        let query = format!("SELECT * FROM issues WHERE id IN ({}) ORDER BY position", placeholders.join(","));
        let mut q = sqlx::query_as::<_, Issue>(&query);
        for id in &input.issue_ids {
            q = q.bind(id);
        }
        q.fetch_all(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn search_issues(state: State<AppState>, project_id: i64, query: String) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE project_id = ? AND (title LIKE ? OR description LIKE ? OR identifier LIKE ?) ORDER BY updated_at DESC"
        )
        .bind(project_id)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sub_issues(state: State<AppState>, parent_id: i64) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE parent_id = ? ORDER BY position")
            .bind(parent_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_issue_labels(state: State<AppState>, issue_id: i64, label_ids: Vec<i64>) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM issue_labels WHERE issue_id = ?")
            .bind(issue_id).execute(&state.pool).await?;
        for label_id in &label_ids {
            sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES (?, ?)")
                .bind(issue_id).bind(label_id).execute(&state.pool).await?;
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_activity_log(state: State<AppState>, issue_id: i64) -> Result<Vec<ActivityLogEntry>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, ActivityLogEntry>("SELECT * FROM activity_log WHERE issue_id = ? ORDER BY timestamp DESC")
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}
