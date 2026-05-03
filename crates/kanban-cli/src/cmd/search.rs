//! `kanban search` subcommand. FTS5-backed full-text search over issues.

use crate::output::Out;
use clap::Args;
use kanban_core::query::IssueFilter;
use kanban_core::types::Priority;
use kanban_core::{Project, Result, Workspace};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// FTS5 query string.
    pub query: String,
    /// Restrict to a single project (id or prefix).
    #[arg(long)]
    pub project: Option<String>,
    /// Repeatable: include issues whose priority matches.
    #[arg(long = "priority")]
    pub priorities: Vec<String>,
    /// Repeatable: include issues whose status name matches.
    #[arg(long = "status")]
    pub statuses: Vec<String>,
    /// Repeatable: include issues that have any of these label names.
    #[arg(long = "label")]
    pub labels: Vec<String>,
    /// Cap the number of results.
    #[arg(long)]
    pub limit: Option<i64>,
}

/// Dispatch a `kanban search` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace::search`] call and from
/// resolving the optional project / status / label filters.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: SearchArgs, ws: &Workspace, out: &Out) -> Result<()> {
    let mut filter = IssueFilter::default();
    let project = match cmd.project.as_deref() {
        Some(s) => Some(resolve_project(ws, s)?),
        None => None,
    };
    if let Some(p) = &project {
        filter.project_id = Some(p.id);
    }
    for prio in &cmd.priorities {
        filter.priorities.push(Priority::from_str(prio)?);
    }
    if !cmd.statuses.is_empty() {
        let pid = project
            .as_ref()
            .ok_or(kanban_core::Error::Validation(
                kanban_core::ValidationError {
                    field: "status".into(),
                    reason: "--status requires --project".into(),
                },
            ))?
            .id;
        let resolved = ws.query_statuses_for_project(pid)?;
        for name in &cmd.statuses {
            let s =
                resolved
                    .iter()
                    .find(|s| &s.name == name)
                    .ok_or(kanban_core::Error::NotFound {
                        kind: kanban_core::EntityKind::Status,
                        id: name.clone(),
                    })?;
            filter.status_ids.push(s.id);
        }
    }
    if !cmd.labels.is_empty() {
        let pid = project
            .as_ref()
            .ok_or(kanban_core::Error::Validation(
                kanban_core::ValidationError {
                    field: "label".into(),
                    reason: "--label requires --project".into(),
                },
            ))?
            .id;
        let resolved = ws.query_labels_for_project(pid)?;
        for name in &cmd.labels {
            let l =
                resolved
                    .iter()
                    .find(|l| &l.name == name)
                    .ok_or(kanban_core::Error::NotFound {
                        kind: kanban_core::EntityKind::Label,
                        id: name.clone(),
                    })?;
            filter.label_ids.push(l.id);
        }
    }
    filter.limit = cmd.limit;

    let hits = ws.search(&cmd.query, filter)?;
    if out.json {
        out.print_json(&hits)?;
    } else {
        for i in &hits {
            println!("{}  {}  {}", i.identifier, i.priority.as_str(), i.title);
        }
    }
    Ok(())
}

fn resolve_project(ws: &Workspace, id_or_prefix: &str) -> Result<Project> {
    if let Ok(uuid) = Uuid::parse_str(id_or_prefix) {
        return ws.query_project_by_id(uuid);
    }
    ws.query_projects()?
        .into_iter()
        .find(|p| p.prefix == id_or_prefix)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Project,
            id: id_or_prefix.to_string(),
        })
}
