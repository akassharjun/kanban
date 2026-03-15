use crate::models::agent::{Agent, AgentStats};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct RegisterAgentInput {
    pub name: String,
    pub agent_type: Option<String>,
    pub skills: Vec<String>,
    pub task_types: Vec<String>,
    pub max_concurrent: Option<i64>,
    pub max_complexity: Option<String>,
}

#[tauri::command]
pub fn register_agent(state: State<AppState>, input: RegisterAgentInput) -> Result<Agent, String> {
    state.rt.block_on(async {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let skills = serde_json::to_string(&input.skills).unwrap_or_else(|_| "[]".to_string());
        let task_types = serde_json::to_string(&input.task_types).unwrap_or_else(|_| "[]".to_string());
        let max_concurrent = input.max_concurrent.unwrap_or(1);
        let max_complexity = input.max_complexity.unwrap_or_else(|| "large".to_string());

        let mut tx = state.pool.begin().await?;

        sqlx::query(
            "INSERT INTO agents (id, name, agent_type, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat) VALUES ($1, $2, $3, $4, $5, $6, $7, 'idle', $8, $9)"
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.agent_type)
        .bind(&skills)
        .bind(&task_types)
        .bind(max_concurrent)
        .bind(&max_complexity)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES ($1, 0, 0, 0.0, 0, '{}')"
        )
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(agent)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn agent_heartbeat(state: State<AppState>, agent_id: String) -> Result<Agent, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Update last_heartbeat
        sqlx::query("UPDATE agents SET last_heartbeat = $1 WHERE id = $2")
            .bind(&now)
            .bind(&agent_id)
            .execute(&state.pool)
            .await?;

        // Count active tasks (claimed or executing)
        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')"
        )
        .bind(&agent_id)
        .fetch_one(&state.pool)
        .await?;

        // Get agent to check max_concurrent
        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_one(&state.pool)
            .await?;

        let new_status = if active_count >= agent.max_concurrent { "busy" } else { "idle" };

        sqlx::query("UPDATE agents SET status = $1 WHERE id = $2")
            .bind(new_status)
            .bind(&agent_id)
            .execute(&state.pool)
            .await?;

        let updated = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_one(&state.pool)
            .await?;

        Ok(updated)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn deregister_agent(state: State<AppState>, agent_id: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Find all active tasks for this agent
        let active_tasks: Vec<(i64,)> = sqlx::query_as(
            "SELECT issue_id FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')"
        )
        .bind(&agent_id)
        .fetch_all(&state.pool)
        .await?;

        // Reclaim each active task back to queued
        for (issue_id,) in &active_tasks {
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1 AND claimed_by = $2"
            )
            .bind(issue_id)
            .bind(&agent_id)
            .execute(&state.pool)
            .await?;

            // Log reclaim in execution_logs
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, 0, 'reclaimed', 'Agent deregistered, task reclaimed to queue', $3)"
            )
            .bind(issue_id)
            .bind(&agent_id)
            .bind(&now)
            .execute(&state.pool)
            .await?;
        }

        // Delete the agent (cascade deletes agent_stats)
        sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(&agent_id)
            .execute(&state.pool)
            .await?;

        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_agents(state: State<AppState>) -> Result<Vec<Agent>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at DESC")
            .fetch_all(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_agent_stats(state: State<AppState>, agent_id: String) -> Result<AgentStats, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = $1")
            .bind(&agent_id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}
