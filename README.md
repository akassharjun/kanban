# Kanban

A fast, offline-first desktop project management tool inspired by Linear, built with Tauri + React + Rust.

## Features

### Project Management
- **Multiple projects** with independent kanban boards, issue lists, and settings
- **Custom statuses** per project with categories (unstarted, started, blocked, completed, discarded)
- **Custom fields** - define text, number, date, and select fields per project
- **Labels** - color-coded labels per project, manageable from issue detail view
- **Team members** - workspace-level member management with avatar colors

### Issue Tracking
- **Board view** - drag-and-drop kanban board with position-aware reordering
- **List view** - sortable table of all issues
- **Tree view** - recursive parent/child hierarchy with unlimited nesting depth
- **Issue detail dialog** - full modal view with markdown description, comments, activity log, sub-issues, and custom fields
- **Quick create** - inline issue creation from board columns
- **Bulk update** - update multiple issues at once with full undo/activity logging
- **Duplicate issues** - clone issues with labels
- **Sub-issues** - parent/child relationships with auto-complete when all children are done
- **Estimates** - point-based estimation with debounced input

### Search & Filter
- **Global search** (Cmd+K) - search issues across all projects
- **Per-project filter bar** - filter by status, priority, assignee, and label
- **Project search** - find projects by name

### Undo / Redo
- **Full undo/redo** (Cmd+Z / Cmd+Shift+Z) for issue and project mutations
- Refreshes issues, statuses, and projects on undo/redo

### Keyboard Shortcuts
| Shortcut | Action |
|----------|--------|
| `C` | Create issue |
| `Cmd+K` | Search |
| `Cmd+B` | Toggle sidebar |
| `1` / `2` / `3` | Board / List / Tree view |
| `Escape` | Close panel/dialog |
| `Cmd+Z` | Undo |
| `Cmd+Shift+Z` | Redo |

### CLI (`kanban-cli`)
Standalone command-line interface for scripting and AI agent workflows:
```bash
kanban-cli issue create --project 2 --title "Bug fix" --status 10 --priority high --assignee 3
kanban-cli issue update KAN-42 --status 13
kanban-cli issue list --project 2 --status 10
kanban-cli issue search --project 2 "auth bug"
```

### MCP Server (`kanban-mcp`)
JSON-RPC 2.0 over stdio for AI agent integration. Supports issue CRUD, search, and project management.

### Live Refresh
- DB file watcher detects external changes (from CLI/MCP) and auto-refreshes the UI
- Optimistic updates for instant feedback on mutations

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri v2 (Rust backend) |
| Frontend | React 18, TypeScript, Vite |
| Styling | Tailwind CSS, shadcn/ui-style components, @tailwindcss/typography |
| Database | SQLite (sqlx, WAL mode) |
| Drag & Drop | @dnd-kit/core |
| Markdown | react-markdown + remark-gfm |

## Installation

### Homebrew (macOS)

```bash
brew tap akassharjun/kanban https://github.com/akassharjun/kanban
```

**Desktop app** (installs `Kanban.app` to `/Applications`):
```bash
brew install --cask kanban
```

**CLI only** (installs `kanban` binary):
```bash
brew install kanban
```

### Manual

Download the latest `.dmg` (macOS) or `.deb` (Linux) from [Releases](https://github.com/akassharjun/kanban/releases).

## Development

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for release
npm run tauri build

# Build CLI
cd src-tauri && cargo build --release --bin kanban-cli
```

## Data Storage

All data is stored locally in `~/.kanban/data.db` (SQLite). No cloud, no accounts, full data ownership.
