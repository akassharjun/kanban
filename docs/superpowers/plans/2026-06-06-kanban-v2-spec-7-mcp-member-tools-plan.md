# Spec #7 — MCP Member Tools — Implementation Plan

**Design:** `docs/superpowers/specs/2026-06-06-kanban-v2-spec-7-mcp-member-tools-design.md`
**Scope:** MCP-surface + output-shape only. No schema, no core op, no migration.

TDD throughout: the end-to-end member test lands red first, then the surface is built until green; gates (`fmt`, `clippy -D warnings`, `test --workspace`) stay green.

## Task 1 — Failing e2e test (RED)
`crates/kanban-mcp/tests/members_flow.rs`, mirroring `e2e_flow.rs` over the in-process `tokio::io::duplex` transport:
- **lifecycle:** create project + issue → `list_members` empty → `create_member` ×2 → `list_members` ordered → `assign_issue` (member) → `get_issue` reflects assignee → `update_member` rename (assignment follows) → `assign_issue` (omit member) unassigns → `delete_member` ×2 → roster empty.
- **errors:** duplicate `create_member`, `assign_issue` to unknown member (lists available), `delete_member` of an unknown member.

Run; confirm it fails with "tool not found" (the tools don't exist).

## Task 2 — Output shapes (`convert.rs`)
- Add `MemberOut { name }` (`From<Member>`).
- Add `member_name_map(&[Member]) -> HashMap<Uuid, String>`.
- `IssueOut` gains `assignee: Option<String>`; `from_issue` gains an `assignee: Option<String>` argument.

## Task 3 — Inputs (`inputs.rs`)
Add `CreateMemberInput { project, name }`, `UpdateMemberInput { project, name, new_name }`, `MemberRef { project, name }`, `AssignIssueInput { key, member? }`. (`ProjectRef` reused by `list_members`.)

## Task 4 — Thread assignee through existing reads (`server.rs`)
Every issue-returning tool loads the project's members (one extra `query_members_for_project` inside the existing lock) and resolves `issue.assignee_id`:
- `list_issues`, `get_issue`, `update_issue`, `move_issue` — single project.
- `search_issues` — collect members across all projects alongside statuses.
- `create_issue` — newly created issues are always unassigned (`None`).

## Task 5 — Member tools (`server.rs`)
- `resolve_member(prefix, name)` helper → member UUID (`not_found` project / `unknown_name` member).
- `list_members` (via `with_project`), `create_member`, `update_member` (rename via `MemberPatch`), `delete_member`, `assign_issue` (resolve member-or-unassign → `UpdateIssueField{ Assignee }`, re-read, return with resolved assignee).

## Task 6 — Verify GREEN + docs
- `members_flow.rs` passes; extend `e2e_flow.rs` to assert `assignee` is `null` by default.
- `cargo test --workspace`, `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt --check` all green.
- Update `CLAUDE.md` (MCP layer tool list + Spec #7 note, Spec #6 deferral resolved, spec trail), `README.md`, `DEVELOPMENT.md` tool list.

## Acceptance
Over MCP an assistant can list/create/rename/delete members per project and assign/unassign issues; every issue the server returns shows its `assignee` (or `null`); member deletes remain undoable via `undo`.
