//! Output shapes returned to the MCP client. Human-facing: prefixes/keys/names,
//! never UUIDs or sort keys.
use kanban_core::types::{Issue, Label, Project, Status};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProjectOut {
    pub prefix: String,
    pub name: String,
    pub description: Option<String>,
}
impl From<Project> for ProjectOut {
    fn from(p: Project) -> Self {
        Self {
            prefix: p.prefix,
            name: p.name,
            description: p.description,
        }
    }
}

#[derive(Serialize)]
pub struct StatusOut {
    pub name: String,
    pub category: String,
}
impl From<Status> for StatusOut {
    fn from(s: Status) -> Self {
        Self {
            name: s.name,
            category: s.category.as_str().to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct LabelOut {
    pub name: String,
    pub color: String,
}
impl From<Label> for LabelOut {
    fn from(l: Label) -> Self {
        Self {
            name: l.name,
            color: l.color,
        }
    }
}

// `IssueOut` is consumed by the issue tools in Task 6; allow dead_code until then.
#[allow(dead_code)]
#[derive(Serialize)]
pub struct IssueOut {
    pub key: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub description: Option<String>,
}

impl IssueOut {
    /// Build from a core `Issue` plus a resolved status NAME.
    #[allow(dead_code)]
    #[must_use]
    pub fn from_issue(i: Issue, status_name: &str) -> Self {
        Self {
            key: i.identifier,
            title: i.title,
            status: status_name.to_owned(),
            priority: i.priority.as_str().to_owned(),
            due_date: i.due_date.map(|d| d.to_string()),
            description: i.description,
        }
    }
}
