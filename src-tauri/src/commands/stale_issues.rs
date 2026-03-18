use crate::state::AppState;
use crate::models::Issue;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateStaleConfigInput {
    pub stale_days: Option<i64>,
    pub stale_close_status_id: Option<i64>,
}

#[tauri::command]
pub fn update_stale_config(state: State<AppState>, project_id: i64, input: UpdateStaleConfigInput) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // stale_days: None means disabled, Some(n) means n days
        let stale_days = input.stale_days;
        let stale_close_status_id = input.stale_close_status_id;

        sqlx::query("UPDATE projects SET stale_days = $1, stale_close_status_id = $2, updated_at = $3 WHERE id = $4")
            .bind(stale_days)
            .bind(stale_close_status_id)
            .bind(&now)
            .bind(project_id)
            .execute(&state.pool)
            .await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn check_stale_issues(state: State<AppState>, project_id: i64) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Get project stale config
        let row: Option<(Option<i64>, Option<i64>, String)> = sqlx::query_as(
            "SELECT stale_days, stale_close_status_id, prefix FROM projects WHERE id = $1"
        )
        .bind(project_id)
        .fetch_optional(&state.pool)
        .await?;

        let (stale_days, stale_close_status_id, _prefix) = match row {
            Some((Some(days), Some(status_id), prefix)) => (days, status_id, prefix),
            _ => return Ok(vec![]), // Stale check disabled or not configured
        };

        // Find issues in 'unstarted' category statuses that haven't been updated in stale_days
        let stale_issues = sqlx::query_as::<_, Issue>(
            "SELECT i.* FROM issues i \
             JOIN statuses s ON i.status_id = s.id \
             WHERE i.project_id = $1 \
             AND s.category = 'unstarted' \
             AND datetime(i.updated_at) < datetime('now', $2) \
             ORDER BY i.updated_at ASC"
        )
        .bind(project_id)
        .bind(format!("-{} days", stale_days))
        .fetch_all(&state.pool)
        .await?;

        let mut closed_issues = Vec::new();

        for issue in &stale_issues {
            // Move to stale close status
            sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                .bind(stale_close_status_id)
                .bind(&now)
                .bind(issue.id)
                .execute(&state.pool)
                .await?;

            // Log activity
            let msg = format!("Auto-closed as stale (no activity for {} days)", stale_days);
            sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)")
                .bind(issue.id)
                .bind("status")
                .bind(issue.status_id.to_string())
                .bind(stale_close_status_id.to_string())
                .execute(&state.pool)
                .await?;

            sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, 'comment', NULL, $2, $3)")
                .bind(issue.id)
                .bind(&msg)
                .bind(&now)
                .execute(&state.pool)
                .await?;

            // Create notification
            let notif_msg = format!("{} was auto-closed as stale", issue.identifier);
            sqlx::query("INSERT INTO notifications (type, issue_id, message, read, created_at) VALUES ('stale_close', $1, $2, 0, $3)")
                .bind(issue.id)
                .bind(&notif_msg)
                .bind(&now)
                .execute(&state.pool)
                .await?;

            // Re-fetch updated issue
            let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue.id)
                .fetch_one(&state.pool)
                .await?;
            closed_issues.push(updated);
        }

        Ok(closed_issues)
    }).map_err(|e: sqlx::Error| e.to_string())
}
