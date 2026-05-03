#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{CreateProject, Operation};
use kanban_core::types::ProjectStatus;
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

#[test]
fn query_projects_returns_all_in_creation_order() {
    let mut ws = Workspace::open_in_memory().unwrap();
    for prefix in ["AAA", "BBB", "CCC"] {
        ws.apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: prefix.into(),
            prefix: prefix.into(),
            description: None,
            icon: None,
        }))
        .unwrap();
    }
    let projects = ws.query_projects().unwrap();
    let prefixes: Vec<_> = projects.iter().map(|p| p.prefix.clone()).collect();
    assert_eq!(prefixes, vec!["AAA", "BBB", "CCC"]);
}

#[test]
fn query_project_by_id_returns_not_found_for_missing() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.query_project_by_id(new_id()).unwrap_err();
    assert!(err.to_string().contains("Project not found"), "{err}");
}

#[test]
fn newly_created_project_has_active_status() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "XYZ".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.status, ProjectStatus::Active);
}

use kanban_core::operation::{ProjectPatch, UpdateProject};

#[test]
fn update_project_changes_name_and_updates_timestamp() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Old".into(),
        prefix: "UPD".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let before = ws.query_project_by_id(id).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
    ws.apply(Operation::UpdateProject(UpdateProject {
        id,
        patch: ProjectPatch {
            name: Some("New".into()),
            ..Default::default()
        },
    }))
    .unwrap();

    let after = ws.query_project_by_id(id).unwrap();
    assert_eq!(after.name, "New");
    assert!(after.updated_at >= before.updated_at);
}

#[test]
fn update_project_clears_description_with_some_none() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "DSC".into(),
        description: Some("hi".into()),
        icon: None,
    }))
    .unwrap();
    ws.apply(Operation::UpdateProject(UpdateProject {
        id,
        patch: ProjectPatch {
            description: Some(None),
            ..Default::default()
        },
    }))
    .unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert!(p.description.is_none());
}

#[test]
fn update_project_unknown_id_returns_not_found() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::UpdateProject(UpdateProject {
            id: new_id(),
            patch: ProjectPatch {
                name: Some("nope".into()),
                ..Default::default()
            },
        }))
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

use kanban_core::operation::ArchiveProject;

#[test]
fn archive_project_sets_status_archived() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "ARC".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.apply(Operation::ArchiveProject(ArchiveProject { id }))
        .unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.status, ProjectStatus::Archived);
}

#[test]
fn archive_project_unknown_id_errors() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::ArchiveProject(ArchiveProject { id: new_id() }))
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

use kanban_core::operation::DeleteProject;

#[test]
fn delete_project_removes_row_and_cascades_statuses() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "DEL".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    ws.apply(Operation::DeleteProject(DeleteProject { id }))
        .unwrap();

    assert!(ws.query_project_by_id(id).is_err());
    let s = ws.query_statuses_for_project(id).unwrap();
    assert!(s.is_empty(), "statuses should cascade: got {s:?}");
}

#[test]
fn delete_project_unknown_id_errors() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::DeleteProject(DeleteProject { id: new_id() }))
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

#[test]
fn inverse_of_create_project_is_delete_with_same_id() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    let op = Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "INV".into(),
        description: None,
        icon: None,
    });
    ws.apply(op.clone()).unwrap();
    let inv: Operation = ws.last_inverse().unwrap();
    match inv {
        Operation::DeleteProject(d) => assert_eq!(d.id, id),
        other => panic!("expected DeleteProject, got {other:?}"),
    }
}

#[test]
fn inverse_of_delete_project_restores_full_record() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Restore Me".into(),
        prefix: "RES".into(),
        description: Some("d".into()),
        icon: Some("\u{1f680}".into()),
    }))
    .unwrap();
    ws.apply(Operation::DeleteProject(DeleteProject { id }))
        .unwrap();
    // The inverse of DeleteProject is now an ImportSnapshot of the project
    // subtree (so cascaded statuses/labels/issues are preserved through
    // undo). The captured snapshot must include the project row itself.
    let inv = ws.last_inverse().unwrap();
    match inv {
        Operation::ImportSnapshot(snap) => {
            assert_eq!(snap.snapshot.projects.len(), 1);
            let p = &snap.snapshot.projects[0];
            assert_eq!(p.id, id);
            assert_eq!(p.name, "Restore Me");
            assert_eq!(p.prefix, "RES");
            assert_eq!(p.description.as_deref(), Some("d"));
            // Default statuses are seeded on Create and must be in the inverse.
            assert!(!snap.snapshot.statuses.is_empty());
        }
        other => panic!("expected ImportSnapshot, got {other:?}"),
    }
}
