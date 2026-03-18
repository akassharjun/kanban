use crate::state::AppState;
use crate::models::Epic;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateEpicInput {
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateEpicInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub fn list_epics(state: State<AppState>, project_id: i64) -> Result<Vec<Epic>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Epic>("SELECT * FROM epics WHERE project_id = $1 ORDER BY created_at ASC")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_epic(state: State<AppState>, id: i64) -> Result<Epic, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Epic>("SELECT * FROM epics WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_epic(state: State<AppState>, input: CreateEpicInput) -> Result<Epic, String> {
    if input.title.trim().is_empty() {
        return Err("title cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let color = input.color.unwrap_or_else(|| "#6366f1".to_string());
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO epics (project_id, title, description, color, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&color)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, Epic>("SELECT * FROM epics WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_epic(state: State<AppState>, id: i64, input: UpdateEpicInput) -> Result<Epic, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        if let Some(ref title) = input.title {
            sqlx::query("UPDATE epics SET title = $1, updated_at = $2 WHERE id = $3")
                .bind(title).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref description) = input.description {
            sqlx::query("UPDATE epics SET description = $1, updated_at = $2 WHERE id = $3")
                .bind(description).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref color) = input.color {
            sqlx::query("UPDATE epics SET color = $1, updated_at = $2 WHERE id = $3")
                .bind(color).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref status) = input.status {
            sqlx::query("UPDATE epics SET status = $1, updated_at = $2 WHERE id = $3")
                .bind(status).bind(&now).bind(id).execute(&state.pool).await?;
        }

        sqlx::query_as::<_, Epic>("SELECT * FROM epics WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_epic(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        // Unlink issues from this epic first
        sqlx::query("UPDATE issues SET epic_id = NULL WHERE epic_id = $1")
            .bind(id).execute(&state.pool).await?;
        let result = sqlx::query("DELETE FROM epics WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
