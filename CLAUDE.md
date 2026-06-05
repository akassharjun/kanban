# Kanban — Project Brain

Local-first project management. v1 covers projects, issues, labels, default statuses, search, sort, undo/redo, and JSON I/O via a Rust core library and CLI. GUI (Tauri), MCP server, and AI-agent orchestration ship in later specs.

## Stack

- **Edition:** Rust 2024, MSRV 1.85
- **Workspace:** two crates — `kanban-core` (sync library, `rusqlite`) and `kanban-cli` (`kanban` binary, `clap`)
- **Storage:** SQLite WAL, hand-rolled migrations in `crates/kanban-core/migrations/NNNN_name.sql`
- **Search:** SQLite FTS5 (porter+unicode61), kept in sync via triggers
- **Tests:** `cargo test` (unit + integration), `proptest` for invariants, `insta` for CLI snapshots, `assert_cmd` for binary tests

## Quick reference

```sh
~/.cargo/bin/cargo test --workspace                                       # /test
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings        # /lint (part 1)
~/.cargo/bin/cargo fmt --all -- --check                                   # /lint (part 2)
~/.cargo/bin/cargo check --workspace                                      # /check (fast)
~/.cargo/bin/cargo test --workspace --release -- --ignored                # /perf
```

(The shell alias `cargo` may not be on PATH in some environments. Use `~/.cargo/bin/cargo` if `cargo: command not found`.)

## Architecture invariants — do not break

- **Single public mutator.** All writes go through `Workspace::apply(Operation)`. The `kanban_core::store::write` module is `pub(crate)` only; the compiler enforces it. Never expose write functions outside the crate.
- **Sync core.** `kanban-core` is sync. Future async callers (GUI, MCP) wrap in `tokio::task::spawn_blocking` and own their connection pooling.
- **Migrations are append-only.** `0001_init.sql` is **frozen**. Schema changes go in `0002_*.sql`, `0003_*.sql`, etc. Never edit a released migration.
- **Race-safe identifier allocation.** Each project carries `next_seq INTEGER`; issue creation uses `UPDATE projects SET next_seq = next_seq + 1 ... RETURNING next_seq - 1` in one transaction. Never compute `seq` outside a transaction.
- **Inverse via ImportSnapshot.** `inverse_of_delete_*` returns `Operation::ImportSnapshot { snapshot, policy: Overwrite }` capturing the entity + cascaded children. This preserves status, seq, identifier, sort_key, attachments. Don't regress to `Create*` inverses.
- **Lossless f64 in JSON payloads.** `Operation::ReorderIssue::new_sort_key` and `Issue::sort_key` use the `serde_f64::bits` helper. Don't switch to default serde f64.

## Tauri layer (Spec #2 onward)

The GUI is `crates/kanban-tauri` (Rust shell) + `ui/` (Vite + React 19 + Tailwind v4, pnpm). Design: `docs/superpowers/specs/2026-05-05-kanban-v2-spec-2-gui-shell-design.md`.

- **All DB commands wrap the sync core in `tokio::task::spawn_blocking`**, cloning an `Arc<Mutex<Workspace>>` and locking *inside* the blocking closure — the guard never crosses `.await`. `AppState` holds `Arc<Mutex<Workspace>>`.
- **`apply` is the single mutation command.** It takes the op as `serde_json::Value` (core's `Operation` is not a `specta::Type`), deserializes to `Operation`, and calls `Workspace::apply`. The frontend builds the `{op, args}` shape via `ui/src/data/ops.ts`. The 14 Operation variants remain the only path to issue/project/label/status writes.
- **`workspace_settings`** (migration `0002`) stores app-level prefs (theme) via `Workspace::get_setting`/`set_setting` — a **documented exception** to the single-mutator invariant; settings have no undo semantics.
- **DTOs, not core types, cross the bridge.** `kanban-tauri/src/dto.rs` defines specta `Type` DTOs (`ProjectDto`, `IssueDto`, …) with ids/dates/enums as strings; core types stay specta-free.
- **`tauri-specta` generates `ui/src/data/bindings.ts`** (git-tracked) via the `export_bindings` test; CI fails on drift. `specta-typescript` is configured with `BigIntExportBehavior::Number` so `i64` fields become TS `number`.
- **No live cross-process watcher.** The GUI refetches on window focus (`refetchOnWindowFocus`); a SQLite `update_hook` watcher is deferred to Spec #3.
- **E2E (`tauri-driver`) is deferred to Spec #3** (no macOS WebDriver support); Spec #2 ships Vitest + RTL coverage. See `ui/e2e/README.md`.
- **UI tooling on Apple Silicon:** if the default `node` is x86_64 (nvm), run ui scripts under a native arm64 node (`PATH="/opt/homebrew/bin:$PATH" npx pnpm@9.12.0 …`) or rollup/esbuild crash. See `DEVELOPMENT.md`.

## TDD discipline

- Every `Operation` variant lands as a failing test first (apply on fresh in-memory DB, assert post-state, undo, assert pre-state restored). Then the applier code lands.
- Every CLI subcommand has a happy-path snapshot and at least one error-path snapshot via `insta`.
- The CI gate (`.github/workflows/ci.yml`) runs `cargo fmt --check && cargo clippy -D warnings && cargo test --workspace`. Don't ship anything red.
- Performance smoke (`crates/kanban-cli/tests/perf.rs`) is `#[ignore]`'d by default; run locally with `cargo test --workspace --release -- --ignored`.

## Repository conventions

- **Repo:** `akassharjun/kanban` (origin: `git@github.com:akassharjun/kanban.git`)
- **PRs target `dev`**, never `main`. The only path to `main` is through `dev`.
- **Conventional commits.** Prefixes: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `ci:`. Scope optional (`feat(core):`, `feat(cli):`).
- **No Co-Authored-By lines on commits, ever.** Use `/commit` or `/commit-push-pr` skills.
- **Cargo.lock is committed** (we ship a binary).
- **Worktrees branch from `dev`** (or current working branch), never from `main`.

## Database & runtime

- Default DB path: `~/.kanban/data.db`. Override via `KANBAN_DB` env var.
- Cold-start CLI invocation: ~5–30ms (open + migration check). Fine for interactive use; `kanban batch` keeps one connection open across NDJSON lines for high-throughput piping.
- Multi-process safe via SQLite WAL — each `kanban` invocation opens its own connection, runs in one transaction, exits.

## Known low-priority debt

These don't block v1 but should be addressed before broader release:

- `AttachLabel`/`DetachLabel` inverse is unconditional (programmatic re-attach corner case; CLI is fine).
- Activity log only emits on `apply`, not `undo`/`redo`. `kanban issue history` shows forward changes only.
- `Workspace::open_default` with neither `KANBAN_DB` nor `HOME` set returns exit code 3 (Validation), but it's really a config error.
- `kanban issue list` default human output omits status column (visible in `--json` mode).
- Reorder cross-status edge case: `--before/--after OTHER` doesn't validate that OTHER is in the same status as the issue being moved.

## Spec/plan trail

- `docs/superpowers/specs/2026-05-03-kanban-v2-core-cli-design.md` — the v1 design.
- `docs/superpowers/plans/2026-05-03-kanban-v2-core-cli-plan.md` — the 41-task TDD plan that built it.

Future specs (GUI, MCP, AI-agent orchestration) live alongside in `docs/superpowers/specs/`.
