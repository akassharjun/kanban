# Agent Lifecycle

Detailed reference of the agent lifecycle and state transitions.

## Lifecycle States

An agent progresses through the following states:

```
Register
   ↓
[idle] ← → [busy] ← → [offline]
   ↓
Deregister
```

## Idle State

Agent is online and has capacity to claim tasks.

**Conditions:**
- Agent is registered
- Number of active tasks (claimed + executing) < `max_concurrent`
- Heartbeat received within timeout (default: 30 minutes)

**Transitions:**
- From: Initial state after registration
- To: Busy (when claiming tasks up to `max_concurrent`)
- Automatic heartbeat maintains this state

**Example:**

```bash
# Register agent (automatically enters idle)
kanban agent register \
  --name "code-analyzer" \
  --agent-type claude-code \
  --skills "python,testing"

# Check status
kanban agent list
# Shows status: idle
```

## Busy State

Agent has active tasks but is still responsive.

**Conditions:**
- Number of active tasks = `max_concurrent`
- Heartbeat received within timeout

**Transitions:**
- From: Idle (when claiming tasks)
- To: Idle (when tasks complete/fail)
- Automatic heartbeat maintains this state

**Example:**

```bash
# Agent with max_concurrent=2
kanban agent register \
  --name "multi-task" \
  --agent-type claude \
  --max-concurrent 2

# Claim first task
kanban task next --agent <agent-id>

# Claim second task (now busy)
kanban task next --agent <agent-id>

# Status shows: busy
# Active tasks: 2/2

# Complete a task
kanban task complete KAN-42 --agent <agent-id> --confidence 0.95 --summary "Done"

# Status reverts to: idle
# Active tasks: 1/2
```

## Offline State

Agent has stopped sending heartbeats (timed out).

**Conditions:**
- No heartbeat received for > 30 minutes
- System automatically detects and transitions

**What happens:**
1. All claimed/executing tasks are reclaimed
2. Agent status set to `offline`
3. Agent can reconnect by sending heartbeat

**Transitions:**
- From: Idle or Busy (automatic, via timeout detection)
- To: Idle (when heartbeat received)

**Example:**

```bash
# Agent goes offline (no heartbeat for 30+ minutes)
# System detects and reclaims tasks
kanban task list --project 1 --json | jq '.[] | select(.claimed_by == "agent-id")'
# Empty (tasks reclaimed)

# Agent reconnects
kanban agent heartbeat --id <agent-id>

# Status: idle
# Can claim tasks again
```

## Registration

Create a new agent instance.

**CLI Command:**

```bash
kanban agent register [OPTIONS]
```

**Parameters:**
- `--name <NAME>` - Optional agent name (auto-generated if not provided)
- `--agent-type <TYPE>` - Agent type (claude, claude-code, codex, gemini, custom)
- `--skills <SKILLS>` - Comma-delimited skills (required)
- `--task-types <TYPES>` - Comma-delimited task types (optional)
- `--max-concurrent <N>` - Max concurrent tasks (default: 1)
- `--max-complexity <LEVEL>` - Max complexity: small, medium, large (default: large)
- `--worktree-path <PATH>` - Working directory (optional)

**Actions on registration:**

1. Generate unique UUID for agent
2. Create member record (for issue assignments)
3. Store agent metadata (skills, type, capabilities)
4. Initialize stats record (0 tasks, 0 failures)
5. Set status to `idle`
6. Record `registered_at` timestamp

**Example:**

```bash
AGENT=$(kanban agent register \
  --name "backend-analyzer" \
  --agent-type claude \
  --skills "rust,database,api,sql" \
  --max-concurrent 3 \
  --max-complexity large \
  --json)

echo $AGENT | jq '.id'
# 550e8400-e29b-41d4-a716-446655440000
```

## Heartbeat

Agents send periodic heartbeats to indicate they are alive and responsive.

**CLI Command:**

```bash
kanban agent heartbeat --id <AGENT_ID>
```

**Actions on heartbeat:**

1. Update `last_heartbeat` timestamp to now
2. Count active tasks (claimed + executing)
3. Determine new status:
   - If active_count >= max_concurrent: set to `busy`
   - Else: set to `idle`
4. Update `last_activity_at` timestamp
5. Return updated agent state

**Example:**

```bash
# Heartbeat every 60 seconds
while true; do
  kanban agent heartbeat --id <agent-id>
  sleep 60
done
```

**Timing:**
- Recommended: Every 30-60 seconds
- Timeout threshold: 30 minutes
- After timeout: Agent marked offline, tasks reclaimed

## Deregistration

Remove an agent from the board.

**CLI Command:**

```bash
kanban agent deregister --id <AGENT_ID>
```

**Actions on deregistration:**

1. Find all active tasks (claimed or executing)
2. For each active task:
   - Set `task_state` to `unclaimed`
   - Set `claimed_by` to NULL
   - Reset claimed/started timestamps
3. Update agent status to `deregistered`
4. Preserve agent record and member for history

**Example:**

```bash
# List active tasks
kanban task list --agent <agent-id> --json

# Deregister
kanban agent deregister --id <agent-id>

# Verify reclamation
kanban task list --agent <agent-id> --json
# Empty (all reclaimed)
```

## Timeout and Recovery

### Timeout Detection

The system automatically detects offline agents:

```bash
# Background process checks every 5 minutes
SELECT id, last_heartbeat FROM agents
WHERE status != 'deregistered'
  AND datetime(last_heartbeat) < datetime('now', '-30 minutes')
```

### Automatic Reclamation

When timeout is detected:

1. Find all claimed/executing tasks
2. Set them to `unclaimed` and `claimed_by = NULL`
3. Mark agent as `offline`
4. Log timeout event

### Recovery

Agent reconnects by sending heartbeat:

```bash
kanban agent heartbeat --id <agent-id>
```

Board detects the heartbeat and:
1. Updates `last_heartbeat` to now
2. Transitions status from `offline` to `idle`
3. Agent can claim tasks again

**Note:** Tasks that were reclaimed are still available, not automatically re-assigned.

## Skill Matching

When claiming tasks, the board matches agent skills to task requirements.

**Match Algorithm:**

```
required_skills = task.required_skills  # [python, testing, refactoring]
agent_skills = agent.skills              # [python, rust, testing]

matching = count(skill in agent_skills for skill in required_skills)
matched = matching / len(required_skills)  # 2/3 = 0.66

if matched >= threshold (e.g., 2/3):
  offer_task_to_agent()
```

**Example:**

```bash
# Task requires these skills
kanban task create \
  --skills "python,testing,refactoring"

# Agent has these skills
kanban agent register \
  --skills "python,rust,testing"

# Match: 2/3 required skills → Task offered

# Agent can also override skills when claiming
kanban task next --agent <agent-id> --skills "python,refactoring"
```

## Complexity Filtering

Tasks have complexity levels. Agents have maximum complexity they can handle.

**Levels:**
- `small` - Well-scoped, straightforward
- `medium` - Multi-step, moderate difficulty
- `large` - Complex, architectural

**Filtering:**

Agent with `max_complexity=medium` will NOT be offered:
- Large complexity tasks

Agent with `max_complexity=large` will be offered:
- Small, medium, and large tasks

**Example:**

```bash
# Register agent with complexity limit
kanban agent register \
  --name "junior-dev" \
  --skills "python" \
  --max_complexity small  # Only small tasks

# Create different complexity tasks
kanban task create --title "Easy fix" --complexity small
kanban task create --title "Feature" --complexity medium
kanban task create --title "Refactor" --complexity large

# Junior can only claim the first one
kanban task next --agent <junior-id>
# Returns: Easy fix (small complexity)
```

## Concurrency Control

`max_concurrent` limits the number of simultaneous active tasks per agent.

**Example:**

```bash
# Register with capacity for 2 concurrent tasks
kanban agent register \
  --name "multi-agent" \
  --max_concurrent 2

# Claim first task
kanban task next --agent <agent-id>
# Status: idle (1/2 active)

# Claim second task
kanban task next --agent <agent-id>
# Status: busy (2/2 active)

# Try to claim third
kanban task next --agent <agent-id>
# Error: Agent at capacity

# Complete one
kanban task complete KAN-40 --agent <agent-id> ...
# Status: idle (1/2 active)

# Can claim again
kanban task next --agent <agent-id>
# Status: busy (2/2 active)
```

## Statistics and Metrics

The board tracks agent performance over time:

**Metrics:**
- `tasks_completed` - Total successful tasks
- `tasks_failed` - Total failed tasks
- `total_confidence` - Sum of confidence scores
- `total_completion_time_seconds` - Total time across all tasks
- `skills_breakdown` - Task count by skill

**View metrics:**

```bash
kanban agent stats <agent-id>

# Or via JSON
kanban metrics --agent <agent-id> --json
```

**Example output:**

```json
{
  "agent_id": "550e8400-e29b-41d4-a716-446655440000",
  "tasks_completed": 42,
  "tasks_failed": 3,
  "total_confidence": 39.84,
  "total_completion_time_seconds": 882000,
  "skills_breakdown": {
    "python": 25,
    "testing": 18,
    "rust": 12
  }
}
```

## State Diagram

```
┌─────────────────────────────────────────────┐
│         AGENT LIFECYCLE DIAGRAM             │
└─────────────────────────────────────────────┘

         ┌──────────────┐
         │  Registered  │
         └────────┬─────┘
                  │
                  ▼
         ┌──────────────┐
         │   (idle)     │ ◄─────────┐
         └────────┬─────┘           │
                  │                 │
        (claim task or              │
         task complete)             │
                  │                 │
                  ▼                 │
         ┌──────────────┐           │
         │   (busy)     │───────────┘
         └────────┬─────┘
                  │
           (no heartbeat
            > 30 min)
                  │
                  ▼
         ┌──────────────┐
         │  (offline)   │
         └────────┬─────┘
                  │
          (heartbeat
           received)
                  │
                  └────────────────┘

        ┌──────────────┐
        │ Deregister   │ (manual)
        │  (reclaim    │
        │   tasks)     │
        └──────────────┘
```

## See Also

- [Agent Protocol Overview](./index.md)
- [Task Contracts](./task-contracts.md)
- [Agent Examples](./examples.md)
