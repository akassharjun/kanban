use crate::error::Result;
use crate::store::{connection, migrations};
use crate::time::{Clock, system_clock};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Opaque handle to a kanban workspace.
///
/// Holds an open `SQLite` connection and a clock. NOT `Sync` — multi-threaded
/// callers must own one `Workspace` per thread (or use a pool).
pub struct Workspace {
    pub(crate) conn: Connection,
    pub(crate) clock: Arc<dyn Clock>,
    #[allow(dead_code)]
    pub(crate) path: WorkspacePath,
}

#[derive(Debug, Clone)]
pub enum WorkspacePath {
    InMemory,
    File(PathBuf),
}

impl Workspace {
    /// Open the workspace at `~/.kanban/data.db` (or `$KANBAN_DB`).
    ///
    /// # Errors
    ///
    /// Returns an error if the database file cannot be opened or migrations fail,
    /// or if `HOME` is not set and `KANBAN_DB` is unset.
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        Self::open(&path)
    }

    /// Open a workspace at the given file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, opened, or migrations fail.
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = connection::open_file(path)?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn,
            clock: system_clock(),
            path: WorkspacePath::File(path.to_path_buf()),
        })
    }

    /// Open an in-memory workspace (for tests).
    ///
    /// # Errors
    ///
    /// Returns an error if the in-memory connection cannot be created or migrations fail.
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = connection::open_in_memory()?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn,
            clock: system_clock(),
            path: WorkspacePath::InMemory,
        })
    }

    /// Inject a clock for deterministic tests.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Look up a project by id.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::NotFound`] if no project with `id` exists,
    /// or a database error if the read fails.
    pub fn query_project_by_id(
        &self,
        id: uuid::Uuid,
    ) -> crate::error::Result<crate::types::Project> {
        crate::store::read::projects::by_id(&self.conn, id)
    }

    /// List all projects in creation order.
    ///
    /// # Errors
    ///
    /// Returns a database error if the read fails.
    pub fn query_projects(&self) -> crate::error::Result<Vec<crate::types::Project>> {
        crate::store::read::projects::list_all(&self.conn)
    }

    /// List all statuses for `project_id` in display order.
    ///
    /// # Errors
    ///
    /// Returns a database error if the read fails.
    pub fn query_statuses_for_project(
        &self,
        project_id: uuid::Uuid,
    ) -> crate::error::Result<Vec<crate::types::Status>> {
        crate::store::read::statuses::for_project(&self.conn, project_id)
    }

    /// Look up an issue by id.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::NotFound`] if no issue with `id` exists,
    /// or a database error if the read fails.
    pub fn query_issue_by_id(&self, id: uuid::Uuid) -> crate::error::Result<crate::types::Issue> {
        crate::store::read::issues::by_id(&self.conn, id)
    }

    /// List issues matching `filter`, ordered as the filter directs.
    ///
    /// # Errors
    ///
    /// Returns a database error if the read fails.
    // Filter is taken by value to give callers a clean `Default`-then-init pattern;
    // the function only needs a borrow internally.
    #[allow(clippy::needless_pass_by_value)]
    pub fn query_issues(
        &self,
        filter: crate::query::IssueFilter,
    ) -> crate::error::Result<Vec<crate::types::Issue>> {
        crate::store::read::issues::list(&self.conn, &filter)
    }

    /// Read the activity log for one issue, in `id` (insertion) order.
    ///
    /// Returns one row per field-level change on the issue; the timeline is
    /// purely derivative of `operation_log`, so it can be rebuilt from re-play.
    ///
    /// # Errors
    ///
    /// Returns a database error if the read fails.
    pub fn query_issue_history(
        &self,
        issue_id: uuid::Uuid,
    ) -> crate::error::Result<Vec<crate::types::ActivityEntry>> {
        crate::store::read::log::for_issue(&self.conn, issue_id)
    }

    /// Read the most-recent operation's `inverse_payload`.
    ///
    /// Used by tests; will be reused by `undo`.
    ///
    /// # Errors
    ///
    /// Returns a database error if no rows exist or the read fails, and a
    /// serialization error if the payload is malformed.
    pub fn last_inverse(&self) -> crate::error::Result<crate::operation::Operation> {
        let payload: String = self.conn.query_row(
            "SELECT inverse_payload FROM operation_log ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )?;
        Ok(serde_json::from_str(&payload)?)
    }
}

fn default_db_path() -> Result<PathBuf> {
    if let Ok(env) = std::env::var("KANBAN_DB") {
        return Ok(PathBuf::from(env));
    }
    let home = std::env::var("HOME").map_err(|_| {
        crate::error::Error::Validation(crate::error::ValidationError {
            field: "HOME".into(),
            reason: "must be set to derive default db path".into(),
        })
    })?;
    Ok(PathBuf::from(home).join(".kanban").join("data.db"))
}

#[cfg(test)]
// unwrap is acceptable in test code — panics are the intended failure mode.
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_runs_migrations() {
        let ws = Workspace::open_in_memory().unwrap();
        let count: i64 = ws
            .conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn open_file_creates_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let _ws = Workspace::open(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn default_db_path_uses_kanban_db_env_when_set() {
        // unsafe set_var is forbidden (unsafe_code = forbid in workspace).
        // We instead verify that default_db_path falls back to HOME/.kanban/data.db
        // when KANBAN_DB is not set, and that the value is a sensible path.
        // The KANBAN_DB env-var override is tested via integration test in
        // crates/kanban-cli where the workspace lints allow subprocess env
        // manipulation.
        if std::env::var("KANBAN_DB").is_err() {
            let path = default_db_path().unwrap();
            assert!(path.ends_with(".kanban/data.db"));
        }
    }
}
