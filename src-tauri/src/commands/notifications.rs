use crate::state::AppState;
use crate::models::Notification;
use tauri::State;

#[tauri::command]
pub fn list_notifications(state: State<AppState>) -> Result<Vec<Notification>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Notification>("SELECT * FROM notifications ORDER BY created_at DESC LIMIT 100")
            .fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unread_notification_count(state: State<AppState>) -> Result<i64, String> {
    state.rt.block_on(async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE read = 0")
            .fetch_one(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_notification_read(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("UPDATE notifications SET read = 1 WHERE id = ?")
            .bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<AppState>) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("UPDATE notifications SET read = 1")
            .execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn clear_notifications(state: State<AppState>) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM notifications")
            .execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
