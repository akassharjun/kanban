# Kanban v2 — Core + CLI (Spec #1)

**Date:** 2026-05-03
**Author:** akassharjun
**Status:** Draft, awaiting user review
**Supersedes:** v1 (`REQUIREMENTS.md`, deleted on this branch)

---

## 1. Why v2 exists

v1 was wiped because it accumulated four problems the team is unwilling to repeat:

1. **Scope creep.** Too many features in flight at once; nothing fully finished; AI-agent layer bolted onto an unstable kanban core.
2. **Architecture rot.** The schema kept being patched, the command layer tangled, frontend and backend types drifted.
3. **Concurrency / correctness.** Race conditions on claims, undo/redo bugs, drag-and-drop state desync.
4. **Slow feedback loop.** Bugs reached the user because there was no fast way to catch them; e2e was slow, unit coverage thin.

**v2's first deliverable is intentionally narrow** so each of those four failure modes is structurally addressed before any new surface area is added.

---

## 2. Scope of this spec

This is **Spec #1** in a planned series. Future specs cover GUI, MCP server, AI-agent orchestration, and platform features (notifications, hooks, etc.). Each gets its own spec → plan → implementation cycle.

### In scope (v1)

- A `kanban-core` Rust library: data model, mutation applier, queries, migrations.
- A `kanban-cli` binary: clap-based CLI that calls the library.
- Entities: workspace, projects, issues, labels, statuses (default 7 only).
- Issue properties: title, description (markdown text only — no rendering yet), status, priority, due date, sort key, labels.
- Operations: create / update / delete / archive on each entity.
- Full-text search across issues.
- Sorting (priority, created, updated, due, manual).
- Undo / redo with persistent operation log.
- JSON export and import.

### Out of scope (will be future specs)

Members & assignees, parent/child hierarchy, issue relations, custom statuses, issue templates, saved views, notifications, automation hooks, AI agents, MCP server, GUI / Tauri, drag-and-drop, markdown rendering, due-date reminders, recently-viewed, starred, batch GitHub sync, activity-feed UI, replay viewer.

---

## 3. Architecture

### Crate layout

```
kanban-core/   library  — data model, Operation, applier, queries, migrations, FTS
kanban-cli/    binary   — clap CLI; only consumer of kanban-core in v1
```

Two crates. No `xtask`, no shared "common" crate, no premature splitting.

### Threading & async model

`kanban-core` is **synchronous** and uses `rusqlite`. No async in v1. The CLI is single-threaded per invocation.

Future async callers (GUI via Tauri, MCP server) own their own connection pooling — `kanban-core` ships `Workspace::open(path) -> Workspace` and the caller decides how many `Workspace` handles to keep alive. We deliberately do **not** infect the core with `async`; that costs more than it gives at v1's scale.

Note: `rusqlite::Connection` is `Send` but not `Sync`. A future multi-threaded caller must either use a connection pool (e.g., `r2d2 + r2d2_sqlite`) or wrap a single connection in `Mutex<Connection>`. This is a documented future cost, not a hidden one.

### Database

- File location: `~/.kanban/data.db`. Override via the `KANBAN_DB` environment variable.
- SQLite, opened with `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`.
- Multi-process safe by SQLite's own guarantees: each CLI invocation opens a fresh connection, runs its work in a single transaction, exits.
- Cold-start cost per CLI invocation: ~5–30ms (open + migration check). Acceptable for interactive use; `kanban batch` keeps a single connection open across many commands for high-throughput piping.

### Migrations

Hand-rolled, no library. SQL files live in `kanban-core/migrations/NNNN_name.sql` and are embedded with `include_str!`. On `Workspace::open`, a small runner applies any unapplied migrations in numeric order inside one transaction each, recording success in `schema_migrations(version PK INTEGER, applied_at TEXT)`.

v1 ships exactly one migration: `0001_init.sql`. After v1 ships, `0001_init.sql` is **never edited**. Schema changes are new files: `0002_*.sql`, `0003_*.sql`, etc. This is the rule that prevents the schema rot v1 suffered.

### Write discipline (the architecture-rot fix)

All functions that issue `INSERT / UPDATE / DELETE` live inside `kanban_core::store::write` and are `pub(crate)`. The single public mutator on `Workspace` is:

```rust
pub fn apply(&self, op: Operation) -> Result<OperationOutcome>
```

This means the **compiler enforces** that the only way to mutate state is through `apply`. There is no test-based check, no convention, no review burden — it is structurally impossible to bypass. Reads use a separate `pub(crate) read` module exposed via `Workspace::query_*` methods.

### Identifier generation (the concurrency fix)

Each project carries a `next_seq INTEGER NOT NULL DEFAULT 1` column. Issue creation runs in one transaction:

```sql
UPDATE projects SET next_seq = next_seq + 1 WHERE id = ?1 RETURNING next_seq - 1;
INSERT INTO issues (..., seq, identifier) VALUES (..., ?seq, ?prefix || '-' || ?seq);
```

SQLite serializes writers, so two concurrent `kanban issue create` invocations cannot collide on `(project_id, seq)`. The unique constraint on `(project_id, seq)` is a belt-and-braces guard.

Identifiers are stored denormalized (`identifier TEXT NOT NULL`) for cheap filtering and stable search results.

---

## 4. Data model

### Tables (v1)

```sql
schema_migrations (
  version    INTEGER PRIMARY KEY,
  applied_at TEXT    NOT NULL
);

projects (
  id          TEXT PRIMARY KEY,         -- UUIDv7
  name        TEXT NOT NULL,
  prefix      TEXT NOT NULL UNIQUE,     -- e.g., 'KAN'
  description TEXT,
  icon        TEXT,
  status      TEXT NOT NULL,            -- active|paused|completed|archived
  next_seq    INTEGER NOT NULL DEFAULT 1,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);

statuses (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  category   TEXT NOT NULL,             -- unstarted|started|blocked|completed|discarded
  color      TEXT NOT NULL,
  position   INTEGER NOT NULL,
  UNIQUE (project_id, name)
);

issues (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  seq          INTEGER NOT NULL,
  identifier   TEXT NOT NULL,            -- prefix-seq, denormalized
  title        TEXT NOT NULL,
  description  TEXT,
  status_id    TEXT NOT NULL REFERENCES statuses(id),
  priority     TEXT NOT NULL DEFAULT 'none',  -- none|urgent|high|medium|low
  due_date     TEXT,
  sort_key     REAL NOT NULL,            -- for manual ordering within a status
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  UNIQUE (project_id, seq),
  UNIQUE (identifier)
);

labels (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  color      TEXT NOT NULL,
  UNIQUE (project_id, name)
);

issue_labels (
  issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (issue_id, label_id)
);

operation_log (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  op_type          TEXT NOT NULL,
  payload          TEXT NOT NULL,        -- JSON, the forward Operation
  inverse_payload  TEXT NOT NULL,        -- JSON, the Operation that undoes it
  applied_at       TEXT NOT NULL,
  undone           INTEGER NOT NULL DEFAULT 0
);

activity_log (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  op_id     INTEGER NOT NULL REFERENCES operation_log(id),
  issue_id  TEXT REFERENCES issues(id) ON DELETE SET NULL,
  field     TEXT NOT NULL,
  old_value TEXT,
  new_value TEXT,
  at        TEXT NOT NULL
);

issue_search USING fts5 (
  title, description,
  content='issues', content_rowid='rowid',
  tokenize = 'porter unicode61'
);
-- triggers keep FTS in sync on INSERT/UPDATE/DELETE of issues
```

### Default statuses

Seeded on project creation:

| position | name        | category   |
| -------- | ----------- | ---------- |
| 0        | Todo        | unstarted  |
| 1        | Backlog     | unstarted  |
| 2        | In Progress | started    |
| 3        | In Review   | started    |
| 4        | Blocked     | blocked    |
| 5        | Discarded   | discarded  |
| 6        | Done        | completed  |

These are not user-editable in v1 — custom statuses ship in a future spec.

### Identifiers and IDs

- Internal primary keys (`id`) are **UUIDv7** strings. Time-ordered, sortable, no central allocator, comparable as plain text.
- User-visible identifiers (`identifier`) are `prefix-seq`, e.g., `KAN-42`. The prefix is set on project create (3+ uppercase chars, unique workspace-wide).

---

## 5. The Operation taxonomy

`Operation` is the entire write surface of `kanban-core` in v1.

```rust
enum Operation {
    // Project
    CreateProject  { id: Uuid, name: String, prefix: String, description: Option<String>, icon: Option<String> },
    UpdateProject  { id: Uuid, fields: ProjectPatch },
    ArchiveProject { id: Uuid },
    DeleteProject  { id: Uuid },

    // Issue
    CreateIssue       { id: Uuid, project_id: Uuid, title: String, description: Option<String>,
                        status_id: Uuid, priority: Priority, due_date: Option<NaiveDate>,
                        label_ids: Vec<Uuid> },
    UpdateIssueField  { id: Uuid, change: IssueFieldChange },  // single field per op
    ReorderIssue      { id: Uuid, new_sort_key: f64 },
    DeleteIssue       { id: Uuid },

    // Labels
    CreateLabel  { id: Uuid, project_id: Uuid, name: String, color: String },
    UpdateLabel  { id: Uuid, fields: LabelPatch },
    DeleteLabel  { id: Uuid },
    AttachLabel  { issue_id: Uuid, label_id: Uuid },
    DetachLabel  { issue_id: Uuid, label_id: Uuid },

    // Workspace-level
    ImportSnapshot { snapshot: WorkspaceSnapshot },  // single op for full JSON import
}

enum IssueFieldChange {
    Title       { new: String },
    Description { new: Option<String> },
    Status      { new: Uuid },
    Priority    { new: Priority },
    DueDate     { new: Option<NaiveDate> },
}
```

**Why single-field updates instead of patches:** undo and activity log become one row per logical field change without forking logic. Bulk updates from the CLI translate to N operations applied in one transaction — see "batch" below.

**`ImportSnapshot` is one operation.** Importing a JSON file is a single, atomic `Operation` from undo's perspective: `kanban undo` after an import rolls the whole thing back.

---

## 6. Undo / redo

Both `operation_log` and `activity_log` are populated by `apply` **inside the same transaction** as the mutation. They cannot drift from the data they describe.

### Forward path

1. `Workspace::apply(op)` opens a `BEGIN IMMEDIATE` transaction.
2. The applier validates the op against current state, computes its inverse, executes the mutation, and inserts both the forward `payload` and the `inverse_payload` into `operation_log` along with derivative `activity_log` rows.
3. If the operation is the first new forward op after one or more undos (i.e., the redo branch was non-empty), `DELETE FROM operation_log WHERE undone = 1` runs first to clear the redo branch.
4. Commit.

### Undo

1. Take the most recent `operation_log` row where `undone = 0`.
2. Apply its `inverse_payload` via an internal `apply_inverse` path that performs the mutation **without** inserting a new `operation_log` row (the existing row is simply re-flagged in step 3). It does insert one or more `activity_log` rows linked back to the original `op_id` so the timeline still records "this field was changed back".
3. Mark the original `operation_log` row `undone = 1`.
4. Commit.

### Redo

1. Take the most recent `operation_log` row where `undone = 1` and `id` is contiguous from the current cursor.
2. Re-apply its `payload`.
3. Mark `undone = 0`. Commit.

### Persistence across restarts

Falls out for free — both logs live in SQLite. The "current undo cursor" is `MAX(id) WHERE undone = 0`.

### Property test (TDD-mandatory)

```text
For any sequence of N valid Operations applied to a fresh DB:
  snapshot S0 = empty DB
  apply each op, snapshot Si after each
  for k in N..0:
    undo
    assert DB == S(k-1)
  for k in 0..N:
    redo
    assert DB == S(k)
```

This test must exist and pass before merging any new `Operation` variant.

---

## 7. JSON export and import

### Export

`kanban export -o snapshot.json` writes a `WorkspaceSnapshot`:

```json
{
  "schema_version": 1,
  "exported_at": "2026-05-03T12:00:00Z",
  "projects": [...],
  "statuses": [...],
  "issues": [...],
  "labels": [...],
  "issue_labels": [...]
}
```

`operation_log` and `activity_log` are **not** exported by default — they are local history, not portable data. A `--with-history` flag includes them for full forensic exports.

### Import

`kanban import snapshot.json [--conflict skip|overwrite|fail]` validates the snapshot against the current schema version, then applies a single `ImportSnapshot` operation. The conflict policy controls per-row behaviour for ID collisions; default is `fail`.

A successful import is one `Operation` in the log — `kanban undo` reverts the whole import.

---

## 8. CLI surface

### Top-level subcommands

```
kanban project   list | create | show | update | archive | delete
kanban issue     list | create | show | update | delete | move | reorder | history
kanban label     list | create | update | delete | attach | detach
kanban status    list
kanban export    -o FILE [--with-history]
kanban import    FILE [--conflict skip|overwrite|fail]
kanban undo
kanban redo
kanban search    QUERY [--project ID]
kanban batch     (reads NDJSON from stdin)
```

### `issue list` filtering and sorting

`kanban issue list` accepts:

- `--project ID` (required when more than one project exists)
- `--status NAME` (repeatable for OR)
- `--priority urgent|high|medium|low|none` (repeatable)
- `--label NAME` (repeatable)
- `--due-before DATE`, `--due-after DATE`
- `--sort priority|created|updated|due|manual` (default `manual`)
- `--reverse`
- `--limit N`

### Global flags

- `--db PATH` — override DB location (also `KANBAN_DB` env var).
- `--json` — switch output to a documented stable JSON schema.
- `--quiet` / `--verbose`.

### Output

Default human-readable: tables for lists, key-value blocks for single entities. `--json` produces a stable, documented schema per command. The schema is part of the public contract; future changes are additive only.

### Exit codes

| code | meaning |
|------|---------|
| 0 | success |
| 1 | user error (bad arguments, missing subcommand) |
| 2 | not found (no such project/issue/label) |
| 3 | validation error (constraint violation, invalid value) |
| 4 | internal error (DB locked, IO failure) |

### Batch mode

`kanban batch` reads NDJSON from stdin, one `{ "cmd": "...", "args": {...} }` object per line. One DB connection across the loop. Each line runs as its own transaction unless preceded by `{"cmd": "begin"}` / closed with `{"cmd": "commit"}`. Errors print one JSON result line and continue, unless `--fail-fast` is set.

---

## 9. Search

SQLite **FTS5** virtual table over `issues.title` and `issues.description`, configured with `content='issues'` so the FTS table doesn't duplicate row data. Triggers keep it in sync:

```sql
CREATE TRIGGER issues_ai AFTER INSERT ON issues BEGIN
  INSERT INTO issue_search(rowid, title, description) VALUES (new.rowid, new.title, new.description);
END;
-- similarly for AFTER UPDATE and AFTER DELETE
```

Filters compose with the FTS match in one SQL: a `kanban search "auth" --priority high --status "In Progress"` becomes:

```sql
SELECT i.* FROM issues i
JOIN issue_search s ON s.rowid = i.rowid
WHERE issue_search MATCH ?1
  AND i.priority = ?2
  AND i.status_id IN (SELECT id FROM statuses WHERE name = ?3)
ORDER BY rank;
```

No external search engine, no separate index process.

---

## 10. Testing strategy (TDD-mandatory)

Every behaviour in v1 is written test-first. The CI gate is `cargo test && cargo clippy -- -D warnings && cargo fmt --check`. No external services are required to run the suite.

### Test pyramid

| Layer | What it covers | Tooling | Target time |
|-------|----------------|---------|-------------|
| Unit | Pure functions: validators (`prefix_is_valid`), parsers, identifier formatting, sort-key math | `cargo test` | <100ms |
| Integration | `kanban-core` against `:memory:` SQLite. Every Operation variant. Every query function. | `cargo test` | <1s |
| Property | Undo/redo invariants over random Operation sequences | `proptest` inside `cargo test` | <500ms |
| CLI snapshot | `kanban-cli` end-to-end: seed temp DB, run binary, assert stdout/stderr/exit code | `assert_cmd` + `insta` | <2s |

### Per-feature TDD rule

For every new `Operation` variant, the first commit in the implementation plan must be a failing test:

1. Construct the `Operation` value.
2. Apply it on a fresh in-memory DB.
3. Assert the post-state via queries.
4. Apply undo, assert pre-state is restored.

The applier code lands in the next commit. No `Operation` variant is merged without this pair of tests.

### CLI snapshot policy

Every CLI subcommand has at least one happy-path snapshot and one error-path snapshot. The snapshots live next to the test (`kanban-cli/tests/snapshots/`) and are reviewed by hand on every change. New behaviour = new snapshot file = explicit, visible PR diff. Snapshot churn is the early-warning system for accidental output changes.

---

## 11. Error model

`kanban-core` uses `thiserror` with a single `Error` enum:

```rust
pub enum Error {
    Validation(ValidationError),  // e.g., bad prefix, duplicate name
    NotFound { kind: EntityKind, id: String },
    Conflict(String),             // unique constraint violations, missing FK
    Db(rusqlite::Error),
    Io(std::io::Error),
    InvalidSnapshot(String),
}
```

The CLI maps these to exit codes (see §8) and produces structured JSON when `--json` is set. The mapping is exhaustive — adding a new error variant requires updating the CLI mapper, enforced by `#[deny(non_exhaustive_omitted_patterns)]` (or an explicit match).

---

## 12. Deliverable definition

v1 is **done** when:

- [ ] `cargo test` passes with all of: unit, integration, property, CLI snapshot tests.
- [ ] `cargo clippy -- -D warnings && cargo fmt --check` pass.
- [ ] Every `Operation` variant has a paired apply-and-undo test.
- [ ] Every CLI subcommand has happy-path and error-path snapshot tests.
- [ ] `kanban export` round-trips through `kanban import --conflict fail` losslessly (test).
- [ ] Cold-start CLI invocation completes in <50ms p95 on an empty DB (smoke test).
- [ ] One end-to-end smoke script: create project, create 3 issues, label them, search, export, undo back to empty.

No GUI, no MCP, no agents — those are spec #2, #3, #4.

---

## 13. Decisions log (one-liners)

- Two crates, sync API, `rusqlite`, hand-rolled migrations.
- One public mutator (`Workspace::apply(Operation)`) enforced by module visibility.
- UUIDv7 internal IDs; `prefix-seq` user-visible identifiers; per-project `next_seq` counter for race-free allocation.
- Default 7 statuses, no custom statuses in v1.
- Single-field updates as the unit of `Operation` (not patches).
- Forward and inverse payloads stored in `operation_log` for trivial undo/redo.
- FTS5 with content-table linkage, no external search engine.
- TDD: failing test first for every Operation variant and every CLI subcommand.
- DB path `~/.kanban/data.db`, override via `KANBAN_DB`.
- DB ships migration `0001_init.sql`; future changes are new files only.

---

## 14. Open questions for the user

None remain at design level. Everything in §1–§13 is a confirmed decision through brainstorming Q&A.

---

## 15. What this spec deliberately does not commit to

- A specific dependency-version pinning policy.
- A release / packaging story (Homebrew formula, etc.) — handled when v1 is feature-complete.
- A logging/tracing format (`tracing` is implied; level and target structure decided in the implementation plan).
- A specific date for v1 completion.

These are implementation-plan concerns, not design concerns.
