//! Integration tests for `kanban issue` subcommands.
//!
//! Each test isolates the database via `KANBAN_DB` pointing at a fresh
//! temporary file, so the suite is hermetic and can be run in parallel.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use assert_cmd::Command;
use std::path::PathBuf;

fn isolated_db() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    (dir, path)
}

fn cli(db: &PathBuf) -> Command {
    let mut c = Command::cargo_bin("kanban").unwrap();
    c.env("KANBAN_DB", db);
    c
}

#[test]
fn issue_create_and_list() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args([
            "issue",
            "create",
            "--project",
            "PRJ",
            "--title",
            "add login",
        ])
        .assert()
        .success();
    cli(&db)
        .args([
            "issue",
            "create",
            "--project",
            "PRJ",
            "--title",
            "fix crash",
            "--priority",
            "high",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["issue", "list", "--project", "PRJ", "--sort", "priority"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("issue_list_priority_sort", stdout);
}

#[test]
fn issue_update_title_via_field_arg() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "old"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "update", "PRJ-1", "--title", "new"])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "issue", "show", "PRJ-1"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["title"], "new");
}

#[test]
fn issue_move_changes_status() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "x"])
        .assert()
        .success();
    // Default status is `Todo`, so the issue should appear in that filter.
    let out = cli(&db)
        .args([
            "--json",
            "issue",
            "list",
            "--project",
            "PRJ",
            "--status",
            "Todo",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);

    cli(&db)
        .args(["issue", "move", "PRJ-1", "--status", "In Progress"])
        .assert()
        .success();

    let out = cli(&db)
        .args([
            "--json",
            "issue",
            "list",
            "--project",
            "PRJ",
            "--status",
            "Todo",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);

    let out = cli(&db)
        .args([
            "--json",
            "issue",
            "list",
            "--project",
            "PRJ",
            "--status",
            "In Progress",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["title"], "x");
}

#[test]
fn issue_history_shows_field_changes() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "x"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "update", "PRJ-1", "--priority", "high"])
        .assert()
        .success();
    let out = cli(&db)
        .args(["issue", "history", "PRJ-1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::with_settings!({
        filters => vec![
            (r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})", "[TIMESTAMP]"),
        ],
    }, {
        insta::assert_snapshot!("issue_history_after_priority", stdout);
    });
}

#[test]
fn issue_create_invalid_priority_exits_validation() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    let assert = cli(&db)
        .args([
            "issue",
            "create",
            "--project",
            "PRJ",
            "--title",
            "x",
            "--priority",
            "bogus",
        ])
        .assert()
        .failure();
    assert.code(3);
}

#[test]
fn issue_show_unknown_identifier_exits_not_found() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    let assert = cli(&db)
        .args(["issue", "show", "PRJ-99"])
        .assert()
        .failure();
    assert.code(2);
}
