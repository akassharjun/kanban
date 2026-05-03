//! `kanban undo` and `kanban redo` subcommands. Stubbed until undo-CLI task.

use crate::output::Out;
use kanban_core::{Result, Workspace};

/// Undo the most recent operation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run_undo(_ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "undo subcommand not yet implemented".into(),
    ))
}

/// Redo the most recently undone operation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run_redo(_ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "redo subcommand not yet implemented".into(),
    ))
}
