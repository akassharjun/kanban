#![allow(clippy::unwrap_used)]
use kanban_core::Workspace;
use kanban_core::operation::{CreateProject, Operation};
use uuid::Uuid;

#[test]
fn project_by_prefix_hit_and_miss() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Auth Service".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let hit = ws.query_project_by_prefix("AUTH").unwrap();
    assert_eq!(hit.map(|p| p.id), Some(id));
    assert!(ws.query_project_by_prefix("NOPE").unwrap().is_none());
}

#[test]
fn issue_by_identifier_hit_and_miss() {
    use kanban_core::operation::CreateIssue;
    use kanban_core::types::Priority;

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

    let issue = ws.query_issue_by_identifier("AUTH-1").unwrap();
    assert_eq!(issue.map(|i| i.id), Some(iid));
    assert!(ws.query_issue_by_identifier("AUTH-999").unwrap().is_none());
}
