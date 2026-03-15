# Agent Work Protocol

## Overview

This project has an AI agent orchestration system built into the Kanban board. Agents interact via CLI (`kanban-cli`) or MCP (`kanban-mcp`). The board is the shared brain — agents that can't talk to each other coordinate through it.

## Prerequisites

- Docker must be running: `docker compose up -d` (Postgres + Redis)
- CLI binary: `src-tauri/target/release/kanban-cli`
- Project ID for KAN board: check with `kanban-cli project list --json`

## Agent Lifecycle

### 1. Register on session start

```bash
kanban-cli agent register \
  --name "" \
  --agent-type "claude" \
  --skills rust,typescript,react,sql \
  --max-concurrent 3 \
  --json
```

Leave name empty for auto-generated name. Agent types: `claude`, `codex`, `gemini`, `custom`.
This also creates a member in the board so the agent shows up in assignee lists.

Save the returned `agent_id` — you need it for all subsequent commands.

### 2. Pull work

```bash
kanban-cli task next --agent <AGENT_ID> --json
```

Returns the highest-priority task you're qualified for, atomically claimed. If null, no work available.

### 3. Start execution

```bash
kanban-cli task start <IDENTIFIER> --agent <AGENT_ID> --json
```

### 4. Log your work (important!)

As you work, log key actions. These appear on the ticket and in replay:

```bash
kanban-cli task log <IDENTIFIER> --agent <AGENT_ID> --entry-type reasoning --message "Analyzing the auth module structure"
kanban-cli task log <IDENTIFIER> --agent <AGENT_ID> --entry-type file_read --message "Read src/auth/handler.rs"
kanban-cli task log <IDENTIFIER> --agent <AGENT_ID> --entry-type file_edit --message "Added JWT validation" --meta '{"file":"src/auth/handler.rs"}'
kanban-cli task log <IDENTIFIER> --agent <AGENT_ID> --entry-type command --message "cargo test" --meta '{"exit_code":0}'
kanban-cli task log <IDENTIFIER> --agent <AGENT_ID> --entry-type discovery --message "Found missing test coverage for edge case"
```

Entry types: `reasoning`, `file_read`, `file_edit`, `command`, `discovery`, `error`, `checkpoint`

### 5. Complete

```bash
kanban-cli task complete <IDENTIFIER> --agent <AGENT_ID> \
  --confidence 0.95 \
  --summary "Implemented JWT validation with refresh token support" \
  --json
```

Confidence thresholds:
- >= 0.85: auto-accepted as completed
- 0.50-0.84: moves to "validating", review task auto-created
- < 0.50: auto-rejected, requeued for retry

### 6. Fail (if you can't complete)

```bash
kanban-cli task fail <IDENTIFIER> --agent <AGENT_ID> \
  --reason "Missing database migration for new table" \
  --json
```

### 7. Create sub-tasks (if you discover more work)

```bash
kanban-cli task create --project <ID> \
  --title "Add missing migration" \
  --objective "Create migration for user_sessions table" \
  --status <TODO_STATUS_ID> \
  --skills sql \
  --complexity small \
  --parent <PARENT_IDENTIFIER> \
  --json
```

### 8. Heartbeat (keep-alive)

```bash
kanban-cli agent heartbeat --id <AGENT_ID> --json
```

Send every 60 seconds. If you miss 3 heartbeats, you're marked offline and tasks are reclaimed.

### 9. Deregister on session end

```bash
kanban-cli agent deregister --id <AGENT_ID> --json
```

## Auto-commenting

The system auto-posts comments on tickets at every state change:
- Claimed: "🤖 Task claimed..."
- Started: "🔧 Execution started."
- Completed: "✅ Task completed (confidence: X.XX). Summary..."
- Failed: "❌ Task failed: reason"
- Unclaimed: "↩️ Task unclaimed..."

## Useful Commands

```bash
# View task contract
kanban-cli task get <IDENTIFIER> --json

# View execution replay
kanban-cli task replay <IDENTIFIER>

# View dependency graph
kanban-cli task graph <IDENTIFIER>

# Project metrics
kanban-cli metrics --project <ID>

# Agent stats
kanban-cli metrics --agent <AGENT_ID>

# List all agents
kanban-cli agent list --json

# Search tasks
kanban-cli task search --project <ID> "query"
```

## Key Principles

1. **Log everything** — your execution trace is how humans debug your work
2. **Comment findings** — use task log + the auto-comment system
3. **Be honest about confidence** — low confidence triggers review, which is good
4. **Create sub-tasks** — if you discover work, create it on the board
5. **Don't hoard tasks** — unclaim if you're stuck, let another agent try
