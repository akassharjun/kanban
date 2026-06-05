# Spec #5 — Custom Statuses

**Date:** 2026-06-06
**Builds on:** Spec #1 (core + CLI), Spec #2 (GUI), Spec #3 (MCP), Spec #4 (GUI completeness)
**Roadmap:** E11 Custom Statuses — let users add / rename / recolor / reorder / delete the board's status columns.

## Goal

Make the board's columns user-editable. Today the 7 default statuses are seeded at project creation and are immutable (no status `Operation` exists). This spec adds four core operations — `CreateStatus`, `UpdateStatus`, `DeleteStatus`, `ReorderStatus` — fully undoable, then exposes them in the GUI (board column management) and the CLI. The Tauri `apply` command and the MCP server need no changes for writes (both go through `Workspace::apply`).

## Decisions

- **Delete is guarded (block-until-empty).** `DeleteStatus` refuses (`Error::Conflict`) if the status still has issues — the user must move/delete them first. It also refuses if it is the project's **last** status (a project always has ≥1 column). Fully recoverable via undo (the status row is captured in the inverse before removal).
- **No migration.** The `statuses` table (`id, project_id, name, category, color, position`, `UNIQUE(project_id,name)`) already exists. `category` stays one of `unstarted|started|blocked|completed|discarded`.
- **Reorder re-packs integer positions.** `ReorderStatus { id, new_position }` moves the status to the target index and renumbers the project's columns `0..n-1` in one transaction. Its inverse is an `ImportSnapshot` of the project's statuses captured *before* the move (restores every column's prior position exactly) — `snapshot::import` already upserts statuses.

## Architecture (mirrors the existing label ops)

Core `Operation` is `#[serde(tag="op", content="args")]`, so the GUI/CLI/MCP all build `{op, args}` and `Workspace::apply` dispatches. New variants flow through the same machinery: `apply/mod.rs` (`dispatch`, `op_type_name`, `capture_inverse`), a new `apply/statuses.rs` (appliers + `inverse_of_*`), additions to `store/write/statuses.rs` (`insert`/`delete`/`update_fields`/`update_position`) and `store/read/statuses.rs` (`by_id_via_tx`), and one snapshot helper for the delete/reorder inverses. Undo/redo, the operation log, and activity come for free.

### Operations
- `CreateStatus { id, project_id, name, category, color, position }` — validate (non-empty name, hex color, unique `(project_id,name)`); inverse `DeleteStatus{id}`.
- `UpdateStatus { id, patch: StatusPatch{ name?, category?, color? } }` — rename/recategorize/recolor; inverse re-applies the prior values for the touched fields.
- `DeleteStatus { id }` — guarded (no issues, not the last); inverse `ImportSnapshot` of the captured row.
- `ReorderStatus { id, new_position }` — re-pack; inverse `ImportSnapshot` of the project's prior status ordering.

### GUI (board)
`ui/src/data/ops.ts` gains `createStatus`/`updateStatus`/`deleteStatus`/`reorderStatus` builders. The board (`BoardColumn` header) gets: a per-column menu (rename, recolor, delete — delete disabled/blocked with a toast if non-empty or last) and an "+ Add column" affordance; reorder via left/right move buttons (or drag — buttons are simpler and testable). Category is chosen from the five fixed values when adding/editing. Writes go through `useApply`; `invalidateFor` already invalidates `qk.statuses(prefix)` (verify) so the board refetches.

### CLI
`kanban status` gains `create`/`update`/`delete`/`reorder` subcommands (mirroring `kanban label`), each building the op and calling `apply`, with happy-path + error snapshots.

## Non-goals / deferred
- MCP status-management tools (the MCP server targets issue workflows; status setup via GUI/CLI suffices) — note for a later spec.
- Drag-and-drop column reordering (buttons first).
- Members/assignees — **Spec #6** (needs a migration).

## Testing
- Core: `tests/apply_statuses.rs` — create / duplicate-name conflict / update-rename / delete-undo-restores / delete-blocked-when-nonempty / delete-blocked-when-last / reorder-changes-order / reorder-undo-restores-order. Each follows the apply→assert→undo→assert-restored cycle.
- CLI: `insta` snapshots (happy + error) via `assert_cmd`.
- GUI: Vitest + RTL for the ops builders and the board column management component(s).
- All gates green: `cargo fmt/clippy/test --workspace`; ui `typecheck/lint/test:unit/build`; bindings drift clean (no DTO change expected — statuses already have a DTO).

## Acceptance
A user can, from the GUI or CLI: add a new column (name + category + color), rename/recolor it, reorder columns, and delete an empty column (blocked with a clear message if it still holds issues or is the last column) — every change undoable.
