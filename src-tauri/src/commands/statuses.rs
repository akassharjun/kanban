use crate::state::AppState;
use crate::models::Status;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateStatusInput {
    pub project_id: i64,
    pub name: String,
    pub category: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateStatusInput {
    pub name: Option<String>,
    pub category: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: Option<i64>,
}

#[tauri::command]
pub fn list_statuses(state: State<AppState>, project_id: i64) -> Result<Vec<Status>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE project_id = $1 ORDER BY position")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_status(state: State<AppState>, input: CreateStatusInput) -> Result<Status, String> {
    if input.name.trim().is_empty() {
        return Err(sqlx::Error::Protocol("Status name cannot be empty".to_string()).to_string());
    }
    let valid_categories = ["unstarted", "started", "blocked", "completed", "discarded"];
    if !valid_categories.contains(&input.category.as_str()) {
        return Err(sqlx::Error::Protocol(format!("Invalid category '{}'. Must be one of: {}", input.category, valid_categories.join(", "))).to_string());
    }
    state.rt.block_on(async {
        // Get max position
        let max_pos: Option<i64> = sqlx::query_scalar("SELECT MAX(position) FROM statuses WHERE project_id = $1")
            .bind(input.project_id)
            .fetch_one(&state.pool)
            .await?;
        let position = max_pos.unwrap_or(-1) + 1;

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO statuses (project_id, name, category, color, icon, position) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(input.project_id)
        .bind(input.name.trim())
        .bind(&input.category)
        .bind(&input.color)
        .bind(&input.icon)
        .bind(position)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_status(state: State<AppState>, id: i64, input: UpdateStatusInput) -> Result<Status, String> {
    state.rt.block_on(async {
        if let Some(name) = &input.name {
            sqlx::query("UPDATE statuses SET name = $1 WHERE id = $2")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(category) = &input.category {
            sqlx::query("UPDATE statuses SET category = $1 WHERE id = $2")
                .bind(category).bind(id).execute(&state.pool).await?;
        }
        if let Some(color) = &input.color {
            sqlx::query("UPDATE statuses SET color = $1 WHERE id = $2")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        if let Some(icon) = &input.icon {
            sqlx::query("UPDATE statuses SET icon = $1 WHERE id = $2")
                .bind(icon).bind(id).execute(&state.pool).await?;
        }
        if let Some(position) = &input.position {
            sqlx::query("UPDATE statuses SET position = $1 WHERE id = $2")
                .bind(position).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_status(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM issues WHERE status_id = $1"
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

        if count > 0 {
            return Err(sqlx::Error::Protocol(
                format!("Cannot delete status: {} issue(s) are currently using it. Move or delete those issues first.", count)
            ));
        }

        sqlx::query("DELETE FROM statuses WHERE id = $1").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn reorder_statuses(state: State<AppState>, status_ids: Vec<i64>) -> Result<(), String> {
    state.rt.block_on(async {
        for (i, id) in status_ids.iter().enumerate() {
            sqlx::query("UPDATE statuses SET position = $1 WHERE id = $2")
                .bind(i as i64).bind(id).execute(&state.pool).await?;
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
