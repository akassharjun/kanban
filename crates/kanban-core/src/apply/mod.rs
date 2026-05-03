use crate::error::{Error, Result};
use crate::operation::{DeleteProject, Operation, OperationOutcome};
use crate::store::write::operation_log;
use crate::workspace::Workspace;

pub(crate) mod projects;

impl Workspace {
    /// The single public mutator. Validates, executes, and records `op` in one transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails, the underlying database errors, or the
    /// operation variant has no apply implementation yet.
    // The op is taken by value to give callers a clean ownership model even though
    // every internal site borrows it.
    #[allow(clippy::needless_pass_by_value)]
    pub fn apply(&mut self, op: Operation) -> Result<OperationOutcome> {
        let now = self.clock.now();
        let payload = serde_json::to_string(&op)?;
        let tx = self.conn.transaction()?;
        operation_log::truncate_redo_branch(&tx)?;

        // Capture pre-state needed to invert this op.
        let inverse = capture_inverse(&tx, &op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        match &op {
            Operation::CreateProject(args) => projects::create(&tx, args, now)?,
            Operation::UpdateProject(args) => projects::update(&tx, args, now)?,
            Operation::ArchiveProject(args) => projects::archive(&tx, args, now)?,
            Operation::DeleteProject(args) => projects::delete(&tx, args)?,
            // Issue/label arms land in Phase 8/9 — until then return InvalidSnapshot.
            other => return Err(Error::InvalidSnapshot(format!("unsupported op: {other:?}"))),
        }

        let op_id = operation_log::insert_operation(
            &tx,
            op_type_name(&op),
            &payload,
            &inverse_payload,
            now,
        )?;
        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

fn op_type_name(op: &Operation) -> &'static str {
    match op {
        Operation::CreateProject(_) => "CreateProject",
        Operation::UpdateProject(_) => "UpdateProject",
        Operation::ArchiveProject(_) => "ArchiveProject",
        Operation::DeleteProject(_) => "DeleteProject",
        Operation::CreateIssue(_) => "CreateIssue",
        Operation::UpdateIssueField(_) => "UpdateIssueField",
        Operation::ReorderIssue(_) => "ReorderIssue",
        Operation::DeleteIssue(_) => "DeleteIssue",
        Operation::CreateLabel(_) => "CreateLabel",
        Operation::UpdateLabel(_) => "UpdateLabel",
        Operation::DeleteLabel(_) => "DeleteLabel",
        Operation::AttachLabel(_) => "AttachLabel",
        Operation::DetachLabel(_) => "DetachLabel",
    }
}

fn capture_inverse(tx: &rusqlite::Transaction<'_>, op: &Operation) -> Result<Operation> {
    match op {
        Operation::CreateProject(args) => {
            Ok(Operation::DeleteProject(DeleteProject { id: args.id }))
        }
        Operation::DeleteProject(args) => projects::inverse_of_delete(tx, args),
        Operation::UpdateProject(args) => projects::inverse_of_update(tx, args),
        Operation::ArchiveProject(args) => projects::inverse_of_archive(tx, args),
        // Issue/label inverses come in later phases.
        other => Err(Error::InvalidSnapshot(format!(
            "inverse not yet implemented for {other:?}"
        ))),
    }
}
