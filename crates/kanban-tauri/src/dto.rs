//! Specta-friendly data-transfer objects mirroring `kanban_core::types`.
//!
//! The core types are `Serialize` but do not derive `specta::Type`, so Tauri
//! command return types cannot be the core types directly. These DTOs render
//! ids, timestamps, dates, and enums as plain strings, and `sort_key` as a
//! plain `f64`, so the generated TypeScript bindings stay simple.

use kanban_core::types::{Issue, Label, Project, Status};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Project> for ProjectDto {
    fn from(p: Project) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            prefix: p.prefix,
            description: p.description,
            icon: p.icon,
            status: p.status.as_str().to_owned(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StatusDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub category: String,
    pub color: String,
    pub position: i64,
}

impl From<Status> for StatusDto {
    fn from(s: Status) -> Self {
        Self {
            id: s.id.to_string(),
            project_id: s.project_id.to_string(),
            name: s.name,
            category: s.category.as_str().to_owned(),
            color: s.color,
            position: s.position,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IssueDto {
    pub id: String,
    pub project_id: String,
    pub seq: i64,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub status_id: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub sort_key: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Issue> for IssueDto {
    fn from(i: Issue) -> Self {
        Self {
            id: i.id.to_string(),
            project_id: i.project_id.to_string(),
            seq: i.seq,
            identifier: i.identifier,
            title: i.title,
            description: i.description,
            status_id: i.status_id.to_string(),
            priority: i.priority.as_str().to_owned(),
            due_date: i.due_date.map(|d| d.to_string()),
            sort_key: i.sort_key,
            created_at: i.created_at.to_rfc3339(),
            updated_at: i.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LabelDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
}

impl From<Label> for LabelDto {
    fn from(l: Label) -> Self {
        Self {
            id: l.id.to_string(),
            project_id: l.project_id.to_string(),
            name: l.name,
            color: l.color,
        }
    }
}
