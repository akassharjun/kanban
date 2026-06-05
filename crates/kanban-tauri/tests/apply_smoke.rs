#![allow(clippy::unwrap_used)]
use kanban_core::Workspace;
use kanban_core::operation::{CreateProject, Operation};
use kanban_tauri::commands::{apply_inner, list_projects_inner};
use uuid::Uuid;

#[test]
fn apply_create_project_then_list_returns_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap(); // &Path; mut because apply needs &mut

    let op = Operation::CreateProject(CreateProject {
        id: Uuid::now_v7(),
        name: "Auth Service".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    });
    apply_inner(&mut ws, op).unwrap();

    let projects = list_projects_inner(&ws).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].prefix, "AUTH");
}
