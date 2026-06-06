# Kanban — Project Brain

Local-first project management. v1 covers projects, issues, labels, default statuses, search, sort, undo/redo, and JSON I/O via a Rust core library and CLI. The GUI (Tauri, Spec #2) and the MCP server (Spec #3) have shipped; AI-agent orchestration follows in later specs.

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
- **No live cross-process watcher.** The GUI refetches on window focus (`refetchOnWindowFocus`); a SQLite `update_hook` watcher is deferred to a later spec.
- **E2E (`tauri-driver`) is deferred to a later spec** (no macOS WebDriver support); Spec #2 ships Vitest + RTL coverage. See `ui/e2e/README.md`.
- **UI tooling on Apple Silicon:** if the default `node` is x86_64 (nvm), run ui scripts under a native arm64 node (`PATH="/opt/homebrew/bin:$PATH" npx pnpm@9.12.0 …`) or rollup/esbuild crash. If `node_modules` ends up with the wrong-arch rollup binary (a plain `install` reports "up to date" but `build` fails on `@rollup/rollup-darwin-arm64`), repair with `… pnpm@9.12.0 install --force`. `test:unit` can pass while `build` fails, so always run `build`. See `DEVELOPMENT.md`.
- **GUI product completeness (Spec #4).** Issue detail-panel editing of priority, due date, and labels (chips + attach/detach/create), issue delete, ⌘Z/⌘⇧Z undo-redo (wrapping `commands.undo`/`redo` + broad query invalidation), and project rename/archive/delete from the sidebar — all via `ops.ts` builders through `apply`. The one core addition is `Workspace::query_labels_for_issue`, surfaced by `get_issue` as `IssueDto.labels: Option<Vec<LabelDto>>` (None for `list_issues`). Members/assignee (not in schema) remain deferred to Spec #6.

- **Custom statuses (Spec #5).** Four undoable core ops — `CreateStatus`, `UpdateStatus` (via `StatusPatch`), `DeleteStatus`, `ReorderStatus` — mirroring the label ops (`apply/statuses.rs` + `store/{write,read}/statuses.rs` + `apply/mod.rs` arms; no migration, no DTO change). **`DeleteStatus` is guarded:** it refuses (`Error::Conflict`) if the status still has issues or is the project's last column (block-until-empty). `ReorderStatus` re-packs the project's columns to dense `0..n` positions; its undo inverse is an `ImportSnapshot` of the project's prior status ordering (delete's inverse snapshots the single row). Exposed in the GUI (board column add/rename/recolor/reorder/delete) and the CLI (`kanban status create/update/delete/reorder`); the generic Tauri `apply` and the MCP server need no changes.

- **Members & assignees (Spec #6).** Migration `0003_members.sql` adds a per-project `members` table + a nullable `issues.assignee_id` (`REFERENCES members(id) ON DELETE SET NULL`). Ops `CreateMember`/`UpdateMember`/`DeleteMember` mirror the label ops; **assignment reuses `UpdateIssueField` via `IssueFieldChange::Assignee(Option<Uuid>)`** (validated: the member must be in the issue's project). `WorkspaceSnapshot` gained `members` and bumped to **schema v2**; `DeleteMember`'s undo inverse is an `ImportSnapshot` capturing the member + its assigned issues **and their `issue_labels`** (the Overwrite re-import tears down + restores those rows). `Issue` gained `assignee_id` — it threads through `row_to_issue`, every snapshot issue read/write, and `IssueDto`. Exposed in the GUI (assignee picker + member roster in the detail panel) and the CLI (`kanban member …`, `kanban issue assign`). MCP member tools are deferred to Spec #7.

## MCP layer (Spec #3)

The MCP server is `crates/kanban-mcp` — a stdio server (on `rmcp`) exposing `kanban-core` to AI assistants (Claude Desktop/Code). Design: `docs/superpowers/specs/2026-06-05-kanban-v2-spec-3-mcp-server-design.md`.

- Opens the default workspace (`Workspace::open_default`) — shares `~/.kanban/data.db` with the CLI and GUI (SQLite WAL, multi-process safe).
- Each async `#[tool]` runs the sync core inside `tokio::task::spawn_blocking` over an `Arc<Mutex<Workspace>>` (the `blocking_read`/`blocking_mut` helpers); the guard never crosses `.await`. **Writes go through `Workspace::apply`** — single-mutator invariant preserved.
- **Semantic tools**, not a generic apply: reads (`list_projects`, `list_statuses`, `list_labels`, `list_issues`, `get_issue`, `search_issues`) + writes (`create_project`, `create_issue`, `update_issue`, `move_issue`, `undo`, `redo`). Tools resolve human identifiers — project `prefix`, issue `key`, status/label `name` — to core UUIDs; misses map to `not_found`/`unknown_name` MCP errors. Output DTOs (`convert.rs`) are human-facing (no UUIDs/sort_keys).
- Resolution uses the existing unique indexes via `Workspace::query_project_by_prefix`/`query_issue_by_identifier` (no new migration). `move_issue` computes a fractional sort-key midpoint (1024-gap, like the GUI). Tested per-tool over an in-process `tokio::io::duplex` transport.
- **Agent orchestration (registry, task contracts, decomposition, validation) is deferred to Spec #4+.**

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

- `docs/superpowers/specs/2026-05-03-kanban-v2-core-cli-design.md` — the v1 (core + CLI) design.
- `docs/superpowers/plans/2026-05-03-kanban-v2-core-cli-plan.md` — the 41-task TDD plan that built it.
- `docs/superpowers/specs/2026-05-05-kanban-v2-spec-2-gui-shell-design.md` — Spec #2 (Tauri GUI shell).
- `docs/superpowers/specs/2026-06-05-kanban-v2-spec-3-mcp-server-design.md` — Spec #3 (MCP server).
- `docs/superpowers/specs/2026-06-06-kanban-v2-spec-4-gui-product-completeness-design.md` — Spec #4 (GUI product completeness).
- `docs/superpowers/specs/2026-06-06-kanban-v2-spec-5-custom-statuses-design.md` — Spec #5 (custom statuses).
- `docs/superpowers/specs/2026-06-06-kanban-v2-spec-6-members-assignees-design.md` — Spec #6 (members & assignees).

Each spec has a matching plan in `docs/superpowers/plans/`. AI-agent orchestration, MCP member tools, and other follow-ups (Spec #7+) will live alongside.
