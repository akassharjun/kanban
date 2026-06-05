//! `kanban status` subcommands.
//!
//! Statuses are scoped to a project and addressed by name within that project.
//! Besides the read-only `list`, this module provides `create`, `update`,
//! `delete`, and `reorder` write subcommands mirroring `kanban label`.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::operation::{
    CreateStatus, DeleteStatus, Operation, ReorderStatus, StatusPatch, UpdateStatus,
};
use kanban_core::{Project, Result, Status, StatusCategory, Workspace, new_id};
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct StatusCmd {
    #[command(subcommand)]
    pub sub: StatusSub,
}

#[derive(Debug, Subcommand)]
pub enum StatusSub {
    /// List statuses for a project.
    List {
        #[arg(long)]
        project: String,
    },
    /// Create a status.
    Create {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        /// One of: unstarted, started, blocked, completed, discarded.
        #[arg(long)]
        category: String,
        #[arg(long)]
        color: String,
        /// Zero-based position. Defaults to appending after the last status.
        #[arg(long)]
        position: Option<i64>,
    },
    /// Update an existing status by current name.
    Update {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        rename: Option<String>,
        /// One of: unstarted, started, blocked, completed, discarded.
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        color: Option<String>,
    },
    /// Delete a status (requires `--yes`).
    Delete {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        yes: bool,
    },
    /// Reorder a status to a new zero-based position.
    Reorder {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        position: i64,
    },
}

/// Dispatch a `kanban status` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: StatusCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        StatusSub::List { project } => list(ws, out, &project),
        StatusSub::Create {
            project,
            name,
            category,
            color,
            position,
        } => create(ws, out, &project, name, &category, color, position),
        StatusSub::Update {
            project,
            name,
            rename,
            category,
            color,
        } => update(ws, out, &project, &name, rename, category, color),
        StatusSub::Delete { project, name, yes } => delete(ws, out, &project, &name, yes),
        StatusSub::Reorder {
            project,
            name,
            position,
        } => reorder(ws, out, &project, &name, position),
    }
}

fn parse_category(s: &str) -> Result<StatusCategory> {
    match s {
        "unstarted" => Ok(StatusCategory::Unstarted),
        "started" => Ok(StatusCategory::Started),
        "blocked" => Ok(StatusCategory::Blocked),
        "completed" => Ok(StatusCategory::Completed),
        "discarded" => Ok(StatusCategory::Discarded),
        other => Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "category".into(),
                reason: format!(
                    "unknown category '{other}'; expected one of \
                     unstarted|started|blocked|completed|discarded"
                ),
            },
        )),
    }
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

fn resolve_status_in_project(ws: &Workspace, project_id: Uuid, name: &str) -> Result<Status> {
    ws.query_statuses_for_project(project_id)?
        .into_iter()
        .find(|s| s.name == name)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Status,
            id: name.to_string(),
        })
}

fn list(ws: &Workspace, out: &Out, project: &str) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let statuses = ws.query_statuses_for_project(p.id)?;
    if out.json {
        out.print_json(&statuses)?;
    } else {
        for s in &statuses {
            println!("{}  {}  {}", s.name, s.category.as_str(), s.color);
        }
    }
    Ok(())
}

fn create(
    ws: &mut Workspace,
    out: &Out,
    project: &str,
    name: String,
    category: &str,
    color: String,
    position: Option<i64>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let category = parse_category(category)?;
    let position = match position {
        Some(pos) => pos,
        None => i64::try_from(ws.query_statuses_for_project(p.id)?.len()).unwrap_or(i64::MAX),
    };
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: p.id,
        name,
        category,
        color,
        position,
    }))?;
    let created = resolve_status_by_id(ws, p.id, id)?;
    if out.json {
        out.print_json(&created)?;
    } else {
        println!(
            "created {} ({}, {})",
            created.name,
            created.category.as_str(),
            created.color
        );
    }
    Ok(())
}

fn update(
    ws: &mut Workspace,
    out: &Out,
    project: &str,
    name: &str,
    rename: Option<String>,
    category: Option<String>,
    color: Option<String>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let status = resolve_status_in_project(ws, p.id, name)?;
    if rename.is_none() && category.is_none() && color.is_none() {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "fields".into(),
                reason: "supply --rename, --category, or --color".into(),
            },
        ));
    }
    let category = category.map(|c| parse_category(&c)).transpose()?;
    ws.apply(Operation::UpdateStatus(UpdateStatus {
        id: status.id,
        patch: StatusPatch {
            name: rename,
            category,
            color,
        },
    }))?;
    let updated = resolve_status_by_id(ws, p.id, status.id)?;
    if out.json {
        out.print_json(&updated)?;
    } else {
        println!("updated {}", updated.name);
    }
    Ok(())
}

fn delete(ws: &mut Workspace, out: &Out, project: &str, name: &str, yes: bool) -> Result<()> {
    if !yes {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "confirm".into(),
                reason: "pass --yes to confirm deletion".into(),
            },
        ));
    }
    let p = resolve_project(ws, project)?;
    let status = resolve_status_in_project(ws, p.id, name)?;
    ws.apply(Operation::DeleteStatus(DeleteStatus { id: status.id }))?;
    if !out.json {
        println!("deleted {}", status.name);
    }
    Ok(())
}

fn reorder(ws: &mut Workspace, out: &Out, project: &str, name: &str, position: i64) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let status = resolve_status_in_project(ws, p.id, name)?;
    ws.apply(Operation::ReorderStatus(ReorderStatus {
        id: status.id,
        new_position: position,
    }))?;
    let statuses = ws.query_statuses_for_project(p.id)?;
    if out.json {
        out.print_json(&statuses)?;
    } else {
        for s in &statuses {
            println!("{}  {}  {}", s.name, s.category.as_str(), s.color);
        }
    }
    Ok(())
}

fn resolve_status_by_id(ws: &Workspace, project_id: Uuid, id: Uuid) -> Result<Status> {
    ws.query_statuses_for_project(project_id)?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Status,
            id: id.to_string(),
        })
}
