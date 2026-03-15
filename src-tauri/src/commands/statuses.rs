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
        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE project_id = ? ORDER BY position")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_status(state: State<AppState>, input: CreateStatusInput) -> Result<Status, String> {
    state.rt.block_on(async {
        // Get max position
        let max_pos: Option<i64> = sqlx::query_scalar("SELECT MAX(position) FROM statuses WHERE project_id = ?")
            .bind(input.project_id)
            .fetch_one(&state.pool)
            .await?;
        let position = max_pos.unwrap_or(-1) + 1;

        let result = sqlx::query(
            "INSERT INTO statuses (project_id, name, category, color, icon, position) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(input.project_id)
        .bind(&input.name)
        .bind(&input.category)
        .bind(&input.color)
        .bind(&input.icon)
        .bind(position)
        .execute(&state.pool)
        .await?;

        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_status(state: State<AppState>, id: i64, input: UpdateStatusInput) -> Result<Status, String> {
    state.rt.block_on(async {
        if let Some(name) = &input.name {
            sqlx::query("UPDATE statuses SET name = ? WHERE id = ?")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(category) = &input.category {
            sqlx::query("UPDATE statuses SET category = ? WHERE id = ?")
                .bind(category).bind(id).execute(&state.pool).await?;
        }
        if let Some(color) = &input.color {
            sqlx::query("UPDATE statuses SET color = ? WHERE id = ?")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        if let Some(icon) = &input.icon {
            sqlx::query("UPDATE statuses SET icon = ? WHERE id = ?")
                .bind(icon).bind(id).execute(&state.pool).await?;
        }
        if let Some(position) = &input.position {
            sqlx::query("UPDATE statuses SET position = ? WHERE id = ?")
                .bind(position).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Status>("SELECT * FROM statuses WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_status(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM issues WHERE status_id = ?"
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

        if count > 0 {
            return Err(sqlx::Error::Protocol(
                format!("Cannot delete status: {} issue(s) are currently using it. Move or delete those issues first.", count)
            ));
        }

        sqlx::query("DELETE FROM statuses WHERE id = ?").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn reorder_statuses(state: State<AppState>, status_ids: Vec<i64>) -> Result<(), String> {
    state.rt.block_on(async {
        for (i, id) in status_ids.iter().enumerate() {
            sqlx::query("UPDATE statuses SET position = ? WHERE id = ?")
                .bind(i as i64).bind(id).execute(&state.pool).await?;
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
