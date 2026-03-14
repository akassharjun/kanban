# Kanban - Project Rules

## Workflow: Everything Goes Through the Board

**All work MUST be tracked on the Kanban board (project ID: 2, prefix: KAN).**

When the user requests a feature, bug fix, or improvement:

1. **Create an issue on the board first** using the CLI (`kanban-cli`) before doing any work.
2. **Triage into the correct status:**
   - **Backlog** (status_id: 8): Big, unplanned items that need scoping, design, or breakdown before work can begin. Vague requests, large features, or items missing clear acceptance criteria go here.
   - **Todo** (status_id: 9): Scoped issues with a clear outcome. The "what" and "how" are understood. Ready to be picked up.
   - **In Progress** (status_id: 10): Actively being worked on. Only move here when you start implementation.
   - **In Review** (status_id: 11): Implementation complete. Spawn a QA agent (code-reviewer or yume--guardian) to verify the work.
   - **Done** (status_id: 13): QA passed, work is verified complete.
   - **Blocked** (status_id: 12): Cannot proceed due to a dependency or external blocker.
   - **Discarded** (status_id: 14): Intentionally abandoned.

3. **On every status change, leave a comment on the issue** explaining why it moved. Until KAN-23 (comments system) is implemented, log the comment via `activity_log` using this pattern:
   ```bash
   sqlite3 ~/.kanban/data.db "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES (<issue_id>, 'comment', NULL, '<message>', datetime('now'));"
   ```

4. **When starting work on a task:**
   - Move the issue to In Progress (status_id: 10)
   - Log a comment: "Starting implementation"

5. **When work is complete:**
   - Move the issue to In Review (status_id: 11)
   - Log a comment: "Implementation complete, spawning QA review"
   - Spawn a QA agent (`code-reviewer` or `yume--guardian` subagent) to review the changes
   - If QA passes, move to Done (status_id: 13) with comment "QA passed"
   - If QA fails, move back to In Progress with comment describing what needs fixing

6. **Assign all issues to Claude (member_id: 3).**

## CLI Reference

The CLI binary is at: `src-tauri/target/release/kanban-cli`

```bash
# Create an issue
kanban-cli issue create --project 2 --title "..." --status <status_id> --priority <urgent|high|medium|low> --assignee 3 --description "..."

# Update status
kanban-cli issue update <IDENTIFIER> --status <status_id>

# List issues
kanban-cli issue list --project 2
kanban-cli issue list --project 2 --status 10  # In Progress only

# Search
kanban-cli issue search --project 2 "query"
```

## Status IDs (Project: KAN)

| Status | ID | Category | Use |
|--------|----|----------|-----|
| Backlog | 8 | unstarted | Needs scoping |
| Todo | 9 | unstarted | Ready to work |
| In Progress | 10 | started | Actively working |
| In Review | 11 | started | QA pending |
| Blocked | 12 | blocked | Waiting on dependency |
| Done | 13 | completed | Verified complete |
| Discarded | 14 | discarded | Abandoned |

## Tech Stack

- **Backend:** Tauri v2, Rust, SQLite (sqlx), tokio
- **Frontend:** React 18, TypeScript, Vite, Tailwind CSS
- **Database:** `~/.kanban/data.db` (SQLite with WAL mode)
- **CLI:** `kanban-cli` (clap-based, standalone binary)
- **MCP:** `kanban-mcp` (JSON-RPC 2.0 over stdio)

## Key Paths

- Frontend source: `src/`
- Rust backend: `src-tauri/src/`
- Tauri commands: `src-tauri/src/commands/`
- Models: `src-tauri/src/models/mod.rs`
- Migrations: `src-tauri/migrations/`
- CLI binary: `src-tauri/src/bin/cli.rs`
- MCP binary: `src-tauri/src/bin/mcp.rs`
