//! `kanban import` subcommand. Applies an `ImportSnapshot` operation.

use crate::output::Out;
use clap::Args;
use kanban_core::operation::{ConflictPolicy, ImportSnapshot, Operation};
use kanban_core::snapshot::WorkspaceSnapshot;
use kanban_core::{Result, Workspace};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Input snapshot JSON file.
    pub file: PathBuf,
    /// Conflict resolution policy: `skip`, `overwrite`, or `fail`.
    #[arg(long, default_value = "fail")]
    pub conflict: String,
}

/// Dispatch a `kanban import` invocation.
///
/// # Errors
///
/// Propagates I/O errors reading the snapshot file, JSON deserialization
/// errors, validation of the `--conflict` policy, and any error from the
/// underlying [`Workspace::apply`] of `ImportSnapshot`.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: ImportArgs, ws: &mut Workspace, out: &Out) -> Result<()> {
    let policy = parse_policy(&args.conflict)?;
    let s = fs::read_to_string(&args.file)?;
    let snapshot: WorkspaceSnapshot = serde_json::from_str(&s)?;
    let outcome = ws.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot,
        policy,
    }))?;
    if out.json {
        out.print_json(&serde_json::json!({"ok": true, "op_id": outcome.op_id}))?;
    } else {
        println!("imported (op_id={})", outcome.op_id);
    }
    Ok(())
}

fn parse_policy(s: &str) -> Result<ConflictPolicy> {
    match s {
        "skip" => Ok(ConflictPolicy::Skip),
        "overwrite" => Ok(ConflictPolicy::Overwrite),
        "fail" => Ok(ConflictPolicy::Fail),
        other => Err(kanban_core::Error::Validation(
            kanban_core::ValidationError {
                field: "conflict".into(),
                reason: format!("unknown value '{other}', expected skip|overwrite|fail"),
            },
        )),
    }
}
