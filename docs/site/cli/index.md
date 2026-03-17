# Kanban CLI Reference

Kanban includes a powerful command-line interface for managing projects, issues, and agents. The CLI is the primary way to interact with the Kanban board programmatically.

## Installation

The CLI binary is built with the Tauri application and is available as `kanban`.

```bash
# List available commands
kanban --help

# Get help for a specific command
kanban issue --help
kanban project create --help
```

## Global Options

All commands support the following global flags:

| Flag | Purpose |
|------|---------|
| `--json` | Output results as JSON (useful for scripting) |
| `--database-url` | Custom SQLite database path (defaults to `~/.kanban/data.db`) |

### JSON Output

Use `--json` to get machine-readable output:

```bash
kanban project list --json
```

Output:
```json
[
  {
    "id": 1,
    "name": "Example Project",
    "prefix": "EX",
    "status": "active",
    "description": "An example project",
    "issue_counter": 42,
    "created_at": "2025-03-15T10:00:00Z",
    "updated_at": "2025-03-15T10:00:00Z"
  }
]
```

## Command Structure

All commands follow this pattern:

```bash
kanban <resource> <action> [options]
```

**Resources:**
- `project` - Manage projects
- `issue` - Manage issues
- `member` - Manage team members
- `label` - Manage labels
- `agent` - Manage AI agents
- `task` - Manage task contracts
- `metrics` - View system metrics
- `export` - Export all data to JSON
- `import` - Import data from JSON

## Quick Examples

### Create a project

```bash
kanban project create "My Project" --prefix PRJ
```

### Create an issue

```bash
kanban issue create --project 1 --title "Fix login bug" --status 9 --priority high
```

### Register an agent

```bash
kanban agent register --name "my-agent" --agent-type claude --skills "python,testing"
```

### Claim and complete a task

```bash
# Get next available task
kanban task next --agent agent-id-123

# Start the task
kanban task start KAN-42 --agent agent-id-123

# Complete it
kanban task complete KAN-42 --agent agent-id-123 --confidence 0.95 --summary "Fixed the issue"
```

## Output Formats

By default, the CLI outputs human-readable text. Use `--json` for scripting:

```bash
# Human-readable
kanban issue list --project 1

# Machine-readable JSON
kanban issue list --project 1 --json
```

## See Also

- [Projects](./projects.md)
- [Issues](./issues.md)
- [Members](./members.md)
- [Labels](./labels.md)
- [Agents](./agents.md)
- [Tasks](./tasks.md)
- [Metrics](./metrics.md)
