//! Integration tests for `kanban member` write subcommands.

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
fn member_create_then_list() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .success();
    let out = cli(&db)
        .args(["member", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("member_list_with_alice", stdout);
}

#[test]
fn member_create_duplicate_name_exits_nonzero() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .success();
    let assert = cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .failure();
    // Conflict/validation maps to EXIT_VALIDATION (3).
    assert.code(3);
}

#[test]
fn member_update_rename() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .success();
    cli(&db)
        .args([
            "member",
            "update",
            "--project",
            "PRJ",
            "--name",
            "Alice",
            "--rename",
            "Alicia",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "member", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Alicia"));
    assert!(!names.contains(&"Alice"));
}

#[test]
fn member_delete_with_yes_succeeds() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .success();
    cli(&db)
        .args([
            "member",
            "delete",
            "--project",
            "PRJ",
            "--name",
            "Alice",
            "--yes",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "member", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"Alice"));
}

#[test]
fn member_delete_without_yes_exits_validation() {
    let (_d, db) = isolated_db();
    make_project(&db);
    cli(&db)
        .args(["member", "create", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .success();
    let assert = cli(&db)
        .args(["member", "delete", "--project", "PRJ", "--name", "Alice"])
        .assert()
        .failure();
    assert.code(3);
}

#[test]
fn member_update_unknown_name_exits_nonzero() {
    let (_d, db) = isolated_db();
    make_project(&db);
    let assert = cli(&db)
        .args([
            "member",
            "update",
            "--project",
            "PRJ",
            "--name",
            "Ghost",
            "--rename",
            "Boo",
        ])
        .assert()
        .failure();
    // NotFound maps to EXIT_NOT_FOUND (2).
    assert.code(2);
}
