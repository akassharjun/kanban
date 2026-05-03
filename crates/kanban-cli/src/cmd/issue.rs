//! `kanban issue` subcommands.
//!
//! Identifier arguments accept either a UUID (e.g. via JSON workflows) or a
//! Linear-style identifier such as `KAN-42`; UUID parsing is tried first and
//! prefix/seq lookup is the fallback.

use crate::output::Out;
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use kanban_core::operation::{
    CreateIssue, DeleteIssue, IssueFieldChange, Operation, ReorderIssue, UpdateIssueField,
};
use kanban_core::query::{IssueFilter, SortBy};
use kanban_core::types::Priority;
use kanban_core::{Issue, Project, Result, Status, Workspace, new_id};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct IssueCmd {
    #[command(subcommand)]
    pub sub: IssueSub,
}

#[derive(Debug, Subcommand)]
pub enum IssueSub {
    /// Create a new issue.
    Create {
        #[arg(long)]
        project: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "none")]
        priority: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        due: Option<String>,
    },
    /// List issues in a project, with optional filtering and sorting.
    List {
        #[arg(long)]
        project: String,
        /// Repeatable: include issues whose status name matches.
        #[arg(long = "status")]
        statuses: Vec<String>,
        /// Repeatable: include issues whose priority matches.
        #[arg(long = "priority")]
        priorities: Vec<String>,
        /// Repeatable: include issues that have any of these label names.
        #[arg(long = "label")]
        labels: Vec<String>,
        /// `YYYY-MM-DD`: keep issues whose due date is strictly before this date.
        #[arg(long = "due-before")]
        due_before: Option<String>,
        /// `YYYY-MM-DD`: keep issues whose due date is strictly after this date.
        #[arg(long = "due-after")]
        due_after: Option<String>,
        /// One of `manual`, `priority`, `created`, `updated`, `due`.
        #[arg(long, default_value = "manual")]
        sort: String,
        /// Reverse the sort direction.
        #[arg(long)]
        reverse: bool,
        /// Cap the number of results returned.
        #[arg(long)]
        limit: Option<i64>,
    },
    /// Show a single issue by id or identifier (e.g. `KAN-42`).
    Show { identifier: String },
    /// Update issue fields.
    Update {
        identifier: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        due: Option<String>,
    },
    /// Move an issue to a different status (alias for `update --status`).
    Move {
        identifier: String,
        #[arg(long)]
        status: String,
    },
    /// Reorder an issue relative to another, using sort-key half-step math.
    Reorder {
        identifier: String,
        #[arg(long, conflicts_with = "after")]
        before: Option<String>,
        #[arg(long, conflicts_with = "before")]
        after: Option<String>,
    },
    /// Delete an issue (requires `--yes`).
    Delete {
        identifier: String,
        #[arg(long)]
        yes: bool,
    },
    /// Show the activity log for an issue.
    History { identifier: String },
}

/// Dispatch a `kanban issue` invocation.
///
/// # Errors
///
/// Propagates errors from the underlying [`Workspace`] operations and from
/// validation in the dispatched subcommand handlers.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_lines)]
pub fn run(cmd: IssueCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        IssueSub::Create {
            project,
            title,
            description,
            priority,
            status,
            due,
        } => create(
            ws,
            out,
            &project,
            title,
            description,
            &priority,
            status,
            due,
        ),
        IssueSub::List {
            project,
            statuses,
            priorities,
            labels,
            due_before,
            due_after,
            sort,
            reverse,
            limit,
        } => list(
            ws,
            out,
            &project,
            &statuses,
            &priorities,
            &labels,
            due_before.as_deref(),
            due_after.as_deref(),
            &sort,
            reverse,
            limit,
        ),
        IssueSub::Show { identifier } => show(ws, out, &identifier),
        IssueSub::Update {
            identifier,
            title,
            description,
            priority,
            status,
            due,
        } => update(
            ws,
            out,
            &identifier,
            title,
            description,
            priority,
            status,
            due,
        ),
        IssueSub::Move { identifier, status } => move_status(ws, out, &identifier, &status),
        IssueSub::Reorder {
            identifier,
            before,
            after,
        } => reorder(ws, out, &identifier, before.as_deref(), after.as_deref()),
        IssueSub::Delete { identifier, yes } => delete(ws, out, &identifier, yes),
        IssueSub::History { identifier } => history(ws, out, &identifier),
    }
}

/// Resolve an issue handle from a UUID string or a `PREFIX-SEQ` identifier.
fn resolve_issue(ws: &Workspace, identifier: &str) -> Result<Issue> {
    if let Ok(uuid) = Uuid::parse_str(identifier) {
        return ws.query_issue_by_id(uuid);
    }
    let (prefix, _seq) = split_identifier(identifier)?;
    let project = resolve_project(ws, prefix)?;
    let issues = ws.query_issues(IssueFilter::for_project(project.id))?;
    issues
        .into_iter()
        .find(|i| i.identifier == identifier)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Issue,
            id: identifier.to_string(),
        })
}

/// Resolve a project handle from a UUID string or a prefix.
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

fn split_identifier(identifier: &str) -> Result<(&str, i64)> {
    let (prefix, seq_s) = identifier.split_once('-').ok_or_else(|| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "identifier".into(),
            reason: format!("expected `PREFIX-SEQ`, got '{identifier}'"),
        })
    })?;
    let seq: i64 = seq_s.parse().map_err(|_| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "identifier".into(),
            reason: format!("seq part of '{identifier}' is not a positive integer"),
        })
    })?;
    Ok((prefix, seq))
}

fn resolve_status(ws: &Workspace, project_id: Uuid, name: &str) -> Result<Status> {
    let statuses = ws.query_statuses_for_project(project_id)?;
    statuses
        .into_iter()
        .find(|s| s.name == name)
        .ok_or(kanban_core::Error::NotFound {
            kind: kanban_core::EntityKind::Status,
            id: name.to_string(),
        })
}

fn parse_date(s: &str, field: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        kanban_core::Error::Validation(kanban_core::ValidationError {
            field: field.into(),
            reason: format!("expected YYYY-MM-DD, got '{s}': {e}"),
        })
    })
}

fn parse_sort(s: &str) -> Result<SortBy> {
    match s {
        "manual" => Ok(SortBy::Manual),
        "priority" => Ok(SortBy::Priority),
        "created" => Ok(SortBy::Created),
        "updated" => Ok(SortBy::Updated),
        "due" => Ok(SortBy::Due),
        other => Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "sort".into(),
                reason: format!("unknown value '{other}'"),
            },
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn create(
    ws: &mut Workspace,
    out: &Out,
    project: &str,
    title: String,
    description: Option<String>,
    priority: &str,
    status: Option<String>,
    due: Option<String>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let priority: Priority = Priority::from_str(priority)?;
    let status_id = if let Some(name) = status {
        resolve_status(ws, p.id, &name)?.id
    } else {
        let statuses = ws.query_statuses_for_project(p.id)?;
        statuses
            .first()
            .ok_or(kanban_core::Error::NotFound {
                kind: kanban_core::EntityKind::Status,
                id: "default".into(),
            })?
            .id
    };
    let due_date = match due {
        Some(s) => Some(parse_date(&s, "due")?),
        None => None,
    };
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: p.id,
        title,
        description,
        status_id,
        priority,
        due_date,
        label_ids: vec![],
    }))?;
    let issue = ws.query_issue_by_id(id)?;
    if out.json {
        out.print_json(&issue)?;
    } else {
        println!("created {} ({})", issue.identifier, issue.title);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn list(
    ws: &Workspace,
    out: &Out,
    project: &str,
    statuses: &[String],
    priorities: &[String],
    labels: &[String],
    due_before: Option<&str>,
    due_after: Option<&str>,
    sort: &str,
    reverse: bool,
    limit: Option<i64>,
) -> Result<()> {
    let p = resolve_project(ws, project)?;
    let mut filter = IssueFilter::for_project(p.id);

    if !statuses.is_empty() {
        let resolved = ws.query_statuses_for_project(p.id)?;
        for name in statuses {
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
    for prio in priorities {
        filter.priorities.push(Priority::from_str(prio)?);
    }
    if !labels.is_empty() {
        let resolved = ws.query_labels_for_project(p.id)?;
        for name in labels {
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
    if let Some(d) = due_before {
        filter.due_before = Some(parse_date(d, "due-before")?);
    }
    if let Some(d) = due_after {
        filter.due_after = Some(parse_date(d, "due-after")?);
    }
    filter.sort = parse_sort(sort)?;
    filter.reverse = reverse;
    filter.limit = limit;

    let issues = ws.query_issues(filter)?;
    if out.json {
        out.print_json(&issues)?;
    } else {
        for i in &issues {
            println!("{}  {}  {}", i.identifier, i.priority.as_str(), i.title);
        }
    }
    Ok(())
}

fn show(ws: &Workspace, out: &Out, identifier: &str) -> Result<()> {
    let i = resolve_issue(ws, identifier)?;
    if out.json {
        out.print_json(&i)?;
    } else {
        println!("{}  {}  {}", i.identifier, i.priority.as_str(), i.title);
        if let Some(d) = &i.description {
            println!("description: {d}");
        }
        if let Some(due) = i.due_date {
            println!("due: {due}");
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn update(
    ws: &mut Workspace,
    out: &Out,
    identifier: &str,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    due: Option<String>,
) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    let mut applied = 0u32;
    if let Some(t) = title {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Title(t),
        }))?;
        applied += 1;
    }
    if let Some(d) = description {
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Description(Some(d)),
        }))?;
        applied += 1;
    }
    if let Some(p) = priority {
        let prio = Priority::from_str(&p)?;
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Priority(prio),
        }))?;
        applied += 1;
    }
    if let Some(name) = status {
        let s = resolve_status(ws, issue.project_id, &name)?;
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::Status(s.id),
        }))?;
        applied += 1;
    }
    if let Some(d) = due {
        let date = parse_date(&d, "due")?;
        ws.apply(Operation::UpdateIssueField(UpdateIssueField {
            id: issue.id,
            change: IssueFieldChange::DueDate(Some(date)),
        }))?;
        applied += 1;
    }
    if applied == 0 {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "fields".into(),
                reason: "supply at least one of --title/--description/--priority/--status/--due"
                    .into(),
            },
        ));
    }
    let after = ws.query_issue_by_id(issue.id)?;
    if out.json {
        out.print_json(&after)?;
    } else {
        println!("updated {}", after.identifier);
    }
    Ok(())
}

fn move_status(ws: &mut Workspace, out: &Out, identifier: &str, status: &str) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    let s = resolve_status(ws, issue.project_id, status)?;
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id: issue.id,
        change: IssueFieldChange::Status(s.id),
    }))?;
    let after = ws.query_issue_by_id(issue.id)?;
    if out.json {
        out.print_json(&after)?;
    } else {
        println!("moved {} to {}", after.identifier, status);
    }
    Ok(())
}

fn reorder(
    ws: &mut Workspace,
    out: &Out,
    identifier: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;

    // Read all peers within the same project + status, sorted by sort_key ASC.
    let peers = {
        let mut f = IssueFilter::for_project(issue.project_id);
        f.status_ids = vec![issue.status_id];
        ws.query_issues(f)?
    };

    let new_sort_key = match (before, after) {
        (Some(other), None) => {
            let other = resolve_issue(ws, other)?;
            let prev = peers
                .iter()
                .filter(|i| i.id != issue.id && i.sort_key < other.sort_key)
                .max_by(|a, b| {
                    a.sort_key
                        .partial_cmp(&b.sort_key)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match prev {
                Some(p) => 0.5 * (p.sort_key + other.sort_key),
                None => other.sort_key - 1.0,
            }
        }
        (None, Some(other)) => {
            let other = resolve_issue(ws, other)?;
            let next = peers
                .iter()
                .filter(|i| i.id != issue.id && i.sort_key > other.sort_key)
                .min_by(|a, b| {
                    a.sort_key
                        .partial_cmp(&b.sort_key)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match next {
                Some(n) => 0.5 * (other.sort_key + n.sort_key),
                None => other.sort_key + 1.0,
            }
        }
        _ => {
            return Err(kanban_core::Error::Validation(
                kanban_core::ValidationError {
                    field: "reorder".into(),
                    reason: "supply exactly one of --before or --after".into(),
                },
            ));
        }
    };

    ws.apply(Operation::ReorderIssue(ReorderIssue {
        id: issue.id,
        new_sort_key,
    }))?;
    let after_issue = ws.query_issue_by_id(issue.id)?;
    if out.json {
        out.print_json(&after_issue)?;
    } else {
        println!("reordered {}", after_issue.identifier);
    }
    Ok(())
}

fn delete(ws: &mut Workspace, out: &Out, identifier: &str, yes: bool) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    if !yes {
        return Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "confirm".into(),
                reason: "pass --yes to confirm deletion".into(),
            },
        ));
    }
    ws.apply(Operation::DeleteIssue(DeleteIssue { id: issue.id }))?;
    if !out.json {
        println!("deleted {}", issue.identifier);
    }
    Ok(())
}

fn history(ws: &Workspace, out: &Out, identifier: &str) -> Result<()> {
    let issue = resolve_issue(ws, identifier)?;
    let entries = ws.query_issue_history(issue.id)?;
    if out.json {
        out.print_json(&entries)?;
    } else {
        for e in &entries {
            let old = e.old_value.as_deref().unwrap_or("");
            let new = e.new_value.as_deref().unwrap_or("");
            println!("{}  {}: {} -> {}", e.at.to_rfc3339(), e.field, old, new);
        }
    }
    Ok(())
}
