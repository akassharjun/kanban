//! `kanban project` subcommands. Currently stubbed; Task 33 implements them.

use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::{Result, Workspace};

#[derive(Debug, Args)]
pub struct ProjectCmd {
    #[command(subcommand)]
    pub sub: ProjectSub,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSub {
    /// List all projects.
    List,
    /// Create a new project.
    Create {
        name: String,
        #[arg(long)]
        prefix: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        icon: Option<String>,
    },
    /// Show one project by id or prefix.
    Show { id_or_prefix: String },
    /// Update project metadata.
    Update {
        id_or_prefix: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Archive a project (soft delete; preserves issues).
    Archive { id_or_prefix: String },
    /// Permanently delete a project (requires `--yes`).
    Delete {
        id_or_prefix: String,
        #[arg(long)]
        yes: bool,
    },
}

/// Dispatch a `kanban project` invocation.
///
/// # Errors
///
/// Returns [`kanban_core::Error::InvalidSnapshot`] until Task 33 implements
/// the real subcommand bodies.
pub fn run(_cmd: ProjectCmd, _ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot(
        "project subcommand not yet implemented".into(),
    ))
}
