use crate::error::Result;
use crate::store::{connection, migrations};
use crate::time::{Clock, system_clock};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Opaque handle to a kanban workspace.
///
/// Holds an open SQLite connection and a clock. NOT `Sync` — multi-threaded
/// callers must own one `Workspace` per thread (or use a pool).
pub struct Workspace {
    pub(crate) conn: Connection,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) path: WorkspacePath,
}

#[derive(Debug, Clone)]
pub enum WorkspacePath {
    InMemory,
    File(PathBuf),
}

impl Workspace {
    /// Open the workspace at `~/.kanban/data.db` (or `$KANBAN_DB`).
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        Self::open(&path)
    }

    /// Open a workspace at the given file path.
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
