//! Small output helpers for human-readable / JSON output modes.
//!
//! Subcommand implementations branch on [`Out::json`] and call either
//! [`Out::print_json`] (always emits JSON) or [`Out::print_human`]
//! (emits nothing in JSON mode).

use serde::Serialize;
use std::io::{Write, stdout};

pub struct Out {
    pub json: bool,
}

impl Out {
    /// Emit `value` either as `Display` (human mode) or as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if writing to stdout fails, or a
    /// [`serde_json`] error if serialization fails (impossible for the
    /// concrete types we use, but propagated for completeness).
    #[allow(dead_code)]
    pub fn print<T: Serialize + std::fmt::Display>(&self, value: &T) -> kanban_core::Result<()> {
        if self.json {
            let s = serde_json::to_string_pretty(value)?;
            writeln!(stdout(), "{s}")?;
        } else {
            writeln!(stdout(), "{value}")?;
        }
        Ok(())
    }

    /// Emit `value` as pretty JSON regardless of `self.json`.
    ///
    /// Used by subcommands that branch on `self.json` themselves.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if writing to stdout fails, or a
    /// [`serde_json`] error if serialization fails.
    #[allow(dead_code)]
    pub fn print_json<T: Serialize>(&self, value: &T) -> kanban_core::Result<()> {
        let _ = self;
        let s = serde_json::to_string_pretty(value)?;
        writeln!(stdout(), "{s}")?;
        Ok(())
    }

    /// Emit a pre-formatted string only in human-readable mode.
    ///
    /// In JSON mode this is a no-op so structured output remains parseable.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if writing to stdout fails.
    #[allow(dead_code)]
    pub fn print_human(&self, s: &str) -> std::io::Result<()> {
        if !self.json {
            writeln!(stdout(), "{s}")?;
        }
        Ok(())
    }
}
