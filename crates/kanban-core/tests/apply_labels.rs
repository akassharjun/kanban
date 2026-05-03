#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    CreateLabel, CreateProject, DeleteLabel, LabelPatch, Operation, UpdateLabel,
};
use kanban_core::{Workspace, new_id};

fn fresh_with_project() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "L".into(),
        prefix: "LBL".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    (ws, pid)
}

#[test]
fn create_label_inserts() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id,
        project_id: pid,
        name: "backend".into(),
        color: "#3b82f6".into(),
    }))
    .unwrap();
    let labels = ws.query_labels_for_project(pid).unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "backend");
}

#[test]
fn create_label_rejects_duplicate_name_per_project() {
    let (mut ws, pid) = fresh_with_project();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: new_id(),
        project_id: pid,
        name: "dup".into(),
        color: "#000000".into(),
    }))
    .unwrap();
    let err = ws
        .apply(Operation::CreateLabel(CreateLabel {
            id: new_id(),
            project_id: pid,
            name: "dup".into(),
            color: "#000000".into(),
        }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn create_label_validates_color() {
    let (mut ws, pid) = fresh_with_project();
    let err = ws
        .apply(Operation::CreateLabel(CreateLabel {
            id: new_id(),
            project_id: pid,
            name: "x".into(),
            color: "blue".into(),
        }))
        .unwrap_err();
    assert!(err.to_string().contains("color"), "{err}");
}

#[test]
fn update_label_renames() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id,
        project_id: pid,
        name: "old".into(),
        color: "#000000".into(),
    }))
    .unwrap();
    ws.apply(Operation::UpdateLabel(UpdateLabel {
        id,
        patch: LabelPatch {
            name: Some("new".into()),
            ..Default::default()
        },
    }))
    .unwrap();
    let labels = ws.query_labels_for_project(pid).unwrap();
    assert_eq!(labels[0].name, "new");
}

#[test]
fn delete_label_undo_restores_it() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id,
        project_id: pid,
        name: "kept".into(),
        color: "#000000".into(),
    }))
    .unwrap();
    ws.apply(Operation::DeleteLabel(DeleteLabel { id }))
        .unwrap();
    assert!(ws.query_labels_for_project(pid).unwrap().is_empty());
    ws.undo().unwrap();
    assert_eq!(ws.query_labels_for_project(pid).unwrap().len(), 1);
}
