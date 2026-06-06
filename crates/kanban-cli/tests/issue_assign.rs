//! Integration tests for `kanban issue assign` / `--unassign`.

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

fn setup(db: &PathBuf) {
    cli(db)
        .args(["project", "create", "Auth", "--prefix", "AUTH"])
        .assert()
        .success();
    cli(db)
        .args(["member", "create", "--project", "AUTH", "--name", "Alice"])
        .assert()
        .success();
    cli(db)
        .args(["issue", "create", "--project", "AUTH", "--title", "login"])
        .assert()
        .success();
}

fn assignee(db: &PathBuf) -> Option<String> {
    let out = cli(db)
        .args(["--json", "issue", "show", "AUTH-1"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["assignee_id"].as_str().map(ToString::to_string)
}

#[test]
fn issue_assign_sets_assignee() {
    let (_d, db) = isolated_db();
    setup(&db);
    assert!(assignee(&db).is_none());
    cli(&db)
        .args(["issue", "assign", "AUTH-1", "Alice"])
        .assert()
        .success();
    assert!(assignee(&db).is_some());
}

#[test]
fn issue_unassign_clears_assignee() {
    let (_d, db) = isolated_db();
    setup(&db);
    cli(&db)
        .args(["issue", "assign", "AUTH-1", "Alice"])
        .assert()
        .success();
    assert!(assignee(&db).is_some());
    cli(&db)
        .args(["issue", "assign", "AUTH-1", "--unassign"])
        .assert()
        .success();
    assert!(assignee(&db).is_none());
}

#[test]
fn issue_assign_unknown_member_exits_not_found() {
    let (_d, db) = isolated_db();
    setup(&db);
    let assert = cli(&db)
        .args(["issue", "assign", "AUTH-1", "Ghost"])
        .assert()
        .failure();
    // NotFound maps to EXIT_NOT_FOUND (2).
    assert.code(2);
}

#[test]
fn issue_assign_foreign_member_exits_validation() {
    let (_d, db) = isolated_db();
    setup(&db);
    // A member in a different project must not be assignable to AUTH-1.
    cli(&db)
        .args(["project", "create", "Other", "--prefix", "OTH"])
        .assert()
        .success();
    cli(&db)
        .args(["member", "create", "--project", "OTH", "--name", "Bob"])
        .assert()
        .success();
    // Resolving by name within AUTH's project won't find Bob -> NotFound (2).
    let assert = cli(&db)
        .args(["issue", "assign", "AUTH-1", "Bob"])
        .assert()
        .failure();
    assert.code(2);
}
