use crate::error::{Error, Result};
use crate::operation::{Operation, OperationOutcome};
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
        let inverse = compute_inverse(&op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        let tx = self.conn.transaction()?;
        // Discard redo branch when a new forward op lands.
        operation_log::truncate_redo_branch(&tx)?;

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

/// Inverse computation. Implemented incrementally (Task 15, 23, 26).
/// Until the inverse for a variant is implemented, we return a placeholder operation
/// that, when applied, would error — undo against an unsupported op simply fails loud.
pub(crate) fn compute_inverse(op: &Operation) -> Result<Operation> {
    match op {
        // Project inverses land in Task 15.
        Operation::CreateProject(args) => {
            Ok(Operation::DeleteProject(crate::operation::DeleteProject {
                id: args.id,
            }))
        }
        Operation::DeleteProject(args) => {
            Ok(Operation::CreateProject(crate::operation::CreateProject {
                id: args.id,
                name: "<undo placeholder>".into(),
                prefix: "UNDO".into(),
                description: None,
                icon: None,
            }))
        }
        Operation::UpdateProject(args) => Ok(Operation::UpdateProject(args.clone())),
        Operation::ArchiveProject(args) => Ok(Operation::ArchiveProject(args.clone())),
        // Issue/label inverses land in later phases.
        other => Err(Error::InvalidSnapshot(format!(
            "inverse not yet implemented for {other:?}"
        ))),
    }
}
