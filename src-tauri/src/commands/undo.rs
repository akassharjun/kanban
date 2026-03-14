use crate::state::AppState;
use crate::models::UndoLogEntry;
use tauri::State;

#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<Option<UndoLogEntry>, String> {
    state.rt.block_on(async {
        // Find the last non-undone entry
        let entry: Option<UndoLogEntry> = sqlx::query_as(
            "SELECT * FROM undo_log WHERE undone = 0 ORDER BY id DESC LIMIT 1"
        ).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        let Some(entry) = entry else { return Ok(None); };

        match (entry.operation_type.as_str(), entry.entity_type.as_str()) {
            ("create", "issue") => {
                sqlx::query("DELETE FROM issues WHERE id = ?")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("update", "issue") => {
                if let Some(ref before) = entry.snapshot_before {
                    let issue: crate::models::Issue = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE issues SET title = ?, description = ?, status_id = ?, priority = ?, assignee_id = ?, parent_id = ?, position = ?, estimate = ?, due_date = ?, updated_at = ? WHERE id = ?")
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
                    sqlx::query("INSERT INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(issue.id).bind(issue.project_id).bind(&issue.identifier)
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.created_at).bind(&issue.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("create", "project") => {
                sqlx::query("DELETE FROM projects WHERE id = ?")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("update", "project") => {
                if let Some(ref before) = entry.snapshot_before {
                    let project: crate::models::Project = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE projects SET name = ?, description = ?, icon = ?, status = ?, updated_at = ? WHERE id = ?")
                        .bind(&project.name).bind(&project.description).bind(&project.icon)
                        .bind(&project.status).bind(&project.updated_at).bind(project.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "project") => {
                if let Some(ref before) = entry.snapshot_before {
                    let project: crate::models::Project = serde_json::from_str(before).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(project.id).bind(&project.name).bind(&project.description)
                        .bind(&project.icon).bind(&project.status).bind(&project.prefix)
                        .bind(project.issue_counter).bind(&project.created_at).bind(&project.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            _ => {}
        }

        sqlx::query("UPDATE undo_log SET undone = 1 WHERE id = ?")
            .bind(entry.id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(Some(entry))
    })
}

#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<Option<UndoLogEntry>, String> {
    state.rt.block_on(async {
        let entry: Option<UndoLogEntry> = sqlx::query_as(
            "SELECT * FROM undo_log WHERE undone = 1 ORDER BY id ASC LIMIT 1"
        ).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        let Some(entry) = entry else { return Ok(None); };

        match (entry.operation_type.as_str(), entry.entity_type.as_str()) {
            ("create", "issue") => {
                if let Some(ref after) = entry.snapshot_after {
                    let issue: crate::models::Issue = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
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
                    sqlx::query("UPDATE issues SET title = ?, description = ?, status_id = ?, priority = ?, assignee_id = ?, parent_id = ?, position = ?, estimate = ?, due_date = ?, updated_at = ? WHERE id = ?")
                        .bind(&issue.title).bind(&issue.description).bind(issue.status_id)
                        .bind(&issue.priority).bind(issue.assignee_id).bind(issue.parent_id)
                        .bind(issue.position).bind(issue.estimate).bind(&issue.due_date)
                        .bind(&issue.updated_at).bind(issue.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "issue") => {
                sqlx::query("DELETE FROM issues WHERE id = ?")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            ("create", "project") => {
                if let Some(ref after) = entry.snapshot_after {
                    let project: crate::models::Project = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("INSERT INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(project.id).bind(&project.name).bind(&project.description)
                        .bind(&project.icon).bind(&project.status).bind(&project.prefix)
                        .bind(project.issue_counter).bind(&project.created_at).bind(&project.updated_at)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("update", "project") => {
                if let Some(ref after) = entry.snapshot_after {
                    let project: crate::models::Project = serde_json::from_str(after).map_err(|e| e.to_string())?;
                    sqlx::query("UPDATE projects SET name = ?, description = ?, icon = ?, status = ?, updated_at = ? WHERE id = ?")
                        .bind(&project.name).bind(&project.description).bind(&project.icon)
                        .bind(&project.status).bind(&project.updated_at).bind(project.id)
                        .execute(&state.pool).await.map_err(|e| e.to_string())?;
                }
            }
            ("delete", "project") => {
                sqlx::query("DELETE FROM projects WHERE id = ?")
                    .bind(entry.entity_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        sqlx::query("UPDATE undo_log SET undone = 0 WHERE id = ?")
            .bind(entry.id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(Some(entry))
    })
}
