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
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE project_id = ? ORDER BY name")
            .bind(project_id).fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_label(state: State<AppState>, input: CreateLabelInput) -> Result<Label, String> {
    state.rt.block_on(async {
        let result = sqlx::query("INSERT INTO labels (project_id, name, color) VALUES (?, ?, ?)")
            .bind(input.project_id).bind(&input.name).bind(&input.color)
            .execute(&state.pool).await?;
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
            .bind(result.last_insert_rowid()).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_label(state: State<AppState>, id: i64, input: UpdateLabelInput) -> Result<Label, String> {
    state.rt.block_on(async {
        if let Some(ref name) = input.name {
            sqlx::query("UPDATE labels SET name = ? WHERE id = ?")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref color) = input.color {
            sqlx::query("UPDATE labels SET color = ? WHERE id = ?")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_label(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM labels WHERE id = ?").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
