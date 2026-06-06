use crate::error::{Error, Result};
use crate::operation::{
    ConflictPolicy, CreateMember, DeleteMember, ImportSnapshot, MemberPatch, Operation,
    UpdateMember,
};
use crate::store::write::members as wm;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateMember, now: DateTime<Utc>) -> Result<()> {
    validate::nonempty_field("name", &args.name)?;
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM members WHERE project_id = ?1 AND name = ?2",
        params![args.project_id.to_string(), &args.name],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if exists {
        return Err(Error::Conflict(format!(
            "member '{}' already exists in project",
            args.name
        )));
    }
    wm::insert(tx, args.id, args.project_id, &args.name, now)?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateMember) -> Result<()> {
    if let Some(n) = &args.patch.name {
        validate::nonempty_field("name", n)?;
    }
    wm::update_fields(tx, args.id, args.patch.name.as_deref())?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteMember) -> Result<()> {
    // member must exist — by_id_via_tx errors NotFound if absent.
    crate::store::read::members::by_id_via_tx(tx, args.id)?;
    // The DB nulls any dependent issues.assignee_id via ON DELETE SET NULL.
    wm::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateMember) -> Operation {
    Operation::DeleteMember(DeleteMember { id: args.id })
}

pub(crate) fn inverse_of_update(tx: &Transaction<'_>, args: &UpdateMember) -> Result<Operation> {
    let m = crate::store::read::members::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateMember(UpdateMember {
        id: m.id,
        patch: MemberPatch {
            name: args.patch.name.as_ref().map(|_| m.name),
        },
    }))
}

/// Capture the inverse of `DeleteMember` as an `ImportSnapshot` carrying the
/// member row + every issue whose `assignee_id` referenced it, so undo restores
/// the member AND re-applies the assignments that `ON DELETE SET NULL` cleared.
pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteMember) -> Result<Operation> {
    let snapshot = crate::apply::snapshot::export_member_subtree(tx, args.id)?;
    Ok(Operation::ImportSnapshot(ImportSnapshot {
        snapshot,
        policy: ConflictPolicy::Overwrite,
    }))
}
