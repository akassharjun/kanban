use crate::error::{Error, Result};
use crate::operation::{
    ConflictPolicy, CreateStatus, DeleteStatus, ImportSnapshot, Operation, ReorderStatus,
    StatusPatch, UpdateStatus,
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

/// Delete a status, guarded so it cannot orphan issues or empty a project's
/// status set. Both guards roll back the whole transaction on failure, so the
/// inverse captured before dispatch is discarded.
pub(crate) fn delete(tx: &Transaction<'_>, a: &DeleteStatus) -> Result<()> {
    // status must exist (and we need its project_id) — by_id_via_tx errors NotFound if absent
    let s = crate::store::read::statuses::by_id_via_tx(tx, a.id)?;
    // guard 1: no issues may reference it
    let issue_count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM issues WHERE status_id = ?1",
        params![a.id.to_string()],
        |r| r.get(0),
    )?;
    if issue_count > 0 {
        return Err(Error::Conflict(format!(
            "status still has {issue_count} issue(s); move or delete them first"
        )));
    }
    // guard 2: a project must keep at least one status
    let status_count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM statuses WHERE project_id = ?1",
        params![s.project_id.to_string()],
        |r| r.get(0),
    )?;
    if status_count <= 1 {
        return Err(Error::Conflict(
            "a project must keep at least one status".into(),
        ));
    }
    ws::delete(tx, a.id)?;
    Ok(())
}

/// Re-pack a project's status positions so that `a.id` lands at `new_position`.
/// Positions are reassigned densely (`0..n`) so the order stays gap-free.
pub(crate) fn reorder(tx: &Transaction<'_>, a: &ReorderStatus) -> Result<()> {
    let target = crate::store::read::statuses::by_id_via_tx(tx, a.id)?;
    // for_project_via_tx orders by position ASC.
    let mut ordered = crate::store::read::statuses::for_project_via_tx(tx, target.project_id)?;
    ordered.retain(|s| s.id != a.id);
    // Clamp the requested position into `0..=ordered.len()`. `usize::try_from`
    // maps any negative request to 0 (treated as "front").
    let idx = usize::try_from(a.new_position.max(0))
        .unwrap_or(0)
        .min(ordered.len());
    ordered.insert(idx, target);
    for (pos, s) in ordered.iter().enumerate() {
        // `pos` is a vector index, always well within i64 range.
        let position = i64::try_from(pos).unwrap_or(i64::MAX);
        ws::update_position(tx, s.id, position)?;
    }
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

/// Capture every status row of the affected project as an `ImportSnapshot`, so
/// undo re-imports them under `Overwrite` and restores all prior positions.
pub(crate) fn inverse_of_reorder(tx: &Transaction<'_>, a: &ReorderStatus) -> Result<Operation> {
    let s = crate::store::read::statuses::by_id_via_tx(tx, a.id)?;
    Ok(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: crate::apply::snapshot::export_project_statuses(tx, s.project_id)?,
        policy: ConflictPolicy::Overwrite,
    }))
}
