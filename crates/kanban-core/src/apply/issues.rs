use crate::error::{Error, Result};
use crate::operation::{CreateIssue, DeleteIssue, Operation};
use crate::store::write::issues as wi;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateIssue, now: DateTime<Utc>) -> Result<()> {
    validate::nonempty_field("title", &args.title)?;

    // Project + status FKs validated implicitly by INSERT, but produce a clearer error.
    let project_exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM projects WHERE id = ?1",
        params![args.project_id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !project_exists {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Project,
            id: args.project_id.to_string(),
        });
    }

    let status_exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM statuses WHERE id = ?1 AND project_id = ?2",
        params![args.status_id.to_string(), args.project_id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !status_exists {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Status,
            id: args.status_id.to_string(),
        });
    }

    // Sort key: place at end. Compute MAX(sort_key)+1.0.
    let max_sort: f64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_key), 0.0) FROM issues
         WHERE project_id = ?1 AND status_id = ?2",
        params![args.project_id.to_string(), args.status_id.to_string()],
        |r| r.get(0),
    )?;

    let title = validate::nonempty_field("title", &args.title)?.to_string();

    wi::insert(
        tx,
        args.id,
        args.project_id,
        &title,
        args.description.as_deref(),
        args.status_id,
        args.priority,
        args.due_date,
        max_sort + 1.0,
        now,
    )?;

    for label_id in &args.label_ids {
        tx.execute(
            "INSERT INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
            params![args.id.to_string(), label_id.to_string()],
        )?;
    }

    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteIssue) -> Result<()> {
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM issues WHERE id = ?1",
        params![args.id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !exists {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Issue,
            id: args.id.to_string(),
        });
    }
    wi::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateIssue) -> Operation {
    Operation::DeleteIssue(DeleteIssue { id: args.id })
}

pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteIssue) -> Result<Operation> {
    let issue = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    let mut label_ids = Vec::new();
    let mut stmt = tx.prepare("SELECT label_id FROM issue_labels WHERE issue_id = ?1")?;
    let rows = stmt.query_map(params![args.id.to_string()], |r| r.get::<_, String>(0))?;
    for r in rows {
        let id_s = r?;
        label_ids.push(Uuid::parse_str(&id_s).map_err(|e| Error::InvalidSnapshot(e.to_string()))?);
    }
    Ok(Operation::CreateIssue(CreateIssue {
        id: issue.id,
        project_id: issue.project_id,
        title: issue.title,
        description: issue.description,
        status_id: issue.status_id,
        priority: issue.priority,
        due_date: issue.due_date,
        label_ids,
    }))
}
