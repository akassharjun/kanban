use crate::models::agent::{Agent, AgentStats};
use crate::state::AppState;
use crate::db::compat::jsonb_cast;
use serde::Deserialize;
use tauri::{Emitter, State};

#[derive(Deserialize)]
pub struct RegisterAgentInput {
    pub name: String,
    pub agent_type: Option<String>,
    pub skills: Vec<String>,
    pub task_types: Vec<String>,
    pub max_concurrent: Option<i64>,
    pub max_complexity: Option<String>,
    pub worktree_path: Option<String>,
}

#[tauri::command]
pub fn register_agent(app: tauri::AppHandle, state: State<AppState>, input: RegisterAgentInput) -> Result<Agent, String> {
    state.rt.block_on(async {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let skills = serde_json::to_string(&input.skills).unwrap_or_else(|_| "[]".to_string());
        let task_types = serde_json::to_string(&input.task_types).unwrap_or_else(|_| "[]".to_string());
        let max_concurrent = input.max_concurrent.unwrap_or(1);
        let max_complexity = input.max_complexity.unwrap_or_else(|| "large".to_string());

        // Generate name if not provided
        let agent_name = if input.name.is_empty() {
            crate::orchestration::names::generate_agent_name()
        } else {
            input.name.clone()
        };

        // Determine avatar color based on agent type
        let agent_type_str = input.agent_type.as_deref().unwrap_or("custom");
        let avatar_color = match agent_type_str {
            "claude" | "claude-code" => "#f97316",
            "codex" => "#22c55e",
            "gemini" => "#3b82f6",
            _ => "#8b5cf6",
        };

        let mut tx = state.pool.begin().await?;

        // Create a member for this agent
        let member_id: i64 = sqlx::query_scalar(
            "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(format!("[{}] {}", agent_type_str, &agent_name))
        .bind(&agent_name)
        .bind(Option::<String>::None)
        .bind(avatar_color)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await?;

        let jb = jsonb_cast(&state.backend);
        sqlx::query(
            &format!("INSERT INTO agents (id, name, agent_type, skills, task_types, max_concurrent, max_complexity, member_id, status, registered_at, last_heartbeat, worktree_path) VALUES ($1, $2, $3, $4{jb}, $5{jb}, $6, $7, $8, 'idle', $9, $10, $11)")
        )
        .bind(&id)
        .bind(&agent_name)
        .bind(&input.agent_type)
        .bind(&skills)
        .bind(&task_types)
        .bind(max_concurrent)
        .bind(&max_complexity)
        .bind(member_id)
        .bind(&now)
        .bind(&now)
        .bind(&input.worktree_path)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            &format!("INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES ($1, 0, 0, 0.0, 0, '{{}}'{jb})")
        )
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        let _ = app.emit("db-changed", ());
        Ok(agent)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn agent_heartbeat(app: tauri::AppHandle, state: State<AppState>, agent_id: String) -> Result<Agent, String> {
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

        let _ = app.emit("db-changed", ());
        Ok(updated)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn deregister_agent(app: tauri::AppHandle, state: State<AppState>, agent_id: String) -> Result<(), String> {
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

        // Reassign agent's member to canonical agent-type member before deleting
        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_one(&state.pool)
            .await?;

        if let Some(member_id) = agent.member_id {
            let agent_type_str = agent.agent_type.as_deref().unwrap_or("custom");
            let canonical_name = match agent_type_str {
                "claude" | "claude-code" => "[claude] Claude",
                "codex" => "[codex] Codex",
                "gemini" => "[gemini] Gemini",
                _ => "[custom] Agent",
            };
            let avatar_color = match agent_type_str {
                "claude" | "claude-code" => "#f97316",
                "codex" => "#22c55e",
                "gemini" => "#3b82f6",
                _ => "#8b5cf6",
            };

            let canonical_id: i64 = match sqlx::query_scalar::<_, i64>("SELECT id FROM members WHERE name = $1")
                .bind(canonical_name).fetch_optional(&state.pool).await? {
                Some(id) => id,
                None => {
                    sqlx::query_scalar("INSERT INTO members (name, display_name, avatar_color, created_at) VALUES ($1, $2, $3, $4) RETURNING id")
                        .bind(canonical_name)
                        .bind(canonical_name.split("] ").last().unwrap_or("Agent"))
                        .bind(avatar_color)
                        .bind(&now)
                        .fetch_one(&state.pool).await?
                }
            };

            if member_id != canonical_id {
                sqlx::query("UPDATE issues SET assignee_id = $1 WHERE assignee_id = $2")
                    .bind(canonical_id).bind(member_id).execute(&state.pool).await?;
                sqlx::query("UPDATE comments SET member_id = $1 WHERE member_id = $2")
                    .bind(canonical_id).bind(member_id).execute(&state.pool).await?;
            }

            sqlx::query("DELETE FROM agents WHERE id = $1")
                .bind(&agent_id).execute(&state.pool).await?;

            if member_id != canonical_id {
                sqlx::query("DELETE FROM members WHERE id = $1")
                    .bind(member_id).execute(&state.pool).await?;
            }
        } else {
            sqlx::query("DELETE FROM agents WHERE id = $1")
                .bind(&agent_id).execute(&state.pool).await?;
        }

        let _ = app.emit("db-changed", ());
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

#[tauri::command]
pub fn get_project_agent_config(state: State<AppState>, project_id: i64) -> Result<crate::models::ProjectAgentConfig, String> {
    state.rt.block_on(async {
        match sqlx::query_as::<_, crate::models::ProjectAgentConfig>(
            "SELECT * FROM project_agent_config WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_optional(&state.pool)
        .await? {
            Some(config) => Ok(config),
            None => {
                sqlx::query("INSERT INTO project_agent_config (project_id) VALUES ($1) ON CONFLICT DO NOTHING")
                    .bind(project_id)
                    .execute(&state.pool)
                    .await?;
                sqlx::query_as::<_, crate::models::ProjectAgentConfig>(
                    "SELECT * FROM project_agent_config WHERE project_id = $1"
                )
                .bind(project_id)
                .fetch_one(&state.pool)
                .await
            }
        }
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[derive(Deserialize)]
pub struct UpdateAgentConfigInput {
    pub auto_accept_threshold: Option<f64>,
    pub human_review_threshold: Option<f64>,
    pub max_attempts: Option<i64>,
    pub heartbeat_interval_seconds: Option<i64>,
    pub missed_heartbeats_before_offline: Option<i64>,
}

#[tauri::command]
pub fn update_project_agent_config(state: State<AppState>, project_id: i64, input: UpdateAgentConfigInput) -> Result<crate::models::ProjectAgentConfig, String> {
    state.rt.block_on(async {
        sqlx::query("INSERT INTO project_agent_config (project_id) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(project_id)
            .execute(&state.pool)
            .await?;

        let mut qb = sqlx::QueryBuilder::new("UPDATE project_agent_config SET project_id = ");
        qb.push_bind(project_id);

        if let Some(v) = input.auto_accept_threshold {
            qb.push(", auto_accept_threshold = "); qb.push_bind(v);
        }
        if let Some(v) = input.human_review_threshold {
            qb.push(", human_review_threshold = "); qb.push_bind(v);
        }
        if let Some(v) = input.max_attempts {
            qb.push(", max_attempts = "); qb.push_bind(v);
        }
        if let Some(v) = input.heartbeat_interval_seconds {
            qb.push(", heartbeat_interval_seconds = "); qb.push_bind(v);
        }
        if let Some(v) = input.missed_heartbeats_before_offline {
            qb.push(", missed_heartbeats_before_offline = "); qb.push_bind(v);
        }

        qb.push(" WHERE project_id = "); qb.push_bind(project_id);
        qb.build().execute(&state.pool).await?;

        sqlx::query_as::<_, crate::models::ProjectAgentConfig>(
            "SELECT * FROM project_agent_config WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
    }).map_err(|e: sqlx::Error| e.to_string())
}
