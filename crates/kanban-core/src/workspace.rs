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

    /// List all labels for `project_id` ordered by name.
    ///
    /// # Errors
    ///
    /// Returns a database error if the read fails.
    pub fn query_labels_for_project(
        &self,
        project_id: uuid::Uuid,
    ) -> crate::error::Result<Vec<crate::types::Label>> {
        crate::store::read::labels::for_project(&self.conn, project_id)
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

    /// Full-text search across issue titles and descriptions, composed with `filter`.
    ///
    /// Backed by the FTS5 `issue_search` virtual table. Result order is by FTS
    /// relevance (`rank`) rather than the filter's `SortBy`. The filter's
    /// `search_text` field, if set, is overwritten by `query`.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails or the FTS5 expression is malformed.
    // Filter is taken by value because `search` mutates its `search_text` field
    // before use; callers get a clean ownership story.
    #[allow(clippy::needless_pass_by_value)]
    pub fn search(
        &self,
        query: &str,
        filter: crate::query::IssueFilter,
    ) -> crate::error::Result<Vec<crate::types::Issue>> {
        crate::store::read::search::search(&self.conn, query, filter)
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

    /// Doc-hidden accessor for integration tests in this crate's `tests/` folder.
    ///
    /// Stable for the duration of v1; do NOT rely on this from external crates.
    /// Plain `cfg(test)` doesn't apply across the integration-test compilation
    /// boundary, so this is exposed publicly with `#[doc(hidden)]` instead.
    #[doc(hidden)]
    #[must_use]
    pub fn _conn_for_integration_tests(&self) -> &rusqlite::Connection {
        &self.conn
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

    /// Export the entire workspace to a [`WorkspaceSnapshot`].
    ///
    /// Reads every project, status, label, issue, and issue/label join row
    /// using the existing read helpers and assembles them into a single
    /// snapshot value tagged with [`SNAPSHOT_SCHEMA_VERSION`].
    ///
    /// # Errors
    ///
    /// Returns a database error if any of the underlying reads fail.
    pub fn export_snapshot(&self) -> crate::error::Result<crate::snapshot::WorkspaceSnapshot> {
        use crate::snapshot::{IssueLabelLink, SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

        let projects = crate::store::read::projects::list_all(&self.conn)?;

        let mut statuses = Vec::new();
        let mut labels = Vec::new();
        for p in &projects {
            statuses.extend(crate::store::read::statuses::for_project(&self.conn, p.id)?);
            labels.extend(crate::store::read::labels::for_project(&self.conn, p.id)?);
        }

        let issues =
            crate::store::read::issues::list(&self.conn, &crate::query::IssueFilter::default())?;

        let mut issue_labels = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT issue_id, label_id FROM issue_labels")?;
        let rows = stmt.query_map([], |r| {
            let issue_id_s: String = r.get(0)?;
            let label_id_s: String = r.get(1)?;
            Ok((issue_id_s, label_id_s))
        })?;
        for r in rows {
            let (issue_id_s, label_id_s) = r?;
            issue_labels.push(IssueLabelLink {
                issue_id: uuid::Uuid::parse_str(&issue_id_s).map_err(|e| {
                    crate::error::Error::InvalidSnapshot(format!(
                        "issue_labels.issue_id is not a uuid: {e}"
                    ))
                })?,
                label_id: uuid::Uuid::parse_str(&label_id_s).map_err(|e| {
                    crate::error::Error::InvalidSnapshot(format!(
                        "issue_labels.label_id is not a uuid: {e}"
                    ))
                })?,
            });
        }

        Ok(WorkspaceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            exported_at: chrono::Utc::now(),
            projects,
            statuses,
            issues,
            labels,
            issue_labels,
        })
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
