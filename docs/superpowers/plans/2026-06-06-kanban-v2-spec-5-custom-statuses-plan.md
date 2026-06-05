# Kanban v2 — Spec #5 Custom Statuses Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** User-editable board columns — `CreateStatus`/`UpdateStatus`/`DeleteStatus`/`ReorderStatus` core operations (undoable), exposed in the GUI and CLI.

**Architecture:** New write ops mirror the existing label ops: `operation.rs` (variants + structs), `apply/mod.rs` (dispatch/op_type_name/capture_inverse), new `apply/statuses.rs` (appliers + inverses), `store/write/statuses.rs` + `store/read/statuses.rs` additions, one `apply/snapshot.rs` helper. Tauri `apply` (generic) and MCP need no write changes. GUI adds `ops.ts` builders + board column management; CLI adds `status` write subcommands.

**Tech:** Rust 2024 / rusqlite; React 19 / Vite / Tailwind (Spec #2 stack). TDD throughout.

**Spec:** `docs/superpowers/specs/2026-06-06-kanban-v2-spec-5-custom-statuses-design.md`

## Conventions
- Rust via `~/.cargo/bin/cargo`. UI via `cd ui && PATH="/opt/homebrew/bin:$PATH" npx --yes pnpm@9.12.0 <script>` (if `build` fails on `@rollup/rollup-darwin-arm64`, run `… pnpm@9.12.0 install --force`). CONFIRM each gate's REAL exit code (`cmd; echo EXIT=$?`); never trust a piped grep/tail.
- Gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; ui `typecheck`/`lint`/`test:unit`/`build`. No `Co-Authored-By`. Copy clippy-allow headers from existing test files (`tests/apply_labels.rs`).
- Core `Operation` enum is `#[serde(tag="op", content="args")]`. `StatusCategory` (`unstarted|started|blocked|completed|discarded`) has `as_str()`. `validate::nonempty_field`, `validate::hex_color`, `Error::{Validation,NotFound{kind:EntityKind::Status,id},Conflict(String)}`, `ConflictPolicy::Overwrite`, `Workspace::open_in_memory`, `new_id` all exist. `apply/snapshot.rs::import` already upserts `snapshot.statuses`.

---

### Task 1: Core — `CreateStatus` + `UpdateStatus` (test-first)

**Files:** `crates/kanban-core/src/operation.rs`, `src/apply/mod.rs`, new `src/apply/statuses.rs`, `src/store/write/statuses.rs`, `src/store/read/statuses.rs`, new `tests/apply_statuses.rs`.

- `operation.rs`: add enum variants `CreateStatus(CreateStatus)`, `UpdateStatus(UpdateStatus)` (also declare `DeleteStatus`/`ReorderStatus` variants now to keep the enum stable, even though their appliers land in Task 2 — OR add all four in Task 1 and implement appliers incrementally; pick one and keep `cargo build` green). Structs: `CreateStatus{ id:Uuid, project_id:Uuid, name:String, category:StatusCategory, color:String, position:i64 }`, `UpdateStatus{ id:Uuid, patch:StatusPatch }`, `StatusPatch{ name:Option<String>, category:Option<StatusCategory>, color:Option<String> }` (derive `Default`). Use `crate::types::StatusCategory`.
- `store/read/statuses.rs`: add `pub(crate) fn by_id_via_tx(tx, id) -> Result<Status>` (reuse `row_to_status`; `QueryReturnedNoRows -> Error::NotFound{kind:EntityKind::Status,...}`).
- `store/write/statuses.rs`: add `insert(tx,id,project_id,name,category,color,position)`, `update_fields(tx,id,name:Option<&str>,category:Option<StatusCategory>,color:Option<&str>)` (one UPDATE per Some field).
- `apply/statuses.rs` (new): `create` (validate name+color, reject duplicate `(project_id,name)` with `Error::Conflict`, then `write::statuses::insert`), `update` (validate touched fields, `write::statuses::update_fields`), `inverse_of_create(args) -> DeleteStatus{id}`, `inverse_of_update(tx,args)` (read current via `by_id_via_tx`, build a `StatusPatch` echoing the prior values of the touched fields).
- `apply/mod.rs`: `pub(crate) mod statuses;`; add `CreateStatus`/`UpdateStatus` arms to `dispatch`, `op_type_name`, `capture_inverse`.
- `tests/apply_statuses.rs`: create-inserts; create-rejects-duplicate-name; update-renames; create-undo-removes; update-undo-restores-name. (apply→assert→undo→assert-restored.)

- [ ] Write failing tests → implement → `cargo test -p kanban-core`, `clippy`, `fmt` green. Commit `feat(core): add CreateStatus and UpdateStatus operations`.

### Task 2: Core — `DeleteStatus` (guarded) + `ReorderStatus` (test-first)

**Files:** `apply/statuses.rs`, `store/write/statuses.rs`, `apply/snapshot.rs`, `apply/mod.rs`, `tests/apply_statuses.rs`.

- `store/write/statuses.rs`: add `delete(tx,id)` and `update_position(tx,id,pos)`.
- `apply/snapshot.rs`: add `pub(crate) fn export_status_row(tx, status_id) -> Result<WorkspaceSnapshot>` (a snapshot with just that one status) and `pub(crate) fn export_project_statuses(tx, project_id) -> Result<WorkspaceSnapshot>` (all of a project's statuses). Both use `WorkspaceSnapshot{ schema_version:SNAPSHOT_SCHEMA_VERSION, exported_at:Utc::now(), projects:vec![], statuses, issues:vec![], labels:vec![], issue_labels:vec![] }`.
- `apply/statuses.rs`:
  - `delete(tx,args)`: read the status (`by_id_via_tx`) for its `project_id`; guard 1 — `SELECT COUNT(*) FROM issues WHERE status_id=?` > 0 → `Error::Conflict("status still has N issue(s); move or delete them first")`; guard 2 — `SELECT COUNT(*) FROM statuses WHERE project_id=?` <= 1 → `Error::Conflict("a project must keep at least one status")`; else `write::statuses::delete`.
  - `inverse_of_delete(tx,args)`: `ImportSnapshot{ snapshot: export_status_row(tx, args.id)?, policy: Overwrite }` (captured pre-delete — `capture_inverse` runs before `dispatch`).
  - `reorder(tx,args)`: read status; load project statuses ordered by position (`read::statuses::for_project_via_tx`); remove the target, clamp `new_position` into `0..=len`, insert at that index, then `update_position` for every status to its new index `0..n-1`.
  - `inverse_of_reorder(tx,args)`: `ImportSnapshot{ snapshot: export_project_statuses(tx, project_id)?, policy: Overwrite }` (captures all prior positions).
- `apply/mod.rs`: add `DeleteStatus`/`ReorderStatus` arms to `dispatch`, `op_type_name`, `capture_inverse`.
- `tests/apply_statuses.rs`: delete-empty-undo-restores; delete-blocked-when-issues-present (create issue in the status → Conflict); delete-blocked-when-last (single status → Conflict); reorder-changes-order; reorder-undo-restores-order (3 columns, move, undo, assert original order).

- [ ] Write failing tests → implement → `cargo test --workspace`, `clippy`, `fmt` green. Commit `feat(core): add DeleteStatus (guarded) and ReorderStatus operations`.

### Task 3: GUI — status op builders + board column management (test-first)

**Files:** `ui/src/data/ops.ts` (+ test), board components (`ui/src/components/board/BoardColumn.tsx` + a new `ColumnMenu`/`AddColumn`), `ui/src/data/mutations.ts` (verify `qk.statuses(prefix)` invalidation on status ops), tests.

- `ops.ts`: `createStatus({id,project_id,name,category,color,position})`, `updateStatus({id,name?,category?,color?})` (→ `{op:"UpdateStatus",args:{id,patch:{…}}}`), `deleteStatus({id})`, `reorderStatus({id,new_position})`. Verify shapes vs `operation.rs`. Tests.
- `mutations.ts`: ensure `invalidateFor` invalidates `qk.statuses(prefix)` for `Create/Update/Delete/ReorderStatus` (and `qk.issues(prefix)` since a status delete/move can affect the board). Add the branch if missing.
- Board: `BoardColumn` header gains a small menu — **Rename** (inline input → `updateStatus({id,name})`), **Recolor** (color input → `updateStatus({id,color})`), **Delete** (→ `deleteStatus({id})`; the button shows a toast/inline error when the apply rejects — non-empty or last). **Move left/right** buttons → `reorderStatus({id,new_position: currentIndex∓1})`. An **"+ Add column"** affordance at the board end → a small form (name + category select from the 5 values + color) → `createStatus({ id:uuidv7(), project_id, name, category, color, position: columns.length })`.
- Tests (Vitest+RTL): builders; the column menu dispatches the right ops (rename/recolor/delete/move); add-column dispatches `CreateStatus`. Mock `useApply`. No `any`.

- [ ] Test-first → implement → ui `typecheck`/`lint`/`test:unit`/`build` green (confirm EXIT=0). Commit `feat(ui): add, rename, recolor, reorder and delete board columns`.

### Task 4: CLI — `status` write subcommands (test-first)

**Files:** `crates/kanban-cli/src/cmd/status.rs`, snapshot tests.

- Extend `StatusSub` with `Create{project,name,category,color,position?}`, `Update{id,name?,category?,color?}`, `Delete{id}`, `Reorder{id,position}` (mirror `cmd/label.rs`). A `parse_category(&str)->Result<StatusCategory>` helper. Each resolves the project (for Create), builds the op, calls `ws.apply`, prints the result (reuse the list/table formatting).
- `insta` snapshots: create happy-path + a duplicate-name error; delete happy + blocked (non-empty) error.

- [ ] Test-first → `cargo test --workspace` green. Commit `feat(cli): add status create/update/delete/reorder subcommands`.

### Task 5: Acceptance + docs + PR

- [ ] Full gates: `cargo fmt/clippy/test --workspace`; ui `typecheck/lint/test:unit/build`; bindings drift clean.
- [ ] Update README/CLAUDE/DEVELOPMENT to note custom statuses (and that members/assignees remain Spec #6).
- [ ] Commit; superpowers:finishing-a-development-branch → PR to `dev`.

## Self-review notes
- All four ops are undoable: Create→Delete, Update→Update(prior), Delete→ImportSnapshot(row), Reorder→ImportSnapshot(project statuses). `capture_inverse` runs before `dispatch`, so delete/reorder inverses capture pre-state.
- Guards live in the applier (`dispatch`), so a blocked delete rolls back the whole transaction (inverse already captured is discarded). Block-until-empty + keep-≥1 per the design decision.
- No migration; no DTO change (StatusDto already exists) → no bindings regen expected (verify the drift guard stays clean).
- Verify every op struct's field set against `operation.rs` before building `ops.ts` builders / CLI args.
