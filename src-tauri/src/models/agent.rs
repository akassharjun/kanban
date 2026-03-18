use serde::{Deserialize, Serialize, Serializer};

/// Helper to parse a JSON string field into serde_json::Value.
pub fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
}


#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub agent_type: Option<String>,
    #[sqlx(rename = "skills")]
    pub skills: String,
    #[sqlx(rename = "task_types")]
    pub task_types: String,
    pub max_concurrent: i64,
    pub max_complexity: String,
    pub member_id: Option<i64>,
    pub status: String,
    pub registered_at: String,
    pub last_heartbeat: String,
    pub last_activity_at: Option<String>,
    pub worktree_path: Option<String>,
}

impl Serialize for Agent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Agent", 13)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("agent_type", &self.agent_type)?;
        s.serialize_field("skills", &parse_json(&self.skills))?;
        s.serialize_field("task_types", &parse_json(&self.task_types))?;
        s.serialize_field("max_concurrent", &self.max_concurrent)?;
        s.serialize_field("max_complexity", &self.max_complexity)?;
        s.serialize_field("member_id", &self.member_id)?;
        s.serialize_field("status", &self.status)?;
        s.serialize_field("registered_at", &self.registered_at)?;
        s.serialize_field("last_heartbeat", &self.last_heartbeat)?;
        s.serialize_field("last_activity_at", &self.last_activity_at)?;
        s.serialize_field("worktree_path", &self.worktree_path)?;
        s.end()
    }
}

impl Agent {
    pub fn skills_json(&self) -> serde_json::Value {
        parse_json(&self.skills)
    }
    pub fn task_types_json(&self) -> serde_json::Value {
        parse_json(&self.task_types)
    }
}

#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
pub struct AgentStats {
    pub agent_id: String,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub total_confidence: f64,
    pub total_completion_time_seconds: i64,
    pub skills_breakdown: String,
}

impl Serialize for AgentStats {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AgentStats", 6)?;
        s.serialize_field("agent_id", &self.agent_id)?;
        s.serialize_field("tasks_completed", &self.tasks_completed)?;
        s.serialize_field("tasks_failed", &self.tasks_failed)?;
        s.serialize_field("total_confidence", &self.total_confidence)?;
        s.serialize_field("total_completion_time_seconds", &self.total_completion_time_seconds)?;
        s.serialize_field("skills_breakdown", &parse_json(&self.skills_breakdown))?;
        s.end()
    }
}

impl AgentStats {
    pub fn skills_breakdown_json(&self) -> serde_json::Value {
        parse_json(&self.skills_breakdown)
    }
}

#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
pub struct TaskContract {
    pub issue_id: i64,
    #[sqlx(rename = "type")]
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

impl Serialize for TaskContract {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("TaskContract", 14)?;
        s.serialize_field("issue_id", &self.issue_id)?;
        s.serialize_field("type", &self.r#type)?;
        s.serialize_field("task_state", &self.task_state)?;
        s.serialize_field("objective", &self.objective)?;
        s.serialize_field("context", &parse_json(&self.context))?;
        s.serialize_field("constraints", &parse_json(&self.constraints))?;
        s.serialize_field("success_criteria", &parse_json(&self.success_criteria))?;
        s.serialize_field("required_skills", &parse_json(&self.required_skills))?;
        s.serialize_field("estimated_complexity", &self.estimated_complexity)?;
        s.serialize_field("claimed_by", &self.claimed_by)?;
        s.serialize_field("claimed_at", &self.claimed_at)?;
        s.serialize_field("timeout_minutes", &self.timeout_minutes)?;
        s.serialize_field("result", &self.result.as_ref().map(|r| parse_json(r)))?;
        s.serialize_field("attempt_count", &self.attempt_count)?;
        s.end()
    }
}

impl TaskContract {
    pub fn context_json(&self) -> serde_json::Value {
        parse_json(&self.context)
    }
    pub fn constraints_json(&self) -> serde_json::Value {
        parse_json(&self.constraints)
    }
    pub fn success_criteria_json(&self) -> serde_json::Value {
        parse_json(&self.success_criteria)
    }
    pub fn required_skills_json(&self) -> serde_json::Value {
        parse_json(&self.required_skills)
    }
    pub fn result_json(&self) -> Option<serde_json::Value> {
        self.result.as_ref().map(|s| parse_json(s))
    }
}

#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
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

impl Serialize for ExecutionLog {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ExecutionLog", 8)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("issue_id", &self.issue_id)?;
        s.serialize_field("agent_id", &self.agent_id)?;
        s.serialize_field("attempt_number", &self.attempt_number)?;
        s.serialize_field("entry_type", &self.entry_type)?;
        s.serialize_field("message", &self.message)?;
        s.serialize_field("metadata", &self.metadata.as_ref().map(|m| parse_json(m)))?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.end()
    }
}

impl ExecutionLog {
    pub fn metadata_json(&self) -> Option<serde_json::Value> {
        self.metadata.as_ref().map(|s| parse_json(s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectAgentConfig {
    pub project_id: i64,
    pub auto_accept_threshold: f64,
    pub human_review_threshold: f64,
    pub max_attempts: i64,
    pub heartbeat_interval_seconds: i64,
    pub missed_heartbeats_before_offline: i64,
    pub use_wsjf_scoring: bool,
}
