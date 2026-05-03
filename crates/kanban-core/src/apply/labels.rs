use crate::error::{Error, Result};
use crate::operation::{
    AttachLabel, CreateLabel, DeleteLabel, DetachLabel, LabelPatch, Operation, UpdateLabel,
};
use crate::store::write::labels as wl;
use crate::validate;
use rusqlite::{Transaction, params};

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateLabel) -> Result<()> {
    validate::nonempty_field("name", &args.name)?;
    validate::hex_color(&args.color)?;
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM labels WHERE project_id = ?1 AND name = ?2",
        params![args.project_id.to_string(), &args.name],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if exists {
        return Err(Error::Conflict(format!(
            "label '{}' already exists in project",
            args.name
        )));
    }
    wl::insert(tx, args.id, args.project_id, &args.name, &args.color)?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateLabel) -> Result<()> {
    if let Some(c) = &args.patch.color {
        validate::hex_color(c)?;
    }
    if let Some(n) = &args.patch.name {
        validate::nonempty_field("name", n)?;
    }
    wl::update_fields(
        tx,
        args.id,
        args.patch.name.as_deref(),
        args.patch.color.as_deref(),
    )?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteLabel) -> Result<()> {
    wl::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn attach(tx: &Transaction<'_>, args: &AttachLabel) -> Result<()> {
    wl::attach(tx, args.issue_id, args.label_id)?;
    Ok(())
}

pub(crate) fn detach(tx: &Transaction<'_>, args: &DetachLabel) -> Result<()> {
    wl::detach(tx, args.issue_id, args.label_id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateLabel) -> Operation {
    Operation::DeleteLabel(DeleteLabel { id: args.id })
}

pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteLabel) -> Result<Operation> {
    let l = crate::store::read::labels::by_id_via_tx(tx, args.id)?;
    Ok(Operation::CreateLabel(CreateLabel {
        id: l.id,
        project_id: l.project_id,
        name: l.name,
        color: l.color,
    }))
}

pub(crate) fn inverse_of_update(tx: &Transaction<'_>, args: &UpdateLabel) -> Result<Operation> {
    let l = crate::store::read::labels::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateLabel(UpdateLabel {
        id: l.id,
        patch: LabelPatch {
            name: args.patch.name.as_ref().map(|_| l.name),
            color: args.patch.color.as_ref().map(|_| l.color),
        },
    }))
}

pub(crate) fn inverse_of_attach(args: &AttachLabel) -> Operation {
    Operation::DetachLabel(DetachLabel {
        issue_id: args.issue_id,
        label_id: args.label_id,
    })
}

pub(crate) fn inverse_of_detach(args: &DetachLabel) -> Operation {
    Operation::AttachLabel(AttachLabel {
        issue_id: args.issue_id,
        label_id: args.label_id,
    })
}
