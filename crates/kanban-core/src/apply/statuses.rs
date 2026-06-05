use crate::error::{Error, Result};
use crate::operation::{
    ConflictPolicy, CreateStatus, DeleteStatus, ImportSnapshot, Operation, StatusPatch,
    UpdateStatus,
};
use crate::store::write::statuses as ws;
use crate::validate;
use rusqlite::{Transaction, params};

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateStatus) -> Result<()> {
    validate::nonempty_field("name", &args.name)?;
    validate::hex_color(&args.color)?;
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND name = ?2",
        params![args.project_id.to_string(), &args.name],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if exists {
        return Err(Error::Conflict(format!(
            "status '{}' already exists in project",
            args.name
        )));
    }
    ws::insert(
        tx,
        args.id,
        args.project_id,
        &args.name,
        args.category,
        &args.color,
        args.position,
    )?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateStatus) -> Result<()> {
    if let Some(n) = &args.patch.name {
        validate::nonempty_field("name", n)?;
    }
    if let Some(c) = &args.patch.color {
        validate::hex_color(c)?;
    }
    ws::update_fields(
        tx,
        args.id,
        args.patch.name.as_deref(),
        args.patch.category,
        args.patch.color.as_deref(),
    )?;
    Ok(())
}

/// Plain delete (no usage guards). Guards land in Task 2 alongside
/// `ReorderStatus`.
pub(crate) fn delete_plain(tx: &Transaction<'_>, args: &DeleteStatus) -> Result<()> {
    ws::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateStatus) -> Operation {
    Operation::DeleteStatus(DeleteStatus { id: args.id })
}

pub(crate) fn inverse_of_update(tx: &Transaction<'_>, args: &UpdateStatus) -> Result<Operation> {
    let s = crate::store::read::statuses::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateStatus(UpdateStatus {
        id: s.id,
        patch: StatusPatch {
            name: args.patch.name.as_ref().map(|_| s.name.clone()),
            category: args.patch.category.as_ref().map(|_| s.category),
            color: args.patch.color.as_ref().map(|_| s.color.clone()),
        },
    }))
}

/// Capture the inverse of `DeleteStatus` as an `ImportSnapshot` carrying the
/// status row, so undo restores `id`/`category`/`color`/`position` exactly.
pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteStatus) -> Result<Operation> {
    let snapshot = crate::apply::snapshot::export_status_row(tx, args.id)?;
    Ok(Operation::ImportSnapshot(ImportSnapshot {
        snapshot,
        policy: ConflictPolicy::Overwrite,
    }))
}
