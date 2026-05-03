//! Integration tests for `status`, `search`, `export`, `import`, `undo`, `redo`.
//!
//! Grouped in one test file for brevity.

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
fn status_list_outputs_default_statuses() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    let out = cli(&db)
        .args(["status", "list", "--project", "PRJ"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("status_list_default", stdout);
}

#[test]
fn search_returns_match() {
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
            "fix login bug",
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
            "ship marketing page",
        ])
        .assert()
        .success();
    let out = cli(&db)
        .args(["--json", "search", "login"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "fix login bug");
}

#[test]
fn export_then_import_round_trips() {
    let (_d1, src_db) = isolated_db();
    cli(&src_db)
        .args(["project", "create", "Auth", "--prefix", "AUTH"])
        .assert()
        .success();
    cli(&src_db)
        .args([
            "issue",
            "create",
            "--project",
            "AUTH",
            "--title",
            "impl OAuth",
        ])
        .assert()
        .success();

    let dump_dir = tempfile::tempdir().unwrap();
    let dump_path = dump_dir.path().join("snapshot.json");
    cli(&src_db)
        .args(["export", "-o", dump_path.to_str().unwrap()])
        .assert()
        .success();

    let (_d2, dst_db) = isolated_db();
    cli(&dst_db)
        .args([
            "import",
            dump_path.to_str().unwrap(),
            "--conflict",
            "overwrite",
        ])
        .assert()
        .success();

    // Compare project + issue counts.
    let out = cli(&dst_db)
        .args(["--json", "project", "list"])
        .output()
        .unwrap();
    let projects: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(projects.as_array().unwrap().len(), 1);
    assert_eq!(projects[0]["prefix"], "AUTH");

    let out = cli(&dst_db)
        .args(["--json", "issue", "list", "--project", "AUTH"])
        .output()
        .unwrap();
    let issues: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(issues.as_array().unwrap().len(), 1);
    assert_eq!(issues[0]["title"], "impl OAuth");
}

#[test]
fn undo_then_redo_returns_state() {
    let (_d, db) = isolated_db();
    cli(&db)
        .args(["project", "create", "P", "--prefix", "PRJ"])
        .assert()
        .success();
    // Undo: project disappears.
    cli(&db).args(["undo"]).assert().success();
    let out = cli(&db)
        .args(["--json", "project", "list"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);

    // Redo: project comes back.
    cli(&db).args(["redo"]).assert().success();
    let out = cli(&db)
        .args(["--json", "project", "list"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["prefix"], "PRJ");
}

#[test]
fn undo_on_empty_log_exits_3() {
    let (_d, db) = isolated_db();
    let assert = cli(&db).args(["undo"]).assert().failure();
    assert.code(3);
}
