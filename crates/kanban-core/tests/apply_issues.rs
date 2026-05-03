#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    CreateIssue, CreateProject, IssueFieldChange, Operation, UpdateIssueField,
};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

fn fresh_with_project() -> (Workspace, Uuid, Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let project_id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: project_id,
        name: "T".into(),
        prefix: "TST".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let todo_id = ws.query_statuses_for_project(project_id).unwrap()[0].id;
    (ws, project_id, todo_id)
}

#[test]
fn create_issue_assigns_seq_1_and_identifier_with_prefix() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: pid,
        title: "first".into(),
        description: None,
        status_id: sid,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();
    let issue = ws.query_issue_by_id(id).unwrap();
    assert_eq!(issue.seq, 1);
    assert_eq!(issue.identifier, "TST-1");
}

#[test]
fn create_issue_increments_seq_per_project() {
    let (mut ws, pid, sid) = fresh_with_project();
    for _ in 0..3 {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(),
            project_id: pid,
            title: "x".into(),
            description: None,
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
    }
    let p = ws.query_project_by_id(pid).unwrap();
    assert_eq!(p.next_seq, 4);
    let issues = ws
        .query_issues(kanban_core::query::IssueFilter::for_project(pid))
        .unwrap();
    let identifiers: Vec<_> = issues.iter().map(|i| i.identifier.clone()).collect();
    assert_eq!(
        identifiers,
        vec!["TST-1".to_string(), "TST-2".into(), "TST-3".into()]
    );
}

#[test]
fn create_issue_attaches_labels_in_one_op() {
    let (ws, pid, sid) = fresh_with_project();
    // Insert a label directly via apply once labels land — but for now we seed via the writer.
    // This test is enabled in Task 25 once AttachLabel is wired through apply.
    let _ = (ws, pid, sid);
}

#[test]
fn create_issue_validates_title_nonempty() {
    let (mut ws, pid, sid) = fresh_with_project();
    let err = ws
        .apply(Operation::CreateIssue(CreateIssue {
            id: new_id(),
            project_id: pid,
            title: "   ".into(),
            description: None,
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap_err();
    assert!(err.to_string().contains("title"), "{err}");
}

#[test]
fn create_issue_unknown_project_errors() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::CreateIssue(CreateIssue {
            id: new_id(),
            project_id: new_id(),
            title: "x".into(),
            description: None,
            status_id: new_id(),
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("not found"),
        "{err}"
    );
}

#[test]
fn concurrent_creates_do_not_collide() {
    // Two workspaces against the same file simulate concurrent CLI invocations.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");

    let project_id = new_id();
    let mut bootstrap = Workspace::open(&path).unwrap();
    bootstrap
        .apply(Operation::CreateProject(CreateProject {
            id: project_id,
            name: "C".into(),
            prefix: "CON".into(),
            description: None,
            icon: None,
        }))
        .unwrap();
    let sid = bootstrap.query_statuses_for_project(project_id).unwrap()[0].id;
    drop(bootstrap);

    let path = Arc::new(path);
    let sid = Arc::new(Mutex::new(sid));
    let mut handles = vec![];
    for _ in 0..8 {
        let path = Arc::clone(&path);
        let sid = *sid.lock().unwrap();
        handles.push(std::thread::spawn(move || {
            let mut ws = Workspace::open(&path).unwrap();
            ws.apply(Operation::CreateIssue(CreateIssue {
                id: new_id(),
                project_id,
                title: "race".into(),
                description: None,
                status_id: sid,
                priority: Priority::None,
                due_date: None,
                label_ids: vec![],
            }))
            .unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let ws = Workspace::open(&path).unwrap();
    let issues = ws
        .query_issues(kanban_core::query::IssueFilter::for_project(project_id))
        .unwrap();
    assert_eq!(issues.len(), 8);
    let mut seqs: Vec<_> = issues.iter().map(|i| i.seq).collect();
    seqs.sort_unstable();
    assert_eq!(seqs, (1..=8).collect::<Vec<_>>());
}

#[test]
fn update_issue_title() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: pid,
        title: "old".into(),
        description: None,
        status_id: sid,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id,
        change: IssueFieldChange::Title("new".into()),
    }))
    .unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().title, "new");
}

#[test]
fn update_issue_priority_and_undo_restores_it() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: pid,
        title: "p".into(),
        description: None,
        status_id: sid,
        priority: Priority::High,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id,
        change: IssueFieldChange::Priority(Priority::Low),
    }))
    .unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().priority, Priority::Low);
    ws.undo().unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().priority, Priority::High);
}

#[test]
fn update_issue_status_to_status_in_other_project_errors() {
    let (mut ws, pid_a, sid_a) = fresh_with_project();
    // create a second project
    let pid_b = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid_b,
        name: "B".into(),
        prefix: "BTW".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid_b = ws.query_statuses_for_project(pid_b).unwrap()[0].id;

    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: pid_a,
        title: "x".into(),
        description: None,
        status_id: sid_a,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let err = ws
        .apply(Operation::UpdateIssueField(UpdateIssueField {
            id,
            change: IssueFieldChange::Status(sid_b),
        }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("status"), "{err}");
}
