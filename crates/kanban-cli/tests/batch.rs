//! Integration tests for `kanban batch` (NDJSON stdin).

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use assert_cmd::Command;

#[test]
fn batch_runs_create_then_list() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");

    let input = r#"{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
{"cmd":"issue.create","args":{"project":"BAT","title":"first"}}
{"cmd":"issue.list","args":{"project":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban")
        .unwrap()
        .env("KANBAN_DB", &db)
        .args(["batch"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<_> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(lines.len(), 3);
    let last: serde_json::Value = serde_json::from_str(&lines[2]).unwrap();
    assert_eq!(last["ok"], true);
    assert_eq!(last["data"][0]["title"], "first");
}

#[test]
fn batch_continues_past_error_unless_fail_fast() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");
    let input = r#"{"cmd":"project.show","args":{"id_or_prefix":"NONE"}}
{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban")
        .unwrap()
        .env("KANBAN_DB", &db)
        .args(["batch"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<_> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(lines.len(), 2);
    let l0: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    let l1: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
    assert_eq!(l0["ok"], false);
    assert_eq!(l1["ok"], true);
    assert_eq!(l1["data"]["prefix"], "BAT");
}

#[test]
fn batch_fail_fast_aborts_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");
    let input = r#"{"cmd":"project.show","args":{"id_or_prefix":"NONE"}}
{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban")
        .unwrap()
        .env("KANBAN_DB", &db)
        .args(["batch", "--fail-fast"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(!out.status.success());
    // Project should NOT have been created.
    let out2 = Command::cargo_bin("kanban")
        .unwrap()
        .env("KANBAN_DB", &db)
        .args(["--json", "project", "list"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
}
