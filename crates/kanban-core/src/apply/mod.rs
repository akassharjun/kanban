use crate::error::{Error, Result};
use crate::operation::{DeleteProject, Operation, OperationOutcome};
use crate::store::write::operation_log;
use crate::workspace::Workspace;

pub(crate) mod issues;
pub(crate) mod labels;
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

        // Capture pre-state needed to invert this op (read DB while it still
        // reflects the old state).
        let inverse = capture_inverse(&tx, &op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        // Insert the operation_log row first so we have an op_id to attach
        // to any activity rows the dispatch produces.
        let op_id = operation_log::insert_operation(
            &tx,
            op_type_name(&op),
            &payload,
            &inverse_payload,
            now,
        )?;

        // Capture pre-state for activity emission, then mutate, then emit.
        let pre = capture_activity_pre(&tx, &op)?;
        dispatch(&tx, &op, now)?;
        emit_activity(&tx, op_id, &op, &pre, now)?;

        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

/// Execute the per-op mutation inside an existing transaction without touching
/// the operation log. Shared by `Workspace::apply`, `undo`, and `redo`.
pub(crate) fn dispatch(
    tx: &rusqlite::Transaction<'_>,
    op: &Operation,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    match op {
        Operation::CreateProject(args) => projects::create(tx, args, now)?,
        Operation::UpdateProject(args) => projects::update(tx, args, now)?,
        Operation::ArchiveProject(args) => projects::archive(tx, args, now)?,
        Operation::DeleteProject(args) => projects::delete(tx, args)?,
        Operation::CreateIssue(args) => issues::create(tx, args, now)?,
        Operation::UpdateIssueField(args) => issues::update_field(tx, args, now)?,
        Operation::ReorderIssue(args) => issues::reorder(tx, args, now)?,
        Operation::DeleteIssue(args) => issues::delete(tx, args)?,
        Operation::CreateLabel(args) => labels::create(tx, args)?,
        Operation::UpdateLabel(args) => labels::update(tx, args)?,
        Operation::DeleteLabel(args) => labels::delete(tx, args)?,
        Operation::AttachLabel(args) => labels::attach(tx, args)?,
        Operation::DetachLabel(args) => labels::detach(tx, args)?,
    }
    Ok(())
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
        Operation::CreateIssue(args) => Ok(issues::inverse_of_create(args)),
        Operation::UpdateIssueField(args) => issues::inverse_of_update_field(tx, args),
        Operation::ReorderIssue(args) => issues::inverse_of_reorder(tx, args),
        Operation::DeleteIssue(args) => issues::inverse_of_delete(tx, args),
        Operation::CreateLabel(args) => Ok(labels::inverse_of_create(args)),
        Operation::DeleteLabel(args) => labels::inverse_of_delete(tx, args),
        Operation::UpdateLabel(args) => labels::inverse_of_update(tx, args),
        Operation::AttachLabel(args) => Ok(labels::inverse_of_attach(args)),
        Operation::DetachLabel(args) => Ok(labels::inverse_of_detach(args)),
    }
}

/// Pre-dispatch state captured to populate `activity_log` rows after dispatch
/// has mutated the database. Only the variants that need history rows are
/// populated — everything else stays `None`.
#[derive(Default)]
pub(crate) struct ActivityPre {
    pub(crate) issue_pre: Option<crate::types::Issue>,
}

fn capture_activity_pre(tx: &rusqlite::Transaction<'_>, op: &Operation) -> Result<ActivityPre> {
    let mut pre = ActivityPre::default();
    if let Operation::UpdateIssueField(args) = op {
        pre.issue_pre = Some(crate::store::read::issues::by_id_via_tx(tx, args.id)?);
    }
    Ok(pre)
}

fn emit_activity(
    tx: &rusqlite::Transaction<'_>,
    op_id: i64,
    op: &Operation,
    pre: &ActivityPre,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    if let Operation::UpdateIssueField(args) = op {
        // capture_activity_pre always populates issue_pre for this op variant.
        let Some(pre_issue) = pre.issue_pre.as_ref() else {
            return Err(Error::InvalidSnapshot(
                "internal: missing issue_pre for UpdateIssueField".into(),
            ));
        };
        let (field, old, new) = match &args.change {
            crate::operation::IssueFieldChange::Title(v) => {
                ("title", Some(pre_issue.title.clone()), Some(v.clone()))
            }
            crate::operation::IssueFieldChange::Description(v) => {
                ("description", pre_issue.description.clone(), v.clone())
            }
            crate::operation::IssueFieldChange::Status(v) => (
                "status",
                Some(pre_issue.status_id.to_string()),
                Some(v.to_string()),
            ),
            crate::operation::IssueFieldChange::Priority(v) => (
                "priority",
                Some(pre_issue.priority.as_str().to_string()),
                Some(v.as_str().to_string()),
            ),
            crate::operation::IssueFieldChange::DueDate(v) => (
                "due_date",
                pre_issue.due_date.map(|d| d.to_string()),
                v.map(|d| d.to_string()),
            ),
        };
        let issue_id_s = args.id.to_string();
        crate::store::write::operation_log::insert_activity(
            tx,
            op_id,
            Some(issue_id_s.as_str()),
            field,
            old.as_deref(),
            new.as_deref(),
            now,
        )?;
    }
    Ok(())
}
