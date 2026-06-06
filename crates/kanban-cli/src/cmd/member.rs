//! `kanban member` subcommands.
//!
//! Members are scoped to a project and addressed by name within that project.
//! Besides the read-only `list`, this module provides `create`, `update`, and
//! `delete` write subcommands mirroring `kanban label`.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::operation::{CreateMember, DeleteMember, MemberPatch, Operation, UpdateMember};
use kanban_core::{Member, Project, Result, Workspace, new_id};
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct MemberCmd {
    #[command(subcommand)]
    pub sub: MemberSub,
}

#[derive(Debug, Subcommand)]
pub enum MemberSub {
    /// List members for a project.
    List {
        #[arg(long)]
        project: String,
    },
    /// Create a member.
    Create {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
    },
    /// Update an existing member by current name.
    Update {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        rename: Option<String>,
    },
    /// Delete a member (requires `--yes`).
    Delete {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        yes: bool,
    },
}

/// Dispatch a `kanban member` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: MemberCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        MemberSub::List { project } => list(ws, out, &project),
        MemberSub::Create { project, name } => create(ws, out, &project, name),
        MemberSub::Update {
            project,
            name,
            rename,
        } => update(ws, out, &project, &name, rename),
        MemberSub::Delete { project, name, yes } => delete(ws, out, &project, &name, yes),
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

fn resolve_member_by_name(ws: &Workspace, project_id: Uuid, name: &str) -> Result<Member> {
    ws.query_members_for_project(project_id)?
        .into_iter()
        .find(|m| m.name == name)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Member,
            id: name.to_string(),
        })
}

fn list(ws: &Workspace, out: &Out, project: &str) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let members = ws.query_members_for_project(p.id)?;
    if out.json {
        out.print_json(&members)?;
    } else {
        for m in &members {
            println!("{}", m.name);
        }
    }
    Ok(())
}

fn create(ws: &mut Workspace, out: &Out, project: &str, name: String) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: p.id,
        name,
    }))?;
    let created = resolve_member_by_id(ws, p.id, id)?;
    if out.json {
        out.print_json(&created)?;
    } else {
        println!("created {}", created.name);
    }
    Ok(())
}

fn update(
    ws: &mut Workspace,
    out: &Out,
    project: &str,
    name: &str,
    rename: Option<String>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let member = resolve_member_by_name(ws, p.id, name)?;
    if rename.is_none() {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "fields".into(),
                reason: "supply --rename".into(),
            },
        ));
    }
    ws.apply(Operation::UpdateMember(UpdateMember {
        id: member.id,
        patch: MemberPatch { name: rename },
    }))?;
    let updated = resolve_member_by_id(ws, p.id, member.id)?;
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
    let member = resolve_member_by_name(ws, p.id, name)?;
    ws.apply(Operation::DeleteMember(DeleteMember { id: member.id }))?;
    if !out.json {
        println!("deleted {}", member.name);
    }
    Ok(())
}

fn resolve_member_by_id(ws: &Workspace, project_id: Uuid, id: Uuid) -> Result<Member> {
    ws.query_members_for_project(project_id)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Member,
            id: id.to_string(),
        })
}
