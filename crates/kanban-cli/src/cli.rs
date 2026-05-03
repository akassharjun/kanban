//! Top-level clap definitions for the `kanban` binary.
//!
//! Global flags (`--db`, `--json`, `--quiet`, `--verbose`) are parsed here and
//! threaded into [`crate::output::Out`] / [`crate::cmd`] dispatch.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "kanban",
    version,
    about = "Local-first kanban for projects, issues, labels, undo/redo, and JSON I/O."
)]
pub struct Cli {
    /// Override database path (also `$KANBAN_DB`).
    #[arg(long, env = "KANBAN_DB", global = true)]
    pub db: Option<PathBuf>,

    /// Emit machine-readable JSON instead of human-readable output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-error output.
    #[arg(long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Verbose diagnostic output on stderr.
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Manage projects (list/create/show/update/archive/delete).
    Project(crate::cmd::project::ProjectCmd),
    /// Manage issues.
    Issue(crate::cmd::issue::IssueCmd),
    /// Manage labels.
    Label(crate::cmd::label::LabelCmd),
    /// Manage statuses.
    Status(crate::cmd::status::StatusCmd),
    /// Full-text search across issues.
    Search(crate::cmd::search::SearchArgs),
    /// Export the workspace as a JSON snapshot.
    Export(crate::cmd::export::ExportArgs),
    /// Import a JSON snapshot into the workspace.
    Import(crate::cmd::import::ImportArgs),
    /// Undo the most recent operation.
    Undo,
    /// Redo the most recently undone operation.
    Redo,
    /// Apply a batch of operations from a JSON file.
    Batch(crate::cmd::batch::BatchArgs),
}
