#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{CreateProject, Operation};
use kanban_core::{Workspace, new_id};

#[test]
fn undo_create_project_removes_it() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Tmp".into(),
        prefix: "TMP".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.undo().unwrap();
    assert!(ws.query_project_by_id(id).is_err());
}

#[test]
fn undo_then_redo_restores_state() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "RDO".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.undo().unwrap();
    ws.redo().unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.prefix, "RDO");
}

#[test]
fn forward_op_truncates_redo_branch() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "A".into(),
        prefix: "AAA".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.undo().unwrap();
    let id2 = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: id2,
        name: "B".into(),
        prefix: "BBB".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    // Redo should now error: nothing to redo.
    let err = ws.redo().unwrap_err();
    assert!(err.to_string().contains("nothing to redo"), "{err}");
}

#[test]
fn undo_with_empty_log_errors() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws.undo().unwrap_err();
    assert!(err.to_string().contains("nothing to undo"), "{err}");
}

#[test]
fn undo_persists_across_workspace_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    let id = new_id();
    {
        let mut ws = Workspace::open(&path).unwrap();
        ws.apply(Operation::CreateProject(CreateProject {
            id,
            name: "P".into(),
            prefix: "PER".into(),
            description: None,
            icon: None,
        }))
        .unwrap();
        ws.undo().unwrap();
    }
    {
        let mut ws = Workspace::open(&path).unwrap();
        // After reopen, undo branch survives — redo should still work.
        ws.redo().unwrap();
        let p = ws.query_project_by_id(id).unwrap();
        assert_eq!(p.prefix, "PER");
    }
}

#[test]
fn forward_op_after_undoing_an_activity_emitting_op_succeeds() {
    // Regression: a new forward op truncates the redo branch by deleting the
    // undone `operation_log` rows. Ops like `UpdateIssueField` also write an
    // `activity_log` row (FK `op_id -> operation_log`, NO ACTION). Those
    // children must be removed before their parent rows, or the truncation
    // fails with "FOREIGN KEY constraint failed". `CreateProject` emits no
    // activity row, which is why the existing truncation test missed this.
    use kanban_core::operation::{CreateIssue, IssueFieldChange, UpdateIssueField};
    use kanban_core::types::Priority;

    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "PRJ".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let status_id = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let iid = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid,
        project_id: pid,
        title: "v1".into(),
        description: None,
        status_id,
        priority: Priority::Medium,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    // An UpdateIssueField writes an activity_log row linked to its op_id.
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id: iid,
        change: IssueFieldChange::Title("v2".into()),
    }))
    .unwrap();

    // Undo marks that op as a redo-branch entry; its activity row remains.
    ws.undo().unwrap();

    // A new forward op must truncate the redo branch without an FK error.
    let iid2 = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid2,
        project_id: pid,
        title: "another".into(),
        description: None,
        status_id,
        priority: Priority::Medium,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    // The new issue exists and the redo branch is gone.
    assert!(ws.query_issue_by_id(iid2).is_ok());
    assert!(ws.redo().is_err());
}
