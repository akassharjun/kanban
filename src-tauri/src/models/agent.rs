use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub agent_type: Option<String>,
    pub skills: serde_json::Value,
    pub task_types: serde_json::Value,
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
    pub skills_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskContract {
    pub issue_id: i64,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub task_state: String,
    pub objective: String,
    pub context: serde_json::Value,
    pub constraints: serde_json::Value,
    pub success_criteria: serde_json::Value,
    pub required_skills: serde_json::Value,
    pub estimated_complexity: Option<String>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<String>,
    pub timeout_minutes: i64,
    pub result: Option<serde_json::Value>,
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
    pub metadata: Option<serde_json::Value>,
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
