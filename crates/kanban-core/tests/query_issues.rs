#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use chrono::NaiveDate;
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::query::{IssueFilter, SortBy};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

fn seeded() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "Q".into(),
        prefix: "QRY".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let statuses = ws.query_statuses_for_project(pid).unwrap();
    let todo = statuses[0].id;
    let inprog = statuses[2].id;
    for (title, st, pr, due) in [
        ("a-todo-high", todo, Priority::High, Some("2026-06-01")),
        ("b-todo-low", todo, Priority::Low, None),
        (
            "c-prog-medium",
            inprog,
            Priority::Medium,
            Some("2026-05-15"),
        ),
    ] {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(),
            project_id: pid,
            title: title.into(),
            description: None,
            status_id: st,
            priority: pr,
            due_date: due.map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap()),
            label_ids: vec![],
        }))
        .unwrap();
    }
    (ws, pid)
}

#[test]
fn filter_by_status() {
    let (ws, pid) = seeded();
    let todo = ws.query_statuses_for_project(pid).unwrap()[0].id;
    let issues = ws
        .query_issues(IssueFilter {
            project_id: Some(pid),
            status_ids: vec![todo],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(issues.len(), 2);
}

#[test]
fn filter_by_priority() {
    let (ws, pid) = seeded();
    let issues = ws
        .query_issues(IssueFilter {
            project_id: Some(pid),
            priorities: vec![Priority::High, Priority::Medium],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(issues.len(), 2);
}

#[test]
fn filter_due_before() {
    let (ws, pid) = seeded();
    let cutoff = NaiveDate::parse_from_str("2026-05-20", "%Y-%m-%d").unwrap();
    let issues = ws
        .query_issues(IssueFilter {
            project_id: Some(pid),
            due_before: Some(cutoff),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "c-prog-medium");
}

#[test]
fn sort_by_priority() {
    let (ws, pid) = seeded();
    let issues = ws
        .query_issues(IssueFilter {
            project_id: Some(pid),
            sort: SortBy::Priority,
            ..Default::default()
        })
        .unwrap();
    let titles: Vec<_> = issues.iter().map(|i| i.title.clone()).collect();
    assert_eq!(
        titles,
        vec![
            "a-todo-high".to_string(),
            "c-prog-medium".into(),
            "b-todo-low".into(),
        ]
    );
}

#[test]
fn limit_caps_results() {
    let (ws, pid) = seeded();
    let issues = ws
        .query_issues(IssueFilter {
            project_id: Some(pid),
            limit: Some(2),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(issues.len(), 2);
}
