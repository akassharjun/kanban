#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::query::IssueFilter;
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

fn seeded() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "S".into(),
        prefix: "SRC".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    for (t, d) in [
        ("Add OAuth login", Some("user can authenticate via OAuth")),
        ("Fix board crash", Some("crash when dragging cards")),
        ("Document API", None),
    ] {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(),
            project_id: pid,
            title: t.into(),
            description: d.map(str::to_string),
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
    }
    (ws, pid)
}

#[test]
fn search_by_title_returns_match() {
    let (ws, pid) = seeded();
    let hits = ws.search("OAuth", IssueFilter::for_project(pid)).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Add OAuth login");
}

#[test]
fn search_by_description() {
    let (ws, pid) = seeded();
    let hits = ws
        .search("dragging", IssueFilter::for_project(pid))
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Fix board crash");
}

#[test]
fn search_combines_with_filter() {
    let (ws, pid) = seeded();
    let hits = ws
        .search(
            "API",
            IssueFilter {
                project_id: Some(pid),
                priorities: vec![Priority::Urgent],
                ..Default::default()
            },
        )
        .unwrap();
    assert!(hits.is_empty(), "no urgent issues match 'API'");
}

#[test]
fn search_after_delete_drops_match() {
    let (mut ws, pid) = seeded();
    let issues = ws.query_issues(IssueFilter::for_project(pid)).unwrap();
    let oauth = issues.iter().find(|i| i.title.contains("OAuth")).unwrap();
    ws.apply(Operation::DeleteIssue(
        kanban_core::operation::DeleteIssue { id: oauth.id },
    ))
    .unwrap();
    let hits = ws.search("OAuth", IssueFilter::for_project(pid)).unwrap();
    assert!(hits.is_empty());
}
