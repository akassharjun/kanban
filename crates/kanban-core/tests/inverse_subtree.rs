//! Regression tests for the `inverse_of_delete_*` -> `ImportSnapshot`
//! refactor. Each test exercises a specific piece of state that the previous
//! `Create*` shape silently dropped on undo.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    AttachLabel, CreateIssue, CreateLabel, CreateProject, DeleteIssue, DeleteLabel, DeleteProject,
    Operation, ProjectPatch, UpdateProject,
};
use kanban_core::query::IssueFilter;
use kanban_core::types::{Priority, ProjectStatus};
use kanban_core::{Workspace, new_id};

#[test]
fn delete_undo_restores_non_active_project_status() {
    // Before the refactor, undo of DeleteProject re-created the project as
    // Active regardless of its prior ProjectStatus, because the captured
    // inverse was a CreateProject op (which has no `status` field).
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Paused Project".into(),
        prefix: "PAU".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.apply(Operation::UpdateProject(UpdateProject {
        id,
        patch: ProjectPatch {
            status: Some(ProjectStatus::Paused),
            ..Default::default()
        },
    }))
    .unwrap();

    ws.apply(Operation::DeleteProject(DeleteProject { id }))
        .unwrap();
    assert!(ws.query_project_by_id(id).is_err());

    ws.undo().unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.status, ProjectStatus::Paused, "status must round-trip");
    assert_eq!(p.name, "Paused Project");
    assert_eq!(p.prefix, "PAU");
}

#[test]
fn delete_undo_restores_project_with_cascaded_issues_and_labels() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "PRJ".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let iid = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid,
        project_id: pid,
        title: "issue 1".into(),
        description: None,
        status_id: sid,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let lid = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: lid,
        project_id: pid,
        name: "bug".into(),
        color: "#ff0000".into(),
    }))
    .unwrap();
    ws.apply(Operation::AttachLabel(AttachLabel {
        issue_id: iid,
        label_id: lid,
    }))
    .unwrap();

    // Capture the issue's identifier so we can verify it survives the
    // delete -> undo cycle (the previous CreateIssue inverse would have
    // re-derived a fresh seq/identifier on undo).
    let original_identifier = ws.query_issue_by_id(iid).unwrap().identifier;

    ws.apply(Operation::DeleteProject(DeleteProject { id: pid }))
        .unwrap();
    assert!(ws.query_project_by_id(pid).is_err());

    ws.undo().unwrap();
    assert!(ws.query_project_by_id(pid).is_ok());

    let restored_issue = ws.query_issue_by_id(iid).unwrap();
    assert_eq!(restored_issue.identifier, original_identifier);
    assert_eq!(restored_issue.title, "issue 1");

    let labels = ws.query_labels_for_project(pid).unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "bug");

    // The issue_labels attachment must round-trip too.
    let attached_count: i64 = ws
        ._conn_for_integration_tests()
        .query_row(
            "SELECT COUNT(*) FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
            rusqlite::params![iid.to_string(), lid.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(attached_count, 1);
}

#[test]
fn delete_undo_preserves_issue_identifier_and_seq() {
    // Before the refactor, DeleteIssue + undo would re-create the issue
    // with the next available seq (and corresponding identifier), so
    // KAN-7 could come back as KAN-8.
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "SEQ".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

    // Create three issues so the deleted one is not at seq 1; this makes
    // the regression visible if seq is re-derived.
    let mut ids = Vec::new();
    for n in 0..3 {
        let id = new_id();
        ws.apply(Operation::CreateIssue(CreateIssue {
            id,
            project_id: pid,
            title: format!("t{n}"),
            description: None,
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
        ids.push(id);
    }

    let target = ids[1];
    let pre = ws.query_issue_by_id(target).unwrap();
    let original_seq = pre.seq;
    let original_identifier = pre.identifier.clone();
    let original_sort_key_bits = pre.sort_key.to_bits();

    ws.apply(Operation::DeleteIssue(DeleteIssue { id: target }))
        .unwrap();
    assert!(ws.query_issue_by_id(target).is_err());

    ws.undo().unwrap();
    let post = ws.query_issue_by_id(target).unwrap();
    assert_eq!(post.seq, original_seq);
    assert_eq!(post.identifier, original_identifier);
    assert_eq!(post.sort_key.to_bits(), original_sort_key_bits);
}

#[test]
fn delete_undo_restores_label_attachments() {
    // Before the refactor, deleting a label CASCADE'd its issue_labels
    // rows but the captured inverse only re-created the label row, so
    // attachments were lost forever.
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "ATT".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let lid = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: lid,
        project_id: pid,
        name: "backend".into(),
        color: "#00ff00".into(),
    }))
    .unwrap();

    // Attach the label to two issues to exercise the full attachment set.
    let mut issue_ids = Vec::new();
    for n in 0..2 {
        let iid = new_id();
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: iid,
            project_id: pid,
            title: format!("issue {n}"),
            description: None,
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
        ws.apply(Operation::AttachLabel(AttachLabel {
            issue_id: iid,
            label_id: lid,
        }))
        .unwrap();
        issue_ids.push(iid);
    }

    let pre_count = label_attachment_count(&ws, lid);
    assert_eq!(pre_count, 2);

    ws.apply(Operation::DeleteLabel(DeleteLabel { id: lid }))
        .unwrap();
    assert_eq!(label_attachment_count(&ws, lid), 0);

    ws.undo().unwrap();
    assert_eq!(
        label_attachment_count(&ws, lid),
        2,
        "both attachments must be restored"
    );

    // Sanity-check the label itself is back with the same fields.
    let restored = ws
        .query_labels_for_project(pid)
        .unwrap()
        .into_iter()
        .find(|l| l.id == lid)
        .expect("label must be restored");
    assert_eq!(restored.name, "backend");
    assert_eq!(restored.color, "#00ff00");
}

#[test]
fn reorder_undo_round_trips_arbitrary_f64_via_bit_pattern() {
    // Regression for the f64 precision drift on the Operation payload:
    // the value 2.2901265025181274 used to lose 1 ULP when the inverse
    // was deserialized from operation_log.payload at undo time.
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "REO".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let iid = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid,
        project_id: pid,
        title: "t".into(),
        description: None,
        status_id: sid,
        priority: Priority::None,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let original = ws.query_issue_by_id(iid).unwrap().sort_key;
    let new_key = 2.290_126_502_518_127_4_f64;
    ws.apply(Operation::ReorderIssue(
        kanban_core::operation::ReorderIssue {
            id: iid,
            new_sort_key: new_key,
        },
    ))
    .unwrap();
    assert_eq!(
        ws.query_issue_by_id(iid).unwrap().sort_key.to_bits(),
        new_key.to_bits(),
    );

    ws.undo().unwrap();
    let after_undo = ws.query_issue_by_id(iid).unwrap().sort_key;
    assert_eq!(after_undo.to_bits(), original.to_bits());

    ws.redo().unwrap();
    let after_redo = ws.query_issue_by_id(iid).unwrap().sort_key;
    assert_eq!(after_redo.to_bits(), new_key.to_bits());
}

fn label_attachment_count(ws: &Workspace, label_id: uuid::Uuid) -> i64 {
    ws._conn_for_integration_tests()
        .query_row(
            "SELECT COUNT(*) FROM issue_labels WHERE label_id = ?1",
            rusqlite::params![label_id.to_string()],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn delete_project_undo_restores_query_listing() {
    // End-to-end check that querying the issues for a project after
    // DeleteProject + undo returns exactly what existed before the delete.
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "P".into(),
        prefix: "LST".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let mut titles_before = Vec::new();
    for n in 0..4 {
        let iid = new_id();
        let title = format!("issue {n}");
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: iid,
            project_id: pid,
            title: title.clone(),
            description: None,
            status_id: sid,
            priority: Priority::None,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
        titles_before.push(title);
    }

    let mut listing_before: Vec<String> = ws
        .query_issues(IssueFilter::for_project(pid))
        .unwrap()
        .into_iter()
        .map(|i| i.identifier)
        .collect();
    listing_before.sort();

    ws.apply(Operation::DeleteProject(DeleteProject { id: pid }))
        .unwrap();
    ws.undo().unwrap();

    let mut listing_after: Vec<String> = ws
        .query_issues(IssueFilter::for_project(pid))
        .unwrap()
        .into_iter()
        .map(|i| i.identifier)
        .collect();
    listing_after.sort();

    assert_eq!(listing_before, listing_after);
}
