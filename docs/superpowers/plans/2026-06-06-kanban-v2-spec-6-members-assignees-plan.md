# Kanban v2 — Spec #6 Members & Assignees Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Per-project member directory + a single nullable assignee per issue, fully undoable, exposed in GUI + CLI.

**Architecture:** New `members` table + `issues.assignee_id` (migration `0003`). New ops `CreateMember`/`UpdateMember`/`DeleteMember` mirror the label ops; assignment reuses `UpdateIssueField` via a new `IssueFieldChange::Assignee(Option<Uuid>)`. `WorkspaceSnapshot` gains `members` (schema v1→v2) so DeleteMember undo restores the member + its assignments. Tauri `apply` stays generic; `IssueDto` gains `assignee_id` + a `list_members` command (bindings regen).

**Tech:** Rust 2024 / rusqlite; React 19 (Spec #2 stack). TDD.

**Spec:** `docs/superpowers/specs/2026-06-06-kanban-v2-spec-6-members-assignees-design.md`

## Conventions
- Rust `~/.cargo/bin/cargo`; UI `cd ui && PATH="/opt/homebrew/bin:$PATH" npx --yes pnpm@9.12.0 <script>` (build fails on rollup arch → `… install --force`). Confirm REAL exit codes (`cmd; echo EXIT=$?`). Gates: cargo fmt/clippy(-D warnings)/test --workspace; ui typecheck/lint/test:unit/build. No `Co-Authored-By`. **Migrations are append-only — never edit 0001/0002.**

---

### Task 1: Migration 0003 + Member entity + member ops + snapshot extension (test-first)

**Files:** new `crates/kanban-core/migrations/0003_members.sql`; `store/migrations.rs` (register); `types.rs` (`Member`, `Issue.assignee_id`); `store/read/members.rs` (new) + `store/read/issues.rs` (+assignee_id) + `store/read/mod.rs`; `store/write/members.rs` (new); `apply/members.rs` (new) + `apply/mod.rs` (wire); `snapshot.rs` (+`members`, bump `SNAPSHOT_SCHEMA_VERSION`→2); `apply/snapshot.rs` (`upsert_member`, `export_member_subtree`, project-subtree includes members, issue upsert/export includes `assignee_id`); `workspace.rs` (`query_members_for_project`); `tests/apply_members.rs` (new) + a migration test.

- `0003_members.sql` per the design (members table + `ALTER TABLE issues ADD COLUMN assignee_id TEXT REFERENCES members(id) ON DELETE SET NULL`).
- `Member { id, project_id, name, created_at }` (mirror `Label` derives, incl. serde). `Issue` gains `pub assignee_id: Option<Uuid>` — update every Issue construction (`row_to_issue` in read, snapshot import/export, any test builders) to include it.
- Ops `CreateMember{id,project_id,name}`, `UpdateMember{id,patch:MemberPatch{name:Option<String>}}`, `DeleteMember{id}` — appliers + inverses (`inverse_of_create→Delete`, `inverse_of_update→prior name`, `inverse_of_delete→ImportSnapshot(export_member_subtree)`). `export_member_subtree(tx,id)`: snapshot with the member + the project's issues whose `assignee_id == id` (so undo restores assignments). Wire `dispatch`/`op_type_name`/`capture_inverse`.
- `snapshot.rs`: add `pub members: Vec<Member>` to `WorkspaceSnapshot` + `members: vec![]` in its constructor; bump version. `apply/snapshot.rs::import` loops `args.snapshot.members` → `upsert_member`. Ensure the issue export/import SQL now reads/writes `assignee_id`.
- Tests: migration applies (0003 in schema_migrations; assignee_id column exists, nullable); create/duplicate/rename; create-undo; update-undo; delete-undo-restores-member; **delete-member-with-assignment**: create member, assign an issue, delete member → issue.assignee_id becomes NULL; undo → member back AND issue re-assigned. Snapshot round-trip (export→import) preserves members + assignee_id.

- [ ] Failing tests → implement → `cargo test -p kanban-core`, clippy, fmt green (EXIT=0). Commit `feat(core): add members table (0003), member ops, and snapshot members`.

### Task 2: Assignment via `UpdateIssueField` (test-first)

**Files:** `operation.rs` (`IssueFieldChange::Assignee(Option<Uuid>)`); `apply/issues.rs` (handle the new change + its inverse) — verify the member belongs to the issue's project else `Error::Conflict`; `tests/apply_issues.rs` (or apply_members.rs).

- Add `Assignee(Option<Uuid>)` to `IssueFieldChange`. In the issue-field applier: on `Assignee(Some(m))`, check the member exists and `member.project_id == issue.project_id` (else `Error::Conflict("member is not in this issue's project")`); set `issues.assignee_id`. On `Assignee(None)` clear it. The inverse mirrors the existing field-change inverse (capture prior `assignee_id`).
- Tests: assign sets assignee_id; unassign clears; assign-foreign-member rejected; assign-undo restores prior; reusing `UpdateIssueField`.

- [ ] Failing tests → implement → `cargo test --workspace`, clippy, fmt green. Commit `feat(core): assign issues to a member via UpdateIssueField`.

### Task 3: Tauri DTOs + list_members + bindings (test-first where practical)

**Files:** `kanban-tauri/src/dto.rs` (`MemberDto`; `IssueDto.assignee_id: Option<String>`; optionally assignee name on the detail), `commands.rs` (`list_members(prefix)`; populate `assignee_id`), `lib.rs` (register command in `collect_commands!`), regenerate `ui/src/data/bindings.ts`.

- `MemberDto{ id, project_id, name }` (specta `Type`, `From<Member>`). `IssueDto` gains `assignee_id: Option<String>` (set from `Issue.assignee_id`). `list_members` mirrors `list_labels`. Add `query_members_for_project` use.
- Regenerate bindings (`cargo test -p kanban-tauri --test export_bindings`); fix any TS fixtures needing `assignee_id: null`. Commit the bindings.

- [ ] `cargo test --workspace` + ui typecheck green; drift guard clean. Commit `feat(tauri): expose members and issue assignee to the GUI`.

### Task 4: GUI — assignee picker + member management (test-first)

**Files:** `ui/src/data/ops.ts` (`createMember`/`updateMember`/`deleteMember` + `change.assignee`), `ui/src/data/queries.ts` (`useMembers(prefix)` + `qk.members`), `ui/src/data/mutations.ts` (invalidate members/issues on member+assignee ops), `IssuePanel` (assignee `<select>` from `useMembers`), a member-management UI (e.g. in `ProjectRow`/a small dialog), tests.

- `change.assignee(id: string | null)` → `{ field: "Assignee", value: id }`. Picker: `<select aria-label="Assignee">` with members + an "Unassigned" option → `updateIssueField(change.assignee(value||null))`.
- Member management: add/rename/delete via the member ops.
- Tests: builder shapes; assignee select dispatches `UpdateIssueField{Assignee}`; member add/rename/delete dispatch the ops. No `any`.

- [ ] ui test:unit/typecheck/lint/build green (EXIT=0). Commit `feat(ui): assign issues to members and manage the member roster`.

### Task 5: CLI — member subcommands + issue assign (test-first)

**Files:** `kanban-cli/src/cmd/member.rs` (new) + `main.rs` (register); extend `cmd/issue.rs` with `assign`; snapshots.

- `kanban member list|create|update|delete` (mirror `label.rs`). `kanban issue assign <identifier> <member-name>` and `--unassign` → `UpdateIssueField{Assignee}`. insta snapshots (happy + error: assign unknown/foreign member).

- [ ] `cargo test --workspace` green. Commit `feat(cli): member subcommands and issue assign`.

### Task 6: Acceptance + docs + PR

- [ ] Full gates (cargo fmt/clippy/test --workspace; ui typecheck/lint/test:unit/build; bindings drift clean).
- [ ] Update README/CLAUDE/DEVELOPMENT (members & assignees; note this is the last of the planned core features; MCP member tools / multi-assignee deferred).
- [ ] superpowers:finishing-a-development-branch → PR to `dev`.

## Self-review notes
- 0003 is the only migration; 0001/0002 untouched. `assignee_id` nullable, FK `ON DELETE SET NULL`.
- All ops undoable; DeleteMember inverse = `ImportSnapshot(member + assigned issues)`; assignment inverse via the existing field-change capture. Snapshot schema bumped to 2 and `members` round-tripped; `Issue.assignee_id` included in every snapshot path.
- Verify each op/struct field set against `operation.rs`; verify the new `IssueFieldChange::Assignee` serde shape used by `ops.ts`/CLI.
- The `Issue` struct gaining `assignee_id` ripples to every `Issue` construction — grep for them and update.
