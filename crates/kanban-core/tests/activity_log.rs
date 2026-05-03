#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    CreateIssue, CreateProject, IssueFieldChange, Operation, UpdateIssueField,
};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn activity_log_records_priority_change() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "X".into(),
        prefix: "ACT".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id,
        project_id: pid,
        title: "t".into(),
        description: None,
        status_id: sid,
        priority: Priority::Low,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id,
        change: IssueFieldChange::Priority(Priority::High),
    }))
    .unwrap();

    let entries = ws.query_issue_history(id).unwrap();
    let priority_changes: Vec<_> = entries.iter().filter(|e| e.field == "priority").collect();
    assert_eq!(priority_changes.len(), 1);
    assert_eq!(priority_changes[0].old_value.as_deref(), Some("low"));
    assert_eq!(priority_changes[0].new_value.as_deref(), Some("high"));
}
