use crate::state::AppState;
use crate::models::Issue;
use tauri::State;

#[tauri::command]
pub fn record_view(state: State<AppState>, issue_id: i64, member_id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        // Upsert: insert or update viewed_at
        sqlx::query(
            "INSERT INTO recently_viewed (issue_id, member_id, viewed_at) VALUES ($1, $2, $3) ON CONFLICT(issue_id, member_id) DO UPDATE SET viewed_at = $3"
        )
        .bind(issue_id)
        .bind(member_id)
        .bind(&now)
        .execute(&state.pool)
        .await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_recently_viewed(state: State<AppState>, member_id: i64, limit: Option<i64>) -> Result<Vec<Issue>, String> {
    let limit = limit.unwrap_or(10);
    state.rt.block_on(async {
        sqlx::query_as::<_, Issue>(
            "SELECT i.* FROM issues i JOIN recently_viewed rv ON i.id = rv.issue_id WHERE rv.member_id = $1 ORDER BY rv.viewed_at DESC LIMIT $2"
        )
        .bind(member_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}
