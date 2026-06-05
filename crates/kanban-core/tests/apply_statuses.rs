#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    CreateIssue, CreateProject, CreateStatus, DeleteStatus, Operation, ReorderStatus, StatusPatch,
    UpdateStatus,
};
use kanban_core::types::{Priority, StatusCategory};
use kanban_core::{Workspace, new_id};

fn fresh_with_project() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "S".into(),
        prefix: "STS".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    (ws, pid)
}

fn status_named(ws: &Workspace, pid: uuid::Uuid, name: &str) -> Option<kanban_core::types::Status> {
    ws.query_statuses_for_project(pid)
        .unwrap()
        .into_iter()
        .find(|s| s.name == name)
}

/// The project's statuses ordered by `position` (the query already sorts ASC).
fn ordered_statuses(ws: &Workspace, pid: uuid::Uuid) -> Vec<kanban_core::types::Status> {
    let mut v = ws.query_statuses_for_project(pid).unwrap();
    v.sort_by_key(|s| s.position);
    v
}

#[test]
fn create_status_inserts() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    let s = status_named(&ws, pid, "QA").unwrap();
    assert_eq!(s.id, id);
    assert_eq!(s.category, StatusCategory::Started);
    assert_eq!(s.color, "#3b82f6");
    assert_eq!(s.position, 7);
}

#[test]
fn create_status_rejects_duplicate_name() {
    let (mut ws, pid) = fresh_with_project();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id: new_id(),
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    let err = ws
        .apply(Operation::CreateStatus(CreateStatus {
            id: new_id(),
            project_id: pid,
            name: "QA".into(),
            category: StatusCategory::Started,
            color: "#3b82f6".into(),
            position: 8,
        }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn update_status_renames() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    ws.apply(Operation::UpdateStatus(UpdateStatus {
        id,
        patch: StatusPatch {
            name: Some("Testing".into()),
            ..Default::default()
        },
    }))
    .unwrap();
    assert!(status_named(&ws, pid, "QA").is_none());
    assert!(status_named(&ws, pid, "Testing").is_some());
}

#[test]
fn create_status_undo_removes() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    assert!(status_named(&ws, pid, "QA").is_some());
    ws.undo().unwrap();
    assert!(status_named(&ws, pid, "QA").is_none());
}

#[test]
fn update_status_undo_restores_name() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    ws.apply(Operation::UpdateStatus(UpdateStatus {
        id,
        patch: StatusPatch {
            name: Some("Testing".into()),
            ..Default::default()
        },
    }))
    .unwrap();
    ws.undo().unwrap();
    assert!(status_named(&ws, pid, "Testing").is_none());
    assert!(status_named(&ws, pid, "QA").is_some());
}

#[test]
fn delete_status_undo_restores() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateStatus(CreateStatus {
        id,
        project_id: pid,
        name: "QA".into(),
        category: StatusCategory::Started,
        color: "#3b82f6".into(),
        position: 7,
    }))
    .unwrap();
    ws.apply(Operation::DeleteStatus(DeleteStatus { id }))
        .unwrap();
    assert!(status_named(&ws, pid, "QA").is_none());
    ws.undo().unwrap();
    let s = status_named(&ws, pid, "QA").unwrap();
    assert_eq!(s.id, id);
    assert_eq!(s.position, 7);
    assert_eq!(s.category, StatusCategory::Started);
}

#[test]
fn delete_status_blocked_when_issues_present() {
    let (mut ws, pid) = fresh_with_project();
    // The seeded "Todo" status is one of the 7 defaults.
    let todo = status_named(&ws, pid, "Todo").unwrap();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(),
        project_id: pid,
        title: "an issue".into(),
        description: None,
        status_id: todo.id,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let err = ws
        .apply(Operation::DeleteStatus(DeleteStatus { id: todo.id }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("issue"), "{err}");
    // The blocked delete must have rolled back: status still present.
    assert!(status_named(&ws, pid, "Todo").is_some());
}

#[test]
fn delete_status_blocked_when_last() {
    let (mut ws, pid) = fresh_with_project();
    // Delete defaults down to exactly one (each is empty), then the last errors.
    let all = ordered_statuses(&ws, pid);
    assert!(all.len() > 1, "project should seed multiple defaults");
    for s in &all[1..] {
        ws.apply(Operation::DeleteStatus(DeleteStatus { id: s.id }))
            .unwrap();
    }
    let remaining = ordered_statuses(&ws, pid);
    assert_eq!(remaining.len(), 1);

    let err = ws
        .apply(Operation::DeleteStatus(DeleteStatus {
            id: remaining[0].id,
        }))
        .unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("at least one"),
        "{err}"
    );
    // Rolled back: the last status survives.
    assert_eq!(ordered_statuses(&ws, pid).len(), 1);
}

#[test]
fn reorder_status_changes_order() {
    let (mut ws, pid) = fresh_with_project();
    let before = ordered_statuses(&ws, pid);
    let last = before.last().unwrap().clone();
    let first_before = before.first().unwrap().clone();

    ws.apply(Operation::ReorderStatus(ReorderStatus {
        id: last.id,
        new_position: 0,
    }))
    .unwrap();

    let after = ordered_statuses(&ws, pid);
    assert_eq!(
        after.first().unwrap().id,
        last.id,
        "moved status is now first"
    );
    assert_eq!(after[1].id, first_before.id, "old first shifted to index 1");
    // positions remain dense 0..n
    for (i, s) in after.iter().enumerate() {
        assert_eq!(s.position, i64::try_from(i).unwrap());
    }
}

#[test]
fn reorder_status_undo_restores_order() {
    let (mut ws, pid) = fresh_with_project();
    let original_names: Vec<String> = ordered_statuses(&ws, pid)
        .into_iter()
        .map(|s| s.name)
        .collect();
    let last_id = ordered_statuses(&ws, pid).last().unwrap().id;

    ws.apply(Operation::ReorderStatus(ReorderStatus {
        id: last_id,
        new_position: 0,
    }))
    .unwrap();
    // sanity: order changed
    let reordered_names: Vec<String> = ordered_statuses(&ws, pid)
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_ne!(reordered_names, original_names);

    ws.undo().unwrap();
    let restored_names: Vec<String> = ordered_statuses(&ws, pid)
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(restored_names, original_names);
}
