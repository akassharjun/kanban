# Spec #4 — GUI Product Completeness

**Date:** 2026-06-06
**Builds on:** Spec #1 (core + CLI), Spec #2 (GUI shell), Spec #3 (MCP server)
**Roadmap epics covered:** E08 Labels (GUI) · E16 Shortcuts (⌘Z/⌘⇧Z undo/redo) · E17 Undo/Redo wired to the GUI · issue priority/due-date editing · issue delete · E02 Projects (rename/archive/delete)
**Deferred to Spec #5+ (need new core ops / schema):** E11 Custom Statuses (no `CreateStatus`/`UpdateStatus`/`DeleteStatus` operation exists) · E03 Members/Assignee (not in schema) · E07 Hierarchy · E09 Multi-views · E19 Agent orchestration

## Goal

Close the remaining gaps that keep the desktop GUI from being a complete daily-driver. Every feature here is **already supported by `kanban-core`** — the 14 `Operation` variants cover labels, deletes, and project edits, and the generic `apply` command dispatches any of them. The only backend change is one small read helper for an issue's labels.

## Non-goals

- Custom status columns, members/assignee, hierarchy, multi-views, templates, import/export UI, notifications, audit UI, agent orchestration — all need new core operations or schema and are out of scope.
- No new migration. No new specta-typed command except the `get_issue` DTO gaining a `labels` field (regenerates `bindings.ts`).

## Architecture notes

- Writes still go through the GUI's `useApply` (→ `apply(Operation)` → `Workspace::apply`). New operations are added as `ops.ts` builders only — no new Tauri commands.
- Undo/redo use the existing `commands.undo`/`commands.redo` bindings, wrapped in mutation hooks that invalidate all queries.
- Showing an issue's labels needs core read support: add `Workspace::query_labels_for_issue(issue_id)`, surface it on the detail read (`get_issue` → a richer `IssueDetailDto { ...IssueDto, labels: LabelDto[] }`), and regenerate `bindings.ts` (git-tracked, drift-guarded by CI).

## Features

### 1. Operation builders (`ops.ts`)
Add builders for the ops the GUI now needs (each produces the `{op, args}` shape; `apply` deserializes): `createLabel`, `updateLabel`, `deleteLabel`, `attachLabel`, `detachLabel`, `updateProject`, `archiveProject`, `deleteProject`. (`deleteIssue` already exists.) Test the wire shapes against the core structs (`CreateLabel{id,project_id,name,color}`, `AttachLabel{issue_id,label_id}`, etc.).

### 2. Detail-panel field editors
In `IssuePanel`:
- **Priority** — a small segmented control / select (none/low/medium/high/urgent) → `updateIssueField(change.priority)`.
- **Due date** — a date input (YYYY-MM-DD, clearable) → `updateIssueField(change.dueDate)`.
- **Delete issue** — a destructive action (with confirm) → `apply(deleteIssue)` then navigate back to the board. (Recoverable via undo.)

### 3. Undo / redo
- `useUndo`/`useRedo` mutation hooks (call `commands.undo`/`redo`, then `queryClient.invalidateQueries()` broadly).
- A global `⌘Z` / `⌘⇧Z` keyboard handler (mounted in the layout) that fires undo/redo and shows a brief toast/status (e.g. "Undone" / "Nothing to undo").

### 4. Labels
- **Core:** `Workspace::query_labels_for_issue(issue_id) -> Vec<Label>` (test-first; joins `issue_labels`).
- **Bridge:** `get_issue` returns `IssueDetailDto` = the issue fields + `labels: LabelDto[]`. Regenerate `bindings.ts`.
- **GUI:** label chips on the detail panel; a label picker that lists the project's labels and toggles attach/detach (`attachLabel`/`detachLabel`); a "new label" affordance (name + color) → `createLabel`. Board cards showing label chips is a stretch (needs `list_issues` to carry labels); deferred unless cheap.

### 5. Project management
- Sidebar affordance (context menu or a small project-settings view) to **rename** (`updateProject`), **archive** (`archiveProject`), and **delete** (`deleteProject`, with confirm) a project; deleting/archiving navigates away if the active project is affected.

## Testing

Vitest + RTL for every component (test-first where the plan says so); `ops.ts` wire-shape tests; one core unit test for `query_labels_for_issue`; the `bindings.ts` drift guard stays green. All existing gates (`cargo` fmt/clippy/test, `pnpm` typecheck/lint/test/build) stay green. The arm64-node invocation (`PATH="/opt/homebrew/bin:$PATH" npx pnpm@9.12.0 …`) applies to all UI commands.

## Acceptance

The desktop app can: set an issue's priority and due date; delete an issue (and ⌘Z it back); create, attach, and detach labels on an issue; rename / archive / delete a project — all persisted to the shared workspace and reflected after refresh.
