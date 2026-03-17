# Agent Routing

Agent routing is the automated process of matching agents to tasks. When an agent calls `next_task()`, Kanban finds the best available task based on skills, complexity, capacity, and priority.

## Overview

The routing algorithm:
1. **Check capacity** — Agent at max concurrent tasks?
2. **Find candidates** — Queued tasks in `unstarted` statuses
3. **Filter** — Remove blocked tasks, complexity mismatches, skill gaps
4. **Sort** — By priority (urgent → high → medium → low)
5. **Claim** — First match, atomically (prevent races)

```
Agent calls: next_task()
      ↓
Check: Active tasks < max_concurrent?
      ↓ Yes
Find: Queued tasks in unstarted status
      ↓
Filter: Remove blocked (via issue_relations)
      ↓
Filter: Remove complexity > agent.max_complexity
      ↓
Filter: Remove missing required_skills
      ↓
Sort: By priority DESC, created_at ASC
      ↓
Claim: First match atomically
      ↓
Return: FullTaskContract or None
```

## Registering an Agent

Before routing, agents must register with their capabilities:

### Via CLI

```bash
kanban cli agent register \
  --name "code-reviewer" \
  --agent-type claude \
  --skills "code-review,typescript,testing,documentation" \
  --task-types "review,decomposition" \
  --max-concurrent 2 \
  --max-complexity large \
  --worktree-path "/tmp/agents/code-reviewer"
```

This creates an agent with:
- **ID** — UUID (auto-generated)
- **Name** — Friendly name
- **Agent Type** — `claude`, `codex`, `gemini`, or custom
- **Skills** — Capabilities the agent has
- **Task Types** — Which task types it handles
- **Max Concurrent** — How many tasks simultaneously (default: 1)
- **Max Complexity** — Highest complexity accepted (default: `large`)
- **Worktree Path** — Working directory (optional)

The system auto-creates a member for the agent (e.g., `[claude] code-reviewer`) and initializes stats.

## How Capacity is Checked

An agent won't receive new tasks if it's at capacity:

```bash
# Agent has max_concurrent=2

# Task 1: Agent claims it (active_count=1, allowed)
kanban cli agent next-task --agent-id "agent-xyz"
# Returns: Task A (task_state='claimed')

# Task 2: Agent claims it (active_count=2, allowed)
kanban cli agent next-task --agent-id "agent-xyz"
# Returns: Task B (task_state='claimed')

# Task 3: Agent tries to claim (active_count=2, NOT allowed)
kanban cli agent next-task --agent-id "agent-xyz"
# Returns: None (agent is at capacity)

# Agent completes Task A (active_count=1)
kanban cli task complete --identifier PROJ-1 --agent-id agent-xyz
# Returns: Task A completed

# Task 3: Now agent can claim (active_count=1 < 2)
kanban cli agent next-task --agent-id "agent-xyz"
# Returns: Task C
```

Active tasks are those in `claimed` or `executing` states:

```bash
SELECT COUNT(*) FROM task_contracts
WHERE claimed_by = 'agent-xyz'
  AND task_state IN ('claimed', 'executing')
```

## How Skills are Matched

Tasks require a set of skills. Agents have a set of skills. **Agents only get tasks where they have all required skills.**

```bash
# Task: --skills "typescript,express,testing"
# Requires: typescript AND express AND testing

# Agent 1: skills = ["typescript", "express", "testing", "databases"]
# ✓ Match (has all required + extra)

# Agent 2: skills = ["typescript", "testing"]
# ✗ No match (missing express)

# Agent 3: skills = ["rust", "golang"]
# ✗ No match (missing all)
```

### Subset Matching
An agent can have extra skills beyond what's required:

```bash
Task requirement: ["coding", "testing"]

Agent 1: ["coding", "testing"]
Agent 2: ["coding", "testing", "documentation"]
Agent 3: ["coding", "testing", "typescript", "javascript", "rust"]

All three agents match. Extra skills are fine.
```

### Skill Matching Example

```bash
# Create a task with strict skill requirement
kanban cli task create \
  --project 1 \
  --title "Implement Redis caching" \
  --type implementation \
  --skills "redis,typescript,databases" \
  --complexity medium

# Agent A: skills = ["redis", "typescript", "databases"]
# ✓ Matches (has exactly required skills)

# Agent B: skills = ["typescript", "javascript"]
# ✗ Doesn't match (missing redis and databases)

# Agent C: skills = ["redis", "typescript", "databases", "devops", "docker"]
# ✓ Matches (has all required + extra)

# When Agent A or C calls next_task(), this task is available.
# When Agent B calls, this task is skipped.
```

## How Complexity is Matched

Tasks have an estimated complexity. Agents have a max_complexity limit. **Tasks can't exceed agent's max.**

```
Complexity ranks:
1 = small
2 = medium
3 = large

Agent max_complexity:
- "small" → can take: small
- "medium" → can take: small, medium
- "large" → can take: small, medium, large
```

### Complexity Matching Example

```bash
# Create three tasks with different complexities
kanban cli task create --project 1 --title "Task A" --complexity small
kanban cli task create --project 1 --title "Task B" --complexity medium
kanban cli task create --project 1 --title "Task C" --complexity large

# Agent 1: max_complexity = "small"
# Can claim: Task A only

# Agent 2: max_complexity = "medium"
# Can claim: Task A, Task B

# Agent 3: max_complexity = "large"
# Can claim: Task A, Task B, Task C
```

## How Blocking Relations are Handled

Tasks with blocking relations are skipped:

```bash
# Scenario:
# Task 1: "Implement API"
# Task 2: "Write tests" (blocked_by Task 1)

# Relation: Task 1 blocks Task 2

# Status:
# Task 1: queued
# Task 2: queued (but blocked)

# When agent calls next_task():
# Agent will get Task 1, not Task 2
# Because Task 2 has a blocking relation to Task 1 that's not completed
```

### Dependency Resolution Example

```bash
# Create tasks with dependencies
kanban cli task create --project 1 --title "Setup DB" --type implementation
# Returns: PROJ-1

kanban cli task create --project 1 --title "Create tables" --type implementation
# Returns: PROJ-2

kanban cli task create --project 1 --title "Seed data" --type implementation
# Returns: PROJ-3

# Define blocking relations
kanban cli issue block PROJ-2 --by PROJ-1  # PROJ-2 blocked by PROJ-1
kanban cli issue block PROJ-3 --by PROJ-2  # PROJ-3 blocked by PROJ-2

# Agent requests work
kanban cli agent next-task --agent-id "agent-1"
# Returns: PROJ-1 (no blockers, can start)

# Agent completes PROJ-1
kanban cli task complete --identifier PROJ-1

# Next time agent requests
kanban cli agent next-task --agent-id "agent-1"
# Returns: PROJ-2 (PROJ-1 done, now available)

# Agent completes PROJ-2
# Now PROJ-3 becomes available
```

## How Sorting Works

When multiple tasks match (same complexity, all skills present), they're sorted by:
1. **Priority** (descending) — urgent > high > medium > low
2. **Created At** (ascending) — older tasks first

```bash
# Candidates after filtering:
# Task A: priority=medium, created=2025-03-15 10:00
# Task B: priority=high, created=2025-03-15 09:00
# Task C: priority=medium, created=2025-03-15 08:00

# Sorted order (agent gets in this order):
# 1. Task B (priority=high, most recent among high)
# 2. Task A (priority=medium, newer than Task C)
# 3. Task C (priority=medium, oldest)
```

## How Claiming Works

When an agent claims a task, it's done **atomically** to prevent race conditions:

```sql
-- Atomic claim (prevents multiple agents from claiming same task)
BEGIN TRANSACTION
  SELECT FOR UPDATE FROM task_contracts WHERE issue_id = X
  UPDATE task_contracts
    SET task_state='claimed', claimed_by='agent-id', claimed_at=NOW()
    WHERE issue_id = X AND task_state='queued'
  -- If UPDATE affected 0 rows, another agent claimed it first
COMMIT
```

If two agents try to claim the same task simultaneously:
- First agent gets it
- Second agent gets `None` from `next_task()` and tries the next candidate

This is **optimistic locking**: no waiting, just retries with next candidate.

## Calling next_task

Agents call this to get work:

### Via CLI

```bash
kanban cli agent next-task \
  --agent-id "550e8400-e29b-41d4-a716-446655440000"

# Returns (if task available):
# {
#   "identifier": "API-42",
#   "title": "Implement password reset",
#   "issue_id": 42,
#   "type": "implementation",
#   "task_state": "claimed",
#   "objective": "Add password reset flow",
#   "context": { "files": [...], "related_tasks": [...] },
#   "constraints": ["Must not break existing API"],
#   "success_criteria": [...],
#   "required_skills": ["typescript", "auth"],
#   "estimated_complexity": "medium",
#   "timeout_minutes": 60,
#   "claimed_by": "agent-xyz",
#   "claimed_at": "2025-03-15T10:00:00Z"
# }

# Or if at capacity or no tasks available:
# null
```

### Return Value
Returns a `FullTaskContract` with all issue + contract data, or `None`.

## Agent Heartbeat

Agents should send heartbeats regularly to stay alive:

```bash
# Agent heartbeat every 30 seconds
kanban cli agent heartbeat --agent-id "550e8400-e29b-41d4-a716-446655440000"

# Updates:
# - last_heartbeat = now
# - status = idle|busy (based on active tasks)
```

Agent status is computed as:
```
if active_count >= max_concurrent:
  status = "busy"
else:
  status = "idle"
```

If an agent misses too many heartbeats, it's marked `offline`:
```
missed_heartbeats_before_offline = 3 (configurable per project)
heartbeat_interval_seconds = 30 (configurable per project)
Timeout = 30 * 3 = 90 seconds without heartbeat
```

## Deregistering Agents

When an agent shuts down, deregister it:

```bash
kanban cli agent deregister --agent-id "550e8400-e29b-41d4-a716-446655440000"

# This:
# 1. Marks agent as offline
# 2. Unassigns all claimed/executing tasks
# 3. Moves them back to queued
# 4. Keeps agent record and stats for history
```

## Agent Stats

Track agent performance:

```bash
kanban cli agent stats --agent-id "550e8400-e29b-41d4-a716-446655440000"

# Returns:
# {
#   "agent_id": "550e8400-e29b-41d4-a716-446655440000",
#   "tasks_completed": 23,
#   "tasks_failed": 2,
#   "avg_confidence": 0.89,
#   "avg_completion_time_minutes": 3.5,
#   "skills_breakdown": {
#     "typescript": 15,
#     "testing": 10,
#     "documentation": 5,
#     "code-review": 3
#   }
# }
```

These stats are updated as tasks complete. Use them to:
- Monitor agent health
- Identify high-performing agents
- Track skill usage
- Calculate SLAs

## Project Agent Configuration

Per-project settings control agent behavior:

```bash
# View project agent config
kanban cli project get 1

# Shows:
# {
#   ...
#   "agent_config": {
#     "auto_accept_threshold": 0.85,
#     "human_review_threshold": 0.50,
#     "max_attempts": 3,
#     "heartbeat_interval_seconds": 60,
#     "missed_heartbeats_before_offline": 3
#   }
# }
```

| Setting | Default | Meaning |
|---------|---------|---------|
| `auto_accept_threshold` | 0.85 | If confidence >= 85%, auto-accept result |
| `human_review_threshold` | 0.50 | If confidence < 50%, require human review |
| `max_attempts` | 3 | Retry task up to 3 times |
| `heartbeat_interval_seconds` | 60 | Expect heartbeat every 60s |
| `missed_heartbeats_before_offline` | 3 | After 3 missed, mark offline |

See **[Validation](/guide/validation.md)** for how `auto_accept_threshold` and `human_review_threshold` work.

## Best Practices

### 1. Register with Accurate Skills
Only list skills agent truly has:
```bash
# Good: What the agent actually does
kanban cli agent register \
  --name "code-reviewer" \
  --skills "code-review,typescript,testing,architecture"

# Bad: Overpromising
kanban cli agent register \
  --name "everything-agent" \
  --skills "coding,testing,devops,design,documentation,pm"
```

### 2. Set max_concurrent Realistically
Match agent's actual capacity:
```bash
# Conservative (1-2 tasks at a time)
--max-concurrent 1

# Balanced (2-3 tasks)
--max-concurrent 2

# High-throughput (4+)
--max-concurrent 4
```

### 3. Set max_complexity Appropriate to Agent
Don't set higher than agent can handle:
```bash
# Simple agent
--max-complexity small

# General-purpose agent
--max-complexity large

# Very capable agent
--max-complexity large
```

### 4. Send Heartbeats Regularly
Especially if agent hangs or crashes:
```bash
# Every 30 seconds, send heartbeat
kanban cli agent heartbeat --agent-id "..."
```

### 5. Deregister Cleanly
On shutdown, deregister instead of just disappearing:
```bash
# Before shutdown
kanban cli agent deregister --agent-id "..."
```

### 6. Monitor Agent Stats
Periodically check performance:
```bash
kanban cli agent stats --agent-id "..."

# Look for:
# - Low tasks_completed: Agent isn't getting work
# - High tasks_failed: Issues with agent logic
# - Low avg_confidence: Agent unsure about results
# - Unbalanced skills_breakdown: Might be overspecialized
```

## Example Routing Scenario

```bash
# Setup
Project: API (1 task requires typescript + testing)
Agent 1: skills=["typescript"], max_complexity=large, max_concurrent=1
Agent 2: skills=["typescript", "testing"], max_complexity=large, max_concurrent=2

kanban cli task create \
  --project 1 \
  --title "Add rate limiting" \
  --type implementation \
  --skills "typescript,testing" \
  --complexity medium
# Returns: API-1 (queued)

# Agent 1 requests work
kanban cli agent next-task --agent-id "agent-1"
# Returns: null (has typescript but missing testing skill)

# Agent 2 requests work
kanban cli agent next-task --agent-id "agent-2"
# Returns: API-1 task (has all skills, not at capacity)
# Internal state: task_state=claimed, claimed_by=agent-2

# Agent 1 tries again (trying to get Agent 2's task)
kanban cli agent next-task --agent-id "agent-1"
# Returns: null (API-1 already claimed by agent-2, no other tasks)

# Agent 2 completes task
kanban cli task complete --identifier API-1 --agent-id agent-2 --confidence 0.92
# Internal: task_state=validating, then completed

# Now both agents idle waiting for new work
```

## Next Steps

- **[Task Contracts](/guide/task-contracts.md)** — Create tasks for agents
- **[Validation](/guide/validation.md)** — How results are validated
- **[Execution & Replay](/guide/execution-replay.md)** — View what agents did
