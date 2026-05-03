//! `--help` lists every subcommand.

#![allow(clippy::unwrap_used)]

use assert_cmd::Command;

#[test]
fn help_lists_all_subcommands() {
    let assert = Command::cargo_bin("kanban")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    for sub in [
        "project", "issue", "label", "status", "search", "export", "import", "undo", "redo",
        "batch",
    ] {
        assert!(
            stdout.contains(sub),
            "missing '{sub}' in --help output:\n{stdout}"
        );
    }
}
