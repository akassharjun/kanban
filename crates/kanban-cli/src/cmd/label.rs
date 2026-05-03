//! `kanban label` subcommands. Stubbed until Task 35.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::{Result, Workspace};

#[derive(Debug, Args)]
pub struct LabelCmd {
    #[command(subcommand)]
    pub sub: LabelSub,
}

#[derive(Debug, Subcommand)]
pub enum LabelSub {
    /// List labels for a project.
    List { project: String },
    /// Create a new label.
    Create {
        project: String,
        name: String,
        #[arg(long)]
        color: String,
    },
    /// Delete a label.
    Delete {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Attach a label to an issue.
    Attach { issue: String, label: String },
    /// Detach a label from an issue.
    Detach { issue: String, label: String },
}

/// Dispatch a `kanban label` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until Task 35 implements it.
pub fn run(_cmd: LabelCmd, _ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "label subcommand not yet implemented".into(),
    ))
}
