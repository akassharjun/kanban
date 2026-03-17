# Agents Command

Manage AI agents and their lifecycle via the CLI.

## Commands

### Register an agent

Register a new AI agent with the Kanban board.

```bash
kanban agent register [OPTIONS]
```

**Options:**
- `--name <NAME>` - Agent name (auto-generated if not provided)
- `--agent-type <TYPE>` - Agent type: claude, claude-code, codex, gemini, or custom
- `--skills <SKILLS>` - Comma-delimited skills (e.g., "python,testing,refactoring")
- `--task-types <TASK_TYPES>` - Comma-delimited task types (optional)
- `--max-concurrent <N>` - Maximum concurrent tasks [default: 1]
- `--max-complexity <LEVEL>` - Maximum complexity level: small, medium, large [default: large]
- `--worktree-path <PATH>` - Local worktree path for agent execution

**Examples:**

```bash
# Simple agent with auto-generated name
kanban agent register --agent-type claude --skills "python,testing"

# Named agent
kanban agent register \
  --name "code-analyzer" \
  --agent-type claude-code \
  --skills "python,rust,analysis,documentation"

# With task types and concurrency
kanban agent register \
  --name "backend-team" \
  --agent-type claude \
  --skills "database,api,rust,sql" \
  --task-types "implementation,bugfix,review" \
  --max-concurrent 3 \
  --max-complexity large

# With worktree path
kanban agent register \
  --name "frontend-agent" \
  --agent-type claude-code \
  --skills "typescript,react,css,testing" \
  --worktree-path "/Users/dev/projects/frontend"

# JSON output
kanban agent register \
  --name "my-agent" \
  --agent-type claude \
  --skills "python,testing" \
  --json
```

**Output:**
```
Registered agent: code-analyzer
ID: 550e8400-e29b-41d4-a716-446655440000
Type: claude-code
Skills: [python, rust, analysis, documentation]
Status: idle
Max concurrent: 1
Max complexity: large
```

**JSON Output:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "code-analyzer",
  "agent_type": "claude-code",
  "skills": ["python", "rust", "analysis", "documentation"],
  "task_types": [],
  "max_concurrent": 1,
  "max_complexity": "large",
  "member_id": 3,
  "status": "idle",
  "registered_at": "2025-03-15T10:00:00Z",
  "last_heartbeat": "2025-03-15T10:00:00Z",
  "last_activity_at": null,
  "worktree_path": null
}
```

### List agents

List all registered agents.

```bash
kanban agent list [OPTIONS]
```

**Examples:**

```bash
# List all agents
kanban agent list

# JSON output
kanban agent list --json
```

**Output (default):**
```
ID                                  NAME              TYPE         STATUS  ACTIVE  SKILLS
550e8400-e29b-41d4-a716-446655440000  code-analyzer     claude-code  idle    0       python,rust,analysis
660e8400-e29b-41d4-a716-446655440001  backend-team      claude       busy    2       database,api,rust
770e8400-e29b-41d4-a716-446655440002  frontend-agent    claude-code  idle    0       typescript,react
```

**JSON Output:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "code-analyzer",
    "agent_type": "claude-code",
    "skills": ["python", "rust", "analysis"],
    "task_types": [],
    "max_concurrent": 1,
    "max_complexity": "large",
    "member_id": 3,
    "status": "idle",
    "registered_at": "2025-03-15T10:00:00Z",
    "last_heartbeat": "2025-03-15T10:00:00Z",
    "last_activity_at": null,
    "worktree_path": null
  }
]
```

### Send heartbeat

Send a heartbeat to keep an agent active and update its status.

```bash
kanban agent heartbeat --id <AGENT_ID>
```

**Arguments:**
- `--id <AGENT_ID>` - Agent UUID

**Examples:**

```bash
# Send heartbeat
kanban agent heartbeat --id 550e8400-e29b-41d4-a716-446655440000

# JSON output
kanban agent heartbeat --id 550e8400-e29b-41d4-a716-446655440000 --json
```

**Output:**
```
Heartbeat sent for agent: code-analyzer
Status: idle (0 active tasks)
Last heartbeat: 2025-03-15T10:05:00Z
```

**JSON Output:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "code-analyzer",
  "agent_type": "claude-code",
  "skills": ["python", "rust", "analysis"],
  "task_types": [],
  "max_concurrent": 1,
  "max_complexity": "large",
  "member_id": 3,
  "status": "idle",
  "registered_at": "2025-03-15T10:00:00Z",
  "last_heartbeat": "2025-03-15T10:05:00Z",
  "last_activity_at": "2025-03-15T10:05:00Z",
  "worktree_path": null
}
```

### Deregister an agent

Unregister an agent and reclaim its tasks.

```bash
kanban agent deregister --id <AGENT_ID>
```

**Arguments:**
- `--id <AGENT_ID>` - Agent UUID to deregister

**Examples:**

```bash
# Deregister an agent
kanban agent deregister --id 550e8400-e29b-41d4-a716-446655440000

# Output
Deregistered agent: code-analyzer
Reclaimed 2 active tasks for reassignment
```

**Notes:**
- Deregistration is permanent
- All active tasks are reclaimed and set back to unclaimed status
- The agent member record is preserved for history

### Show agent stats

Display detailed statistics for an agent.

```bash
kanban agent stats <AGENT_ID>
```

**Arguments:**
- `<AGENT_ID>` - Agent UUID

**Examples:**

```bash
# Show stats
kanban agent stats 550e8400-e29b-41d4-a716-446655440000

# JSON output
kanban agent stats 550e8400-e29b-41d4-a716-446655440000 --json
```

**Output:**
```
Agent: code-analyzer
Type: claude-code
Status: idle

Statistics:
  Tasks completed: 42
  Tasks failed: 3
  Success rate: 93%
  Average confidence: 0.87
  Total execution time: 245 hours
  Average time per task: 5.8 hours

Skills breakdown:
  python: 25 tasks (60%)
  rust: 12 tasks (29%)
  analysis: 5 tasks (12%)
```

**JSON Output:**
```json
{
  "agent_id": "550e8400-e29b-41d4-a716-446655440000",
  "tasks_completed": 42,
  "tasks_failed": 3,
  "total_confidence": 36.54,
  "total_completion_time_seconds": 882000,
  "skills_breakdown": {
    "python": 25,
    "rust": 12,
    "analysis": 5
  }
}
```

## Agent Types

| Type | Description |
|------|-------------|
| `claude` | Anthropic Claude (general capability) |
| `claude-code` | Anthropic Claude with code expertise |
| `codex` | OpenAI Codex (code generation) |
| `gemini` | Google Gemini |
| `custom` | Custom/external agent |

## Complexity Levels

| Level | Description |
|-------|-------------|
| `small` | Simple, well-scoped tasks |
| `medium` | Moderate complexity, multiple steps |
| `large` | Complex tasks, architectural work |

## Agent Lifecycle

```
Register → idle
   ↓
Send heartbeat → idle/busy (based on task count)
   ↓
Claim tasks → executing
   ↓
Complete/fail → idle
   ↓
Deregister (when no longer needed)
```

## Examples

### Complete agent workflow

```bash
# 1. Register an agent
AGENT_ID=$(kanban agent register \
  --name "code-reviewer" \
  --agent-type claude-code \
  --skills "python,testing,review" \
  --max-concurrent 2 \
  --json | jq -r '.id')

echo "Registered agent: $AGENT_ID"

# 2. Verify registration
kanban agent list --json | jq ".[] | select(.id == \"$AGENT_ID\")"

# 3. Send periodic heartbeats
for i in {1..5}; do
  kanban agent heartbeat --id "$AGENT_ID"
  sleep 60
done

# 4. Check stats
kanban agent stats "$AGENT_ID"

# 5. Deregister when done
kanban agent deregister --id "$AGENT_ID"
```

### Agent pool management

```bash
#!/bin/bash
# Create multiple agents for different specialties

# Frontend team
kanban agent register \
  --name "frontend-1" \
  --agent-type claude-code \
  --skills "typescript,react,css" \
  --max-concurrent 2

# Backend team
kanban agent register \
  --name "backend-1" \
  --agent-type claude \
  --skills "rust,database,api" \
  --max-concurrent 2

# QA
kanban agent register \
  --name "qa-bot" \
  --agent-type codex \
  --skills "testing,automation,qa" \
  --max-concurrent 1

# List all
kanban agent list

# Health check all
kanban agent list --json | jq -r '.[].id' | while read agent_id; do
  kanban agent heartbeat --id "$agent_id"
done
```

### Monitor agent activity

```bash
#!/bin/bash
# Script to monitor all agents every 30 seconds

while true; do
  echo "=== Agent Status ($(date)) ==="
  kanban agent list --json | jq '.[] | {
    name: .name,
    status: .status,
    last_heartbeat: .last_heartbeat
  }'
  sleep 30
done
```

## See Also

- [Tasks Command](./tasks.md)
- [Agent Lifecycle Documentation](../agents/lifecycle.md)
