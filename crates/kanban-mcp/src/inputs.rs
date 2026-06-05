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
pub struct SearchIssues {
    /// Full-text query.
    pub query: String,
    /// Optional project prefix to scope the search.
    #[serde(default)]
    pub project: Option<String>,
}
