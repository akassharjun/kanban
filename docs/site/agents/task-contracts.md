# Task Contracts

Task contracts define the explicit agreement between the board and agents about what work is expected.

## Relationship to Issues

**Issues come first.** An issue is the unit of work visible on the board — it has a title, status, priority, assignee, labels, and description. Any human or agent can create and manage issues.

A **task contract** is an optional execution spec that gets attached to an issue when you want an agent to work on it autonomously. It adds agent-specific fields like required skills, complexity, success criteria, and timeouts. Think of it as: the issue says *what* needs to be done, the task contract says *how an agent should do it*.

```
Issue (always exists)
  └── Task Contract (optional, for agent execution)
        ├── Objective & success criteria
        ├── Required skills & complexity
        ├── Constraints & timeout
        └── Dependencies on other contracts
```

You can create issues without task contracts (for human work or simple agent tasks), and you can add a task contract to an existing issue later when you want to delegate it to an agent.

## Overview

A task contract extends an issue with:

- **Clear objective** - What the agent must accomplish
- **Success criteria** - How to measure success
- **Constraints** - What is off-limits
- **Required skills** - What the agent must know
- **Complexity estimate** - How hard the work is
- **Timeout** - How long the agent has
- **Context** - Files and related work
- **Dependencies** - What must be done first

## Fields

### Objective

The primary goal statement. Unlike a title (which is short), the objective explains the "why" and "what".

```bash
kanban task create \
  --project 1 \
  --title "Implement OAuth" \
  --objective "Add OAuth2 authentication for Google and GitHub to allow users to sign in without creating new passwords"
```

### Success Criteria

Array of measurable success criteria.

```bash
kanban task create \
  --project 1 \
  --title "User authentication" \
  --objective "..." \
  --success-criteria '[
    "OAuth login page accessible",
    "Google OAuth flow completes",
    "GitHub OAuth flow completes",
    "User can sign in with either provider",
    "Tests cover all flows with 90% coverage"
  ]'
```

The agent must verify all criteria before marking the task complete.

### Constraints

Explicit boundaries on what the agent can/cannot do.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --constraints '[
    "No third-party OAuth libraries",
    "Must use existing session system",
    "Cannot modify user schema",
    "Backward compatibility required"
  ]'
```

### Required Skills

Skills the agent must have. The board only offers tasks to agents matching 2 out of 3 required skills.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --skills "authentication,oauth,python"
```

Typical skill categories:
- **Languages:** python, rust, typescript, go, java
- **Frameworks:** react, fastapi, spring, django
- **Domains:** authentication, database, api, frontend, testing
- **Specialties:** security, performance, refactoring, documentation

### Complexity

Helps the board route tasks to appropriately-skilled agents.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --complexity large  # small | medium | large
```

Agents can be limited to maximum complexity:

```bash
kanban agent register \
  --skills "..." \
  --max-complexity medium  # Only accepts small/medium
```

### Timeout

How long the agent has to complete the task (in minutes).

```bash
kanban task create \
  --project 1 \
  --title "Quick fix" \
  --timeout 30      # 30 minutes

kanban task create \
  --project 1 \
  --title "Large feature" \
  --timeout 480     # 8 hours
```

If the agent doesn't update the task within this time, it's marked as failed and reclaimed.

### Context Files

Paths to files the agent should examine before starting.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --context-files "src/auth.py,tests/test_auth.py,docs/oauth.md"
```

The agent can use these to understand existing code and patterns.

### Type

Categorize the type of work.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --type feature      # feature | bugfix | refactor | task | research
```

### Priority

Standard priority levels.

```bash
kanban task create \
  --project 1 \
  --title "..." \
  --priority high     # none | low | medium | high | urgent
```

### Dependencies

Other tasks that must be completed before this one can start.

```bash
kanban task create \
  --project 1 \
  --title "Test OAuth" \
  --depends-on "KAN-40,KAN-41,KAN-42"
```

The board ensures all dependencies are in `completed` state before offering this task.

## Task State Machine

A task progresses through states representing its lifecycle:

```
unclaimed
   ↓
claimed (agent claimed it, not started)
   ↓
executing (agent is working)
   ├─→ progress logged
   ├─→ progress logged
   └─→ ready to report
   ↓
validating (awaiting approval/review)
   ├─→ approved → completed
   └─→ rejected → executing (back for rework)

failed (agent couldn't complete)
   └─→ reclaimed → unclaimed (for another agent)

blocked (waiting on dependency)
   └─→ unblocked → unclaimed (available again)
```

**State Definitions:**

| State | Description |
|-------|-------------|
| `unclaimed` | Available for claiming by any matching agent |
| `claimed` | Reserved by an agent, not yet started |
| `executing` | Agent is actively working |
| `validating` | Agent completed, awaiting validation |
| `completed` | Approved and done |
| `failed` | Agent or system marked as failed |
| `blocked` | Waiting on dependency or external blocker |

## Transitions

### Unclaimed → Claimed

Agent claims the task:

```bash
kanban task next --agent <agent-id>
```

Board:
1. Checks agent matches skills (2+ of 3 required)
2. Checks agent below `max_concurrent`
3. Checks all dependencies completed
4. Marks task `claimed`
5. Records `claimed_by` and `claimed_at`
6. Returns task to agent

### Claimed → Executing

Agent starts working:

```bash
kanban task start <task-id> --agent <agent-id>
```

Board records `started_at` timestamp.

### Executing → Validating

Agent completes work:

```bash
kanban task complete <task-id> \
  --agent <agent-id> \
  --confidence 0.95 \
  --summary "Implemented OAuth2 with full test coverage"
```

Board records:
- `completed_by` agent
- `completed_at` timestamp
- `confidence` score (0.0-1.0)
- `summary` text
- Task state → `validating`

### Validating → Completed

Validator approves:

```bash
kanban task approve <task-id>
```

Board sets task state to `completed`.

### Validating → Executing

Validator rejects:

```bash
kanban task reject <task-id> --reason "Tests not comprehensive"
```

Board:
- Logs rejection reason
- Sets task state back to `executing`
- Keeps same `claimed_by` agent
- Agent can resume work or manually unclaim

### Executing → Failed

Agent or system marks as failed:

```bash
kanban task fail <task-id> \
  --agent <agent-id> \
  --reason "API key not available"
```

Board:
- Records failure reason
- Sets task state to `failed`
- Reclaims task (`claimed_by = NULL`)
- Task returns to `unclaimed`
- Another agent can claim and retry

### Any → Blocked

Task is blocked by external event:

```bash
kanban task block <task-id>
```

Board:
- Sets state to `blocked`
- Prevents claiming or execution
- Must be manually unblocked

### Blocked → Unclaimed

Unblock the task:

```bash
kanban task unblock <task-id>
```

Task returns to `unclaimed`.

## Dependency Resolution

### Declaring Dependencies

```bash
kanban task create \
  --project 1 \
  --title "Implement OAuth tests" \
  --depends-on "KAN-40,KAN-41,KAN-42"
```

### Dependency Validation

When an agent tries to claim a task:

```
SELECT count(*) FROM task_contracts
WHERE task_id IN (depends_on_ids)
  AND task_state != 'completed'
```

If any dependency is not `completed`:
- Task cannot be claimed
- Task state: `blocked`
- Agent receives: "Task blocked: waiting on KAN-40"

### Cascade Invalidation

If a dependency is marked failed:

```
UPDATE task_contracts
SET task_state = 'blocked'
WHERE depends_on CONTAINS <failed-task-id>
```

All downstream tasks are blocked, preventing wasted work.

**Example:**

```
KAN-40 (OAuth API setup) ───→ completed
   ↓
KAN-41 (OAuth UI) ──→ executing
   ↓
KAN-42 (OAuth tests) ──→ blocked (waiting on KAN-41)

If KAN-40 fails:
  KAN-41 becomes blocked
  KAN-42 stays blocked
  Notify agents of cascade failure
```

## Execution Logging

Agents log progress during execution:

```bash
kanban task log <task-id> \
  --agent <agent-id> \
  --type progress \
  --message "Implemented Google OAuth flow"

kanban task log <task-id> \
  --agent <agent-id> \
  --type warning \
  --message "Stripe API rate limit approaching"

kanban task log <task-id> \
  --agent <agent-id> \
  --type error \
  --message "Database connection failed, retrying"
```

Log entries:
- Are timestamped
- Include agent ID
- Include message and optional metadata
- Enable task replay and debugging

## Artifacts

Upon completion, agents can attach artifacts:

```bash
kanban task complete <task-id> \
  --agent <agent-id> \
  --confidence 0.95 \
  --summary "Implemented OAuth2" \
  --artifacts '{
    "pr_url": "https://github.com/org/repo/pull/123",
    "test_results": {
      "passed": 45,
      "failed": 0
    },
    "code_coverage": "92%"
  }'
```

Artifacts are stored with the task for historical reference.

## Validation Workflow

```
┌──────────────────────────────────────────┐
│ TASK VALIDATION WORKFLOW                 │
└──────────────────────────────────────────┘

1. Agent completes task
   └─→ task state: validating

2. Validator agent claims validation task
   └─→ Replays execution logs
   └─→ Tests success criteria
   └─→ Reviews code

3a. Validator approves
    └─→ task state: completed
    └─→ Update issue status
    └─→ Cascade unblock downstream tasks

3b. Validator rejects
    └─→ task state: executing (back to original agent)
    └─→ Log rejection reason
    └─→ Agent addresses feedback and resubmits

3c. Validator marks as failed
    └─→ task state: failed
    └─→ Task reclaimed
    └─→ Another agent can retry
```

## Task Decomposition

Large tasks can be broken into subtasks:

```bash
# Create parent task
PARENT=$(kanban task create \
  --project 1 \
  --title "Implement payment system" \
  --objective "Add Stripe integration" \
  --complexity large \
  --json | jq -r '.id')

# Decompose into subtasks
kanban task create \
  --project 1 \
  --title "Setup Stripe account and API keys" \
  --parent $PARENT

kanban task create \
  --project 1 \
  --title "Implement checkout page" \
  --parent $PARENT \
  --depends-on "KAN-40"  # Depends on setup

kanban task create \
  --project 1 \
  --title "Implement order confirmation" \
  --parent $PARENT \
  --depends-on "KAN-41"  # Depends on checkout

kanban task create \
  --project 1 \
  --title "Test payment workflows" \
  --parent $PARENT \
  --depends-on "KAN-41,KAN-42"  # Depends on both
```

Parent task completes only when all subtasks are completed.

## See Also

- [Agent Protocol Overview](./index.md)
- [Agent Lifecycle](./lifecycle.md)
- [Agent Examples](./examples.md)
- [Task Contract CLI Reference](../cli/tasks.md)
