use crate::error::{Error, Result};
use crate::operation::{Operation, OperationOutcome};
use crate::store::write::operation_log;
use crate::workspace::Workspace;

impl Workspace {
    /// Undo the most-recently-applied (non-undone) operation.
    ///
    /// Applies the captured inverse mutation in a single transaction and marks
    /// the original `operation_log` row as `undone = 1` so it becomes redoable.
    /// No new `operation_log` row is inserted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] with `"nothing to undo"` if the log is
    /// empty or every row is already undone, plus any database/serialization
    /// errors from the underlying mutation.
    pub fn undo(&mut self) -> Result<OperationOutcome> {
        let row = peek_undoable(&self.conn)?;
        let op: Operation = serde_json::from_str(&row.inverse_payload)?;

        let now = self.clock.now();
        let tx = self.conn.transaction()?;

        // Apply the inverse mutation directly without writing a new operation_log row.
        crate::apply::dispatch(&tx, &op, now)?;

        operation_log::set_op_undone(&tx, row.id, true)?;
        tx.commit()?;
        Ok(OperationOutcome { op_id: row.id })
    }

    /// Redo the next previously-undone operation in the redo branch.
    ///
    /// Re-applies the original op payload and clears its `undone` flag. No new
    /// `operation_log` row is inserted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] with `"nothing to redo"` if no rows are
    /// currently in the redo branch, plus any database/serialization errors
    /// from the underlying mutation.
    pub fn redo(&mut self) -> Result<OperationOutcome> {
        let row = peek_redoable(&self.conn)?;
        let op: Operation = serde_json::from_str(&row.payload)?;

        let now = self.clock.now();
        let tx = self.conn.transaction()?;

        crate::apply::dispatch(&tx, &op, now)?;

        operation_log::set_op_undone(&tx, row.id, false)?;
        tx.commit()?;
        Ok(OperationOutcome { op_id: row.id })
    }
}

struct OpRow {
    id: i64,
    payload: String,
    inverse_payload: String,
}

fn peek_undoable(conn: &rusqlite::Connection) -> Result<OpRow> {
    conn.query_row(
        "SELECT id, payload, inverse_payload FROM operation_log
         WHERE undone = 0 ORDER BY id DESC LIMIT 1",
        [],
        |r| {
            Ok(OpRow {
                id: r.get(0)?,
                payload: r.get(1)?,
                inverse_payload: r.get(2)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::Conflict("nothing to undo".into()),
        other => other.into(),
    })
}

fn peek_redoable(conn: &rusqlite::Connection) -> Result<OpRow> {
    conn.query_row(
        "SELECT id, payload, inverse_payload FROM operation_log
         WHERE undone = 1 ORDER BY id ASC LIMIT 1",
        [],
        |r| {
            Ok(OpRow {
                id: r.get(0)?,
                payload: r.get(1)?,
                inverse_payload: r.get(2)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::Conflict("nothing to redo".into()),
        other => other.into(),
    })
}
