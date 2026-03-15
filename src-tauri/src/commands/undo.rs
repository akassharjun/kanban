use crate::state::AppState;
use crate::models::UndoLogEntry;
use tauri::State;

#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<Option<UndoLogEntry>, String> {
    state.rt.block_on(async {
        // Find the last non-undone entry
        let entry: Option<UndoLogEntry> = sqlx::query_as(
            "SELECT * FROM undo_log WHERE undone = FALSE ORDER BY id DESC LIMIT 1"
        ).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        let Some(entry) = entry else { return Ok(None); };

        match (entry.operation_type.as_str(), entry.entity_type.as_str()) {
            ("create", "issue") => {
                sqlx::query("DELETE FROM issues WHERE id = $1")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("update", "issue") => {
                if let Some(ref before) = entry.snapshot_before {
                    let issue: crate::models::Issue = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE issues SET title = $1, description = $2, status_id = $3, priority = $4, assignee_id = $5, parent_id = $6, position = $7, estimate = $8, due_date = $9, updated_at = $10 WHERE id = $11")
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.updated_at).bind(issue.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "issue") => {
                if let Some(ref before) = entry.snapshot_before {
                    let issue: crate::models::Issue = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)")
                        .bind(issue.id).bind(issue.project_id).bind(&issue.identifier)
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.created_at).bind(&issue.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;

                    // Restore labels if present in snapshot
                    let snapshot_val: serde_json::Value = serde_json::from_str(before).unwrap_or_default();
                    if let Some(label_ids) = snapshot_val.get("label_ids").and_then(|v| v.as_array()) {
                        for label_val in label_ids {
                            if let Some(label_id) = label_val.as_i64() {
                                sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                                    .bind(issue.id).bind(label_id)
                                    .execute(&state.pool).await.map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
            }
            ("create", "project") => {
                sqlx::query("DELETE FROM projects WHERE id = $1")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("update", "project") => {
                if let Some(ref before) = entry.snapshot_before {
                    let project: crate::models::Project = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE projects SET name = $1, description = $2, icon = $3, status = $4, updated_at = $5 WHERE id = $6")
                        .bind(&project.name).bind(&project.description).bind(&project.icon)
                        .bind(&project.status).bind(&project.updated_at).bind(project.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "project") => {
                if let Some(ref before) = entry.snapshot_before {
                    let project: crate::models::Project = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
                        .bind(project.id).bind(&project.name).bind(&project.description)
                        .bind(&project.icon).bind(&project.status).bind(&project.prefix)
                        .bind(project.issue_counter).bind(&project.created_at).bind(&project.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            _ => {}
        }

        sqlx::query("UPDATE undo_log SET undone = TRUE WHERE id = $1")
            .bind(entry.id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(Some(entry))
    })
}

#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<Option<UndoLogEntry>, String> {
    state.rt.block_on(async {
        let entry: Option<UndoLogEntry> = sqlx::query_as(
            "SELECT * FROM undo_log WHERE undone = TRUE ORDER BY id ASC LIMIT 1"
        ).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        let Some(entry) = entry else { return Ok(None); };

        match (entry.operation_type.as_str(), entry.entity_type.as_str()) {
            ("create", "issue") => {
                if let Some(ref after) = entry.snapshot_after {
                    let issue: crate::models::Issue = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)")
                        .bind(issue.id).bind(issue.project_id).bind(&issue.identifier)
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.created_at).bind(&issue.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("update", "issue") => {
                if let Some(ref after) = entry.snapshot_after {
                    let issue: crate::models::Issue = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE issues SET title = $1, description = $2, status_id = $3, priority = $4, assignee_id = $5, parent_id = $6, position = $7, estimate = $8, due_date = $9, updated_at = $10 WHERE id = $11")
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.updated_at).bind(issue.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "issue") => {
                sqlx::query("DELETE FROM issues WHERE id = $1")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("create", "project") => {
                if let Some(ref after) = entry.snapshot_after {
                    let project: crate::models::Project = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
                        .bind(project.id).bind(&project.name).bind(&project.description)
                        .bind(&project.icon).bind(&project.status).bind(&project.prefix)
                        .bind(project.issue_counter).bind(&project.created_at).bind(&project.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("update", "project") => {
                if let Some(ref after) = entry.snapshot_after {
                    let project: crate::models::Project = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE projects SET name = $1, description = $2, icon = $3, status = $4, updated_at = $5 WHERE id = $6")
                        .bind(&project.name).bind(&project.description).bind(&project.icon)
                        .bind(&project.status).bind(&project.updated_at).bind(project.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "project") => {
                sqlx::query("DELETE FROM projects WHERE id = $1")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        sqlx::query("UPDATE undo_log SET undone = FALSE WHERE id = $1")
            .bind(entry.id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(Some(entry))
    })
}
