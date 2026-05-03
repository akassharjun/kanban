//! `kanban issue` subcommands. Stubbed until Task 34.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::{Result, Workspace};

#[derive(Debug, Args)]
pub struct IssueCmd {
    #[command(subcommand)]
    pub sub: IssueSub,
}

#[derive(Debug, Subcommand)]
pub enum IssueSub {
    /// List issues across the workspace.
    List,
    /// Show a single issue by id or identifier.
    Show { id_or_identifier: String },
    /// Create a new issue.
    Create {
        project: String,
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
    },
    /// Update issue fields.
    Update {
        id_or_identifier: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
    },
    /// Delete an issue.
    Delete {
        id_or_identifier: String,
        #[arg(long)]
        yes: bool,
    },
    /// Show the activity log for an issue.
    History { id_or_identifier: String },
}

/// Dispatch a `kanban issue` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until Task 34 implements it.
pub fn run(_cmd: IssueCmd, _ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "issue subcommand not yet implemented".into(),
    ))
}
