//! Cold-start performance smoke test.
//!
//! This test is `#[ignore]` by default because it measures wall-clock timing
//! which is inherently noisy on shared CI machines. Run it locally with:
//!
//! ```sh
//! cargo test --workspace -- --ignored
//! ```
//!
//! For results closer to real-world conditions (no debug-overhead), prefer:
//!
//! ```sh
//! cargo test --workspace --release -- --ignored
//! ```

#![allow(clippy::unwrap_used)]
#![allow(clippy::cast_possible_truncation)]

use assert_cmd::Command;
use std::path::PathBuf;
use std::time::Instant;

/// Asserts that 40 cold-start invocations complete in <50ms p95.
///
/// This test is ignored by default to avoid flakiness on CI. Run locally with
/// `cargo test --workspace -- --ignored` or `cargo test --workspace --release -- --ignored`.
#[test]
#[ignore = "measures wall-clock timing; noisy on shared CI — run locally with `cargo test --workspace -- --ignored`"]
fn cold_start_invocation_under_50ms_p95() {
    let dir = tempfile::tempdir().unwrap();
    let db: PathBuf = dir.path().join("data.db");
    Command::cargo_bin("kanban")
        .unwrap()
        .env("KANBAN_DB", &db)
        .args(["project", "create", "X", "--prefix", "PRF"])
        .assert()
        .success();

    let mut times = Vec::new();
    for _ in 0..40 {
        let start = Instant::now();
        Command::cargo_bin("kanban")
            .unwrap()
            .env("KANBAN_DB", &db)
            .args(["project", "list"])
            .assert()
            .success();
        times.push(start.elapsed().as_millis() as u64);
    }
    times.sort_unstable();
    let p95 = times[(times.len() * 95) / 100];
    assert!(
        p95 < 50,
        "cold-start p95 {p95}ms exceeds 50ms target; full sample: {times:?}"
    );
}
