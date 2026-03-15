use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub skills: String,
    pub task_types: String,
    pub max_concurrent: i64,
    pub max_complexity: String,
    pub status: String,
    pub registered_at: String,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentStats {
    pub agent_id: String,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub total_confidence: f64,
    pub total_completion_time_seconds: i64,
    pub skills_breakdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskContract {
    pub issue_id: i64,
    pub r#type: String,
    pub task_state: String,
    pub objective: String,
    pub context: String,
    pub constraints: String,
    pub success_criteria: String,
    pub required_skills: String,
    pub estimated_complexity: Option<String>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<String>,
    pub timeout_minutes: i64,
    pub result: Option<String>,
    pub attempt_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExecutionLog {
    pub id: i64,
    pub issue_id: i64,
    pub agent_id: String,
    pub attempt_number: i64,
    pub entry_type: String,
    pub message: String,
    pub metadata: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectAgentConfig {
    pub project_id: i64,
    pub auto_accept_threshold: f64,
    pub human_review_threshold: f64,
    pub max_attempts: i64,
    pub heartbeat_interval_seconds: i64,
    pub missed_heartbeats_before_offline: i64,
}
