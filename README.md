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

## Development

See `DEVELOPMENT.md` for build, test, and contribution conventions.
