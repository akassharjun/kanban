//! `kanban status` subcommand. Read-only listing of statuses per project.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::{Project, Result, Workspace};
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
}

/// Dispatch a `kanban status` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: StatusCmd, ws: &Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        StatusSub::List { project } => list(ws, out, &project),
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
