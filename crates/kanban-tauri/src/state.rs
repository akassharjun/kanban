use std::sync::Mutex;

use kanban_core::Workspace;

/// Shared Tauri state: the single `Workspace`, guarded by a `Mutex`.
///
/// `Workspace` is `Send` but not `Sync`; the `Mutex` provides the `Sync`
/// bound that Tauri's managed-state requires, and serialises DB access.
pub struct AppState {
    #[allow(dead_code)] // consumed by command handlers added in later tasks
    pub workspace: Mutex<Workspace>,
}

impl AppState {
    /// Open the default workspace (`~/.kanban/data.db` or `$KANBAN_DB`) and wrap it.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be opened or migrations fail.
    pub fn from_default() -> Result<Self, kanban_core::Error> {
        let workspace = Workspace::open_default()?;
        Ok(Self {
            workspace: Mutex::new(workspace),
        })
    }
}
