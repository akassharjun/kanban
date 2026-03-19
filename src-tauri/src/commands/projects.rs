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
    pub path: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: Option<String>,
    pub path: Option<String>,
    pub stale_days: Option<i64>,
    pub stale_close_status_id: Option<i64>,
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE deleted_at IS NULL ORDER BY name")
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project(state: State<AppState>, id: i64) -> Result<Project, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(state: State<AppState>, input: CreateProjectInput) -> Result<Project, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Insert project
        let project_id: i64 = sqlx::query_scalar(
            "INSERT INTO projects (name, description, icon, status, prefix, issue_counter, path, created_at, updated_at) VALUES ($1, $2, $3, 'active', $4, 0, $5, $6, $7) RETURNING id"
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.icon)
        .bind(&input.prefix)
        .bind(&input.path)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

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
                "INSERT INTO statuses (project_id, name, category, color, position) VALUES ($1, $2, $3, $4, $5)"
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
        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await?;

        let snapshot = serde_json::to_string(&project).unwrap_or_default();
        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('create', 'project', $1, NULL, $2, $3)")
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
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Get old state for undo
        let old_project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
        let old_snapshot = serde_json::to_string(&old_project).unwrap_or_default();

        let mut qb = sqlx::QueryBuilder::new("UPDATE projects SET updated_at = ");
        qb.push_bind(&now);

        if let Some(name) = &input.name {
            qb.push(", name = ");
            qb.push_bind(name);
        }
        if let Some(desc) = &input.description {
            qb.push(", description = ");
            qb.push_bind(desc);
        }
        if let Some(icon) = &input.icon {
            qb.push(", icon = ");
            qb.push_bind(icon);
        }
        if let Some(status) = &input.status {
            qb.push(", status = ");
            qb.push_bind(status);
        }
        if let Some(path) = &input.path {
            qb.push(", path = ");
            qb.push_bind(path);
        }
        if let Some(stale_days) = &input.stale_days {
            qb.push(", stale_days = ");
            qb.push_bind(*stale_days);
        }
        if let Some(stale_close_status_id) = &input.stale_close_status_id {
            qb.push(", stale_close_status_id = ");
            qb.push_bind(*stale_close_status_id);
        }

        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.build().execute(&state.pool).await?;

        let updated = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let new_snapshot = serde_json::to_string(&updated).unwrap_or_default();

        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('update', 'project', $1, $2, $3, $4)")
            .bind(id).bind(&old_snapshot).bind(&new_snapshot).bind(&now)
            .execute(&state.pool).await?;

        Ok(updated)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let old_project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let old_snapshot = serde_json::to_string(&old_project).unwrap_or_default();

        sqlx::query("UPDATE projects SET deleted_at = $1 WHERE id = $2")
            .bind(&now).bind(id).execute(&state.pool).await?;

        // Cascade: soft-delete related data for the deleted project
        // Remove issues belonging to this project (and their label associations)
        sqlx::query("DELETE FROM issue_labels WHERE issue_id IN (SELECT id FROM issues WHERE project_id = $1)")
            .bind(id).execute(&state.pool).await?;
        sqlx::query("DELETE FROM comments WHERE issue_id IN (SELECT id FROM issues WHERE project_id = $1)")
            .bind(id).execute(&state.pool).await?;
        sqlx::query("DELETE FROM activity_log WHERE issue_id IN (SELECT id FROM issues WHERE project_id = $1)")
            .bind(id).execute(&state.pool).await?;
        sqlx::query("DELETE FROM issues WHERE project_id = $1")
            .bind(id).execute(&state.pool).await?;
        sqlx::query("DELETE FROM statuses WHERE project_id = $1")
            .bind(id).execute(&state.pool).await?;
        sqlx::query("DELETE FROM labels WHERE project_id = $1")
            .bind(id).execute(&state.pool).await?;

        sqlx::query("INSERT INTO undo_log (operation_type, entity_type, entity_id, snapshot_before, snapshot_after, timestamp) VALUES ('delete', 'project', $1, $2, NULL, $3)")
            .bind(id).bind(&old_snapshot).bind(&now)
            .execute(&state.pool).await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn restore_project(state: State<AppState>, id: i64) -> Result<Project, String> {
    state.rt.block_on(async {
        sqlx::query("UPDATE projects SET deleted_at = NULL WHERE id = $1")
            .bind(id).execute(&state.pool).await?;

        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}
