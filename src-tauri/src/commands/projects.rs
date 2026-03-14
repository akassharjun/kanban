use crate::state::AppState;
use crate::models::Project;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub prefix: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY name")
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project(state: State<AppState>, id: i64) -> Result<Project, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(state: State<AppState>, input: CreateProjectInput) -> Result<Project, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Insert project
        let result = sqlx::query(
            "INSERT INTO projects (name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, 0, ?, ?)"
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.icon)
        .bind(&input.prefix)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await?;

        let project_id = result.last_insert_rowid();

        // Create default statuses
        let default_statuses = vec![
            ("Backlog", "unstarted", "#6b7280", 0),
            ("Todo", "unstarted", "#6b7280", 1),
            ("In Progress", "started", "#3b82f6", 2),
            ("In Review", "started", "#8b5cf6", 3),
            ("Blocked", "blocked", "#ef4444", 4),
            ("Done", "completed", "#22c55e", 5),
            ("Discarded", "discarded", "#6b7280", 6),
        ];

        for (name, category, color, position) in default_statuses {
            sqlx::query(
                "INSERT INTO statuses (project_id, name, category, color, position) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(project_id)
            .bind(name)
            .bind(category)
            .bind(color)
            .bind(position)
            .execute(&state.pool)
            .await?;
        }

        // Log undo
        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await?;

        let snapshot = serde_json::to_string(&project).unwrap_or_default();
        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('create', 'project', ?, NULL, ?, ?)")
            .bind(project_id)
            .bind(&snapshot)
            .bind(&now)
            .execute(&state.pool)
            .await?;

        Ok(project)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_project(state: State<AppState>, id: i64, input: UpdateProjectInput) -> Result<Project, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Get old state for undo
        let old_project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
        let old_snapshot = serde_json::to_string(&old_project).unwrap_or_default();

        if let Some(name) = &input.name {
            sqlx::query("UPDATE projects SET name = ?, updated_at = ? WHERE id = ?")
                .bind(name).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(desc) = &input.description {
            sqlx::query("UPDATE projects SET description = ?, updated_at = ? WHERE id = ?")
                .bind(desc).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(icon) = &input.icon {
            sqlx::query("UPDATE projects SET icon = ?, updated_at = ? WHERE id = ?")
                .bind(icon).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(status) = &input.status {
            sqlx::query("UPDATE projects SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status).bind(&now).bind(id).execute(&state.pool).await?;
        }

        let updated = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;
        let new_snapshot = serde_json::to_string(&updated).unwrap_or_default();

        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('update', 'project', ?, ?, ?, ?)")
            .bind(id).bind(&old_snapshot).bind(&new_snapshot).bind(&now)
            .execute(&state.pool).await?;

        Ok(updated)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let old_project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await?;
        let old_snapshot = serde_json::to_string(&old_project).unwrap_or_default();

        sqlx::query("DELETE FROM projects WHERE id = ?").bind(id).execute(&state.pool).await?;

        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('delete', 'project', ?, ?, NULL, ?)")
            .bind(id).bind(&old_snapshot).bind(&now)
            .execute(&state.pool).await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
