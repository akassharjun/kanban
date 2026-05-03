//! Integration tests for `kanban project` subcommands.
//!
//! Each test isolates the database via `KANBAN_DB` pointing at a fresh
//! temporary file, so the suite is hermetic and can be run in parallel.

#![allow(clippy::unwrap_used)]

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
fn project_create_then_list() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "Auth", "--prefix", "AUTH"])
        .assert()
        .success();
    let out = cli(&db).args(["project", "list"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!(&stdout, @"AUTH  Auth  active");
}

#[test]
fn project_create_emits_json_when_requested() {
    let (_d, db) = isolated_db();
    let out = cli(&db)
        .args(["--json", "project", "create", "Auth", "--prefix", "AUTH"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["prefix"], "AUTH");
    assert_eq!(v["name"], "Auth");
}

#[test]
fn project_create_invalid_prefix_exits_validation() {
    let (_d, db) = isolated_db();
    let assert = cli(&db)
        .args(["project", "create", "Bad", "--prefix", "lower"])
        .assert()
        .failure();
    assert.code(3); // EXIT_VALIDATION
}

#[test]
fn project_show_unknown_prefix_exits_not_found() {
    let (_d, db) = isolated_db();
    let assert = cli(&db)
        .args(["project", "show", "NOPE"])
        .assert()
        .failure();
    assert.code(2); // EXIT_NOT_FOUND
}

#[test]
fn project_archive_changes_status() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "X", "--prefix", "ARC"])
        .assert()
        .success();
    cli(&db)
        .args(["project", "archive", "ARC"])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "project", "show", "ARC"])
        .output()
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&out.stdout).unwrap()).unwrap();
    assert_eq!(v["status"], "archived");
}
