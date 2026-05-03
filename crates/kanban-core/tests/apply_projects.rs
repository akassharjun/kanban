#![allow(clippy::unwrap_used)]

use kanban_core::operation::{CreateProject, Operation};
use kanban_core::{Workspace, new_id};

#[test]
fn create_project_inserts_row_and_seeds_default_statuses() {
    let mut ws = Workspace::open_in_memory().unwrap();

    let id = new_id();
    let outcome = ws
        .apply(Operation::CreateProject(CreateProject {
            id,
            name: "Auth Service".into(),
            prefix: "AUTH".into(),
            description: Some("oauth flows".into()),
            icon: None,
        }))
        .unwrap();
    assert!(outcome.op_id > 0);

    let project = ws.query_project_by_id(id).unwrap();
    assert_eq!(project.name, "Auth Service");
    assert_eq!(project.prefix, "AUTH");
    assert_eq!(project.next_seq, 1);

    let statuses = ws.query_statuses_for_project(id).unwrap();
    let names: Vec<_> = statuses.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Todo",
            "Backlog",
            "In Progress",
            "In Review",
            "Blocked",
            "Discarded",
            "Done"
        ]
    );
}

#[test]
fn create_project_rejects_duplicate_prefix() {
    let mut ws = Workspace::open_in_memory().unwrap();
    ws.apply(Operation::CreateProject(CreateProject {
        id: new_id(),
        name: "A".into(),
        prefix: "DUP".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let err = ws
        .apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: "B".into(),
            prefix: "DUP".into(),
            description: None,
            icon: None,
        }))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("conflict"), "got: {msg}");
}

#[test]
fn create_project_rejects_invalid_prefix() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: "A".into(),
            prefix: "lower".into(),
            description: None,
            icon: None,
        }))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("uppercase"), "got: {msg}");
}
