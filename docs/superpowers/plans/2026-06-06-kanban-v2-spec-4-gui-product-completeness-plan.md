# Kanban v2 — Spec #4 GUI Product Completeness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Close the daily-driver gaps in the desktop GUI — issue priority/due-date editing, delete, labels, undo/redo (⌘Z), and project rename/archive/delete — all on top of the existing `kanban-core` operations.

**Architecture:** Frontend-first. Writes go through the existing `useApply` → `apply(Operation)`. New operations are `ops.ts` builders only (no new Tauri write commands). One small core read (`query_labels_for_issue`) + a `get_issue` DTO `labels` field (regenerates the git-tracked `bindings.ts`). Undo/redo wrap the existing `commands.undo`/`redo`.

**Tech Stack:** React 19 / Vite / Tailwind v4 / TanStack Query (Spec #2 stack). Rust 2024 / rusqlite (core). Tests: Vitest + RTL (test-first), `cargo test` for the core helper.

**Spec:** `docs/superpowers/specs/2026-06-06-kanban-v2-spec-4-gui-product-completeness-design.md`

## Conventions (every task)
- UI commands run under the arm64 node: `cd ui && PATH="/opt/homebrew/bin:$PATH" npx --yes pnpm@9.12.0 <script>` (test:unit / typecheck / lint / build).
- Rust via `~/.cargo/bin/cargo`. Gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; ui `typecheck`/`lint`/`test:unit`/`build`. CONFIRM each gate exits 0 explicitly (don't trust a tail).
- ESLint: `@typescript-eslint/no-explicit-any` is an ERROR — no `as any` anywhere; unused vars prefixed `_`. Use `vi.hoisted` for any mock referenced in a hoisted `vi.mock` factory.
- No `Co-Authored-By` / AI attribution in commits. Conventional-commit prefixes.

---

### Task 1: Operation builders for labels + projects (`ops.ts`) — test-first

**Files:** `ui/src/data/ops.ts`, `ui/tests/data/ops.test.ts`

Add builders producing the exact `{op, args}` serde shapes (verify against `crates/kanban-core/src/operation.rs`): `createLabel({id, project_id, name, color})`, `updateLabel({id, name?, color?})`, `deleteLabel({id})`, `attachLabel({issue_id, label_id})`, `detachLabel({issue_id, label_id})`, `updateProject({id, ...})`, `archiveProject({id})`, `deleteProject({id})`. (`deleteIssue` already exists.) Verify each struct's fields against core (e.g. `CreateLabel{id,project_id,name,color}`, `AttachLabel{issue_id,label_id}`, `ArchiveProject`/`DeleteProject` — confirm whether they take `{id}` only). Test the produced shapes.

- [ ] Write failing `ops.test.ts` cases for the new builders (assert `{op, args}`).
- [ ] Implement the builders.
- [ ] `test:unit -- ops`, typecheck, lint green. Commit `feat(ui): add label and project operation builders`.

### Task 2: Priority editor in the detail panel

**Files:** `ui/src/components/detail/IssuePanel.tsx` (+ a small `PrioritySelect` component if cleaner), test.

Add a priority control to `IssuePanel` (segmented buttons or a `<select>`, values none/low/medium/high/urgent) bound to the issue's current priority; on change → `apply.mutate(ops.updateIssueField({ id, change: change.priority(value) }))`. Test (RTL): rendering with a mocked `useIssue`/`useApply`, changing priority dispatches `UpdateIssueField` with `change == {field:"Priority", value:<v>}`.

- [ ] Test-first; implement; gates green. Commit `feat(ui): edit issue priority in the detail panel`.

### Task 3: Due-date editor in the detail panel

**Files:** `IssuePanel.tsx`, test.

Add a date `<input type="date">` (value = issue.due_date or empty; clearable) → `apply.mutate(ops.updateIssueField({ id, change: change.dueDate(value || null) }))`. Test: setting a date dispatches `{field:"DueDate", value:"YYYY-MM-DD"}`; clearing dispatches `value:null`.

- [ ] Test-first; implement; gates green. Commit `feat(ui): edit issue due date in the detail panel`.

### Task 4: Delete-issue action in the detail panel

**Files:** `IssuePanel.tsx`, test.

Add a destructive "Delete issue" button (with an inline confirm) → `apply.mutate(ops.deleteIssue({ id }))`, then `navigate(\`/p/${prefix}\`)` to close the panel/return to the board. Test: confirm → dispatches `DeleteIssue` + navigates.

- [ ] Test-first; implement; gates green. Commit `feat(ui): delete an issue from the detail panel`.

### Task 5: Undo/redo hooks + ⌘Z / ⌘⇧Z

**Files:** `ui/src/data/mutations.ts` (or `queries.ts`), `ui/src/routes/_layout.tsx` (or a small `useKeyboardUndo` hook), tests.

- `useUndo()` / `useRedo()` — `useMutation` calling `commands.undo()` / `commands.redo()` (unwrap the Result), `onSettled: () => qc.invalidateQueries()` (broad). A "nothing to undo" error is swallowed/toasted, not thrown to the UI.
- A global keyboard handler (mounted once, e.g. in `_layout.tsx` via `useEffect` on `window` keydown): `⌘Z` (metaKey+z, no shift) → undo; `⌘⇧Z` → redo. Show a brief toast/status line.
- Tests: the hooks call the right command + invalidate; the key handler maps the right combos (can unit-test the handler logic).

- [ ] Test-first; implement; gates green. Commit `feat(ui): wire ⌘Z / ⌘⇧Z undo and redo`.

### Task 6: Core `query_labels_for_issue` + `get_issue` labels + regen bindings

**Files:** `crates/kanban-core/src/workspace.rs` (+ `store/read/labels.rs`), `crates/kanban-core/tests/`, `crates/kanban-tauri/src/{commands.rs,dto.rs}`, regenerate `ui/src/data/bindings.ts`.

- Core (test-first): `Workspace::query_labels_for_issue(&self, issue_id: Uuid) -> Result<Vec<Label>>` — `SELECT label_id FROM issue_labels WHERE issue_id=?` joined to labels (or reuse `labels::for_project` + filter). Unit test: attach two labels, assert both returned.
- Tauri: a `LabelDto` already exists; make `get_issue` return an `IssueDetailDto { #[serde(flatten)] issue: IssueDto, labels: Vec<LabelDto> }` (or add a `labels: Vec<LabelDto>` field to a detail struct) by calling `query_labels_for_issue` in `get_issue_inner`. Keep `list_issues` returning plain `IssueDto` (no per-issue label load for the board in this spec).
- Regenerate bindings: `cargo test -p kanban-tauri --test export_bindings` (or the binary's debug export); commit the updated `ui/src/data/bindings.ts`.

- [ ] Test-first core helper; wire the DTO; regen bindings; `cargo test --workspace` + ui typecheck green (bindings drift guard clean). Commit `feat(core): expose an issue's labels via get_issue`.

### Task 7: Label display + attach/detach + create in the detail panel

**Files:** `IssuePanel.tsx` (+ a `LabelPicker` component), `ui/src/data/queries.ts` (a `useLabels(prefix)` hook already exists — reuse), tests.

- Show the issue's labels (from the `get_issue` detail) as colored chips.
- A label picker: lists the project's labels (`useLabels`), each toggling attach/detach via `ops.attachLabel`/`ops.detachLabel` (using `issue.id` + `label.id`); on toggle, `apply.mutate` and invalidate the issue.
- A "+ New label" affordance (name + a color) → `ops.createLabel({ id: uuidv7(), project_id, name, color })`, then it appears in the picker.
- Test: rendering shows existing chips; toggling a label dispatches attach/detach with the right ids; creating dispatches `CreateLabel`.

- [ ] Test-first; implement; gates green. Commit `feat(ui): label chips, attach/detach, and create in the detail panel`.

### Task 8: Project rename / archive / delete from the sidebar

**Files:** `ui/src/components/sidebar/ProjectList.tsx` (+ a `ProjectMenu`/`ProjectSettingsDialog`), tests.

- Per-project affordance (hover menu or a settings dialog) to: **rename** (`ops.updateProject({ id, name })`), **archive** (`ops.archiveProject({ id })`), **delete** (`ops.deleteProject({ id })`, with confirm). Verify `UpdateProject`/`ArchiveProject`/`DeleteProject` arg shapes against core.
- After delete/archive of the active project, navigate to `/`.
- Test: rename dispatches `UpdateProject`; delete (confirmed) dispatches `DeleteProject` and navigates.

- [ ] Test-first; implement; gates green. Commit `feat(ui): rename, archive, and delete projects from the sidebar`.

### Task 9: Acceptance + docs

- [ ] Full gates: `cargo fmt/clippy/test --workspace`; ui `typecheck`/`lint`/`test:unit`/`build`; bindings drift clean.
- [ ] Update README/CLAUDE feature lists to reflect the now-complete GUI (labels, priority/due editing, delete, undo shortcut, project management). Note custom statuses / members deferred to a later spec.
- [ ] Commit, then superpowers:finishing-a-development-branch → PR to `dev`.

## Self-review notes
- Every write reuses `useApply` → `apply` → `Workspace::apply` (single-mutator invariant). Only `ops.ts` gains builders; only `get_issue` gains a `labels` field (one bindings regen). No new migration.
- Deferred (need new core ops/schema): custom statuses, members/assignee, hierarchy, multi-views — flagged for Spec #5+.
- Verify each op's arg struct against `operation.rs` before building its `ops.ts` builder (don't assume field sets).
