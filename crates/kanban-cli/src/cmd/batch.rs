//! `kanban batch` subcommand. Reads NDJSON commands from stdin and emits one
//! JSON result line per input line.
//!
//! v1 supports a deliberate subset of the full CLI surface:
//! - `project.create`, `project.show`, `project.update`, `project.delete`
//! - `issue.create`, `issue.list`, `issue.update`, `issue.move`, `issue.delete`
//!
//! The batch command opens its own [`Workspace`] from the optional `db` path
//! or `$KANBAN_DB`, so its `run` signature does NOT take `&mut Workspace`.

use chrono::NaiveDate;
use clap::Args;
use kanban_core::operation::{
    CreateIssue, CreateProject, DeleteIssue, DeleteProject, IssueFieldChange, Operation,
    ProjectPatch, UpdateIssueField, UpdateProject,
};
use kanban_core::query::IssueFilter;
use kanban_core::types::Priority;
use kanban_core::{Result, Workspace};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write, stdin, stdout};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct BatchArgs {
    /// Abort batch on first failing command instead of continuing.
    #[arg(long)]
    pub fail_fast: bool,
}

#[derive(Debug, Deserialize)]
struct Line {
    cmd: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OkLine<'a> {
    ok: bool,
    cmd: &'a str,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ErrLine<'a> {
    ok: bool,
    cmd: &'a str,
    error: String,
}

/// Dispatch a `kanban batch` invocation by streaming stdin line-by-line.
///
/// # Errors
///
/// Returns the first error if `--fail-fast` is set; otherwise per-line errors
/// are emitted as `{"ok": false, ...}` rows on stdout and the overall command
/// completes successfully.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: BatchArgs, db: Option<&Path>, _out: &crate::output::Out) -> Result<()> {
    let mut ws = match db {
        Some(p) => Workspace::open(p)?,
        None => Workspace::open_default()?,
    };
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.map_err(kanban_core::Error::Io)?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed: std::result::Result<Line, _> = serde_json::from_str(&line);
        let (cmd_name, result) = match parsed {
            Ok(l) => {
                let cmd = l.cmd.clone();
                let r = dispatch(&mut ws, &l);
                (cmd, r.map(|v| (l.cmd, v)))
            }
            Err(e) => (
                String::new(),
                Err(kanban_core::Error::InvalidSnapshot(format!(
                    "invalid ndjson: {e}"
                ))),
            ),
        };
        match result {
            Ok((cmd, data)) => {
                let s = serde_json::to_string(&OkLine {
                    ok: true,
                    cmd: &cmd,
                    data,
                })?;
                writeln!(stdout, "{s}")?;
            }
            Err(e) => {
                let s = serde_json::to_string(&ErrLine {
                    ok: false,
                    cmd: &cmd_name,
                    error: e.to_string(),
                })?;
                writeln!(stdout, "{s}")?;
                if args.fail_fast {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

fn dispatch(ws: &mut Workspace, l: &Line) -> Result<serde_json::Value> {
    match l.cmd.as_str() {
        "project.create" => project_create(ws, &l.args),
        "project.show" => project_show(ws, &l.args),
        "project.update" => project_update(ws, &l.args),
        "project.delete" => project_delete(ws, &l.args),
        "issue.create" => issue_create(ws, &l.args),
        "issue.list" => issue_list(ws, &l.args),
        "issue.update" => issue_update(ws, &l.args),
        "issue.move" => issue_move(ws, &l.args),
        "issue.delete" => issue_delete(ws, &l.args),
        other => Err(kanban_core::Error::InvalidSnapshot(format!(
            "unsupported batch cmd: {other}"
        ))),
    }
}

// --- helpers shared across batch handlers ---

fn resolve_project_value(ws: &Workspace, id_or_prefix: &str) -> Result<kanban_core::Project> {
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

fn resolve_issue_value(ws: &Workspace, identifier: &str) -> Result<kanban_core::Issue> {
    if let Ok(uuid) = Uuid::parse_str(identifier) {
        return ws.query_issue_by_id(uuid);
    }
    let (prefix, _) = identifier.split_once('-').ok_or_else(|| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "identifier".into(),
            reason: format!("expected `PREFIX-SEQ`, got '{identifier}'"),
        })
    })?;
    let project = resolve_project_value(ws, prefix)?;
    ws.query_issues(IssueFilter::for_project(project.id))?
        .into_iter()
        .find(|i| i.identifier == identifier)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Issue,
            id: identifier.to_string(),
        })
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "due".into(),
            reason: format!("expected YYYY-MM-DD, got '{s}': {e}"),
        })
    })
}

// --- project.* ---

fn project_create(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        name: String,
        prefix: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        icon: Option<String>,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let id = kanban_core::new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: a.name,
        prefix: a.prefix,
        description: a.description,
        icon: a.icon,
    }))?;
    let p = ws.query_project_by_id(id)?;
    Ok(serde_json::to_value(p)?)
}

fn project_show(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        id_or_prefix: String,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let p = resolve_project_value(ws, &a.id_or_prefix)?;
    Ok(serde_json::to_value(p)?)
}

fn project_update(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        id_or_prefix: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        description: Option<String>,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let p = resolve_project_value(ws, &a.id_or_prefix)?;
    ws.apply(Operation::UpdateProject(UpdateProject {
        id: p.id,
        patch: ProjectPatch {
            name: a.name,
            description: a.description.map(Some),
            ..Default::default()
        },
    }))?;
    let p = ws.query_project_by_id(p.id)?;
    Ok(serde_json::to_value(p)?)
}

fn project_delete(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        id_or_prefix: String,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let p = resolve_project_value(ws, &a.id_or_prefix)?;
    ws.apply(Operation::DeleteProject(DeleteProject { id: p.id }))?;
    Ok(serde_json::json!({"deleted": p.prefix}))
}

// --- issue.* ---

fn issue_create(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        project: String,
        title: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        priority: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        due: Option<String>,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let p = resolve_project_value(ws, &a.project)?;
    let priority: Priority = Priority::from_str(a.priority.as_deref().unwrap_or("none"))?;
    let status_id = if let Some(name) = a.status.as_deref() {
        ws.query_statuses_for_project(p.id)?
            .into_iter()
            .find(|s| s.name == name)
            .ok_or(kanban_core::Error::NotFound {
                kind: kanban_core::EntityKind::Status,
                id: name.to_string(),
            })?
            .id
    } else {
        ws.query_statuses_for_project(p.id)?
            .first()
            .ok_or(kanban_core::Error::NotFound {
                kind: kanban_core::EntityKind::Status,
                id: "default".into(),
            })?
            .id
    };
    let due_date = match a.due {
        Some(s) => Some(parse_date(&s)?),
        None => None,
    };
    let id = kanban_core::new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: p.id,
        title: a.title,
        description: a.description,
        status_id,
        priority,
        due_date,
        label_ids: vec![],
    }))?;
    let issue = ws.query_issue_by_id(id)?;
    Ok(serde_json::to_value(issue)?)
}

fn issue_list(ws: &Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        project: String,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let p = resolve_project_value(ws, &a.project)?;
    let issues = ws.query_issues(IssueFilter::for_project(p.id))?;
    Ok(serde_json::to_value(issues)?)
}

fn issue_update(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        identifier: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        priority: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        due: Option<String>,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let issue = resolve_issue_value(ws, &a.identifier)?;
    let mut applied = 0u32;
    if let Some(t) = a.title {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Title(t),
        }))?;
        applied += 1;
    }
    if let Some(d) = a.description {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Description(Some(d)),
        }))?;
        applied += 1;
    }
    if let Some(p) = a.priority {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Priority(Priority::from_str(&p)?),
        }))?;
        applied += 1;
    }
    if let Some(name) = a.status {
        let s = ws
            .query_statuses_for_project(issue.project_id)?
            .into_iter()
            .find(|s| s.name == name)
            .ok_or(kanban_core::Error::NotFound {
                kind: kanban_core::EntityKind::Status,
                id: name,
            })?;
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Status(s.id),
        }))?;
        applied += 1;
    }
    if let Some(d) = a.due {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::DueDate(Some(parse_date(&d)?)),
        }))?;
        applied += 1;
    }
    if applied == 0 {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "fields".into(),
                reason: "supply at least one of title/description/priority/status/due".into(),
            },
        ));
    }
    let after = ws.query_issue_by_id(issue.id)?;
    Ok(serde_json::to_value(after)?)
}

fn issue_move(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        identifier: String,
        status: String,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let issue = resolve_issue_value(ws, &a.identifier)?;
    let s = ws
        .query_statuses_for_project(issue.project_id)?
        .into_iter()
        .find(|s| s.name == a.status)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Status,
            id: a.status.clone(),
        })?;
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id: issue.id,
        change: IssueFieldChange::Status(s.id),
    }))?;
    let after = ws.query_issue_by_id(issue.id)?;
    Ok(serde_json::to_value(after)?)
}

fn issue_delete(ws: &mut Workspace, args: &serde_json::Value) -> Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct A {
        identifier: String,
    }
    let a: A = serde_json::from_value(args.clone())?;
    let issue = resolve_issue_value(ws, &a.identifier)?;
    ws.apply(Operation::DeleteIssue(DeleteIssue { id: issue.id }))?;
    Ok(serde_json::json!({"deleted": issue.identifier}))
}
