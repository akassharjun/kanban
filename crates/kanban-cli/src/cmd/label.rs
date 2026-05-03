//! `kanban label` subcommands.
//!
//! Labels are scoped to a project and addressed by name within that project.
//! `attach`/`detach` take an issue identifier (`KAN-42`) and a label name, both
//! resolved relative to the issue's parent project.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::operation::{
    AttachLabel, CreateLabel, DeleteLabel, DetachLabel, LabelPatch, Operation, UpdateLabel,
};
use kanban_core::query::IssueFilter;
use kanban_core::{Issue, Label, Project, Result, Workspace, new_id};
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct LabelCmd {
    #[command(subcommand)]
    pub sub: LabelSub,
}

#[derive(Debug, Subcommand)]
pub enum LabelSub {
    /// List labels for a project.
    List {
        #[arg(long)]
        project: String,
    },
    /// Create a label.
    Create {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        color: String,
    },
    /// Update an existing label by current name.
    Update {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        rename: Option<String>,
        #[arg(long)]
        color: Option<String>,
    },
    /// Delete a label (requires `--yes`).
    Delete {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        yes: bool,
    },
    /// Attach a label to an issue.
    Attach { identifier: String, label: String },
    /// Detach a label from an issue.
    Detach { identifier: String, label: String },
}

/// Dispatch a `kanban label` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
#[allow(clippy::needless_pass_by_value)]
pub fn run(cmd: LabelCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        LabelSub::List { project } => list(ws, out, &project),
        LabelSub::Create {
            project,
            name,
            color,
        } => create(ws, out, &project, name, color),
        LabelSub::Update {
            project,
            name,
            rename,
            color,
        } => update(ws, out, &project, &name, rename, color),
        LabelSub::Delete { project, name, yes } => delete(ws, out, &project, &name, yes),
        LabelSub::Attach { identifier, label } => attach(ws, out, &identifier, &label),
        LabelSub::Detach { identifier, label } => detach(ws, out, &identifier, &label),
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

fn resolve_label_in_project(ws: &Workspace, project_id: Uuid, name: &str) -> Result<Label> {
    ws.query_labels_for_project(project_id)?
        .into_iter()
        .find(|l| l.name == name)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Label,
            id: name.to_string(),
        })
}

fn resolve_issue(ws: &Workspace, identifier: &str) -> Result<Issue> {
    if let Ok(uuid) = Uuid::parse_str(identifier) {
        return ws.query_issue_by_id(uuid);
    }
    let (prefix, _) = identifier.split_once('-').ok_or_else(|| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "identifier".into(),
            reason: format!("expected `PREFIX-SEQ`, got '{identifier}'"),
        })
    })?;
    let project = resolve_project(ws, prefix)?;
    ws.query_issues(IssueFilter::for_project(project.id))?
        .into_iter()
        .find(|i| i.identifier == identifier)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Issue,
            id: identifier.to_string(),
        })
}

fn list(ws: &Workspace, out: &Out, project: &str) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let labels = ws.query_labels_for_project(p.id)?;
    if out.json {
        out.print_json(&labels)?;
    } else {
        for l in &labels {
            println!("{}  {}", l.name, l.color);
        }
    }
    Ok(())
}

fn create(ws: &mut Workspace, out: &Out, project: &str, name: String, color: String) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id,
        project_id: p.id,
        name,
        color,
    }))?;
    let labels = ws.query_labels_for_project(p.id)?;
    let created = labels
        .into_iter()
        .find(|l| l.id == id)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Label,
            id: id.to_string(),
        })?;
    if out.json {
        out.print_json(&created)?;
    } else {
        println!("created {} ({})", created.name, created.color);
    }
    Ok(())
}

fn update(
    ws: &mut Workspace,
    out: &Out,
    project: &str,
    name: &str,
    rename: Option<String>,
    color: Option<String>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let label = resolve_label_in_project(ws, p.id, name)?;
    if rename.is_none() && color.is_none() {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "fields".into(),
                reason: "supply --rename or --color".into(),
            },
        ));
    }
    ws.apply(Operation::UpdateLabel(UpdateLabel {
        id: label.id,
        patch: LabelPatch {
            name: rename,
            color,
        },
    }))?;
    let labels = ws.query_labels_for_project(p.id)?;
    let updated =
        labels
            .into_iter()
            .find(|l| l.id == label.id)
            .ok_or(kanban_core::Error::NotFound {
                kind: kanban_core::EntityKind::Label,
                id: label.id.to_string(),
            })?;
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
    let label = resolve_label_in_project(ws, p.id, name)?;
    ws.apply(Operation::DeleteLabel(DeleteLabel { id: label.id }))?;
    if !out.json {
        println!("deleted {}", label.name);
    }
    Ok(())
}

fn attach(ws: &mut Workspace, out: &Out, identifier: &str, label_name: &str) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    let label = resolve_label_in_project(ws, issue.project_id, label_name)?;
    ws.apply(Operation::AttachLabel(AttachLabel {
        issue_id: issue.id,
        label_id: label.id,
    }))?;
    if !out.json {
        println!("attached {} to {}", label.name, issue.identifier);
    }
    Ok(())
}

fn detach(ws: &mut Workspace, out: &Out, identifier: &str, label_name: &str) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    let label = resolve_label_in_project(ws, issue.project_id, label_name)?;
    ws.apply(Operation::DetachLabel(DetachLabel {
        issue_id: issue.id,
        label_id: label.id,
    }))?;
    if !out.json {
        println!("detached {} from {}", label.name, issue.identifier);
    }
    Ok(())
}
