//! `kanban export` subcommand. Stubbed until snapshot-CLI task.

use crate::output::Out;
use clap::Args;
use kanban_core::{Result, Workspace};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Output file (defaults to stdout).
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// Dispatch a `kanban export` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run(_cmd: ExportArgs, _ws: &Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "export subcommand not yet implemented".into(),
    ))
}
