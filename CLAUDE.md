# Kanban - Project Rules

## GitHub & Git

- **Repo:** `akassharjun/kanban` (public, origin: https://github.com/akassharjun/kanban.git)
- **PAT:** Stored in `.env` as `GITHUB_PAT`. Load it before any authenticated git/GitHub API operation:
  ```bash
  source .env
  # For git push/pull with auth:
  git push https://${GITHUB_PAT}@github.com/akassharjun/kanban.git <branch>
  # For GitHub API:
  curl -H "Authorization: token ${GITHUB_PAT}" https://api.github.com/repos/akassharjun/kanban/...
  ```
- **NEVER commit `.env`** — it's in `.gitignore`.

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

## Commands

```bash
# Frontend dev (Vite on :1420, HMR on :1421)
npm run dev

# Build frontend (tsc + vite)
npm run build

# Tests
npm run test          # vitest watch mode
npm run test:run      # vitest single run (CI)

# Tauri desktop app
cargo tauri dev       # Dev mode (runs npm dev + Rust backend)
cargo tauri build     # Production (.app, AppImage, .msi)

# Rust backend only
cargo build -p kanban --release

# CLI
src-tauri/target/release/kanban-cli issue list --project 2

# MCP server (JSON-RPC 2.0 over stdio)
src-tauri/target/release/kanban-mcp

# Docs (VitePress)
npm run docs:dev      # Dev server
npm run docs:build    # Static build
```

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

- **Backend:** Tauri v2, Rust, SQLite (sqlx with AnyPool), tokio
- **Frontend:** React 18, TypeScript, Vite, Tailwind CSS, shadcn/ui components
- **Database:** `~/.kanban/data.db` (SQLite with WAL mode). Optional Postgres via `DATABASE_URL` env var.
- **CLI:** `kanban-cli` (clap-based, runs as `kanban cli` subcommand)
- **MCP:** `kanban-mcp` (JSON-RPC 2.0 over stdio, runs as `kanban mcp` subcommand)
- **Drag & Drop:** @dnd-kit/core (not react-beautiful-dnd)
- **Testing:** vitest + @testing-library/react + happy-dom

## Architecture

```
src/                              # Frontend (React/TypeScript)
├── App.tsx                       # Root component, global state, keyboard shortcuts
├── main.tsx                      # React 18 entry point
├── components/
│   ├── ui/                       # shadcn primitives (button, dialog, input, etc)
│   ├── BoardView.tsx             # Kanban board with @dnd-kit drag-drop
│   ├── ListView.tsx              # Sortable table view
│   ├── TreeView.tsx              # Hierarchical parent/child view
│   ├── IssueDetailPanel.tsx      # Issue modal (markdown, comments, activity)
│   ├── IssueCard.tsx             # Card component (priority, assignee avatar)
│   ├── AgentDashboard.tsx        # Agent metrics & task visualization
│   └── ReplayViewer.tsx          # Execution log step-through replay
├── hooks/                        # Custom hooks (useIssues, useProjects, useMembers, etc)
├── lib/                          # Utilities (cn(), formatTimestamp, issue filtering)
├── tauri/commands.ts             # Typed wrappers around Tauri invoke()
├── types/index.ts                # TypeScript interfaces (mirrors Rust models)
└── test/setup.ts                 # vitest setup (mocks Tauri API)

src-tauri/                        # Backend (Rust/Tauri v2)
├── src/
│   ├── main.rs                   # Entry point: mode dispatch (GUI/CLI/MCP)
│   ├── lib.rs                    # Tauri setup, plugin registration
│   ├── state.rs                  # AppState (pool, backend type, tokio runtime)
│   ├── cli.rs                    # CLI subcommands (clap)
│   ├── mcp.rs                    # JSON-RPC 2.0 server (~2200 lines)
│   ├── commands/                 # Tauri command handlers
│   │   ├── issues.rs             # CRUD, search, bulk update, activity logging
│   │   ├── projects.rs           # Projects (soft delete + restore)
│   │   ├── statuses.rs           # Status CRUD + reordering
│   │   ├── agents.rs             # Agent registry, heartbeat, stats
│   │   ├── task_contracts.rs     # Task claim/complete/fail/approve (~43KB)
│   │   ├── comments.rs           # Issue comments
│   │   ├── custom_fields.rs      # Custom field definitions + values
│   │   └── undo.rs               # Undo/redo with JSON snapshots
│   ├── models/
│   │   ├── mod.rs                # Core structs (Issue, Project, Status, etc)
│   │   └── agent.rs              # Agent, AgentStats, TaskContract
│   ├── db/
│   │   ├── mod.rs                # init_db(), schema loader, seed_defaults
│   │   ├── watcher.rs            # SQLite WAL file watcher (emits db-changed)
│   │   └── compat.rs             # DB-agnostic SQL helpers (SQLite vs Postgres)
│   └── orchestration/            # Agent task routing, state machine, timeouts
├── migrations/                   # Postgres migrations (sqlx)
├── migrations_sqlite/            # SQLite schema (single file)
└── tauri.conf.json               # Window config (1200x800), CSP, bundle settings
```

## Key Patterns & Gotchas

**Database:**
- sqlx::AnyPool abstracts SQLite/Postgres at runtime; SQL dialect differences handled in `db/compat.rs`
- JSON fields (agent skills, task context, custom field options) stored as TEXT, parsed via `parse_json()` helper
- Issue `position` is REAL (float) for O(1) drag-drop reordering — inserts at midpoint between neighbors
- Activity logging is manual — no ORM hooks; commands explicitly call `log_activity()` after mutations
- Undo stack: `DELETE FROM undo_log WHERE undone=true` clears redo on new ops (no branching)
- SQLite pragmas: WAL mode, foreign_keys=ON, busy_timeout=5000ms

**Backend:**
- Tauri commands use `state.rt.block_on(async { ... })` to run async sqlx queries
- Background threads: WAL watcher (emits "db-changed" event) + timeout recovery (every 30s reclaims stale agent tasks)
- Task state machine with strict transitions in `orchestration/state_machine.rs`
- Agents marked offline if heartbeat exceeds `heartbeat_interval * missed_heartbeats_before_offline`
- Error handling: `Result<T, String>` (simple string errors, no custom error types)

**Frontend:**
- No router — `page` state in App.tsx: `"project" | "members" | "settings" | "agents"`
- State: React useState + custom hooks; no Redux/Zustand
- DB change events trigger full UI refresh via Tauri `listen("db-changed")`
- `cn()` helper = clsx + tailwind-merge for conditional classes
- Keyboard shortcuts: C (create), Cmd+K (search), Cmd+B (sidebar), 1/2/3 (views), Cmd+Z/Shift+Z (undo/redo)
- Dark mode: class-based (`.dark` on html), CSS vars in `index.css`

**Testing:**
- vitest with happy-dom, globals enabled (no imports needed for describe/it/expect)
- Tauri API mocked in `src/test/setup.ts`
- Tests in `__tests__/` directories alongside source files
- Run `npm run test:run` for CI, `npm run test` for watch mode

**Config:**
- Vite: port 1420, `@/` alias to `./src/`
- TypeScript: strict mode, ES2020 target, bundler resolution
- Tailwind: class-based dark mode, CSS var colors (hsl), @tailwindcss/typography plugin
- Docker: Postgres 16 on port 5433 + Redis 7 on 6379 (for optional Postgres mode)

## Environment

- `.env` — GitHub PAT and other secrets. **Never commit this file.**
- `DATABASE_URL` — Set to `postgresql://...` to use Postgres instead of SQLite. Omit for default SQLite.
- `docker-compose.yml` — Postgres 16 + Redis for cross-process sync (optional `redis-sync` feature)
