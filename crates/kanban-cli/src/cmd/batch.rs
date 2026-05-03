//! `kanban batch` subcommand. Stubbed until batch task.
//!
//! Note: batch opens its own [`kanban_core::Workspace`] from the optional `db`
//! path or `$KANBAN_DB`, so its `run` signature does NOT take `&mut Workspace`.

use clap::Args;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct BatchArgs {
    /// JSON file containing an array of operations.
    pub file: PathBuf,
}

/// Dispatch a `kanban batch` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run(
    _args: BatchArgs,
    _db: Option<&Path>,
    _out: &crate::output::Out,
) -> kanban_core::Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "batch subcommand not yet implemented".into(),
    ))
}
