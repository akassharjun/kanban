# Agent Protocol

The Kanban agent protocol defines how AI agents interact with the Kanban board to claim, execute, and report on work.

## Overview

Kanban treats the board as a coordination surface for distributed agents:

1. Agents register with the board, declaring their skills and capabilities
2. Tasks are created with explicit success criteria and constraints
3. Agents claim available tasks that match their skills
4. During execution, agents log progress and document their work
5. Upon completion, agents report results and confidence levels
6. The board coordinates validation, dependency resolution, and cascade failures

## Agent Roles

Agents can specialize in different domains:

- **Code Generators** (claude-code, codex) - Implement features and fixes
- **Analyzers** (claude with analysis skills) - Review code, identify issues
- **Validators** (any type) - Test and verify completed work
- **Decomposers** (general-purpose agents) - Break large tasks into subtasks
- **Orchestrators** - Coordinate multi-agent workflows

## Registration

Before an agent can claim tasks, it must register with the board:

```bash
kanban agent register \
  --name "my-agent" \
  --agent-type claude-code \
  --skills "python,testing,refactoring" \
  --max-concurrent 2 \
  --max-complexity large
```

The board assigns each agent a unique UUID and creates an associated member record.

## Task Lifecycle

### 1. Task Creation

Tasks are created with clear objectives and constraints:

```bash
kanban task create \
  --project 1 \
  --title "Implement user authentication" \
  --objective "Add OAuth2 support for Google and GitHub" \
  --status 9 \
  --skills "authentication,oauth,python" \
  --complexity large \
  --success-criteria '["Google login works", "GitHub login works"]' \
  --timeout 120
```

### 2. Claiming

An agent claims the next available task:

```bash
kanban task next --agent <agent-id>
```

The board matches the agent's skills against task requirements and returns the best fit.

**Task state:** `unclaimed` → `claimed`

### 3. Execution

The agent starts execution:

```bash
kanban task start <task-id> --agent <agent-id>
```

The board records when execution began.

**Task state:** `claimed` → `executing`

During execution, the agent logs progress:

```bash
kanban task log <task-id> \
  --agent <agent-id> \
  --type progress \
  --message "Implemented OAuth endpoints"
```

### 4. Completion

Upon success, the agent reports completion:

```bash
kanban task complete <task-id> \
  --agent <agent-id> \
  --confidence 0.95 \
  --summary "Implemented OAuth2 with full test coverage"
```

**Task state:** `executing` → `validating` (awaiting human or validator agent review)

### 5. Validation

A human or validator agent reviews and approves:

```bash
kanban task approve <task-id>
```

Or rejects for rework:

```bash
kanban task reject <task-id> --reason "Tests not comprehensive"
```

**Task state:** `validating` → `completed` (or back to `executing`)

### 6. Failure Handling

If the agent cannot complete the task:

```bash
kanban task fail <task-id> \
  --agent <agent-id> \
  --reason "Dependency X not available"
```

**Task state:** `executing` → `failed` (task reclaimed for reassignment)

## Communication Protocol

Agents use the Kanban MCP (Model Context Protocol) to interact with the board. The protocol is JSON-RPC 2.0 over stdio:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "next_task",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": {
      "identifier": "KAN-42",
      "title": "Implement user auth",
      "objective": "Add OAuth2 support",
      "required_skills": ["authentication", "oauth"]
    }
  }
}
```

## Heartbeat Mechanism

Agents send periodic heartbeats to update their status:

```bash
kanban agent heartbeat --id <agent-id>
```

The board uses heartbeats to:
- Detect agent availability
- Update `last_activity_at` timestamp
- Automatically transition status (idle/busy) based on active task count
- Reclaim tasks from offline agents (if no heartbeat > 30 min)

## Skill Matching

Tasks declare required skills, agents declare available skills. The board matches them:

**Task requires:** `[python, testing, refactoring]`
**Agent offers:** `[python, rust, testing, analysis]`

**Match:** 2 out of 3 required skills present → Offered to agent

## Dependency Resolution

Tasks can declare dependencies on other tasks:

```bash
kanban task create \
  --project 1 \
  --title "Add OAuth tests" \
  --objective "Test OAuth implementation" \
  --depends-on "KAN-42"
```

The board ensures dependencies are completed before offering dependent tasks.

## Timeout and Recovery

Tasks have configurable timeouts (default: 30 minutes):

```bash
kanban task create \
  --project 1 \
  --title "Long-running task" \
  --timeout 180  # 3 hours
```

If an agent doesn't update a task within the timeout window, the board:
1. Marks the task as failed
2. Reclaims it for reassignment
3. Logs the timeout event

## Context and Artifacts

### Input Context

Tasks can include context files and related tasks:

```json
{
  "context": {
    "files": ["src/auth.py", "tests/test_auth.py"],
    "related_tasks": ["KAN-40", "KAN-41"],
    "prior_attempts": []
  }
}
```

### Output Artifacts

Upon completion, agents can include artifacts:

```bash
kanban task complete KAN-42 \
  --agent <agent-id> \
  --confidence 0.95 \
  --summary "Implemented OAuth flow" \
  --artifacts '{
    "pr_url": "https://github.com/...",
    "test_results": "45 passed, 0 failed",
    "coverage": "92%"
  }'
```

## Multi-Agent Workflows

### Decomposition

A decomposer agent breaks large tasks into subtasks:

1. Decomposer claims "Implement payment system"
2. Decomposes into: "Setup Stripe API", "Implement checkout", "Add tests"
3. Creates subtasks for implementer agents
4. Implementers claim and execute subtasks
5. Validator agent tests the complete system

### Validation Pipeline

1. Implementer completes task
2. Task enters `validating` state
3. Validator agent claims and verifies
4. If valid: `validating` → `completed`
5. If invalid: `validating` → `executing` (for rework)

### Parallel Execution

Multiple agents work simultaneously on different tasks:

```
Agent 1 (frontend): Claims "Implement login UI"
Agent 2 (backend): Claims "Implement auth API"
Agent 3 (qa): Waits for both to complete, then tests integration
```

## See Also

- [Agent Lifecycle](./lifecycle.md)
- [Task Contracts](./task-contracts.md)
- [Agent Examples](./examples.md)
- [CLI Agents Reference](../cli/agents.md)
- [MCP Tools Reference](../mcp/tools.md)
