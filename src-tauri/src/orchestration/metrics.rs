use serde::{Deserialize, Serialize};
use sqlx::AnyPool;
use crate::db::{DbBackend, compat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub total_tasks: i64,
    pub completed: i64,
    pub queued: i64,
    pub in_progress: i64,
    pub blocked: i64,
    pub validating: i64,
    pub failed_attempts: i64,
    pub agents_online: i64,
    pub avg_confidence: Option<f64>,
    pub avg_completion_time_minutes: Option<f64>,
    pub tasks_completed_24h: i64,
    pub task_type_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub name: String,
    pub status: String,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub success_rate: f64,
    pub avg_confidence: f64,
    pub avg_completion_time_minutes: f64,
    pub current_tasks: Vec<String>,
    pub skills_success_rate: serde_json::Value,
}

pub async fn project_metrics(
    pool: &AnyPool,
    backend: &DbBackend,
    project_id: i64,
) -> Result<ProjectMetrics, sqlx::Error> {
    let bc = compat::bigint_cast(backend);

    let total: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let completed: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'completed'"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let queued: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'queued'"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let in_progress: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state IN ('claimed', 'executing')"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let blocked: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'blocked'"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let validating: i64 = sqlx::query_scalar(
        &format!("SELECT COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'validating'"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let failed_attempts: i64 = sqlx::query_scalar(
        &format!("SELECT COALESCE(SUM(tc.attempt_count), 0){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.attempt_count > 0"),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let agents_online: i64 =
        sqlx::query_scalar(&format!("SELECT COUNT(*){bc} FROM agents WHERE status != 'offline'"))
            .fetch_one(pool)
            .await?;

    let avg_confidence: Option<f64> = sqlx::query_scalar(
        compat::avg_confidence_query(backend),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let tasks_completed_24h: i64 = sqlx::query_scalar(
        compat::tasks_completed_24h_query(backend),
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let type_rows: Vec<(String, i64)> = sqlx::query_as(
        &format!("SELECT tc.type, COUNT(*){bc} FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 GROUP BY tc.type"),
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let mut type_breakdown = serde_json::Map::new();
    for (task_type, count) in type_rows {
        type_breakdown.insert(task_type, serde_json::json!({"count": count}));
    }

    Ok(ProjectMetrics {
        total_tasks: total,
        completed,
        queued,
        in_progress,
        blocked,
        validating,
        failed_attempts,
        agents_online,
        avg_confidence,
        avg_completion_time_minutes: None,
        tasks_completed_24h,
        task_type_breakdown: serde_json::Value::Object(type_breakdown),
    })
}

pub async fn agent_metrics(
    pool: &AnyPool,
    _backend: &DbBackend,
    agent_id: &str,
) -> Result<AgentMetrics, sqlx::Error> {
    let agent = sqlx::query_as::<_, crate::models::Agent>("SELECT * FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_one(pool)
        .await?;

    let stats =
        sqlx::query_as::<_, crate::models::AgentStats>(
            "SELECT * FROM agent_stats WHERE agent_id = $1",
        )
        .bind(agent_id)
        .fetch_one(pool)
        .await?;

    let total = stats.tasks_completed + stats.tasks_failed;
    let success_rate = if total > 0 {
        stats.tasks_completed as f64 / total as f64
    } else {
        0.0
    };
    let avg_confidence = if stats.tasks_completed > 0 {
        stats.total_confidence / stats.tasks_completed as f64
    } else {
        0.0
    };
    let avg_time = if stats.tasks_completed > 0 {
        stats.total_completion_time_seconds as f64 / stats.tasks_completed as f64 / 60.0
    } else {
        0.0
    };

    let current_tasks: Vec<String> = sqlx::query_scalar(
        "SELECT i.identifier FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE tc.claimed_by = $1 AND tc.task_state IN ('claimed', 'executing')",
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await?;

    let skills_breakdown = stats.skills_breakdown_json();

    Ok(AgentMetrics {
        agent_id: agent.id,
        name: agent.name,
        status: agent.status,
        tasks_completed: stats.tasks_completed,
        tasks_failed: stats.tasks_failed,
        success_rate,
        avg_confidence,
        avg_completion_time_minutes: avg_time,
        current_tasks,
        skills_success_rate: skills_breakdown,
    })
}
