use crate::models::Hook;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateHookInput {
    pub project_id: i64,
    pub event_type: String,
    pub command: String,
}

#[tauri::command]
pub fn list_hooks(state: State<AppState>, project_id: i64) -> Result<Vec<Hook>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Hook>("SELECT * FROM hooks WHERE project_id = $1 ORDER BY id")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn create_hook(state: State<AppState>, input: CreateHookInput) -> Result<Hook, String> {
    state.rt.block_on(async {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO hooks (project_id, event_type, command) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.event_type)
        .bind(&input.command)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, Hook>("SELECT * FROM hooks WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_hook(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM hooks WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
