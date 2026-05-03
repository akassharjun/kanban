#![allow(clippy::unwrap_used)]

use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn export_snapshot_contains_all_entities() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "X".into(),
        prefix: "EXP".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(),
        project_id: pid,
        title: "t".into(),
        description: None,
        status_id: sid,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();
    let snap = ws.export_snapshot().unwrap();
    assert_eq!(snap.schema_version, 1);
    assert_eq!(snap.projects.len(), 1);
    assert_eq!(snap.statuses.len(), 7);
    assert_eq!(snap.issues.len(), 1);
}
