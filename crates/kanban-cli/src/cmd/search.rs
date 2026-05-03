//! `kanban search` subcommand. Stubbed until search-CLI task.

use crate::output::Out;
use clap::Args;
use kanban_core::{Result, Workspace};

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// FTS5 query string.
    pub query: String,
    /// Restrict to a single project (id or prefix).
    #[arg(long)]
    pub project: Option<String>,
}

/// Dispatch a `kanban search` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run(_cmd: SearchArgs, _ws: &Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "search subcommand not yet implemented".into(),
    ))
}
