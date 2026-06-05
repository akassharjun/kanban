#![allow(clippy::unwrap_used)]
use kanban_core::Workspace;
use kanban_core::operation::{AttachLabel, CreateIssue, CreateLabel, CreateProject, Operation};
use kanban_core::types::Priority;
use uuid::Uuid;

#[test]
fn labels_for_issue_returns_attached_sorted_and_empty_when_none() {
    let mut ws = Workspace::open_in_memory().unwrap();

    let pid = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let status_id = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let iid = Uuid::now_v7();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid,
        project_id: pid,
        title: "Add OAuth".into(),
        description: None,
        status_id,
        priority: Priority::Medium,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    // Issue with no labels returns an empty vec.
    assert!(ws.query_labels_for_issue(iid).unwrap().is_empty());

    let label_b = Uuid::now_v7();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: label_b,
        project_id: pid,
        name: "bug".into(),
        color: "#ff0000".into(),
    }))
    .unwrap();
    let label_a = Uuid::now_v7();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: label_a,
        project_id: pid,
        name: "api".into(),
        color: "#00ff00".into(),
    }))
    .unwrap();

    ws.apply(Operation::AttachLabel(AttachLabel {
        issue_id: iid,
        label_id: label_b,
    }))
    .unwrap();
    ws.apply(Operation::AttachLabel(AttachLabel {
        issue_id: iid,
        label_id: label_a,
    }))
    .unwrap();

    let labels = ws.query_labels_for_issue(iid).unwrap();
    assert_eq!(labels.len(), 2);
    // Ordered by name.
    let names: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, vec!["api", "bug"]);
}
