use crate::state::AppState;
use crate::models::Label;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateLabelInput {
    pub project_id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Deserialize)]
pub struct UpdateLabelInput {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[tauri::command]
pub fn list_labels(state: State<AppState>, project_id: i64) -> Result<Vec<Label>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE project_id = $1 ORDER BY name")
            .bind(project_id).fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_label(state: State<AppState>, input: CreateLabelInput) -> Result<Label, String> {
    if input.name.trim().is_empty() {
        return Err(sqlx::Error::Protocol("Label name cannot be empty".to_string()).to_string());
    }
    state.rt.block_on(async {
        let id: i64 = sqlx::query_scalar("INSERT INTO labels (project_id, name, color) VALUES ($1, $2, $3) RETURNING id")
            .bind(input.project_id).bind(input.name.trim()).bind(&input.color)
            .fetch_one(&state.pool).await?;
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_label(state: State<AppState>, id: i64, input: UpdateLabelInput) -> Result<Label, String> {
    state.rt.block_on(async {
        if let Some(ref name) = input.name {
            sqlx::query("UPDATE labels SET name = $1 WHERE id = $2")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref color) = input.color {
            sqlx::query("UPDATE labels SET color = $1 WHERE id = $2")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_label(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM labels WHERE id = $1").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
