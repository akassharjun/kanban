use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetric {
    pub date: String,
    pub completed: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    pub agent_id: String,
    pub agent_name: String,
    pub total_tasks: i64,
    pub completed: i64,
    pub failed: i64,
    pub rejected: i64,
    pub timeout: i64,
    pub success_rate: f64,
    pub avg_confidence: f64,
    pub avg_duration_minutes: f64,
    pub total_lines_changed: i64,
    pub tasks_by_type: HashMap<String, i64>,
    pub tasks_by_complexity: HashMap<String, i64>,
    pub daily_completions: Vec<DailyMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRanking {
    pub agent_id: String,
    pub agent_name: String,
    pub score: f64,
    pub success_rate: f64,
    pub avg_confidence: f64,
    pub tasks_completed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAgentSummary {
    pub total_agent_tasks: i64,
    pub total_completed: i64,
    pub total_failed: i64,
    pub avg_completion_time_minutes: f64,
    pub agents_active: i64,
    pub top_performers: Vec<AgentRanking>,
    pub task_type_distribution: HashMap<String, i64>,
    pub completion_trend: Vec<DailyMetric>,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricInput {
    pub agent_id: String,
    pub task_identifier: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub confidence: Option<f64>,
    pub attempt_number: Option<i64>,
    pub outcome: String,
    pub complexity: Option<String>,
    pub task_type: Option<String>,
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
}

#[tauri::command]
pub fn record_task_metric(app: tauri::AppHandle, state: State<AppState>, input: RecordMetricInput) -> Result<(), String> {
    use tauri::Emitter;
    state.rt.block_on(async {
        sqlx::query(
            "INSERT INTO agent_task_metrics (agent_id, task_identifier, started_at, completed_at, duration_seconds, confidence, attempt_number, outcome, complexity, task_type, files_changed, lines_added, lines_removed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
        )
        .bind(&input.agent_id)
        .bind(&input.task_identifier)
        .bind(&input.started_at)
        .bind(&input.completed_at)
        .bind(input.duration_seconds.unwrap_or(0))
        .bind(input.confidence.unwrap_or(0.0))
        .bind(input.attempt_number.unwrap_or(1))
        .bind(&input.outcome)
        .bind(&input.complexity)
        .bind(&input.task_type)
        .bind(input.files_changed.unwrap_or(0))
        .bind(input.lines_added.unwrap_or(0))
        .bind(input.lines_removed.unwrap_or(0))
        .execute(&state.pool)
        .await?;

        let _ = app.emit("db-changed", ());
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_agent_performance(state: State<AppState>, agent_id: String) -> Result<AgentPerformance, String> {
    state.rt.block_on(async {
        // Get agent name
        let agent_name: String = sqlx::query_scalar("SELECT name FROM agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| agent_id.clone());

        // Outcome counts
        let completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 AND outcome = 'completed'"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let failed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 AND outcome = 'failed'"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let rejected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 AND outcome = 'rejected'"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let timeout: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 AND outcome = 'timeout'"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let total_tasks = completed + failed + rejected + timeout;
        let success_rate = if total_tasks > 0 { completed as f64 / total_tasks as f64 } else { 0.0 };

        // Avg confidence
        let avg_confidence: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(confidence), 0.0) FROM agent_task_metrics WHERE agent_id = $1 AND outcome = 'completed'"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        // Avg duration (minutes)
        let avg_duration_seconds: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(duration_seconds), 0.0) FROM agent_task_metrics WHERE agent_id = $1 AND duration_seconds > 0"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        // Lines changed
        let total_lines: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(lines_added + lines_removed), 0) FROM agent_task_metrics WHERE agent_id = $1"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        // Tasks by type
        let type_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT COALESCE(task_type, 'unknown'), COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 GROUP BY task_type"
        ).bind(&agent_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
        let tasks_by_type: HashMap<String, i64> = type_rows.into_iter().collect();

        // Tasks by complexity
        let complexity_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT COALESCE(complexity, 'unknown'), COUNT(*) FROM agent_task_metrics WHERE agent_id = $1 GROUP BY complexity"
        ).bind(&agent_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
        let tasks_by_complexity: HashMap<String, i64> = complexity_rows.into_iter().collect();

        // Daily completions (last 30 days)
        let daily_rows: Vec<(String, i64, i64)> = sqlx::query_as(
            "SELECT date(created_at) as d, SUM(CASE WHEN outcome='completed' THEN 1 ELSE 0 END), SUM(CASE WHEN outcome='failed' THEN 1 ELSE 0 END) FROM agent_task_metrics WHERE agent_id = $1 AND created_at >= datetime('now', '-30 days') GROUP BY d ORDER BY d"
        ).bind(&agent_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
        let daily_completions: Vec<DailyMetric> = daily_rows.into_iter().map(|(date, completed, failed)| {
            DailyMetric { date, completed, failed }
        }).collect();

        Ok(AgentPerformance {
            agent_id,
            agent_name,
            total_tasks,
            completed,
            failed,
            rejected,
            timeout,
            success_rate,
            avg_confidence,
            avg_duration_minutes: avg_duration_seconds / 60.0,
            total_lines_changed: total_lines,
            tasks_by_type,
            tasks_by_complexity,
            daily_completions,
        })
    })
}

#[tauri::command]
pub fn get_project_agent_summary(state: State<AppState>, _project_id: i64) -> Result<ProjectAgentSummary, String> {
    state.rt.block_on(async {
        let total_agent_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics"
        ).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let total_completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE outcome = 'completed'"
        ).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let total_failed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM agent_task_metrics WHERE outcome = 'failed'"
        ).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let avg_seconds: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(duration_seconds), 0.0) FROM agent_task_metrics WHERE duration_seconds > 0"
        ).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let agents_active: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT agent_id) FROM agent_task_metrics WHERE created_at >= datetime('now', '-7 days')"
        ).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        // Task type distribution
        let type_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT COALESCE(task_type, 'unknown'), COUNT(*) FROM agent_task_metrics GROUP BY task_type"
        ).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
        let task_type_distribution: HashMap<String, i64> = type_rows.into_iter().collect();

        // Completion trend (last 30 days)
        let daily_rows: Vec<(String, i64, i64)> = sqlx::query_as(
            "SELECT date(created_at) as d, SUM(CASE WHEN outcome='completed' THEN 1 ELSE 0 END), SUM(CASE WHEN outcome='failed' THEN 1 ELSE 0 END) FROM agent_task_metrics WHERE created_at >= datetime('now', '-30 days') GROUP BY d ORDER BY d"
        ).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
        let completion_trend: Vec<DailyMetric> = daily_rows.into_iter().map(|(date, completed, failed)| {
            DailyMetric { date, completed, failed }
        }).collect();

        // Top performers (leaderboard)
        let top_performers = compute_leaderboard(&state.pool).await?;

        Ok(ProjectAgentSummary {
            total_agent_tasks,
            total_completed,
            total_failed,
            avg_completion_time_minutes: avg_seconds / 60.0,
            agents_active,
            top_performers,
            task_type_distribution,
            completion_trend,
        })
    })
}

async fn compute_leaderboard(pool: &sqlx::AnyPool) -> Result<Vec<AgentRanking>, sqlx::Error> {
    // Get all agents with metrics
    let rows: Vec<(String, i64, i64, f64, f64)> = sqlx::query_as(
        "SELECT agent_id, SUM(CASE WHEN outcome='completed' THEN 1 ELSE 0 END), COUNT(*), COALESCE(AVG(CASE WHEN outcome='completed' THEN confidence ELSE NULL END), 0.0), COALESCE(AVG(CASE WHEN duration_seconds > 0 THEN duration_seconds ELSE NULL END), 0.0) FROM agent_task_metrics GROUP BY agent_id HAVING COUNT(*) > 0"
    ).fetch_all(pool).await?;

    let mut rankings: Vec<AgentRanking> = Vec::new();
    for (agent_id, completed, total, avg_confidence, avg_duration) in rows {
        let success_rate = if total > 0 { completed as f64 / total as f64 } else { 0.0 };
        // Speed score: faster is better. Normalize to 0-1 range (30min = 0, 0min = 1)
        let speed_score = (1.0 - (avg_duration / 1800.0).min(1.0)).max(0.0);
        let score = success_rate * 0.4 + avg_confidence * 0.3 + speed_score * 0.3;

        let agent_name: String = sqlx::query_scalar("SELECT name FROM agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or_else(|| agent_id.clone());

        rankings.push(AgentRanking {
            agent_id,
            agent_name,
            score,
            success_rate,
            avg_confidence,
            tasks_completed: completed,
        });
    }

    rankings.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(rankings)
}

#[tauri::command]
pub fn get_agent_leaderboard(state: State<AppState>, _project_id: i64) -> Result<Vec<AgentRanking>, String> {
    state.rt.block_on(async {
        compute_leaderboard(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}
