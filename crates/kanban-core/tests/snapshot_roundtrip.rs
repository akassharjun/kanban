#![allow(clippy::unwrap_used)]

use kanban_core::operation::{
    ConflictPolicy, CreateIssue, CreateLabel, CreateProject, ImportSnapshot, Operation,
};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn export_then_import_into_empty_yields_equivalent_state() {
    let mut a = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    a.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "RT".into(),
        prefix: "RTR".into(),
        description: Some("d".into()),
        icon: None,
    }))
    .unwrap();
    let sid = a.query_statuses_for_project(pid).unwrap()[0].id;
    let label_id = new_id();
    a.apply(Operation::CreateLabel(CreateLabel {
        id: label_id,
        project_id: pid,
        name: "feat".into(),
        color: "#3b82f6".into(),
    }))
    .unwrap();
    a.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(),
        project_id: pid,
        title: "round-trip me".into(),
        description: Some("body".into()),
        status_id: sid,
        priority: Priority::High,
        due_date: None,
        label_ids: vec![label_id],
    }))
    .unwrap();

    let snap = a.export_snapshot().unwrap();

    let mut b = Workspace::open_in_memory().unwrap();
    b.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: snap,
        policy: ConflictPolicy::Fail,
    }))
    .unwrap();

    let snap_b = b.export_snapshot().unwrap();
    assert_eq!(snap_b.projects.len(), 1);
    assert_eq!(snap_b.projects[0].name, "RT");
    assert_eq!(snap_b.statuses.len(), 7);
    assert_eq!(snap_b.issues.len(), 1);
    assert_eq!(snap_b.labels.len(), 1);
    assert_eq!(snap_b.issue_labels.len(), 1);
}
