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
