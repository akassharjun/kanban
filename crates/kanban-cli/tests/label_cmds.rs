//! Integration tests for `kanban label` subcommands.

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
fn label_create_then_list() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "#ff0000",
        ])
        .assert()
        .success();
    cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "perf",
            "--color",
            "#00ff00",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["label", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("label_list_two_labels", stdout);
}

#[test]
fn label_attach_then_show_includes_label() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "#ff0000",
        ])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "fix it"])
        .assert()
        .success();
    cli(&db)
        .args(["label", "attach", "PRJ-1", "bug"])
        .assert()
        .success();
    // The list filter by label name should now find the issue.
    let out = cli(&db)
        .args([
            "--json",
            "issue",
            "list",
            "--project",
            "PRJ",
            "--label",
            "bug",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["title"], "fix it");
}

#[test]
fn label_detach_removes_association() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "#ff0000",
        ])
        .assert()
        .success();
    cli(&db)
        .args(["issue", "create", "--project", "PRJ", "--title", "fix it"])
        .assert()
        .success();
    cli(&db)
        .args(["label", "attach", "PRJ-1", "bug"])
        .assert()
        .success();
    cli(&db)
        .args(["label", "detach", "PRJ-1", "bug"])
        .assert()
        .success();
    let out = cli(&db)
        .args([
            "--json",
            "issue",
            "list",
            "--project",
            "PRJ",
            "--label",
            "bug",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
}

#[test]
fn label_create_invalid_color_exits_validation() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    let assert = cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "notahex",
        ])
        .assert()
        .failure();
    assert.code(3);
}

#[test]
fn label_create_duplicate_name_exits_validation() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "#ff0000",
        ])
        .assert()
        .success();
    let assert = cli(&db)
        .args([
            "label",
            "create",
            "--project",
            "PRJ",
            "--name",
            "bug",
            "--color",
            "#aa0000",
        ])
        .assert()
        .failure();
    // Conflict maps to EXIT_VALIDATION (3).
    assert.code(3);
}
