use crate::state::AppState;
use crate::models::Issue;
use tauri::State;

#[tauri::command]
pub fn star_issue(state: State<AppState>, issue_id: i64, member_id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO starred_issues (issue_id, member_id, created_at) VALUES ($1, $2, $3)"
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
pub fn unstar_issue(state: State<AppState>, issue_id: i64, member_id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM starred_issues WHERE issue_id = $1 AND member_id = $2")
            .bind(issue_id)
            .bind(member_id)
            .execute(&state.pool)
            .await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_starred(state: State<AppState>, member_id: i64) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Issue>(
            "SELECT i.* FROM issues i JOIN starred_issues s ON i.id = s.issue_id WHERE s.member_id = $1 ORDER BY s.created_at DESC"
        )
        .bind(member_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_starred(state: State<AppState>, issue_id: i64, member_id: i64) -> Result<bool, String> {
    state.rt.block_on(async {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM starred_issues WHERE issue_id = $1 AND member_id = $2"
        )
        .bind(issue_id)
        .bind(member_id)
        .fetch_one(&state.pool)
        .await?;
        Ok(count > 0)
    }).map_err(|e: sqlx::Error| e.to_string())
}
