pub mod agent;
pub use agent::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Priority {
    None,
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum StatusCategory {
    Unstarted,
    Started,
    Blocked,
    Completed,
    Discarded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum RelationType {
    Related,
    Blocks,
    BlockedBy,
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: String,
    pub prefix: String,
    pub issue_counter: i64,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub path: Option<String>,
    pub stale_days: Option<i64>,
    pub stale_close_status_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Status {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub category: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: i64,
    pub project_id: i64,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub status_id: i64,
    pub priority: String,
    pub assignee_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub position: f64,
    pub estimate: Option<f64>,
    pub due_date: Option<String>,
    pub epic_id: Option<i64>,
    pub milestone_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Epic {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Milestone {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Label {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Member {
    pub id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_color: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueRelation {
    pub id: i64,
    pub source_issue_id: i64,
    pub target_issue_id: i64,
    pub relation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueTemplate {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description_template: Option<String>,
    pub default_status_id: Option<i64>,
    pub default_priority: String,
    pub default_label_ids: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityLogEntry {
    pub id: i64,
    pub issue_id: i64,
    pub field_changed: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub actor_id: Option<i64>,
    pub actor_type: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub issue_id: i64,
    pub issue_identifier: String,
    pub issue_title: String,
    pub field_changed: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub actor_id: Option<i64>,
    pub actor_type: Option<String>,
    pub actor_name: Option<String>,
    pub actor_avatar_color: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueHistoryEntry {
    pub id: i64,
    pub issue_id: i64,
    pub field_changed: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub actor_id: Option<i64>,
    pub actor_type: Option<String>,
    pub actor_name: Option<String>,
    pub actor_avatar_color: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mention {
    pub id: i64,
    pub issue_id: i64,
    pub comment_id: Option<i64>,
    pub member_id: i64,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionWithContext {
    pub id: i64,
    pub issue_id: i64,
    pub issue_identifier: String,
    pub issue_title: String,
    pub comment_id: Option<i64>,
    pub member_id: i64,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UndoLogEntry {
    pub id: i64,
    pub operation_type: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub snapshot_before: Option<String>,
    pub snapshot_after: Option<String>,
    pub undone: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: i64,
    pub r#type: String,
    pub issue_id: Option<i64>,
    pub message: String,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hook {
    pub id: i64,
    pub project_id: i64,
    pub event_type: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: i64,
    pub issue_id: i64,
    pub member_id: Option<i64>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomField {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub field_type: String,
    pub options: Option<String>,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomFieldValue {
    pub id: i64,
    pub issue_id: i64,
    pub field_id: i64,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SavedView {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub filters: String,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub view_mode: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StarredIssue {
    pub id: i64,
    pub issue_id: i64,
    pub member_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentlyViewed {
    pub id: i64,
    pub issue_id: i64,
    pub member_id: i64,
    pub viewed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitLink {
    pub id: i64,
    pub issue_id: i64,
    pub link_type: String,
    pub url: Option<String>,
    pub ref_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRule {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub enabled: bool,
    pub trigger_type: String,
    pub trigger_config: String,
    pub conditions: String,
    pub actions: String,
    pub execution_count: i64,
    pub last_executed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationLogEntry {
    pub id: i64,
    pub rule_id: i64,
    pub issue_id: Option<i64>,
    pub trigger_type: String,
    pub actions_executed: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub executed_at: String,
}
