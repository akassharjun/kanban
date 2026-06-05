//! Integration tests for `kanban status` write subcommands.

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

fn make_project(db: &PathBuf) {
    cli(db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
}

#[test]
fn status_create_then_list() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["status", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("status_list_with_review", stdout);
}

#[test]
fn status_create_duplicate_name_exits_validation() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    let assert = cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#123456",
        ])
        .assert()
        .failure();
    // Conflict/validation maps to EXIT_VALIDATION (3).
    assert.code(3);
}

#[test]
fn status_create_unknown_category_exits_validation() {
    let (_d, db) = isolated_db();
    make_project(&db);
    let assert = cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "bogus",
            "--color",
            "#abcdef",
        ])
        .assert()
        .failure();
    assert.code(3);
}

#[test]
fn status_update_rename() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    cli(&db)
        .args([
            "status",
            "update",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--rename",
            "Reviewing",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "status", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Reviewing"));
    assert!(!names.contains(&"Review"));
}

#[test]
fn status_delete_empty_succeeds() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    cli(&db)
        .args([
            "status",
            "delete",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--yes",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "status", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"Review"));
}

#[test]
fn status_delete_without_yes_exits_validation() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    let assert = cli(&db)
        .args(["status", "delete", "--project", "PRJ", "--name", "Review"])
        .assert()
        .failure();
    assert.code(3);
}

#[test]
fn status_delete_with_issue_exits_conflict() {
    let (_d, db) = isolated_db();
    make_project(&db);
    // Default project ships with statuses; find the default status an issue lands in.
    // Create an issue (lands in the project's initial status), then move it into our
    // new status so deletion is blocked.
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "fix it"])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "move", "PRJ-1", "--status", "Review"])
        .assert()
        .success();
    let assert = cli(&db)
        .args([
            "status",
            "delete",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--yes",
        ])
        .assert()
        .failure();
    // Conflict maps to EXIT_VALIDATION (3).
    assert.code(3);
}

#[test]
fn status_reorder_changes_order() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args([
            "status",
            "create",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--category",
            "started",
            "--color",
            "#abcdef",
        ])
        .assert()
        .success();
    // Move the newly-appended status to the front.
    cli(&db)
        .args([
            "status",
            "reorder",
            "--project",
            "PRJ",
            "--name",
            "Review",
            "--position",
            "0",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "status", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let first = v.as_array().unwrap().first().unwrap();
    assert_eq!(first["name"].as_str().unwrap(), "Review");
}
