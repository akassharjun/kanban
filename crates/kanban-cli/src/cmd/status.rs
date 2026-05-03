//! `kanban status` subcommands. Read-only listing of statuses per project.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::{Result, Workspace};

#[derive(Debug, Args)]
pub struct StatusCmd {
    #[command(subcommand)]
    pub sub: StatusSub,
}

#[derive(Debug, Subcommand)]
pub enum StatusSub {
    /// List statuses for a project.
    List { project: String },
}

/// Dispatch a `kanban status` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until a later task wires it.
pub fn run(_cmd: StatusCmd, _ws: &Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "status subcommand not yet implemented".into(),
    ))
}
