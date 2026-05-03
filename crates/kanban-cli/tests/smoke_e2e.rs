//! End-to-end smoke test: create project, label, issues, attach, export, undo.
//!
//! This exercises the full happy path through the CLI binary without inspecting
//! snapshot details — it just confirms all commands exit successfully and that
//! the JSON output matches expectations.

#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use std::path::PathBuf;

fn isolated_db() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    (dir, path)
}

#[test]
fn full_v1_smoke_path() {
    let (_d, db) = isolated_db();
    let env = ("KANBAN_DB", db.to_string_lossy().to_string());
    let run = |args: &[&str]| {
        Command::cargo_bin("kanban")
            .unwrap()
            .env(env.0, &env.1)
            .args(args)
            .assert()
            .success()
    };

    run(&["project", "create", "Smoke", "--prefix", "SMK"]);
    run(&[
        "label",
        "create",
        "--project",
        "SMK",
        "--name",
        "feat",
        "--color",
        "#3b82f6",
    ]);
    run(&["issue", "create", "--project", "SMK", "--title", "first"]);
    run(&["issue", "create", "--project", "SMK", "--title", "second"]);
    run(&["issue", "create", "--project", "SMK", "--title", "third"]);
    run(&["label", "attach", "SMK-1", "feat"]);

    let out = Command::cargo_bin("kanban")
        .unwrap()
        .env(env.0, &env.1)
        .args(["--json", "issue", "list", "--project", "SMK"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 3);

    let dir = tempfile::tempdir().unwrap();
    let snap = dir.path().join("snap.json");
    run(&["export", "-o", snap.to_str().unwrap()]);
    assert!(snap.exists());

    // Undo all the way back: label_attach, issue×3, label_create, project_create = 6 ops.
    for _ in 0..6 {
        run(&["undo"]);
    }
    let out = Command::cargo_bin("kanban")
        .unwrap()
        .env(env.0, &env.1)
        .args(["--json", "project", "list"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
}
