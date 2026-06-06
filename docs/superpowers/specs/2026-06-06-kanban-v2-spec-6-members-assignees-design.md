# Spec #6 — Members & Assignees

**Date:** 2026-06-06
**Builds on:** Spec #1–#5 (core + CLI + GUI + MCP + GUI completeness + custom statuses)
**Roadmap:** E03 Members & Assignees — a per-project people directory; each issue may have one assignee.

## Decisions (confirmed)
- **Per-project members.** Each project owns its member roster (`members.project_id`). Only a project's own members can be assigned to its issues.
- **Single assignee per issue.** Issues gain a nullable `assignee_id` referencing `members(id)`. (Multi-assignee is a later spec if ever needed.)

## Migration `0003_members.sql` (the only new migration; append-only)
```sql
CREATE TABLE members (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (project_id, name)
) STRICT;
CREATE INDEX idx_members_project ON members(project_id);

ALTER TABLE issues ADD COLUMN assignee_id TEXT REFERENCES members(id) ON DELETE SET NULL;
```
SQLite permits `ADD COLUMN` with a nullable `REFERENCES … ON DELETE SET NULL` (default NULL); `foreign_keys` is ON (`connection.rs`), so deleting a member nulls its issues' `assignee_id` at the DB level. Register `(3, "members", include_str!("…/0003_members.sql"))` in `store/migrations.rs`.

## Operations (undoable, mirror the label/status ops)
- `CreateMember { id, project_id, name }` — validate non-empty name; reject duplicate `(project_id, name)`; inverse `DeleteMember{id}`.
- `UpdateMember { id, patch: MemberPatch { name? } }` — rename; inverse re-applies the prior name.
- `DeleteMember { id }` — allowed (no guard); the DB nulls dependent `assignee_id`s. Inverse = `ImportSnapshot` capturing the member row **and** the issues that referenced it (so undo re-creates the member and restores those assignments).
- **Assignment reuses `UpdateIssueField`**: add `IssueFieldChange::Assignee(Option<Uuid>)` — `Some(member_id)` assigns, `None` unassigns. Inverse is the existing field-change inverse (prior assignee). Validation: the member must belong to the issue's project (else `Error::Conflict`/`Validation`).

## Snapshot changes (undo of DeleteMember)
`WorkspaceSnapshot` gains `pub members: Vec<Member>`; bump `SNAPSHOT_SCHEMA_VERSION` 1 → 2. `apply/snapshot.rs::import` upserts members (a new `upsert_member`); the project-subtree export (used by delete-project inverse) includes members. The issue row now carries `assignee_id`, so the issues read/write used by snapshots must include it. A new `export_member_subtree(tx, member_id)` captures the member + the project's issues assigned to it.

## Types / DTO / bridge
- `types.rs`: new `Member { id: Uuid, project_id: Uuid, name: String, created_at: DateTime<Utc> }`. `Issue` gains `assignee_id: Option<Uuid>`.
- `store/read`: `members::for_project` / `by_id_via_tx`; `Workspace::query_members_for_project(project_id)`. Issue reads include `assignee_id`.
- Tauri: `MemberDto { id, project_id, name }`; a `list_members(prefix)` command; `IssueDto` gains `assignee_id: Option<String>` and `IssueDetailDto`/`get_issue` may include the resolved assignee name. Regenerate `bindings.ts` (drift-guarded). `apply` stays generic.
- MCP: optional `list_members` / assignment surfacing — **deferred** to keep scope; note it.

## GUI
- Issue detail panel: an **assignee picker** (lists the project's members via `useMembers(prefix)`; selecting → `updateIssueField(change.assignee(memberId|null))`); show the current assignee.
- A small **member management** affordance (e.g. in the project settings / sidebar or a panel): add/rename/delete members → `createMember`/`updateMember`/`deleteMember`.
- Board card: optionally show an assignee initial/avatar (stretch).
- `ops.ts`: `createMember`/`updateMember`/`deleteMember` builders + `change.assignee`. `mutations.ts`: invalidate a `qk.members(prefix)` key + issues on member/assignee ops.

## CLI
- `kanban member list|create|update|delete` (mirror `label`); `kanban issue assign <identifier> <member-name|--unassign>`.

## Testing
- Core: `tests/apply_members.rs` — create / duplicate / rename / delete-undo-restores-member-and-assignments / assign / unassign / assign-foreign-member-rejected; plus a migration test (0003 applies; assignee_id nullable). Snapshot round-trip includes members + assignee_id.
- CLI: `member_cmds.rs` + an `issue assign` test (insta).
- GUI: Vitest + RTL for the builders, assignee picker, member management.
- All gates green; bindings drift accounted for (regenerated + committed).

## Non-goals
- Multi-assignee; member avatars/emails/auth; cross-project members; MCP member tools (later). 

## Acceptance
A user can, per project: add/rename/delete members; assign an issue to one member (or unassign); deleting a member unassigns their issues — every change undoable (undoing a member delete restores the member and its prior assignments).
