//! `kanban project` subcommands: list, create, show, update, archive, delete.
//!
//! `id_or_prefix` arguments accept either a UUID (e.g. via JSON workflows) or
//! a project prefix (e.g. `AUTH`); UUID parsing is tried first and prefix
//! lookup is the fallback.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::operation::{
    ArchiveProject, CreateProject, DeleteProject, Operation, ProjectPatch, UpdateProject,
};
use kanban_core::{Result, Workspace, new_id};

#[derive(Debug, Args)]
pub struct ProjectCmd {
    #[command(subcommand)]
    pub sub: ProjectSub,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSub {
    /// List all projects.
    List,
    /// Create a new project.
    Create {
        name: String,
        #[arg(long)]
        prefix: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        icon: Option<String>,
    },
    /// Show one project by id or prefix.
    Show { id_or_prefix: String },
    /// Update project metadata.
    Update {
        id_or_prefix: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Archive a project (preserves issues).
    Archive { id_or_prefix: String },
    /// Permanently delete a project (requires `--yes`).
    Delete {
        id_or_prefix: String,
        #[arg(long)]
        yes: bool,
    },
}

/// Dispatch a `kanban project` invocation to the appropriate handler.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
// `ProjectCmd` is a clap Subcommand wrapper; idiomatic clap usage is to
// consume it by value rather than borrow.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: ProjectCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        ProjectSub::List => list(ws, out),
        ProjectSub::Create {
            name,
            prefix,
            description,
            icon,
        } => create(ws, out, name, prefix, description, icon),
        ProjectSub::Show { id_or_prefix } => show(ws, out, &id_or_prefix),
        ProjectSub::Update {
            id_or_prefix,
            name,
            description,
        } => update(ws, out, &id_or_prefix, name, description),
        ProjectSub::Archive { id_or_prefix } => archive(ws, out, &id_or_prefix),
        ProjectSub::Delete { id_or_prefix, yes } => delete(ws, out, &id_or_prefix, yes),
    }
}

/// Resolve a project handle from a UUID string or a prefix.
fn resolve(ws: &Workspace, id_or_prefix: &str) -> Result<kanban_core::Project> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id_or_prefix) {
        return ws.query_project_by_id(uuid);
    }
    let projects = ws.query_projects()?;
    projects
        .into_iter()
        .find(|p| p.prefix == id_or_prefix)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Project,
            id: id_or_prefix.to_string(),
        })
}

fn list(ws: &Workspace, out: &Out) -> Result<()> {
    let ps = ws.query_projects()?;
    if out.json {
        out.print_json(&ps)?;
    } else {
        for p in &ps {
            println!("{}  {}  {}", p.prefix, p.name, p.status.as_str());
        }
    }
    Ok(())
}

fn create(
    ws: &mut Workspace,
    out: &Out,
    name: String,
    prefix: String,
    description: Option<String>,
    icon: Option<String>,
) -> Result<()> {
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name,
        prefix,
        description,
        icon,
    }))?;
    let p = ws.query_project_by_id(id)?;
    if out.json {
        out.print_json(&p)?;
    } else {
        println!("created {} ({})", p.prefix, p.name);
    }
    Ok(())
}

fn show(ws: &Workspace, out: &Out, id_or_prefix: &str) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    if out.json {
        out.print_json(&p)?;
    } else {
        println!("{}  {}  {}", p.prefix, p.name, p.status.as_str());
        if let Some(d) = &p.description {
            println!("description: {d}");
        }
    }
    Ok(())
}

fn update(
    ws: &mut Workspace,
    out: &Out,
    id_or_prefix: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    ws.apply(Operation::UpdateProject(UpdateProject {
        id: p.id,
        patch: ProjectPatch {
            name,
            description: description.map(Some),
            ..Default::default()
        },
    }))?;
    let p = ws.query_project_by_id(p.id)?;
    if out.json {
        out.print_json(&p)?;
    } else {
        println!("updated {}", p.prefix);
    }
    Ok(())
}

fn archive(ws: &mut Workspace, out: &Out, id_or_prefix: &str) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    ws.apply(Operation::ArchiveProject(ArchiveProject { id: p.id }))?;
    if !out.json {
        println!("archived {}", p.prefix);
    }
    Ok(())
}

fn delete(ws: &mut Workspace, out: &Out, id_or_prefix: &str, yes: bool) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    if !yes {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "confirm".into(),
                reason: "pass --yes to confirm deletion".into(),
            },
        ));
    }
    ws.apply(Operation::DeleteProject(DeleteProject { id: p.id }))?;
    if !out.json {
        println!("deleted {}", p.prefix);
    }
    Ok(())
}
