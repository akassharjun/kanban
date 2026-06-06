# Spec #7 — MCP Member Tools

**Date:** 2026-06-06
**Builds on:** Spec #1–#6 (core + CLI + GUI + MCP + GUI completeness + custom statuses + members)
**Roadmap:** E13 MCP (members slice) — surface the Spec #6 member/assignee model over the MCP server so AI assistants can manage the people directory and assign work, closing the deferral noted in Spec #6 ("MCP member tools are deferred to Spec #7").

## Context

The MCP server (`crates/kanban-mcp`) already exposes projects, statuses, labels, and issues (read + create/update/move/undo/redo) over stdio. Spec #6 added a per-project member roster and a single nullable `issues.assignee_id`, exposed in the GUI and CLI but **not** over MCP. Today an assistant can create and move issues but cannot see who is on a project or assign work — and assignment is invisible in every issue the server returns. This spec closes both gaps.

## Decisions

- **Members referenced by name within a project**, exactly like statuses and labels (issues stay keyed by `KEY-N`, projects by prefix). UUIDs never cross the bridge.
- **Five new tools**, mirroring the existing tool shapes:
  - `list_members { project }` — read the roster.
  - `create_member { project, name }` — add a member.
  - `update_member { project, name, new_name }` — rename (resolve the member by its current name within the project).
  - `delete_member { project, name }` — remove (the DB nulls dependent `assignee_id`s via `ON DELETE SET NULL`; undoable via the core's snapshot inverse).
  - `assign_issue { key, member? }` — assign the issue to `member` (by name, within the issue's project) or **unassign** when `member` is omitted/empty.
- **Assignment reuses the core path:** `assign_issue` applies `Operation::UpdateIssueField` with `IssueFieldChange::Assignee(Some|None)`. No new core op. Single-mutator invariant preserved (every write still goes through `Workspace::apply`).
- **Assignee becomes observable in issue reads.** `IssueOut` gains `assignee: Option<String>` (the member name, or `null`). This threads through `get_issue`, `list_issues`, `search_issues`, `create_issue`, `update_issue`, `move_issue`, and `assign_issue`. Without this the feature would be write-only — an assistant could assign but never read back the assignment.

## Output shapes (`convert.rs`)

- New `MemberOut { name }` (`From<Member>`), matching the prefix/name-only convention of `StatusOut`/`LabelOut`.
- `IssueOut` gains `assignee: Option<String>`. `IssueOut::from_issue` takes an added `assignee: Option<String>` argument; a small `member_name_map(&[Member]) -> HashMap<Uuid, String>` resolves `issue.assignee_id` to a name (analogous to `status_name_map`).

## Tool resolution & errors (`server.rs`, `inputs.rs`, `error.rs`)

- New input structs in `inputs.rs`: `MemberRef { project, name }`, `CreateMemberInput { project, name }`, `UpdateMemberInput { project, name, new_name }`, `AssignIssueInput { key, member? }`. `ProjectRef` is reused by `list_members`.
- Resolving a member by name within a project reuses the existing `error::unknown_name(kind, name, project, available)` (lists valid members) when absent — same pattern as status-name resolution.
- Project-scoped reads reuse the existing `with_project` helper (resolves prefix → id + statuses); a member lookup runs inside the same lock via `query_members_for_project`.
- Core errors continue to map through `to_mcp` (duplicate name → `Conflict` → `invalid_request`; empty name → `Validation` → `invalid_params`).

## Read-side threading (member-name resolution)

Each issue-returning tool already resolves the project's statuses; it now also loads that project's members (one extra `query_members_for_project` inside the existing lock) and builds a `member_name_map`. `search_issues` (cross-project) collects members for every project alongside statuses, exactly as it already does for statuses. `assign_issue` re-reads the issue after applying and returns it with the resolved assignee.

## Testing (TDD)

- `crates/kanban-mcp/tests/members_flow.rs` — end-to-end over the in-process stdio transport (mirrors `e2e_flow.rs`):
  - happy path: create project → `list_members` empty → `create_member` → `list_members` shows it → create issue → `assign_issue` sets `assignee` → `get_issue` reflects it → `update_member` rename → `assign_issue` with no member unassigns (`assignee: null`) → `delete_member` → `list_members` empty.
  - error paths: duplicate `create_member` (conflict), `assign_issue` to an unknown member (lists available), `delete_member` for an unknown member.
- Existing `e2e_flow.rs` assertions extended to confirm `assignee` is present and `null` by default on created/read issues.
- All CI gates green: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`.

## Non-goals

- Member emails/avatars/auth, multi-assignee, cross-project members (unchanged from Spec #6).
- Filtering `list_issues` by assignee (could follow; not required to manage assignments).
- Any schema or core-op change — this spec is purely an MCP-surface + output-shape addition.

## Acceptance

Over MCP, an assistant can, per project: list/create/rename/delete members, and assign an issue to a member or unassign it — and every issue the server returns shows its current assignee (or `null`). Member deletes remain undoable through the existing `undo` tool.
