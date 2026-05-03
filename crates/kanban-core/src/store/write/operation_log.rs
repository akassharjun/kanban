use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};

/// Insert one row into `operation_log`. Returns the new `op_id`.
pub(crate) fn insert_operation(
    tx: &Transaction<'_>,
    op_type: &str,
    payload: &str,
    inverse_payload: &str,
    applied_at: DateTime<Utc>,
) -> Result<i64> {
    tx.execute(
        "INSERT INTO operation_log(op_type, payload, inverse_payload, applied_at, undone)
         VALUES (?1, ?2, ?3, ?4, 0)",
        params![op_type, payload, inverse_payload, applied_at.to_rfc3339()],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Insert one row into `activity_log` linked to a previously-inserted op_id.
pub(crate) fn insert_activity(
    tx: &Transaction<'_>,
    op_id: i64,
    issue_id: Option<&str>,
    field: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    at: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "INSERT INTO activity_log(op_id, issue_id, field, old_value, new_value, at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![op_id, issue_id, field, old_value, new_value, at.to_rfc3339()],
    )?;
    Ok(())
}

/// Mark `op_id` as undone (1) or redone (0).
pub(crate) fn set_op_undone(tx: &Transaction<'_>, op_id: i64, undone: bool) -> Result<()> {
    tx.execute(
        "UPDATE operation_log SET undone = ?1 WHERE id = ?2",
        params![i64::from(undone), op_id],
    )?;
    Ok(())
}

/// Discard the redo branch (any rows where undone=1). Called when a new forward op lands
/// while a redo branch existed.
pub(crate) fn truncate_redo_branch(tx: &Transaction<'_>) -> Result<()> {
    tx.execute("DELETE FROM operation_log WHERE undone = 1", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{connection::open_in_memory, migrations};

    fn fresh_tx_owner() -> rusqlite::Connection {
        let mut c = open_in_memory().unwrap();
        migrations::run(&mut c).unwrap();
        c
    }

    #[test]
    fn insert_operation_returns_monotonic_ids() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let a = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        let b = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        tx.commit().unwrap();
        assert!(b > a);
    }

    #[test]
    fn insert_activity_links_to_op_id() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let op_id = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        insert_activity(&tx, op_id, None, "title", Some("a"), Some("b"), now).unwrap();
        tx.commit().unwrap();
        let count: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM activity_log WHERE op_id = ?1",
                params![op_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn set_op_undone_toggles_flag() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let op_id = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        set_op_undone(&tx, op_id, true).unwrap();
        tx.commit().unwrap();
        let undone: i64 = c
            .query_row(
                "SELECT undone FROM operation_log WHERE id = ?1",
                params![op_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(undone, 1);
    }

    #[test]
    fn truncate_redo_branch_only_deletes_undone_rows() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let live = insert_operation(&tx, "L", "{}", "{}", now).unwrap();
        let dead = insert_operation(&tx, "D", "{}", "{}", now).unwrap();
        set_op_undone(&tx, dead, true).unwrap();
        truncate_redo_branch(&tx).unwrap();
        tx.commit().unwrap();
        let live_exists: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM operation_log WHERE id = ?1",
                params![live],
                |r| r.get(0),
            )
            .unwrap();
        let dead_exists: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM operation_log WHERE id = ?1",
                params![dead],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(live_exists, 1);
        assert_eq!(dead_exists, 0);
    }
}
