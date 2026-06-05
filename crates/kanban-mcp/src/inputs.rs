//! Tool input parameter structs (`serde::Deserialize` + `schemars::JsonSchema`).
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectRef {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssues {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
    /// Optional status name to filter by (e.g. "In Progress").
    #[serde(default)]
    pub status: Option<String>,
    /// Optional priority filter: one of none, low, medium, high, urgent.
    #[serde(default)]
    pub priority: Option<String>,
    /// Optional full-text search within the project.
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueKey {
    /// Issue key, e.g. "AUTH-12".
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectInput {
    /// Display name, e.g. "Auth Service".
    pub name: String,
    /// Unique prefix: 2-8 uppercase letters, e.g. "AUTH".
    pub prefix: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueInput {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
    /// Issue title.
    pub title: String,
    /// Optional markdown description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional status name; defaults to the project's first column.
    #[serde(default)]
    pub status: Option<String>,
    /// Optional priority: none, low, medium, high, urgent. Defaults to medium.
    #[serde(default)]
    pub priority: Option<String>,
    /// Optional due date, YYYY-MM-DD.
    #[serde(default)]
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateIssueInput {
    /// Issue key, e.g. "AUTH-12".
    pub key: String,
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New markdown description (empty string clears the visible text).
    #[serde(default)]
    pub description: Option<String>,
    /// New priority: none, low, medium, high, urgent.
    #[serde(default)]
    pub priority: Option<String>,
    /// New status name (e.g. "In Progress").
    #[serde(default)]
    pub status: Option<String>,
    /// New due date, YYYY-MM-DD.
    #[serde(default)]
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchIssues {
    /// Full-text query.
    pub query: String,
    /// Optional project prefix to scope the search.
    #[serde(default)]
    pub project: Option<String>,
}
