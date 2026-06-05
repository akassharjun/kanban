//! Read commands over `kanban-core`.
//!
//! Each command is split into a synchronous, unit-testable `*_inner` function
//! that borrows a `&Workspace`, and an async `#[tauri::command]` wrapper that
//! clones the shared `Arc<Mutex<Workspace>>` handle and runs the inner function
//! on a blocking thread via `tokio::task::spawn_blocking`. The mutex guard never
//! crosses an `.await`: it lives entirely inside the blocking closure.
//!
//! The async wrappers are registered with Tauri's invoke handler in Task 10.

use kanban_core::Workspace;
use kanban_core::query::IssueFilter;
use kanban_core::types::Project;

use crate::dto::{IssueDto, LabelDto, ProjectDto, StatusDto};
use crate::error::ApiError;
use crate::state::AppState;

/// Resolve a project by its `prefix`, returning `None` if no project matches.
fn project_by_prefix(ws: &Workspace, prefix: &str) -> Result<Option<Project>, ApiError> {
    let projects = ws.query_projects()?;
    Ok(projects.into_iter().find(|p| p.prefix == prefix))
}

/// List all projects.
///
/// # Errors
///
/// Returns an error if the underlying query fails.
pub fn list_projects_inner(ws: &Workspace) -> Result<Vec<ProjectDto>, ApiError> {
    Ok(ws
        .query_projects()?
        .into_iter()
        .map(ProjectDto::from)
        .collect())
}

/// Get a single project by its `prefix`.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if no project has the given prefix, or an error
/// if the underlying query fails.
pub fn get_project_inner(ws: &Workspace, prefix: &str) -> Result<ProjectDto, ApiError> {
    match project_by_prefix(ws, prefix)? {
        Some(p) => Ok(ProjectDto::from(p)),
        None => Err(ApiError::NotFound {
            resource: "project".into(),
            key: prefix.into(),
        }),
    }
}

/// List the issues of the project identified by `prefix`.
///
/// An unknown prefix yields an empty list (not an error).
///
/// # Errors
///
/// Returns an error if the underlying query fails.
pub fn list_issues_inner(ws: &Workspace, prefix: &str) -> Result<Vec<IssueDto>, ApiError> {
    let Some(project) = project_by_prefix(ws, prefix)? else {
        return Ok(Vec::new());
    };
    Ok(ws
        .query_issues(IssueFilter::for_project(project.id))?
        .into_iter()
        .map(IssueDto::from)
        .collect())
}

/// Get a single issue by its `identifier` (e.g. `AUTH-12`).
///
/// # Errors
///
/// Returns `ApiError::NotFound` if no issue has the given identifier, or an
/// error if the underlying query fails.
pub fn get_issue_inner(ws: &Workspace, key: &str) -> Result<IssueDto, ApiError> {
    match ws
        .query_issues(IssueFilter::default())?
        .into_iter()
        .find(|i| i.identifier == key)
    {
        Some(i) => Ok(IssueDto::from(i)),
        None => Err(ApiError::NotFound {
            resource: "issue".into(),
            key: key.into(),
        }),
    }
}

/// List the statuses of the project identified by `prefix`.
///
/// An unknown prefix yields an empty list (not an error).
///
/// # Errors
///
/// Returns an error if the underlying query fails.
pub fn list_statuses_inner(ws: &Workspace, prefix: &str) -> Result<Vec<StatusDto>, ApiError> {
    let Some(project) = project_by_prefix(ws, prefix)? else {
        return Ok(Vec::new());
    };
    Ok(ws
        .query_statuses_for_project(project.id)?
        .into_iter()
        .map(StatusDto::from)
        .collect())
}

/// List the labels of the project identified by `prefix`.
///
/// An unknown prefix yields an empty list (not an error).
///
/// # Errors
///
/// Returns an error if the underlying query fails.
pub fn list_labels_inner(ws: &Workspace, prefix: &str) -> Result<Vec<LabelDto>, ApiError> {
    let Some(project) = project_by_prefix(ws, prefix)? else {
        return Ok(Vec::new());
    };
    Ok(ws
        .query_labels_for_project(project.id)?
        .into_iter()
        .map(LabelDto::from)
        .collect())
}

/// Map a `spawn_blocking` join error to an internal `ApiError`.
fn join_error(e: &tokio::task::JoinError) -> ApiError {
    ApiError::Internal {
        message: e.to_string(),
    }
}

/// Lock the shared workspace, mapping a poisoned mutex to an internal error.
fn lock_workspace(
    ws: &std::sync::Mutex<Workspace>,
) -> Result<std::sync::MutexGuard<'_, Workspace>, ApiError> {
    ws.lock().map_err(|_| ApiError::Internal {
        message: "workspace mutex poisoned".into(),
    })
}

/// Tauri command: list all projects.
///
/// # Errors
///
/// Returns an error if the workspace mutex is poisoned, the blocking task fails
/// to join, or the underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: tauri::State<'_, AppState>) -> Result<Vec<ProjectDto>, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        list_projects_inner(&guard)
    })
    .await
    .map_err(|e| join_error(&e))?
}

/// Tauri command: get a single project by `prefix`.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if no project has the given prefix, or an error
/// if the workspace mutex is poisoned, the blocking task fails to join, or the
/// underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn get_project(
    state: tauri::State<'_, AppState>,
    prefix: String,
) -> Result<ProjectDto, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        get_project_inner(&guard, &prefix)
    })
    .await
    .map_err(|e| join_error(&e))?
}

/// Tauri command: list the issues of the project identified by `project` (its prefix).
///
/// # Errors
///
/// Returns an error if the workspace mutex is poisoned, the blocking task fails
/// to join, or the underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn list_issues(
    state: tauri::State<'_, AppState>,
    project: String,
) -> Result<Vec<IssueDto>, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        list_issues_inner(&guard, &project)
    })
    .await
    .map_err(|e| join_error(&e))?
}

/// Tauri command: get a single issue by its `key` (identifier, e.g. `AUTH-12`).
///
/// # Errors
///
/// Returns `ApiError::NotFound` if no issue has the given identifier, or an
/// error if the workspace mutex is poisoned, the blocking task fails to join, or
/// the underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn get_issue(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<IssueDto, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        get_issue_inner(&guard, &key)
    })
    .await
    .map_err(|e| join_error(&e))?
}

/// Tauri command: list the statuses of the project identified by `project` (its prefix).
///
/// # Errors
///
/// Returns an error if the workspace mutex is poisoned, the blocking task fails
/// to join, or the underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn list_statuses(
    state: tauri::State<'_, AppState>,
    project: String,
) -> Result<Vec<StatusDto>, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        list_statuses_inner(&guard, &project)
    })
    .await
    .map_err(|e| join_error(&e))?
}

/// Tauri command: list the labels of the project identified by `project` (its prefix).
///
/// # Errors
///
/// Returns an error if the workspace mutex is poisoned, the blocking task fails
/// to join, or the underlying query fails.
#[tauri::command]
#[specta::specta]
pub async fn list_labels(
    state: tauri::State<'_, AppState>,
    project: String,
) -> Result<Vec<LabelDto>, ApiError> {
    let ws = std::sync::Arc::clone(&state.workspace);
    tokio::task::spawn_blocking(move || {
        let guard = lock_workspace(&ws)?;
        list_labels_inner(&guard, &project)
    })
    .await
    .map_err(|e| join_error(&e))?
}
