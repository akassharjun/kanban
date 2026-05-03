use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

pub fn open_file(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

fn apply_pragmas(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pragma_value(conn: &Connection, name: &str) -> String {
        conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get::<_, String>(0))
            .unwrap_or_else(|_| {
                conn.query_row(&format!("PRAGMA {name}"), [], |r| {
                    Ok(r.get::<_, i64>(0)?.to_string())
                })
                .unwrap()
            })
    }

    #[test]
    fn in_memory_applies_required_pragmas() {
        let conn = open_in_memory().unwrap();
        assert_eq!(pragma_value(&conn, "synchronous"), "1"); // NORMAL
        assert_eq!(pragma_value(&conn, "foreign_keys"), "1");
    }

    #[test]
    fn file_applies_wal_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let conn = open_file(&path).unwrap();
        let mode = pragma_value(&conn, "journal_mode");
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn open_file_creates_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/inner/data.db");
        let _conn = open_file(&path).unwrap();
        assert!(path.exists());
    }
}
