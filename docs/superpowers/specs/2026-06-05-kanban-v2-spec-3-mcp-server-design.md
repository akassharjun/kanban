# Spec #3 — Kanban MCP Server

**Date:** 2026-06-05
**Builds on:** [Spec #1 — Core + CLI](2026-05-03-kanban-v2-core-cli-design.md), [Spec #2 — GUI Shell](2026-05-05-kanban-v2-spec-2-gui-shell-design.md)
**Roadmap epic covered:** E13 MCP
**Deferred to Spec #4+:** E19 Agents (registry, task contracts, auto-decomposition, validation pipelines, notifications)

## Goals

1. Expose `kanban-core` to MCP clients (Claude Desktop, Claude Code) so an LLM can read and drive a kanban board in natural language.
2. Present **semantic, well-described tools** the model can use reliably — not a generic "submit Operation JSON" surface.
3. Preserve every `kanban-core` invariant: sync core, single mutator via `Workspace::apply`, append-only migrations.
4. Share one workspace across CLI, GUI, and MCP — an issue created in any surface is visible in the others.

## Non-goals

- **Agent orchestration** (E19): agent registry, task contracts, decomposition, validation pipelines, claim/complete lifecycles, notifications. All deferred to Spec #4+. The MCP server is the foundation those will build on.
- **HTTP / SSE transport.** Local-first; stdio only. Remote transports are a later concern.
- **Destructive tools.** No `delete_issue` / `delete_project` / `archive` in v1. `undo`/`redo` cover mistake recovery. (Revisit in a later spec if agents need full lifecycle control.)
- **MCP resources and prompts.** Tools only for v1.
- **New schema.** Resolution by `prefix`/`identifier` uses existing unique indexes; no migration ships in Spec #3.

## Stack

- **Crate:** new `crates/kanban-mcp` — a `kanban-mcp` binary, added to the Cargo workspace.
- **SDK:** [`rmcp`](https://crates.io/crates/rmcp) — the official Rust MCP SDK (server side, stdio transport, `#[tool]` macros). Exact version pinned during planning.
- **Runtime:** `tokio` (multi-thread). The sync core runs inside `tokio::task::spawn_blocking` — the same pattern proven in `kanban-tauri`.
- **Tests:** `cargo test` (per-tool handler tests on an in-memory `Workspace`; one stdio integration test). Covered by the existing `cargo test --workspace` / clippy / fmt CI gates.

## Architecture

```
kanban/
├── crates/
│   ├── kanban-core/            # +2 indexed resolution helpers (see below)
│   ├── kanban-cli/             # unchanged
│   ├── kanban-tauri/           # switches get_issue to the new core helper (retires debt)
│   └── kanban-mcp/      (new)  # stdio MCP server
│       ├── src/
│       │   ├── main.rs         # rmcp stdio server bootstrap
│       │   ├── server.rs       # KanbanServer: holds Arc<Mutex<Workspace>>, tool impls
│       │   ├── tools/          # one module per tool group (reads, issues, projects)
│       │   ├── dto.rs          # serde-friendly tool input/output shapes
│       │   └── error.rs        # kanban_core::Error -> rmcp tool error mapping
│       └── Cargo.toml
```

- `KanbanServer` holds `Arc<Mutex<Workspace>>`, opened via `Workspace::open_default()` — so the server shares `~/.kanban/data.db` (or `$KANBAN_DB`) with the CLI and GUI. SQLite WAL keeps the three multi-process-safe.
- Every tool clones the `Arc`, and inside `spawn_blocking` locks the mutex and runs the sync core call — the guard never crosses an `.await` (the `kanban-tauri` pattern).
- **Writes go through `Workspace::apply`.** Each write tool builds the appropriate `Operation` and applies it; the single-mutator invariant is untouched.

### Identifiers

Tools speak the human vocabulary, never UUIDs:
- Projects are addressed by **prefix** (`AUTH`).
- Issues are addressed by **key / identifier** (`AUTH-12`).
- Statuses and labels are referenced **by name** within a project (the server resolves names to ids); ambiguous/unknown names return a clear tool error.

## Tool surface

All tools return structured JSON content. Descriptions and field docs are written for an LLM audience.

### Reads
| Tool | Input | Returns |
|------|-------|---------|
| `list_projects` | — | projects: prefix, name, description |
| `list_statuses` | `project` (prefix) | the project's status columns, in order |
| `list_labels` | `project` | labels: name, color |
| `list_issues` | `project`, optional `status` (name), `priority`, `search` | matching issues (key, title, status, priority, due) |
| `get_issue` | `key` | full issue incl. description (markdown), labels, timestamps |
| `search_issues` | `query`, optional `project` | FTS5 results across titles/descriptions |

### Writes (each builds an `Operation` → `Workspace::apply`)
| Tool | Input | Operation(s) |
|------|-------|--------------|
| `create_project` | `name`, `prefix`, optional `description` | `CreateProject` |
| `create_issue` | `project`, `title`, optional `description`, `status` (name), `priority`, `due_date` | `CreateIssue` (resolves project→id, status name→id, defaults to the project's first status) |
| `update_issue` | `key`, any of `title`/`description`/`priority`/`status`/`due_date` | one `UpdateIssueField` per changed field |
| `move_issue` | `key`, optional `status` (name), optional `before`/`after` (sibling key) | `UpdateIssueField{Status}` if the column changes, then `ReorderIssue` with a server-computed midpoint `sort_key` |
| `undo` | — | `Workspace::undo()` |
| `redo` | — | `Workspace::redo()` |

`move_issue` computes the new `sort_key` from the neighbours' keys using the same fractional-midpoint convention as the GUI (1024 gap; average between two neighbours). `before`/`after` omitted ⇒ append to the column.

## Core additions (`kanban-core`)

Two indexed resolution helpers on `Workspace` (both columns are already uniquely indexed — `idx_projects_prefix`, `UNIQUE(identifier)` — so **no migration**):

- `query_project_by_prefix(&self, prefix: &str) -> Result<Option<Project>>`
- `query_issue_by_identifier(&self, identifier: &str) -> Result<Option<Issue>>`

The MCP server uses these for resolution. `kanban-tauri::commands::get_issue_inner` switches to `query_issue_by_identifier`, **retiring the Task 7 full-table-scan FOLLOWUP debt** (it currently scans all issues and filters in Rust).

No new `Operation` variants are needed — the existing 14 cover everything the tools do.

## Error handling

A `kanban_core::Error` → `rmcp` tool-error mapping (mirrors the Spec #2 `ApiError`), producing LLM-actionable messages:
- `NotFound` → "project AUTH not found" / "issue AUTH-12 not found"
- `Validation` → the field + reason verbatim ("prefix: must be 2–8 uppercase letters")
- name-resolution misses (unknown status/label name) → "status 'Doing' not found in project AUTH; available: To Do, In Progress, Done"
- `Conflict` (e.g. nothing to undo) → the message verbatim

Tool input validation (missing required field, bad enum) is handled by the `rmcp` schema layer before the handler runs.

## Testing (TDD)

- **Per tool:** a test that calls the handler against a fresh `Workspace::open_in_memory()`, asserts the resulting state, and — for writes — asserts `undo` reverses it. Mirrors the core's "every Operation lands as a failing test first" discipline.
- **Resolution helpers:** unit tests for prefix/identifier hit + miss.
- **Integration:** one test that drives the server over an in-process stdio pipe — `initialize`, `tools/list` (assert the tool set), and one `tools/call` round-trip — proving transport + schema wiring.
- **Regression:** the `kanban-tauri` switch to `query_issue_by_identifier` keeps its existing tests green.
- CI: the new crate is covered by the existing `cargo fmt --check`, `cargo clippy --workspace -D warnings`, `cargo test --workspace` gates. No workflow change required.

## Docs

- `README.md`: a short "Use from an AI assistant (MCP)" section.
- `DEVELOPMENT.md`: the `mcpServers` config snippet for **Claude Desktop** (`claude_desktop_config.json`) and **Claude Code** (`.mcp.json` / `claude mcp add`), pointing at the built `kanban-mcp` binary, with `KANBAN_DB` override noted.
- `CLAUDE.md`: a short "MCP layer (Spec #3)" note — stdio server, shares the workspace DB, writes through `apply`, semantic tools, agent orchestration deferred to Spec #4.

## Acceptance criteria

1. `kanban-mcp` builds and starts as a stdio server; `tools/list` returns the full tool set with schemas.
2. From a fresh DB, an MCP `tools/call` sequence — `create_project` → `create_issue` → `move_issue` → `get_issue` — produces the expected board state, and the same DB is visible from `kanban issue list`.
3. `undo` reverses the last write; `redo` replays it.
4. Errors are clear and actionable (unknown project/status/issue).
5. All workspace gates green (fmt, clippy `-D warnings`, `cargo test --workspace`); the `kanban-tauri` debt retirement keeps its suite green.
6. Registering the binary per the docs makes the tools usable from Claude Desktop and Claude Code.
