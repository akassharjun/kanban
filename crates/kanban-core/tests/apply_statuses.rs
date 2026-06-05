#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    CreateProject, CreateStatus, DeleteStatus, Operation, StatusPatch, UpdateStatus,
};
use kanban_core::types::StatusCategory;
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
