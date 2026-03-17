# Tasks Command

Manage task contracts and the agent execution lifecycle via the CLI.

## Commands

### Create a task

Create a new task contract (an issue with execution metadata).

```bash
kanban task create --project <ID> --title <TITLE> --objective <OBJECTIVE> --status <STATUS_ID> [OPTIONS]
```

**Required:**
- `--project <ID>` - Project ID
- `--title <TITLE>` - Task title
- `--objective <OBJECTIVE>` - Clear objective statement
- `--status <STATUS_ID>` - Initial status ID

**Options:**
- `--type <TYPE>` - Task type (task, bugfix, feature, refactor) [default: task]
- `--priority <PRIORITY>` - Priority (none, low, medium, high, urgent) [default: none]
- `--skills <SKILLS>` - Comma-delimited required skills (e.g., "python,testing")
- `--complexity <LEVEL>` - Complexity: small, medium, large [default: medium]
- `--constraints <JSON>` - JSON array of constraints
- `--success-criteria <JSON>` - JSON array of success criteria
- `--context-files <FILES>` - Comma-delimited file paths for context
- `--timeout <MINUTES>` - Task timeout in minutes [default: 30]
- `--depends-on <IDENTIFIERS>` - Comma-delimited task dependencies
- `--description <DESCRIPTION>` - Full description
- `--assignee <MEMBER_ID>` - Initial assignee

**Examples:**

```bash
# Simple task
kanban task create \
  --project 1 \
  --title "Add password reset" \
  --objective "Implement email-based password reset flow" \
  --status 9

# With skills and complexity
kanban task create \
  --project 1 \
  --title "Optimize database queries" \
  --objective "Reduce query time by 50%" \
  --status 9 \
  --skills "sql,performance,postgresql" \
  --complexity large \
  --priority high

# With constraints and success criteria
kanban task create \
  --project 1 \
  --title "Refactor auth module" \
  --objective "Improve code quality and test coverage" \
  --status 9 \
  --type refactor \
  --skills "python,testing,refactoring" \
  --constraints '["no breaking changes", "maintain backward compatibility"]' \
  --success-criteria '["test coverage > 90%", "code review approved", "no regressions"]'

# With dependencies
kanban task create \
  --project 1 \
  --title "Implement feature flag" \
  --objective "Add feature flag system" \
  --status 9 \
  --depends-on "KAN-40,KAN-41"

# With context files
kanban task create \
  --project 1 \
  --title "Write API docs" \
  --objective "Document all endpoints" \
  --status 9 \
  --context-files "src/api.rs,ARCHITECTURE.md" \
  --skills "documentation,rust"

# Complete example with all options
kanban task create \
  --project 1 \
  --title "Implement OAuth integration" \
  --objective "Add OAuth2 support for Google and GitHub" \
  --status 9 \
  --type feature \
  --priority high \
  --skills "authentication,oauth,python" \
  --complexity large \
  --constraints '["no third-party auth libs"]' \
  --success-criteria '["Google login works", "GitHub login works", "E2E tests pass"]' \
  --context-files "src/auth.py,docs/oauth.md" \
  --timeout 120 \
  --depends-on "KAN-42" \
  --json
```

**Output:**
```
Created task: KAN-45 - Add password reset
Objective: Implement email-based password reset flow
Status: Todo
Complexity: medium
Timeout: 30 minutes
```

**JSON Output:**
```json
{
  "id": 45,
  "project_id": 1,
  "identifier": "KAN-45",
  "title": "Add password reset",
  "objective": "Implement email-based password reset flow",
  "description": null,
  "type": "task",
  "priority": "none",
  "status_id": 9,
  "task_state": "unclaimed",
  "skills": [],
  "complexity": "medium",
  "constraints": [],
  "success_criteria": [],
  "context_files": [],
  "timeout_minutes": 30,
  "claimed_by": null,
  "created_at": "2025-03-15T10:00:00Z",
  "updated_at": "2025-03-15T10:00:00Z"
}
```

### Get next task

Get the next available task for an agent.

```bash
kanban task next --agent <AGENT_ID> [OPTIONS]
```

**Required:**
- `--agent <AGENT_ID>` - Agent UUID

**Options:**
- `--skills <SKILLS>` - Override agent skills (comma-delimited)

**Examples:**

```bash
# Get next task for agent
kanban task next --agent 550e8400-e29b-41d4-a716-446655440000

# With skill override
kanban task next \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --skills "python,testing"

# JSON output
kanban task next --agent 550e8400-e29b-41d4-a716-446655440000 --json
```

**Output:**
```
Next task for agent: code-analyzer

KAN-45: Add password reset
Objective: Implement email-based password reset flow
Type: task
Complexity: medium
Skills required: [python, testing]
Timeout: 30 minutes
Status: unclaimed → claimed
```

**JSON Output:**
```json
{
  "id": 45,
  "identifier": "KAN-45",
  "title": "Add password reset",
  "objective": "Implement email-based password reset flow",
  "type": "task",
  "complexity": "medium",
  "required_skills": ["python", "testing"],
  "success_criteria": [],
  "constraints": [],
  "context_files": [],
  "timeout_minutes": 30,
  "task_state": "claimed",
  "claimed_by": "550e8400-e29b-41d4-a716-446655440000",
  "claimed_at": "2025-03-15T10:00:00Z"
}
```

### Start a task

Begin execution of a claimed task.

```bash
kanban task start <IDENTIFIER> --agent <AGENT_ID>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier (e.g., KAN-45)

**Options:**
- `--agent <AGENT_ID>` - Agent UUID executing the task

**Examples:**

```bash
# Start a task
kanban task start KAN-45 --agent 550e8400-e29b-41d4-a716-446655440000

# JSON output
kanban task start KAN-45 --agent 550e8400-e29b-41d4-a716-446655440000 --json
```

**Output:**
```
Started execution: KAN-45
Agent: code-analyzer
State: executing
Started at: 2025-03-15T10:01:00Z
```

### Complete a task

Mark a task as successfully completed.

```bash
kanban task complete <IDENTIFIER> --agent <AGENT_ID> --confidence <CONFIDENCE> --summary <SUMMARY> [OPTIONS]
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Options:**
- `--agent <AGENT_ID>` - Agent UUID that completed it
- `--confidence <CONFIDENCE>` - Confidence level (0.0 to 1.0)
- `--summary <SUMMARY>` - Completion summary
- `--artifacts <JSON>` - Optional JSON object with artifacts

**Examples:**

```bash
# Simple completion
kanban task complete KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --confidence 0.95 \
  --summary "Implemented email reset flow with tests"

# With artifacts
kanban task complete KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --confidence 0.92 \
  --summary "Completed OAuth integration" \
  --artifacts '{"pr_url": "https://github.com/...", "tests_added": 15}'

# JSON output
kanban task complete KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --confidence 0.95 \
  --summary "Done!" \
  --json
```

**Output:**
```
Completed task: KAN-45
State: validating
Confidence: 95%
Summary: Implemented email reset flow with tests
Awaiting validation...
```

**JSON Output:**
```json
{
  "id": 45,
  "identifier": "KAN-45",
  "title": "Add password reset",
  "task_state": "validating",
  "completed_by": "550e8400-e29b-41d4-a716-446655440000",
  "completed_at": "2025-03-15T10:15:00Z",
  "confidence": 0.95,
  "summary": "Implemented email reset flow with tests",
  "artifacts": null
}
```

### Fail a task

Mark a task as failed.

```bash
kanban task fail <IDENTIFIER> --agent <AGENT_ID> --reason <REASON>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Options:**
- `--agent <AGENT_ID>` - Agent UUID that failed it
- `--reason <REASON>` - Reason for failure

**Examples:**

```bash
# Task failed due to external issue
kanban task fail KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --reason "Database schema migration failed"

# JSON output
kanban task fail KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --reason "Dependency not available" \
  --json
```

**Output:**
```
Task KAN-45 failed
Agent: code-analyzer
Reason: Database schema migration failed
State: failed
Task reclaimed for reassignment.
```

### Unclaim a task

Return a claimed/executing task without completion or failure.

```bash
kanban task unclaim <IDENTIFIER> --agent <AGENT_ID> [OPTIONS]
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Options:**
- `--agent <AGENT_ID>` - Agent UUID that claims it
- No additional options (task returns to unclaimed)

**Examples:**

```bash
# Unclaim a task
kanban task unclaim KAN-45 --agent 550e8400-e29b-41d4-a716-446655440000
```

### Log task activity

Log an execution entry for a task.

```bash
kanban task log <IDENTIFIER> --agent <AGENT_ID> --type <ENTRY_TYPE> --message <MESSAGE> [OPTIONS]
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Options:**
- `--agent <AGENT_ID>` - Agent logging the entry
- `--type <ENTRY_TYPE>` - Entry type (info, warning, error, progress)
- `--message <MESSAGE>` - Log message
- `--meta <JSON>` - Optional metadata JSON

**Examples:**

```bash
# Progress update
kanban task log KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --type progress \
  --message "Completed unit tests"

# Error log
kanban task log KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --type error \
  --message "API request timeout" \
  --meta '{"retry_count": 3, "endpoint": "/api/users"}'

# Warning
kanban task log KAN-45 \
  --agent 550e8400-e29b-41d4-a716-446655440000 \
  --type warning \
  --message "High memory usage detected"
```

**Output:**
```
Logged entry for KAN-45
Type: progress
Message: Completed unit tests
```

### Approve a task

Approve a validating task (mark as complete).

```bash
kanban task approve <IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Examples:**

```bash
kanban task approve KAN-45
```

**Output:**
```
Approved task: KAN-45
State: completed
Status: Done
```

### Reject a task

Reject a validating task (return to in progress for rework).

```bash
kanban task reject <IDENTIFIER> --reason <REASON>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Options:**
- `--reason <REASON>` - Reason for rejection

**Examples:**

```bash
kanban task reject KAN-45 --reason "Tests not comprehensive enough"
```

**Output:**
```
Rejected task: KAN-45
Reason: Tests not comprehensive enough
State: returned to executing
```

### Show dependency graph

Display the dependency graph for a task.

```bash
kanban task graph <IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Examples:**

```bash
kanban task graph KAN-45
```

**Output:**
```
Dependency graph for KAN-45:

  KAN-40: Setup database
     ├─> KAN-41: Create schema
     │    └─> KAN-42: Add indexes
     │         └─> KAN-45: Add password reset
     └─> KAN-43: Setup cache
          └─> KAN-45

Legend:
  ├─> depends on
  └─> blocked by
```

### Show execution attempts

Show all execution attempts for a task.

```bash
kanban task attempts <IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Examples:**

```bash
kanban task attempts KAN-45
```

**Output:**
```
Execution attempts for KAN-45:

Attempt 1:
  Agent: code-analyzer
  Claimed: 2025-03-15T10:00:00Z
  Started: 2025-03-15T10:01:00Z
  Failed: 2025-03-15T10:30:00Z
  Reason: Dependency not available

Attempt 2:
  Agent: backend-1
  Claimed: 2025-03-15T11:00:00Z
  Started: 2025-03-15T11:01:00Z
  Completed: 2025-03-15T11:45:00Z
  Confidence: 0.95

Total attempts: 2
Success rate: 50%
```

### Replay execution logs

Replay all execution logs for a task.

```bash
kanban task replay <IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Task identifier

**Examples:**

```bash
kanban task replay KAN-45 --json
```

**Output:**
```
Execution log for KAN-45:

2025-03-15T10:01:00Z [progress] Starting implementation
2025-03-15T10:05:00Z [progress] Created password reset endpoint
2025-03-15T10:10:00Z [progress] Added email validation
2025-03-15T10:15:00Z [progress] Wrote unit tests
2025-03-15T10:20:00Z [info] All tests passing
2025-03-15T10:25:00Z [progress] Code review completed
2025-03-15T10:30:00Z [info] Submitted for validation
```

## Task States

| State | Description |
|-------|-------------|
| `unclaimed` | Available for claiming |
| `claimed` | Claimed by an agent, not started |
| `executing` | Currently being executed |
| `validating` | Waiting for validation/approval |
| `completed` | Successfully completed and approved |
| `failed` | Failed and needs reassignment |
| `blocked` | Waiting for dependency |

## Examples

### Complete task lifecycle

```bash
# 1. Create a task
TASK=$(kanban task create \
  --project 1 \
  --title "Fix critical bug" \
  --objective "Resolve security vulnerability" \
  --status 9 \
  --priority urgent \
  --skills "security,python" \
  --complexity large \
  --json)

IDENTIFIER=$(echo "$TASK" | jq -r '.identifier')

# 2. Get next task for an agent
AGENT_ID="550e8400-e29b-41d4-a716-446655440000"
kanban task next --agent "$AGENT_ID"

# 3. Start execution
kanban task start "$IDENTIFIER" --agent "$AGENT_ID"

# 4. Log progress
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Identified root cause"

# 5. Log more progress
kanban task log "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --type progress \
  --message "Implemented fix and wrote tests"

# 6. Complete with confidence
kanban task complete "$IDENTIFIER" \
  --agent "$AGENT_ID" \
  --confidence 0.98 \
  --summary "Fixed authentication bypass vulnerability"

# 7. Approve the completion
kanban task approve "$IDENTIFIER"

# 8. Check final state
kanban task get "$IDENTIFIER" --json
```

### Parallel task execution

```bash
#!/bin/bash
# Create multiple tasks and assign to different agents

PROJECT_ID=1
TASKS=()

# Create tasks
for i in {1..3}; do
  TASK=$(kanban task create \
    --project $PROJECT_ID \
    --title "Feature $i" \
    --objective "Implement feature $i" \
    --status 9 \
    --json)
  TASKS+=($(echo "$TASK" | jq -r '.identifier'))
done

# Assign agents
AGENTS=(
  "550e8400-e29b-41d4-a716-446655440000"
  "660e8400-e29b-41d4-a716-446655440001"
  "770e8400-e29b-41d4-a716-446655440002"
)

# Claim and start
for i in {0..2}; do
  kanban task next --agent "${AGENTS[$i]}" > /dev/null
  kanban task start "${TASKS[$i]}" --agent "${AGENTS[$i]}"
done

# Monitor
while true; do
  echo "=== Task Status ==="
  for task in "${TASKS[@]}"; do
    kanban task get "$task" --json | jq '.task_state'
  done
  sleep 10
done
```

## See Also

- [Agents Command](./agents.md)
- [Issues Command](./issues.md)
- [Task Contracts Documentation](../agents/task-contracts.md)
