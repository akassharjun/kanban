//! `kanban export` subcommand. Serializes the workspace as a JSON snapshot.

use crate::output::Out;
use clap::Args;
use kanban_core::{Result, Workspace};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Output file (defaults to stdout).
    #[arg(short = 'o', long)]
    pub out: Option<PathBuf>,
    /// Reserved: include the operation/activity log in the snapshot.
    ///
    /// In v1 the log is excluded from snapshots regardless of this flag; the
    /// flag is parsed so external tooling that already passes it keeps working.
    #[arg(long)]
    pub with_history: bool,
}

/// Dispatch a `kanban export` invocation.
///
/// # Errors
///
/// Propagates errors from [`Workspace::export_snapshot`] and from writing to
/// the output file or stdout.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: ExportArgs, ws: &Workspace, _out: &Out) -> Result<()> {
    let _ = args.with_history; // reserved.
    let snap = ws.export_snapshot()?;
    let s = serde_json::to_string_pretty(&snap)?;
    if let Some(path) = &args.out {
        fs::write(path, s)?;
    } else {
        use std::io::Write as _;
        let stdout = std::io::stdout();
        let mut h = stdout.lock();
        writeln!(h, "{s}")?;
    }
    Ok(())
}
