use crate::error::{Error, Result};
use crate::operation::{ArchiveProject, CreateProject, DeleteProject, UpdateProject};
use crate::store::write::{projects as wp, statuses as ws};
use crate::types::ProjectStatus;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateProject, now: DateTime<Utc>) -> Result<()> {
    let name = validate::nonempty_field("name", &args.name)?.to_string();
    validate::project_prefix(&args.prefix)?;

    let exists: Option<i64> = tx
        .query_row(
            "SELECT 1 FROM projects WHERE prefix = ?1",
            params![&args.prefix],
            |r| r.get(0),
        )
        .ok();
    if exists.is_some() {
        return Err(Error::Conflict(format!(
            "project prefix '{}' is already in use",
            args.prefix
        )));
    }

    wp::insert(
        tx,
        args.id,
        &name,
        &args.prefix,
        args.description.as_deref(),
        args.icon.as_deref(),
        now,
    )?;
    ws::seed_defaults(tx, args.id)?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateProject, now: DateTime<Utc>) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Project,
            id: args.id.to_string(),
        });
    }
    if let Some(name) = &args.patch.name {
        validate::nonempty_field("name", name)?;
    }
    wp::update_fields(
        tx,
        args.id,
        args.patch.name.as_deref(),
        args.patch.description.as_ref().map(|o| o.as_deref()),
        args.patch.icon.as_ref().map(|o| o.as_deref()),
        args.patch.status,
        now,
    )?;
    Ok(())
}

pub(crate) fn archive(
    tx: &Transaction<'_>,
    args: &ArchiveProject,
    now: DateTime<Utc>,
) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Project,
            id: args.id.to_string(),
        });
    }
    wp::set_status(tx, args.id, ProjectStatus::Archived, now)?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteProject) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Project,
            id: args.id.to_string(),
        });
    }
    wp::delete(tx, args.id)?;
    Ok(())
}

fn exists(tx: &Transaction<'_>, id: uuid::Uuid) -> Result<bool> {
    let n: i64 = tx.query_row(
        "SELECT COUNT(*) FROM projects WHERE id = ?1",
        params![id.to_string()],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}
