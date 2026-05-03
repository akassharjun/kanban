use crate::error::Result;
use chrono::Utc;
use rusqlite::{Connection, params};

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "init", include_str!("../../migrations/0001_init.sql")),
];

pub fn run(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
           version    INTEGER PRIMARY KEY,
           applied_at TEXT    NOT NULL
         ) STRICT",
    )?;

    for (version, _name, sql) in MIGRATIONS {
        let already: bool = conn.query_row(
            "SELECT 1 FROM schema_migrations WHERE version = ?1",
            params![version],
            |_| Ok(true),
        ).optional()?.unwrap_or(false);

        if already {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

trait OptionalRow<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalRow<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::connection::open_in_memory;

    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name")
            .unwrap();
        stmt.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    #[test]
    fn run_creates_all_v1_tables() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        let tables = table_names(&conn);
        for expected in [
            "activity_log",
            "issue_labels",
            "issue_search",
            "issues",
            "labels",
            "operation_log",
            "projects",
            "schema_migrations",
            "statuses",
        ] {
            assert!(tables.contains(&expected.to_string()), "missing {expected} in {tables:?}");
        }
    }

    #[test]
    fn run_is_idempotent() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        run(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn fts_triggers_keep_search_in_sync() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        // Manually insert minimum data to test triggers — full apply tested elsewhere.
        conn.execute(
            "INSERT INTO projects(id,name,prefix,status,next_seq,created_at,updated_at)
             VALUES('p','P','PPP','active',1,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO statuses(id,project_id,name,category,color,position)
             VALUES('s','p','Todo','unstarted','#000000',0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,priority,sort_key,created_at,updated_at)
             VALUES('i','p',1,'PPP-1','add login','user can login','s','high',1.0,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let hits: i64 = conn.query_row(
            "SELECT count(*) FROM issue_search WHERE issue_search MATCH 'login'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(hits, 1);
    }
}
