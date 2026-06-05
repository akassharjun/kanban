#![allow(clippy::unwrap_used)]

use kanban_core::Workspace;
use kanban_tauri::commands::{list_issues_inner, list_projects_inner};

#[test]
fn list_projects_returns_empty_on_fresh_db() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let projects = list_projects_inner(&ws).unwrap();
    assert!(projects.is_empty());
}

#[test]
fn list_issues_returns_empty_for_unknown_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let issues = list_issues_inner(&ws, "AUTH").unwrap();
    assert!(issues.is_empty());
}
