#![allow(clippy::unwrap_used)]
use kanban_core::Workspace;
use kanban_core::operation::{CreateProject, Operation};
use kanban_tauri::commands::{apply_inner, list_projects_inner, redo_inner, undo_inner};
use uuid::Uuid;

fn create_auth_project() -> Operation {
    Operation::CreateProject(CreateProject {
        id: Uuid::now_v7(),
        name: "Auth Service".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    })
}

#[test]
fn apply_create_project_then_list_returns_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap(); // &Path; mut because apply needs &mut

    apply_inner(&mut ws, create_auth_project()).unwrap();

    let projects = list_projects_inner(&ws).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].prefix, "AUTH");
}

#[test]
fn apply_then_undo_then_redo_round_trips_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();

    apply_inner(&mut ws, create_auth_project()).unwrap();
    assert_eq!(list_projects_inner(&ws).unwrap().len(), 1);

    undo_inner(&mut ws).unwrap();
    assert!(
        list_projects_inner(&ws).unwrap().is_empty(),
        "undo should remove the created project"
    );

    redo_inner(&mut ws).unwrap();
    assert_eq!(
        list_projects_inner(&ws).unwrap().len(),
        1,
        "redo should restore the project"
    );
}
