use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn health_check(state: State<AppState>) -> Result<String, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(&state.pool)
                .await
        })
        .map_err(|e| e.to_string())?;

    Ok("ok".to_string())
}
