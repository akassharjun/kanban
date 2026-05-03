//! Binary entry point for `kanban`.
//!
//! Parses [`cli::Cli`] (clap), opens a [`kanban_core::Workspace`] from
//! `--db` / `$KANBAN_DB` / the default path, and dispatches to a `cmd` module.
//! Errors from `kanban_core` are mapped to stable exit codes via [`exit::code_for`].

mod cli;
mod cmd;
mod exit;
mod output;

use clap::Parser;
use cli::{Cli, Cmd};
use kanban_core::Workspace;

fn main() {
    let args = Cli::parse();
    match run(args) {
        Ok(()) => std::process::exit(exit::EXIT_OK),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(exit::code_for(&e));
        }
    }
}

fn run(args: Cli) -> kanban_core::Result<()> {
    // `Batch` opens its own workspace inside its `run` implementation because
    // batch operations need transactional control over the connection. Every
    // other subcommand reuses the parent-opened workspace.
    if let Cmd::Batch(c) = args.cmd {
        let out = output::Out { json: args.json };
        return cmd::batch::run(c, args.db.as_deref(), &out);
    }

    let mut ws = match args.db.as_ref() {
        Some(path) => Workspace::open(path)?,
        None => Workspace::open_default()?,
    };
    let out = output::Out { json: args.json };
    match args.cmd {
        Cmd::Project(c) => cmd::project::run(c, &mut ws, &out),
        Cmd::Issue(c) => cmd::issue::run(c, &mut ws, &out),
        Cmd::Label(c) => cmd::label::run(c, &mut ws, &out),
        Cmd::Status(c) => cmd::status::run(c, &ws, &out),
        Cmd::Search(c) => cmd::search::run(c, &ws, &out),
        Cmd::Export(c) => cmd::export::run(c, &ws, &out),
        Cmd::Import(c) => cmd::import::run(c, &mut ws, &out),
        Cmd::Undo => cmd::undo::run_undo(&mut ws, &out),
        Cmd::Redo => cmd::undo::run_redo(&mut ws, &out),
        Cmd::Batch(_) => unreachable!("Batch handled above"),
    }
}
