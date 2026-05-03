#![allow(clippy::unwrap_used)]

use kanban_core::Workspace;
use kanban_core::operation::{ConflictPolicy, ImportSnapshot, Operation};

#[test]
fn import_into_empty_db_writes_all_entities() {
    let mut donor = Workspace::open_in_memory().unwrap();
    // seed donor with one project + one issue
    let pid = kanban_core::new_id();
    donor
        .apply(Operation::CreateProject(
            kanban_core::operation::CreateProject {
                id: pid,
                name: "P".into(),
                prefix: "IMP".into(),
                description: None,
                icon: None,
            },
        ))
        .unwrap();
    let sid = donor.query_statuses_for_project(pid).unwrap()[0].id;
    let iid = kanban_core::new_id();
    donor
        .apply(Operation::CreateIssue(
            kanban_core::operation::CreateIssue {
                id: iid,
                project_id: pid,
                title: "x".into(),
                description: None,
                status_id: sid,
                priority: kanban_core::types::Priority::None,
                due_date: None,
                label_ids: vec![],
            },
        ))
        .unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target
        .apply(Operation::ImportSnapshot(ImportSnapshot {
            snapshot: snap,
            policy: ConflictPolicy::Fail,
        }))
        .unwrap();

    assert_eq!(target.query_projects().unwrap().len(), 1);
    assert_eq!(target.query_issue_by_id(iid).unwrap().title, "x");
}

#[test]
fn import_with_id_collision_fails_under_fail_policy() {
    // create same project in target first, then try to import a snapshot containing it
    let mut donor = Workspace::open_in_memory().unwrap();
    let pid = kanban_core::new_id();
    donor
        .apply(Operation::CreateProject(
            kanban_core::operation::CreateProject {
                id: pid,
                name: "Donor".into(),
                prefix: "DDD".into(),
                description: None,
                icon: None,
            },
        ))
        .unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target
        .apply(Operation::CreateProject(
            kanban_core::operation::CreateProject {
                id: pid,
                name: "Same id different name".into(),
                prefix: "TGT".into(),
                description: None,
                icon: None,
            },
        ))
        .unwrap();
    let err = target
        .apply(Operation::ImportSnapshot(ImportSnapshot {
            snapshot: snap,
            policy: ConflictPolicy::Fail,
        }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn import_skip_policy_keeps_existing_rows() {
    let mut donor = Workspace::open_in_memory().unwrap();
    let pid = kanban_core::new_id();
    donor
        .apply(Operation::CreateProject(
            kanban_core::operation::CreateProject {
                id: pid,
                name: "From Donor".into(),
                prefix: "DON".into(),
                description: None,
                icon: None,
            },
        ))
        .unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target
        .apply(Operation::CreateProject(
            kanban_core::operation::CreateProject {
                id: pid,
                name: "Original".into(),
                prefix: "ORG".into(),
                description: None,
                icon: None,
            },
        ))
        .unwrap();
    target
        .apply(Operation::ImportSnapshot(ImportSnapshot {
            snapshot: snap,
            policy: ConflictPolicy::Skip,
        }))
        .unwrap();
    let p = target.query_project_by_id(pid).unwrap();
    assert_eq!(p.name, "Original", "skip should keep existing row");
}
