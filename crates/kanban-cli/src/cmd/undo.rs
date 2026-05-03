//! `kanban undo` and `kanban redo` subcommands. Thin wrappers over
//! [`Workspace::undo`] and [`Workspace::redo`].

use crate::output::Out;
use kanban_core::{Result, Workspace};

/// Undo the most recent operation.
///
/// # Errors
///
/// Propagates errors from [`Workspace::undo`]. In particular,
/// [`kanban_core::Error::Conflict`] is returned (and maps to exit code 3) when
/// the operation log is empty.
pub fn run_undo(ws: &mut Workspace, out: &Out) -> Result<()> {
    let outcome = ws.undo()?;
    if out.json {
        out.print_json(&serde_json::json!({"ok": true, "op_id": outcome.op_id}))?;
    } else {
        println!("undone (op_id={})", outcome.op_id);
    }
    Ok(())
}

/// Redo the most recently undone operation.
///
/// # Errors
///
/// Propagates errors from [`Workspace::redo`]. In particular,
/// [`kanban_core::Error::Conflict`] is returned (and maps to exit code 3) when
/// the redo branch is empty.
pub fn run_redo(ws: &mut Workspace, out: &Out) -> Result<()> {
    let outcome = ws.redo()?;
    if out.json {
        out.print_json(&serde_json::json!({"ok": true, "op_id": outcome.op_id}))?;
    } else {
        println!("redone (op_id={})", outcome.op_id);
    }
    Ok(())
}
