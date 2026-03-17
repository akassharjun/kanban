use crate::models::agent::{ExecutionLog, TaskContract};
use crate::orchestration::timeout::update_agent_activity;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct LogEntryInput {
    pub identifier: String,
    pub agent_id: String,
    pub entry_type: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub fn log_task_activity(state: State<AppState>, input: LogEntryInput) -> Result<i64, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            update_agent_activity(&state.pool, &input.agent_id).await;

            // Resolve identifier to issue_id
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                    .bind(&input.identifier)
                    .fetch_one(&state.pool)
                    .await?;

            // Get current attempt_count from task_contracts + 1
            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            let attempt_number = contract.attempt_count + 1;

            let metadata_str = input
                .metadata
                .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()));

            // Insert into execution_logs
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            )
            .bind(issue_id)
            .bind(&input.agent_id)
            .bind(attempt_number)
            .bind(&input.entry_type)
            .bind(&input.message)
            .bind(&metadata_str)
            .bind(&now)
            .fetch_one(&state.pool)
            .await?;

            Ok(id)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn task_replay(
    state: State<AppState>,
    identifier: String,
) -> Result<Vec<ExecutionLog>, String> {
    state
        .rt
        .block_on(async {
            // Resolve identifier to issue_id
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            sqlx::query_as::<_, ExecutionLog>(
                "SELECT * FROM execution_logs WHERE issue_id = $1 ORDER BY timestamp ASC",
            )
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn task_attempts(
    state: State<AppState>,
    identifier: String,
) -> Result<serde_json::Value, String> {
    state
        .rt
        .block_on(async {
            // Resolve identifier to issue_id
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            // Get task_contracts row
            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Parse context JSON, extract prior_attempts
            let context: serde_json::Value = contract.context_json();
            let prior_attempts = context
                .get("prior_attempts")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]));

            Ok(serde_json::json!({
                "identifier": identifier,
                "total_attempts": contract.attempt_count,
                "prior_attempts": prior_attempts,
            }))
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn recent_activity(
    state: State<AppState>,
    project_id: i64,
    limit: Option<i64>,
) -> Result<Vec<ExecutionLog>, String> {
    state
        .rt
        .block_on(async {
            let lim = limit.unwrap_or(50);
            sqlx::query_as::<_, ExecutionLog>(
                "SELECT el.* FROM execution_logs el JOIN issues i ON el.issue_id = i.id WHERE i.project_id = $1 ORDER BY el.timestamp DESC LIMIT $2",
            )
            .bind(project_id)
            .bind(lim)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}
