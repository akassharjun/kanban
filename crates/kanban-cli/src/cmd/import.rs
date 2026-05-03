//! `kanban import` subcommand. Stubbed until snapshot-CLI task.

use crate::output::Out;
use clap::Args;
use kanban_core::{Result, Workspace};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Input snapshot JSON file.
    pub file: PathBuf,
    /// Conflict resolution policy: skip|overwrite|fail.
    #[arg(long, default_value = "fail")]
    pub policy: String,
}

/// Dispatch a `kanban import` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run(_cmd: ImportArgs, _ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "import subcommand not yet implemented".into(),
    ))
}
