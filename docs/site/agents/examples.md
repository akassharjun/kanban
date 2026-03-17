# Agent Examples

Complete worked examples showing agent workflows.

## Example 1: Simple Task Execution

A single agent claims and completes a straightforward task.

### Workflow

1. Create a task
2. Register an agent
3. Agent claims the task
4. Agent starts execution
5. Agent logs progress
6. Agent completes the task
7. Human validates and approves

### Step-by-step

**1. Create the task:**

```bash
kanban task create \
  --project 1 \
  --title "Add dark mode toggle" \
  --objective "Implement dark mode support in the UI" \
  --status 9 \
  --type feature \
  --priority medium \
  --skills "typescript,react,css" \
  --complexity medium \
  --success-criteria '[
    "Dark mode toggle visible in settings",
    "All pages render correctly in dark mode",
    "User preference persisted",
    "Tests cover dark mode functionality"
  ]' \
  --timeout 120
```

Output:

```
Created task: KAN-100 - Add dark mode toggle
Objective: Implement dark mode support in the UI
Status: Todo
Complexity: medium
Timeout: 120 minutes
```

**2. Register an agent:**

```bash
AGENT_ID=$(kanban agent register \
  --name "frontend-dev" \
  --agent-type claude-code \
  --skills "typescript,react,css,testing" \
  --max-concurrent 2 \
  --json | jq -r '.id')

echo "Agent registered: $AGENT_ID"
```

**3. Agent claims the task:**

```bash
# Agent asks for next task
TASK=$(kanban task next --agent "$AGENT_ID" --json)
IDENTIFIER=$(echo "$TASK" | jq -r '.identifier')

echo "Claimed task: $IDENTIFIER"
# Output: Claimed task: KAN-100
```

**4. Agent starts execution:**

```bash
kanban task start "$IDENTIFIER" --agent "$AGENT_ID"

# Output:
# Started execution: KAN-100
# Agent: frontend-dev
# State: executing
# Started at: 2025-03-15T10:01:00Z
```

**5. Agent logs progress:**

```bash
# Progress update 1
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Created dark mode CSS variables"

# Progress update 2
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Implemented toggle component"

# Progress update 3
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Added localStorage persistence"

# Progress update 4
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Tests complete, all green"
```

**6. Agent completes the task:**

```bash
kanban task complete "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --confidence 0.96 \
  --summary "Implemented dark mode with full test coverage. All UI pages render correctly in both light and dark modes. User preference is persisted in localStorage." \
  --artifacts '{
    "files_modified": ["src/theme.css", "src/components/Settings.tsx", "src/hooks/useDarkMode.ts"],
    "tests_added": 12,
    "coverage": "94%"
  }'

# Output:
# Completed task: KAN-100
# State: validating
# Confidence: 96%
# Awaiting validation...
```

**7. Human validates and approves:**

```bash
# Human reviews the execution
kanban task replay "$IDENTIFIER"

# Output:
# Execution log for KAN-100:
# 2025-03-15T10:01:00Z [progress] Created dark mode CSS variables
# 2025-03-15T10:05:00Z [progress] Implemented toggle component
# 2025-03-15T10:12:00Z [progress] Added localStorage persistence
# 2025-03-15T10:25:00Z [progress] Tests complete, all green
# 2025-03-15T10:30:00Z [info] Task submitted for validation

# Approve
kanban task approve "$IDENTIFIER"

# Output:
# Approved task: KAN-100
# State: completed
# Status: Done
```

## Example 2: Multi-Agent Workflow with Decomposition

A large task is decomposed by a coordinator agent, executed by implementers, and validated by QA.

### Workflow

```
Coordinator decomposes task
       ↓
Decomposed into 3 subtasks
       ↓
Implementation agents claim and execute
       ↓
QA agent validates all
       ↓
Human approves final state
```

### Step-by-step

**1. Create the main task:**

```bash
MAIN=$(kanban task create \
  --project 1 \
  --title "Payment integration" \
  --objective "Add Stripe payment processing to the platform" \
  --status 9 \
  --type feature \
  --priority high \
  --complexity large \
  --timeout 480 \
  --json)

MAIN_ID=$(echo "$MAIN" | jq -r '.id')
echo "Main task: KAN-101"
```

**2. Register coordinator and implementer agents:**

```bash
COORDINATOR=$(kanban agent register \
  --name "coordinator" \
  --agent-type claude \
  --skills "planning,architecture,breakdown" \
  --json | jq -r '.id')

BACKEND=$(kanban agent register \
  --name "backend-dev" \
  --agent-type claude \
  --skills "python,api,stripe" \
  --max-concurrent 2 \
  --json | jq -r '.id')

FRONTEND=$(kanban agent register \
  --name "frontend-dev" \
  --agent-type claude-code \
  --skills "typescript,react,forms" \
  --max-concurrent 2 \
  --json | jq -r '.id')

QA=$(kanban agent register \
  --name "qa-bot" \
  --agent-type claude-code \
  --skills "testing,validation,e2e" \
  --json | jq -r '.id')

echo "Agents registered"
```

**3. Coordinator claims and decomposes:**

```bash
# Coordinator claims
kanban task next --agent "$COORDINATOR"

kanban task start KAN-101 --agent "$COORDINATOR"

# Log decomposition
kanban task log KAN-101 \
  --agent "$COORDINATOR" \
  --type progress \
  --message "Decomposing into: Setup API, Checkout UI, Payment Processing"

# Create subtask 1: Setup
SUB1=$(kanban task create \
  --project 1 \
  --title "Setup Stripe API keys and webhooks" \
  --objective "Configure Stripe account and integrate keys" \
  --status 9 \
  --parent 101 \
  --skills "python,api,stripe" \
  --complexity medium \
  --timeout 60 \
  --json)

SUB1_ID=$(echo "$SUB1" | jq -r '.identifier')

# Create subtask 2: Frontend (depends on SUB1)
SUB2=$(kanban task create \
  --project 1 \
  --title "Build checkout form and payment UI" \
  --objective "Create React components for payment collection" \
  --status 9 \
  --parent 101 \
  --skills "typescript,react,forms" \
  --complexity medium \
  --timeout 120 \
  --depends-on "$SUB1_ID" \
  --json)

SUB2_ID=$(echo "$SUB2" | jq -r '.identifier')

# Create subtask 3: Integration (depends on SUB1 and SUB2)
SUB3=$(kanban task create \
  --project 1 \
  --title "Integrate checkout with payment processing" \
  --objective "Wire up frontend form to backend API" \
  --status 9 \
  --parent 101 \
  --skills "python,typescript,integration" \
  --complexity large \
  --timeout 120 \
  --depends-on "$SUB1_ID,$SUB2_ID" \
  --json)

SUB3_ID=$(echo "$SUB3" | jq -r '.identifier')

# Coordinator completes
kanban task complete KAN-101 \
  --agent "$COORDINATOR" \
  --confidence 0.98 \
  --summary "Decomposed payment integration into 3 subtasks with clear dependencies"
```

**4. Implementer agents claim and execute:**

```bash
# Backend agent claims subtask 1
kanban task next --agent "$BACKEND"  # Gets KAN-102 (Setup)

kanban task start KAN-102 --agent "$BACKEND"

kanban task log KAN-102 \
  --agent "$BACKEND" \
  --type progress \
  --message "Configured Stripe API keys and test webhooks"

kanban task complete KAN-102 \
  --agent "$BACKEND" \
  --confidence 0.99 \
  --summary "Stripe keys configured, webhooks active"

# Frontend agent claims subtask 2 (auto-unblocked when SUB1 completes)
kanban task next --agent "$FRONTEND"  # Gets KAN-103 (Checkout UI)

kanban task start KAN-103 --agent "$FRONTEND"

kanban task log KAN-103 \
  --agent "$FRONTEND" \
  --type progress \
  --message "Created checkout form component"

kanban task log KAN-103 \
  --agent "$FRONTEND" \
  --type progress \
  --message "Added form validation"

kanban task complete KAN-103 \
  --agent "$FRONTEND" \
  --confidence 0.97 \
  --summary "Checkout UI complete with form validation"

# Backend agent claims subtask 3 (auto-unblocked when SUB1 and SUB2 complete)
kanban task next --agent "$BACKEND"  # Gets KAN-104 (Integration)

kanban task start KAN-104 --agent "$BACKEND"

kanban task log KAN-104 \
  --agent "$BACKEND" \
  --type progress \
  --message "Wired frontend form to payment API"

kanban task complete KAN-104 \
  --agent "$BACKEND" \
  --confidence 0.96 \
  --summary "End-to-end payment flow implemented"
```

**5. QA validates all subtasks:**

```bash
# QA agent validates subtask 1
kanban task next --agent "$QA"  # Gets KAN-102
kanban task approve KAN-102

# QA agent validates subtask 2
kanban task next --agent "$QA"  # Gets KAN-103
kanban task approve KAN-103

# QA agent validates subtask 3
kanban task next --agent "$QA"  # Gets KAN-104
kanban task approve KAN-104
```

**6. Verify completion:**

```bash
# All subtasks complete
kanban task list --project 1 --parent 101 --json | jq '.[] | {identifier, task_state}'

# Output:
# {
#   "identifier": "KAN-102",
#   "task_state": "completed"
# },
# {
#   "identifier": "KAN-103",
#   "task_state": "completed"
# },
# {
#   "identifier": "KAN-104",
#   "task_state": "completed"
# }

# Parent task is auto-marked complete
kanban task get KAN-101 --json | jq '.task_state'
# "completed"
```

## Example 3: Handling Failures and Retries

An agent fails a task. The system reclaims it, and another agent retries.

### Workflow

```
Agent 1 claims task
       ↓
Agent 1 executes but fails
       ↓
Task reclaimed
       ↓
Agent 2 claims task (retry)
       ↓
Agent 2 succeeds
       ↓
Validated and complete
```

### Step-by-step

**1. Create a task:**

```bash
kanban task create \
  --project 1 \
  --title "Migrate legacy API" \
  --objective "Update endpoints to use new auth system" \
  --status 9 \
  --skills "python,api,migration" \
  --complexity large \
  --timeout 240
```

Output: `KAN-105`

**2. First agent claims and fails:**

```bash
AGENT1=$(kanban agent register \
  --name "junior-dev" \
  --agent-type claude \
  --skills "python,api" \
  --json | jq -r '.id')

kanban task next --agent "$AGENT1"  # Gets KAN-105

kanban task start KAN-105 --agent "$AGENT1"

kanban task log KAN-105 \
  --agent "$AGENT1" \
  --type progress \
  --message "Started migrating endpoints"

# Oops, they hit an issue
kanban task log KAN-105 \
  --agent "$AGENT1" \
  --type error \
  --message "Database schema doesn't support new auth tokens"

# Mark as failed
kanban task fail KAN-105 \
  --agent "$AGENT1" \
  --reason "Database schema incompatible with new auth system"

# Output:
# Task KAN-105 failed
# Agent: junior-dev
# Reason: Database schema incompatible
# Task reclaimed for reassignment
```

**3. Check task history:**

```bash
kanban task attempts KAN-105

# Output:
# Execution attempts for KAN-105:
#
# Attempt 1:
#   Agent: junior-dev
#   Claimed: 2025-03-15T10:00:00Z
#   Started: 2025-03-15T10:01:00Z
#   Failed: 2025-03-15T10:15:00Z
#   Reason: Database schema incompatible
#
# Total attempts: 1
# Status: unclaimed (available for retry)
```

**4. Second agent (more experienced) claims and succeeds:**

```bash
AGENT2=$(kanban agent register \
  --name "senior-dev" \
  --agent-type claude \
  --skills "python,api,migration,database" \
  --json | jq -r '.id')

kanban task next --agent "$AGENT2"  # Gets KAN-105

kanban task start KAN-105 --agent "$AGENT2"

kanban task log KAN-105 \
  --agent "$AGENT2" \
  --type info \
  --message "Analyzed previous attempt: database schema incompatible"

kanban task log KAN-105 \
  --agent "$AGENT2" \
  --type progress \
  --message "Added database migration for auth tokens"

kanban task log KAN-105 \
  --agent "$AGENT2" \
  --type progress \
  --message "Migrated 5 API endpoints to new auth"

kanban task log KAN-105 \
  --agent "$AGENT2" \
  --type progress \
  --message "All tests passing"

kanban task complete KAN-105 \
  --agent "$AGENT2" \
  --confidence 0.94 \
  --summary "Migrated legacy API endpoints to new auth system. Created database migration and updated all 5 affected endpoints."
```

**5. Validation:**

```bash
kanban task approve KAN-105

# Check final history
kanban task attempts KAN-105

# Output:
# Execution attempts for KAN-105:
#
# Attempt 1:
#   Agent: junior-dev
#   ...
#   Failed: Database schema incompatible
#
# Attempt 2:
#   Agent: senior-dev
#   Claimed: 2025-03-15T11:00:00Z
#   Started: 2025-03-15T11:01:00Z
#   Completed: 2025-03-15T11:45:00Z
#   Confidence: 0.94
#   Status: completed
#
# Total attempts: 2
# Success rate: 50%
```

## Example 4: MCP Integration

Using the MCP protocol to interact with Kanban from an external client.

### JSON-RPC Sequence

**Step 1: List projects**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "list_projects",
  "params": {}
}

Response:
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": [
      {
        "id": 1,
        "name": "My Project",
        "prefix": "KAN"
      }
    ]
  }
}
```

**Step 2: Register an agent**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "register_agent",
  "params": {
    "name": "claude-worker",
    "agent_type": "claude",
    "skills": ["python", "testing", "documentation"]
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "data": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "claude-worker",
      "status": "idle",
      "registered_at": "2025-03-15T10:00:00Z"
    }
  }
}
```

**Step 3: Create a task**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "create_task",
  "params": {
    "project_id": 1,
    "title": "Document API",
    "objective": "Write comprehensive API documentation",
    "status_id": 9,
    "skills": ["documentation", "python"],
    "complexity": "medium"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "data": {
      "identifier": "KAN-106",
      "title": "Document API",
      "task_state": "unclaimed"
    }
  }
}
```

**Step 4: Agent gets next task**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "next_task",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "data": {
      "identifier": "KAN-106",
      "title": "Document API",
      "objective": "Write comprehensive API documentation",
      "task_state": "claimed",
      "claimed_by": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**Step 5: Agent starts execution**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "start_task",
  "params": {
    "identifier": "KAN-106",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "data": {
      "identifier": "KAN-106",
      "task_state": "executing",
      "started_at": "2025-03-15T10:01:00Z"
    }
  }
}
```

**Step 6: Agent logs activity**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "log_task_activity",
  "params": {
    "identifier": "KAN-106",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "entry_type": "progress",
    "message": "Documented 10 API endpoints"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "data": {
      "identifier": "KAN-106",
      "logged_at": "2025-03-15T10:15:00Z"
    }
  }
}
```

**Step 7: Agent completes**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "complete_task",
  "params": {
    "identifier": "KAN-106",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "confidence": 0.97,
    "summary": "Documented all API endpoints with examples and error codes"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "data": {
      "identifier": "KAN-106",
      "task_state": "validating",
      "confidence": 0.97,
      "completed_at": "2025-03-15T10:30:00Z"
    }
  }
}
```

**Step 8: Validation**

```json
Request:
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "task_replay",
  "params": {
    "identifier": "KAN-106"
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "data": [
      {
        "timestamp": "2025-03-15T10:01:00Z",
        "entry_type": "info",
        "message": "Started documentation task"
      },
      {
        "timestamp": "2025-03-15T10:15:00Z",
        "entry_type": "progress",
        "message": "Documented 10 API endpoints"
      },
      {
        "timestamp": "2025-03-15T10:30:00Z",
        "entry_type": "info",
        "message": "Submitted for validation"
      }
    ]
  }
}
```

## See Also

- [Agent Protocol Overview](./index.md)
- [Agent Lifecycle](./lifecycle.md)
- [Task Contracts](./task-contracts.md)
