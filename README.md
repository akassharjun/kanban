# Kanban

Local-first project management. Two crates: `kanban-core` (Rust library) and `kanban-cli` (`kanban` binary). v1 covers projects, issues, labels, default statuses, search, sort, undo/redo, and JSON import/export. GUI, MCP, and AI-agent orchestration ship in later specs.

See `docs/superpowers/specs/2026-05-03-kanban-v2-core-cli-design.md` for the v1 design.

## Quick start

```sh
cargo install --path crates/kanban-cli
kanban project create "Auth Service" --prefix AUTH
kanban issue create --project AUTH --title "Add OAuth login" --priority high
kanban issue list --project AUTH --sort priority
kanban undo
```

By default the workspace lives at `~/.kanban/data.db`. Override with `KANBAN_DB=/path/to.db`.

## Desktop app

A macOS desktop GUI (Tauri + React) ships alongside the CLI, built on the same
`kanban-core` library: a drag-and-drop kanban board, a markdown issue detail
panel with priority / due-date / label editing and delete, a project sidebar
with rename / archive / delete, ⌘Z / ⌘⇧Z undo-redo, and light/dark/system
theming.

Download the latest macOS DMG from
[GitHub Releases](https://github.com/akassharjun/kanban/releases). The first
release is **unsigned**; on first launch you may need to clear the quarantine
attribute:

```sh
xattr -dr com.apple.quarantine /Applications/kanban.app
```

Projects and issues created from the CLI appear in the GUI after its window
receives focus (the board refetches on focus; there is no live cross-process
watcher yet).

## Use from an AI assistant (MCP)

`kanban-mcp` is a stdio [MCP](https://modelcontextprotocol.io) server that lets an
AI assistant (Claude Desktop, Claude Code) read and manage your board in natural
language — list/create/update/move issues, search, undo. It shares the same
`~/.kanban/data.db` as the CLI and GUI.

```sh
cargo build -p kanban-mcp --release   # binary at target/release/kanban-mcp
```

Register it with your client (see `DEVELOPMENT.md`), then ask the assistant to,
e.g., "create a project AUTH and add an issue to implement OAuth login."

## Development

See `DEVELOPMENT.md` for build, test, and contribution conventions (CLI, GUI, MCP).
