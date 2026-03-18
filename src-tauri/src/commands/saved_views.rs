use crate::state::AppState;
use crate::models::SavedView;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateSavedViewInput {
    pub project_id: i64,
    pub name: String,
    pub filters: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub view_mode: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSavedViewInput {
    pub name: Option<String>,
    pub filters: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub view_mode: Option<String>,
}

#[tauri::command]
pub fn list_saved_views(state: State<AppState>, project_id: i64) -> Result<Vec<SavedView>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, SavedView>("SELECT * FROM saved_views WHERE project_id = $1 ORDER BY created_at ASC")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_saved_view(state: State<AppState>, input: CreateSavedViewInput) -> Result<SavedView, String> {
    if input.name.trim().is_empty() {
        return Err("name cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let filters = input.filters.unwrap_or_else(|| "{}".to_string());
        let sort_direction = input.sort_direction.unwrap_or_else(|| "asc".to_string());
        let view_mode = input.view_mode.unwrap_or_else(|| "board".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO saved_views (project_id, name, filters, sort_by, sort_direction, view_mode, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.name)
        .bind(&filters)
        .bind(&input.sort_by)
        .bind(&sort_direction)
        .bind(&view_mode)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, SavedView>("SELECT * FROM saved_views WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_saved_view(state: State<AppState>, id: i64, input: UpdateSavedViewInput) -> Result<SavedView, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        if let Some(ref name) = input.name {
            sqlx::query("UPDATE saved_views SET name = $1, updated_at = $2 WHERE id = $3")
                .bind(name).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref filters) = input.filters {
            sqlx::query("UPDATE saved_views SET filters = $1, updated_at = $2 WHERE id = $3")
                .bind(filters).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref sort_by) = input.sort_by {
            sqlx::query("UPDATE saved_views SET sort_by = $1, updated_at = $2 WHERE id = $3")
                .bind(sort_by).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref sort_direction) = input.sort_direction {
            sqlx::query("UPDATE saved_views SET sort_direction = $1, updated_at = $2 WHERE id = $3")
                .bind(sort_direction).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref view_mode) = input.view_mode {
            sqlx::query("UPDATE saved_views SET view_mode = $1, updated_at = $2 WHERE id = $3")
                .bind(view_mode).bind(&now).bind(id).execute(&state.pool).await?;
        }

        sqlx::query_as::<_, SavedView>("SELECT * FROM saved_views WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_saved_view(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM saved_views WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
