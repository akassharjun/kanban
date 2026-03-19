use crate::state::AppState;
use crate::models::{Issue, ActivityLogEntry, AuditLogEntry, IssueHistoryEntry};
use crate::commands::automations::{evaluate_rules, AutomationContext};
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
    pub epic_id: Option<i64>,
    pub milestone_id: Option<i64>,
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
    pub epic_id: Option<i64>,
    pub milestone_id: Option<i64>,
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

// Helper to log activity with optional actor tracking
async fn log_activity(pool: &sqlx::AnyPool, issue_id: i64, field: &str, old_val: Option<String>, new_val: Option<String>) -> Result<(), sqlx::Error> {
    log_activity_with_actor(pool, issue_id, field, old_val, new_val, None, None).await
}

async fn log_activity_with_actor(pool: &sqlx::AnyPool, issue_id: i64, field: &str, old_val: Option<String>, new_val: Option<String>, actor_id: Option<i64>, actor_type: Option<String>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, actor_id, actor_type, timestamp) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(issue_id).bind(field).bind(old_val).bind(new_val).bind(actor_id).bind(actor_type).bind(&now)
        .execute(pool).await?;
    Ok(())
}

// Helper to log undo
async fn log_undo(pool: &sqlx::AnyPool, op_type: &str, entity_type: &str, entity_id: i64, before: Option<String>, after: Option<String>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    // Clear any undone entries after the current position (new operation invalidates redo stack)
    sqlx::query("DELETE FROM undo_log WHERE undone = TRUE")
        .execute(pool).await?;
    sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(op_type).bind(entity_type).bind(entity_id).bind(before).bind(after).bind(&now)
        .execute(pool).await?;
    Ok(())
}

#[tauri::command]
pub fn create_issue(state: State<AppState>, input: CreateIssueInput) -> Result<Issue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let priority = input.priority.unwrap_or_else(|| "none".to_string());

        let mut tx = state.pool.begin().await?;

        // Atomically increment counter and get new value + prefix
        let (counter, prefix): (i64, String) = sqlx::query_as(
            "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
        ).bind(input.project_id).fetch_one(&mut *tx).await?;
        let identifier = format!("{}-{}", prefix, counter);

        // Get max position for this status
        let max_pos: Option<f64> = sqlx::query_scalar("SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2")
            .bind(input.project_id).bind(input.status_id)
            .fetch_one(&mut *tx).await?;
        let position = max_pos.unwrap_or(-1.0) + 1.0;

        let epic_id = input.epic_id.and_then(|v| if v <= 0 { None } else { Some(v) });
        let milestone_id = input.milestone_id.and_then(|v| if v <= 0 { None } else { Some(v) });

        let issue_id: i64 = sqlx::query_scalar(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, epic_id, milestone_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING id"
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
        .bind(epic_id)
        .bind(milestone_id)
        .bind(&now)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await?;

        // Add labels
        if let Some(label_ids) = &input.label_ids {
            for label_id in label_ids {
                sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                    .bind(issue_id).bind(label_id).execute(&mut *tx).await?;
            }
        }

        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id).fetch_one(&mut *tx).await?;

        let snapshot = serde_json::to_string(&issue).unwrap_or_default();

        // Clear redo stack
        sqlx::query("DELETE FROM undo_log WHERE undone = TRUE")
            .execute(&mut *tx).await?;
        // Insert undo entry
        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind("create").bind("issue").bind(issue_id)
            .bind(Option::<String>::None).bind(Some(&snapshot))
            .bind(&now)
            .execute(&mut *tx).await?;

        tx.commit().await?;

        // Trigger automation rules for issue creation
        let ctx = AutomationContext {
            issue_id: Some(issue.id),
            project_id: issue.project_id,
            old_value: None,
            new_value: None,
            actor_name: None,
            agent_name: None,
            task_confidence: None,
            issue_title: Some(issue.title.clone()),
            issue_identifier: Some(issue.identifier.clone()),
            issue_priority: Some(issue.priority.clone()),
            issue_status_id: Some(issue.status_id),
            issue_assignee_id: issue.assignee_id,
        };
        let _ = evaluate_rules(&state.pool, issue.project_id, "issue_created", &ctx).await;

        Ok(issue)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_issue(state: State<AppState>, id: i64) -> Result<IssueWithLabels, String> {
    state.rt.block_on(async {
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let labels = sqlx::query_as::<_, crate::models::Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1"
        ).bind(id).fetch_all(&state.pool).await?;
        Ok(IssueWithLabels { issue, labels })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_issue_by_identifier(state: State<AppState>, identifier: String) -> Result<IssueWithLabels, String> {
    state.rt.block_on(async {
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
            .bind(&identifier).fetch_one(&state.pool).await?;
        let labels = sqlx::query_as::<_, crate::models::Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1"
        ).bind(issue.id).fetch_all(&state.pool).await?;
        Ok(IssueWithLabels { issue, labels })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_issues(state: State<AppState>, filter: ListIssuesFilter) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let mut qb: sqlx::QueryBuilder<sqlx::Any> = sqlx::QueryBuilder::new("SELECT i.* FROM issues i");

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
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let old_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let old_snapshot = serde_json::to_string(&old_issue).unwrap_or_default();

        if let Some(ref title) = input.title {
            if title != &old_issue.title {
                log_activity(&state.pool, id, "title", Some(old_issue.title.clone()), Some(title.clone())).await?;
            }
            sqlx::query("UPDATE issues SET title = $1, updated_at = $2 WHERE id = $3")
                .bind(title).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref desc) = input.description {
            log_activity(&state.pool, id, "description", old_issue.description.clone(), Some(desc.clone())).await?;
            sqlx::query("UPDATE issues SET description = $1, updated_at = $2 WHERE id = $3")
                .bind(desc).bind(&now).bind(id).execute(&state.pool).await?;
            // Re-process @mentions in description
            crate::commands::mentions::clear_mentions(&state.pool, id, None, "description").await?;
            crate::commands::mentions::process_mentions(&state.pool, id, None, "description", desc).await?;
        }
        if let Some(status_id) = input.status_id {
            if status_id != old_issue.status_id {
                log_activity(&state.pool, id, "status_id", Some(old_issue.status_id.to_string()), Some(status_id.to_string())).await?;
            }
            sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                .bind(status_id).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref priority) = input.priority {
            if priority != &old_issue.priority {
                log_activity(&state.pool, id, "priority", Some(old_issue.priority.clone()), Some(priority.clone())).await?;
            }
            sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                .bind(priority).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(assignee_id) = input.assignee_id {
            // Use 0 or -1 to unassign
            let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
            log_activity(&state.pool, id, "assignee_id", old_issue.assignee_id.map(|v| v.to_string()), val.map(|v| v.to_string())).await?;
            if let Some(v) = val {
                sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET assignee_id = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(parent_id) = input.parent_id {
            let val = if parent_id <= 0 { None } else { Some(parent_id) };
            if let Some(v) = val {
                // Validate: cannot be own parent
                if v == id {
                    return Err(sqlx::Error::Protocol("An issue cannot be its own parent".to_string()));
                }
                // Validate: walk parent chain to detect cycles
                let mut current = v;
                let mut visited = std::collections::HashSet::new();
                visited.insert(id); // the issue being updated
                loop {
                    if !visited.insert(current) {
                        // current was already in visited -- cycle detected
                        return Err(sqlx::Error::Protocol("Circular parent-child reference detected".to_string()));
                    }
                    let next: Option<Option<i64>> = sqlx::query_scalar("SELECT parent_id FROM issues WHERE id = $1")
                        .bind(current).fetch_optional(&state.pool).await?;
                    match next.flatten() {
                        Some(pid) => current = pid,
                        None => break, // reached root, no cycle
                    }
                }
                sqlx::query("UPDATE issues SET parent_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET parent_id = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(position) = input.position {
            sqlx::query("UPDATE issues SET position = $1, updated_at = $2 WHERE id = $3")
                .bind(position).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(estimate) = input.estimate {
            // Use negative value as sentinel to clear estimate; otherwise validate >= 0
            let val = if estimate < 0.0 { None } else { Some(estimate) };
            if let Some(v) = val {
                if v < 0.0 {
                    return Err(sqlx::Error::Protocol("Estimate must be >= 0".to_string()));
                }
                sqlx::query("UPDATE issues SET estimate = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET estimate = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(ref due_date) = input.due_date {
            let val = if due_date.is_empty() { None } else { Some(due_date.clone()) };
            if let Some(ref v) = val {
                sqlx::query("UPDATE issues SET due_date = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET due_date = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(epic_id) = input.epic_id {
            let val = if epic_id <= 0 { None } else { Some(epic_id) };
            if let Some(v) = val {
                // Validate epic belongs to the same project
                let epic_project_id: Option<i64> = sqlx::query_scalar(
                    "SELECT project_id FROM epics WHERE id = $1"
                ).bind(v).fetch_optional(&state.pool).await?;
                match epic_project_id {
                    Some(pid) if pid != old_issue.project_id => {
                        return Err(sqlx::Error::Protocol("Epic does not belong to the same project as the issue".to_string()));
                    }
                    None => {
                        return Err(sqlx::Error::Protocol("Epic not found".to_string()));
                    }
                    _ => {}
                }
                sqlx::query("UPDATE issues SET epic_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET epic_id = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
            log_activity(&state.pool, id, "epic_id", old_issue.epic_id.map(|v| v.to_string()), val.map(|v| v.to_string())).await?;
        }
        if let Some(milestone_id) = input.milestone_id {
            let val = if milestone_id <= 0 { None } else { Some(milestone_id) };
            if let Some(v) = val {
                // Validate milestone belongs to the same project
                let milestone_project_id: Option<i64> = sqlx::query_scalar(
                    "SELECT project_id FROM milestones WHERE id = $1"
                ).bind(v).fetch_optional(&state.pool).await?;
                match milestone_project_id {
                    Some(pid) if pid != old_issue.project_id => {
                        return Err(sqlx::Error::Protocol("Milestone does not belong to the same project as the issue".to_string()));
                    }
                    None => {
                        return Err(sqlx::Error::Protocol("Milestone not found".to_string()));
                    }
                    _ => {}
                }
                sqlx::query("UPDATE issues SET milestone_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET milestone_id = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
            log_activity(&state.pool, id, "milestone_id", old_issue.milestone_id.map(|v| v.to_string()), val.map(|v| v.to_string())).await?;
        }

        let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let new_snapshot = serde_json::to_string(&updated).unwrap_or_default();

        log_undo(&state.pool, "update", "issue", id, Some(old_snapshot), Some(new_snapshot)).await?;

        // Trigger automation rules
        {
            let ctx = AutomationContext {
                issue_id: Some(updated.id),
                project_id: updated.project_id,
                old_value: None,
                new_value: None,
                actor_name: None,
                agent_name: None,
                task_confidence: None,
                issue_title: Some(updated.title.clone()),
                issue_identifier: Some(updated.identifier.clone()),
                issue_priority: Some(updated.priority.clone()),
                issue_status_id: Some(updated.status_id),
                issue_assignee_id: updated.assignee_id,
            };

            // Status change
            if let Some(status_id) = input.status_id {
                if status_id != old_issue.status_id {
                    let mut sctx = ctx.clone();
                    sctx.old_value = Some(old_issue.status_id.to_string());
                    sctx.new_value = Some(status_id.to_string());
                    let _ = evaluate_rules(&state.pool, updated.project_id, "status_change", &sctx).await;
                }
            }

            // Priority change
            if let Some(ref priority) = input.priority {
                if priority != &old_issue.priority {
                    let mut pctx = ctx.clone();
                    pctx.old_value = Some(old_issue.priority.clone());
                    pctx.new_value = Some(priority.clone());
                    let _ = evaluate_rules(&state.pool, updated.project_id, "priority_changed", &pctx).await;
                }
            }

            // General issue_updated
            let _ = evaluate_rules(&state.pool, updated.project_id, "issue_updated", &ctx).await;
        }

        // Check if parent should auto-complete (all children done/discarded)
        if input.status_id.is_some() {
            if let Some(parent_id) = updated.parent_id {
                let status = sqlx::query_as::<_, crate::models::Status>("SELECT * FROM statuses WHERE id = $1")
                    .bind(updated.status_id).fetch_one(&state.pool).await?;
                if status.category == "completed" || status.category == "discarded" {
                    // Check all siblings
                    let incomplete: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.parent_id = $1 AND s.category NOT IN ('completed', 'discarded')"
                    ).bind(parent_id).fetch_one(&state.pool).await?;

                    if incomplete == 0 {
                        // Auto-close parent - find a 'completed' status for the project
                        let done_status: Option<crate::models::Status> = sqlx::query_as(
                            "SELECT * FROM statuses WHERE project_id = (SELECT project_id FROM issues WHERE id = $1) AND category = 'completed' LIMIT 1"
                        ).bind(parent_id).fetch_optional(&state.pool).await?;

                        if let Some(done) = done_status {
                            let parent_before = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                                .bind(parent_id).fetch_one(&state.pool).await?;
                            let parent_old_snapshot = serde_json::to_string(&parent_before).unwrap_or_default();
                            let parent_old_status_id = parent_before.status_id;

                            sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                                .bind(done.id).bind(&now).bind(parent_id).execute(&state.pool).await?;

                            let parent_after = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
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
        let old_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;

        // Also snapshot label associations so undo can restore them
        let labels: Vec<i64> = sqlx::query_scalar("SELECT label_id FROM issue_labels WHERE issue_id = $1")
            .bind(id).fetch_all(&state.pool).await?;
        let old_snapshot = serde_json::to_value(&old_issue)
            .ok()
            .and_then(|mut v| {
                v.as_object_mut()?.insert(
                    "label_ids".to_string(),
                    serde_json::to_value(&labels).ok()?,
                );
                serde_json::to_string(&v).ok()
            })
            .unwrap_or_default();

        sqlx::query("DELETE FROM issues WHERE id = $1").bind(id).execute(&state.pool).await?;

        log_undo(&state.pool, "delete", "issue", id, Some(old_snapshot), None).await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn duplicate_issue(state: State<AppState>, id: i64) -> Result<Issue, String> {
    state.rt.block_on(async {
        let original = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        let mut tx = state.pool.begin().await?;

        // Atomically increment counter and get new value + prefix
        let (counter, prefix): (i64, String) = sqlx::query_as(
            "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
        ).bind(original.project_id).fetch_one(&mut *tx).await?;
        let identifier = format!("{}-{}", prefix, counter);

        let new_id: i64 = sqlx::query_scalar(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id"
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
        .fetch_one(&mut *tx)
        .await?;

        // Copy labels
        let labels = sqlx::query_scalar::<_, i64>("SELECT label_id FROM issue_labels WHERE issue_id = $1")
            .bind(id).fetch_all(&mut *tx).await?;
        for label_id in labels {
            sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                .bind(new_id).bind(label_id).execute(&mut *tx).await?;
        }

        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(new_id).fetch_one(&mut *tx).await?;

        let snapshot = serde_json::to_string(&issue).unwrap_or_default();

        // Clear redo stack
        sqlx::query("DELETE FROM undo_log WHERE undone = TRUE")
            .execute(&mut *tx).await?;
        // Insert undo entry
        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind("create").bind("issue").bind(new_id)
            .bind(Option::<String>::None).bind(Some(&snapshot))
            .bind(&now)
            .execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(issue)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn bulk_update_issues(state: State<AppState>, input: BulkUpdateInput) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        for issue_id in &input.issue_ids {
            // Fetch old state before updating
            let old_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id).fetch_one(&state.pool).await?;
            let old_snapshot = serde_json::to_string(&old_issue).unwrap_or_default();

            if let Some(status_id) = input.status_id {
                sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(status_id).bind(&now).bind(issue_id).execute(&state.pool).await?;
            }
            if let Some(ref priority) = input.priority {
                sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                    .bind(priority).bind(&now).bind(issue_id).execute(&state.pool).await?;
            }
            if let Some(assignee_id) = input.assignee_id {
                let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
                if let Some(v) = val {
                    sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                        .bind(v).bind(&now).bind(issue_id).execute(&state.pool).await?;
                } else {
                    sqlx::query("UPDATE issues SET assignee_id = NULL, updated_at = $1 WHERE id = $2")
                        .bind(&now).bind(issue_id).execute(&state.pool).await?;
                }
            }

            // Fetch new state after updating
            let updated_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id).fetch_one(&state.pool).await?;
            let new_snapshot = serde_json::to_string(&updated_issue).unwrap_or_default();

            // Log activity for each changed field
            if let Some(status_id) = input.status_id {
                if status_id != old_issue.status_id {
                    log_activity(&state.pool, *issue_id, "status_id", Some(old_issue.status_id.to_string()), Some(status_id.to_string())).await?;
                }
            }
            if let Some(ref priority) = input.priority {
                if priority != &old_issue.priority {
                    log_activity(&state.pool, *issue_id, "priority", Some(old_issue.priority.clone()), Some(priority.clone())).await?;
                }
            }
            if let Some(assignee_id) = input.assignee_id {
                let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
                log_activity(&state.pool, *issue_id, "assignee_id", old_issue.assignee_id.map(|v| v.to_string()), val.map(|v| v.to_string())).await?;
            }

            // Log undo entry with before/after snapshots
            log_undo(&state.pool, "update", "issue", *issue_id, Some(old_snapshot), Some(new_snapshot)).await?;
        }

        // Fetch updated issues
        let mut qb: sqlx::QueryBuilder<sqlx::Any> = sqlx::QueryBuilder::new("SELECT * FROM issues WHERE id IN (");
        let mut separated = qb.separated(", ");
        for id in &input.issue_ids {
            separated.push_bind(*id);
        }
        separated.push_unseparated(") ORDER BY position");
        qb.build_query_as::<Issue>()
            .fetch_all(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn search_issues(state: State<AppState>, project_id: i64, query: String) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE project_id = $1 AND (title LIKE $2 OR description LIKE $3 OR identifier LIKE $4) ORDER BY updated_at DESC"
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
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE parent_id = $1 ORDER BY position")
            .bind(parent_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_issue_labels(state: State<AppState>, issue_id: i64, label_ids: Vec<i64>) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
            .bind(issue_id).execute(&state.pool).await?;
        for label_id in &label_ids {
            sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                .bind(issue_id).bind(label_id).execute(&state.pool).await?;
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_activity_log(state: State<AppState>, issue_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ActivityLogEntry>, String> {
    state.rt.block_on(async {
        let lim = limit.unwrap_or(50);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, ActivityLogEntry>("SELECT * FROM activity_log WHERE issue_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3")
            .bind(issue_id)
            .bind(lim)
            .bind(off)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct AuditLogFilter {
    pub project_id: i64,
    pub actor_id: Option<i64>,
    pub issue_id: Option<i64>,
    pub field_changed: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub fn get_audit_log(state: State<AppState>, filter: AuditLogFilter) -> Result<Vec<AuditLogEntry>, String> {
    state.rt.block_on(async {
        let mut qb: sqlx::QueryBuilder<sqlx::Any> = sqlx::QueryBuilder::new(
            "SELECT a.id, a.issue_id, i.identifier as issue_identifier, i.title as issue_title, \
             a.field_changed, a.old_value, a.new_value, a.actor_id, a.actor_type, \
             m.display_name as actor_name, m.avatar_color as actor_avatar_color, a.timestamp \
             FROM activity_log a \
             JOIN issues i ON a.issue_id = i.id \
             LEFT JOIN members m ON a.actor_id = m.id \
             WHERE i.project_id = "
        );
        qb.push_bind(filter.project_id);

        if let Some(actor_id) = filter.actor_id {
            qb.push(" AND a.actor_id = ");
            qb.push_bind(actor_id);
        }
        if let Some(issue_id) = filter.issue_id {
            qb.push(" AND a.issue_id = ");
            qb.push_bind(issue_id);
        }
        if let Some(ref field) = filter.field_changed {
            qb.push(" AND a.field_changed = ");
            qb.push_bind(field.clone());
        }
        if let Some(ref date_from) = filter.date_from {
            qb.push(" AND a.timestamp >= ");
            qb.push_bind(date_from.clone());
        }
        if let Some(ref date_to) = filter.date_to {
            qb.push(" AND a.timestamp <= ");
            qb.push_bind(date_to.clone());
        }

        qb.push(" ORDER BY a.timestamp DESC");

        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb.build_query_as::<AuditLogRow>()
            .fetch_all(&state.pool)
            .await?;

        Ok(rows.into_iter().map(|r| AuditLogEntry {
            id: r.id,
            issue_id: r.issue_id,
            issue_identifier: r.issue_identifier,
            issue_title: r.issue_title,
            field_changed: r.field_changed,
            old_value: r.old_value,
            new_value: r.new_value,
            actor_id: r.actor_id,
            actor_type: r.actor_type,
            actor_name: r.actor_name,
            actor_avatar_color: r.actor_avatar_color,
            timestamp: r.timestamp,
        }).collect())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: i64,
    issue_id: i64,
    issue_identifier: String,
    issue_title: String,
    field_changed: String,
    old_value: Option<String>,
    new_value: Option<String>,
    actor_id: Option<i64>,
    actor_type: Option<String>,
    actor_name: Option<String>,
    actor_avatar_color: Option<String>,
    timestamp: String,
}

#[tauri::command]
pub fn get_issue_history(state: State<AppState>, issue_id: i64) -> Result<Vec<IssueHistoryEntry>, String> {
    state.rt.block_on(async {
        let rows = sqlx::query_as::<_, IssueHistoryRow>(
            "SELECT a.id, a.issue_id, a.field_changed, a.old_value, a.new_value, \
             a.actor_id, a.actor_type, m.display_name as actor_name, m.avatar_color as actor_avatar_color, \
             a.timestamp \
             FROM activity_log a \
             LEFT JOIN members m ON a.actor_id = m.id \
             WHERE a.issue_id = $1 \
             ORDER BY a.timestamp DESC"
        )
        .bind(issue_id)
        .fetch_all(&state.pool)
        .await?;

        Ok(rows.into_iter().map(|r| IssueHistoryEntry {
            id: r.id,
            issue_id: r.issue_id,
            field_changed: r.field_changed,
            old_value: r.old_value,
            new_value: r.new_value,
            actor_id: r.actor_id,
            actor_type: r.actor_type,
            actor_name: r.actor_name,
            actor_avatar_color: r.actor_avatar_color,
            timestamp: r.timestamp,
        }).collect())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[derive(sqlx::FromRow)]
struct IssueHistoryRow {
    id: i64,
    issue_id: i64,
    field_changed: String,
    old_value: Option<String>,
    new_value: Option<String>,
    actor_id: Option<i64>,
    actor_type: Option<String>,
    actor_name: Option<String>,
    actor_avatar_color: Option<String>,
    timestamp: String,
}
