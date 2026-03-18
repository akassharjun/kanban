use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GithubConfig {
    pub id: i64,
    pub project_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub access_token: Option<String>,
    pub branch_pattern: String,
    pub auto_link_prs: bool,
    pub auto_transition_on_merge: bool,
    pub merge_target_status_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GithubEvent {
    pub id: i64,
    pub project_id: i64,
    pub event_type: String,
    pub issue_id: Option<i64>,
    pub payload: String,
    pub processed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitLink {
    pub id: i64,
    pub issue_id: i64,
    pub link_type: String,
    pub url: Option<String>,
    pub ref_name: String,
    pub pr_number: Option<i64>,
    pub pr_state: Option<String>,
    pub pr_merged: bool,
    pub ci_status: Option<String>,
    pub review_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIStatus {
    pub status: String,       // "pending", "success", "failure", "neutral"
    pub checks: Vec<CICheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CICheck {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRStatus {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub merged: bool,
    pub review_status: String,   // "approved", "changes_requested", "pending", "none"
    pub ci_status: String,       // "pending", "success", "failure"
    pub url: String,
    pub author: String,
}
