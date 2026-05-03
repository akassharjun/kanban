# Development

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## TDD discipline

Every `Operation` variant lands as a failing test first. Every CLI subcommand has a happy-path snapshot and at least one error-path snapshot. The CI gate runs the full test suite — no `#[ignore]` lands in `main` without a comment explaining why.

## Migrations

SQL files in `crates/kanban-core/migrations/NNNN_name.sql` are embedded with `include_str!` and applied in numeric order on `Workspace::open`. **`0001_init.sql` is frozen after v1 ships** — schema changes go in `0002_*.sql` and beyond. Never edit a released migration.

## Snapshot tests

CLI tests use [`insta`](https://insta.rs). After a change to expected output:

```sh
cargo insta review --workspace-root crates/kanban-cli
```

Approve or reject each diff hunk. Snapshots live in `crates/kanban-cli/tests/snapshots/`.

## Performance

`crates/kanban-cli/tests/perf.rs` checks cold-start <50ms p95. It's `#[ignore]` by default; run locally with `cargo test --workspace -- --ignored`.
