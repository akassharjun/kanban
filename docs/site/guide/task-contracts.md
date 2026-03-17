# Task Contracts

Task contracts are the centerpiece of Kanban's agent orchestration system. A task contract extends an issue with detailed execution specifications: objective, required skills, success criteria (as shell commands), constraints, complexity, and timeout.

Unlike regular issues (which are for human project management), task contracts are **executable definitions** that agents can claim and complete.

## Task vs. Regular Issues

| Aspect | Regular Issue | Task Contract |
|--------|---------------|---------------|
| **Purpose** | Human project tracking | Agent execution |
| **Success** | Someone completes it | Automated validation |
| **Lifecycle** | Manual status updates | Automatic state machine |
| **Validation** | Human review | Shell commands |
| **Details** | Title, description, priority | Objective, skills, criteria, constraints |

You can convert any issue to a task contract by adding the contract extension.

## Creating a Task Contract

A task contract requires:
- **Title** — What it is
- **Objective** — What the agent should accomplish (specific, measurable)
- **Status** — Initial workflow column
- **Type** — `implementation`, `review`, or `decomposition`
- **Required Skills** — List of capabilities agent must have
- **Estimated Complexity** — `small`, `medium`, or `large`
- **Success Criteria** — JSON array of checks (can include shell commands)
- **Constraints** — Rules to follow
- **Timeout** — Minutes before auto-failure
- **Context** — Files, related tasks, prior attempts

### Via CLI

```bash
kanban cli task create \
  --project 1 \
  --title "Implement user authentication" \
  --objective "Add OAuth 2.0 login flow with Google and GitHub" \
  --status 2 \
  --type implementation \
  --skills "typescript,auth,backend" \
  --complexity medium \
  --success-criteria '[
    {
      "check": "OAuth endpoints created",
      "command": "grep -r \"oauth\" src/ && echo success",
      "expect": "exit_code == 0"
    },
    {
      "check": "Login tests pass",
      "command": "npm test -- --testPathPattern=auth",
      "expect": "exit_code == 0"
    },
    {
      "check": "No security vulnerabilities",
      "command": "npm audit --audit-level=moderate",
      "expect": "exit_code == 0"
    }
  ]' \
  --constraints "Must not break existing API, handle token refresh" \
  --timeout 120
```

### Via GUI
1. Click "+" to create issue
2. Toggle "Task Contract" option
3. Fill in objective, skills, criteria
4. Click "Create"

## Task Contract Fields

### Type

| Type | Purpose |
|------|---------|
| `implementation` | Build something new |
| `review` | Review code or design |
| `decomposition` | Break down a large task into smaller ones |

### Objective

A specific, measurable description of what should happen:

Good:
```
Implement rate limiting for the /api/v1/auth endpoint using a token bucket
algorithm. Allow 100 requests per minute per user. Return 429 when exceeded.
```

Bad:
```
Fix the API
```

### Skills

List of required capabilities. Agents only claim tasks where they have **all** required skills:

```bash
--skills "typescript,express,oauth"

# Agent must have: typescript AND express AND oauth
# Subset matching: agent can have additional skills
```

### Complexity

Agents have a `max_complexity`. They'll only take tasks at or below their limit:

```
small   ← Simple, < 1 hour, well-defined
medium  ← Standard, 1-4 hours, mostly clear
large   ← Complex, multiple days, exploratory
```

### Success Criteria

An array of checks. Each check can be:

1. **Non-executable (description only)**
   ```json
   {
     "check": "Code follows style guide",
     "description": "Ensure 2-space indentation, no trailing whitespace"
   }
   ```

2. **Executable (shell command)**
   ```json
   {
     "check": "Tests pass",
     "command": "npm test",
     "expect": "exit_code == 0"
   }
   ```

3. **Complex executable**
   ```json
   {
     "check": "Performance requirement",
     "command": "npm run benchmark",
     "expect": "exit_code == 0",
     "description": "Latency < 100ms (p99)"
   }
   ```

Full example with multiple criteria:

```bash
--success-criteria '[
  {
    "check": "Unit tests pass",
    "command": "go test ./...",
    "expect": "exit_code == 0"
  },
  {
    "check": "Lint passes",
    "command": "golangci-lint run",
    "expect": "exit_code == 0"
  },
  {
    "check": "No vulnerable dependencies",
    "command": "go list -u -m all | grep INDIRECT | wc -l | grep 0",
    "expect": "exit_code == 0"
  },
  {
    "check": "Binary builds",
    "command": "go build -o kanban ./cmd/cli",
    "expect": "exit_code == 0"
  },
  {
    "check": "Follows Go conventions",
    "description": "Code should be idiomatic Go (checked manually by reviewer)"
  }
]'
```

### Constraints

Rules the agent must follow:

```bash
--constraints '[
  "Must not modify the database schema",
  "Must not break existing API contracts",
  "Session tokens must expire in 24 hours",
  "Passwords must use bcrypt with cost >= 12"
]'
```

### Context

Structured input for the agent:

```bash
# Via JSON in command line
--context '{
  "files": [
    "src/auth/oauth.ts",
    "src/auth/tokens.ts",
    "docs/oauth-spec.md"
  ],
  "related_tasks": [
    "API-40",
    "API-41"
  ],
  "prior_attempts": [
    "Attempt 1: Token refresh had race condition, need mutex"
  ]
}'
```

### Timeout

Minutes before the task auto-fails:

```bash
--timeout 120  # 2 hours max

# Common values:
# 15  - Quick tasks (< 30 min expected)
# 30  - Standard (1-2 hours)
# 60  - Complex (2-4 hours)
# 120 - Very complex (4+ hours)
```

## Task State Machine

Each task contract has a `task_state` that progresses through:

```
      START
        ↓
    ┌─ queued ──────────────────────────────────┐
    │   (waiting for agent to claim)             │
    │   ↓                                         │
    ├─ claimed ─────────────────────────────────┤
    │   (agent has it, hasn't started)           │
    │   ↓                                         │
    ├─ executing ───────────────────────────────┤
    │   (agent is working)                       │
    │   ↓                                         │
    ├─ validating ──────────────────────────────┤
    │   (running success criteria)                │
    │   ├─ ✓ passed ──────→ completed            │
    │   └─ ✗ failed ──────→ (requeue or blocked) │
    │                                             │
    ├─ completed ──→ issue.status = Done         │
    │                                             │
    ├─ blocked ────→ issue.status = Blocked      │
    │   (dependency missing, auto-transition)    │
    │                                             │
    └─ cancelled ──→ issue.status = Discarded    │
        (agent gave up or timeout)               │
```

### Queued
Task is waiting for an agent to claim it.
- Status: Issue in `unstarted` status
- Agent can see it via `next_task()`
- No one is working on it

### Claimed
Agent has claimed the task and is preparing to work.
- Status: Issue in `started` status (usually "In Progress")
- Agent has exclusive lock (atomic)
- Agent may be analyzing context before starting

### Executing
Agent is actively working.
- Status: Issue still in `started` status
- Agent is writing code, running tests, etc.
- Execution logs are being written

### Validating
Agent finished working; system is running success criteria.
- Status: Issue in `started` status
- Shell commands from success_criteria are executed
- Each command result is logged

### Completed
Validation passed; task is done.
- Status: Issue moves to first `completed` status
- Result (JSON) is stored
- Confidence score is recorded

### Blocked
Task can't proceed; dependency missing.
- Status: Issue moves to `blocked` status
- Automatic transition (orchestration checks issue_relations)
- Task stays queued; retried when blocker completes

### Cancelled
Agent gave up; task failed.
- Status: Issue moves to `discarded` status
- Happened because: timeout, error, or max attempts reached
- Task won't be picked up again automatically

## Task Lifecycle Example

```bash
# 1. Create task contract
kanban cli task create \
  --project 1 \
  --title "Add password strength validator" \
  --objective "Implement regex validator for password policies" \
  --status 2 \
  --type implementation \
  --skills "typescript,testing" \
  --complexity small \
  --success-criteria '[
    {
      "check": "Tests pass",
      "command": "npm test -- validator.test.ts",
      "expect": "exit_code == 0"
    },
    {
      "check": "Lints",
      "command": "npm run lint",
      "expect": "exit_code == 0"
    }
  ]' \
  --timeout 30

# Returns: API-99 (task created, status: queued)

# 2. Agent checks for work
kanban cli agent next-task --agent-id "agent-xyz"
# Returns: API-99 (if agent has typescript + testing skills)

# 3. Agent claims task
# Internally: UPDATE task_contracts SET task_state='claimed', claimed_by='agent-xyz'
# Issue status auto-updates to "In Progress"

# 4. Agent starts working
kanban cli task log-activity \
  --identifier API-99 \
  --agent-id agent-xyz \
  --entry-type start \
  --message "Starting implementation"

# 5. Agent writes code, runs tests
kanban cli task log-activity \
  --identifier API-99 \
  --agent-id agent-xyz \
  --entry-type file_edit \
  --message "Created src/validator.ts" \
  --metadata '{"file": "src/validator.ts", "lines": 42}'

kanban cli task log-activity \
  --identifier API-99 \
  --agent-id agent-xyz \
  --entry-type command \
  --message "Running tests" \
  --metadata '{"command": "npm test", "exit_code": 0}'

# 6. Agent completes task
kanban cli task complete \
  --identifier API-99 \
  --agent-id agent-xyz \
  --confidence 0.95 \
  --summary "Validator implemented with all test cases passing"

# Internally:
# - task_state → 'validating'
# - Execute success_criteria commands
# - If all pass: task_state → 'completed', issue.status → Done
# - If any fail: task_state → (requeue or blocked), issue.status → (Backlog or Blocked)

# 7. Check result
kanban cli task get API-99
# Returns: task_state=completed, confidence=0.95, result={...}
```

## Updating Task Contracts

Update task state and details:

### Via CLI

```bash
# Update task state
kanban cli task update API-99 --task-state executing

# Update complexity
kanban cli task update API-99 --complexity large

# Update required skills
kanban cli task update API-99 --skills "typescript,express,auth"

# Update success criteria
kanban cli task update API-99 --success-criteria '[...]'

# Update constraints
kanban cli task update API-99 --constraints '[...]'

# Update timeout
kanban cli task update API-99 --timeout 60

# Update objective
kanban cli task update API-99 --objective "New objective text"
```

## Viewing Task Contracts

### Get a Task Contract

```bash
kanban cli task get API-99

# Output:
# {
#   "issue_id": 99,
#   "identifier": "API-99",
#   "title": "Add password strength validator",
#   "objective": "Implement regex validator for password policies",
#   "type": "implementation",
#   "task_state": "completed",
#   "required_skills": ["typescript", "testing"],
#   "estimated_complexity": "small",
#   "success_criteria": [
#     { "check": "Tests pass", "command": "npm test -- validator.test.ts", "expect": "exit_code == 0" },
#     { "check": "Lints", "command": "npm run lint", "expect": "exit_code == 0" }
#   ],
#   "constraints": [],
#   "timeout_minutes": 30,
#   "claimed_by": "agent-xyz",
#   "claimed_at": "2025-03-15T10:05:00Z",
#   "attempt_count": 1,
#   "result": {
#     "confidence": 0.95,
#     "summary": "Validator implemented with all test cases passing",
#     "artifacts": { "file": "src/validator.ts" }
#   }
# }
```

### List Task Contracts

```bash
# List all task contracts in a project
kanban cli task list --project 1

# Filter by state
kanban cli task list --project 1 --state queued
kanban cli task list --project 1 --state executing

# Filter by claimed agent
kanban cli task list --project 1 --claimed-by agent-xyz
```

## Task Decomposition

The orchestration system can automatically decompose large tasks into smaller ones. A task of type `decomposition` is meant to break down work:

```bash
kanban cli task create \
  --project 1 \
  --title "Refactor authentication system" \
  --objective "Break down auth refactor into smaller, parallelizable tasks" \
  --type decomposition \
  --status 2 \
  --skills "architecture,typescript" \
  --complexity large \
  --success-criteria '[
    {
      "check": "Sub-tasks created",
      "description": "Create 3+ child tasks for different auth modules"
    }
  ]'
```

When an agent completes a decomposition task, it returns new task identifiers to create.

## Best Practices

### 1. Write Clear Objectives
Be specific and measurable:

Good:
```
Implement token bucket rate limiter with:
- 100 requests/min per user
- 429 response on limit
- Sliding window algorithm
- Redis for distributed counting
```

Bad:
```
Make the API faster
```

### 2. List All Required Skills
Include everything needed:
```
--skills "typescript,express,redis,testing,architecture"
```

### 3. Set Realistic Complexity
Match agent max_complexity:
```
small   ← Small = 1-2 hours, clearly defined
medium  ← Medium = 1-2 days, mostly clear
large   ← Large = 3+ days, exploratory
```

### 4. Write Executable Success Criteria
Make criteria machine-checkable:

```json
{
  "check": "Tests pass",
  "command": "npm test",
  "expect": "exit_code == 0"
}
```

NOT just:
```json
{
  "check": "Tests should pass"
}
```

### 5. Set Appropriate Timeout
Match expected completion time:
```
15 min   → Simple, quick tasks
30 min   → Standard 1-2 hour tasks
60 min   → Complex 2-4 hour tasks
120 min  → Very complex 4+ hour tasks
```

### 6. Document Constraints
Be explicit about what's off-limits:
```
--constraints '[
  "Do not modify database schema",
  "Keep API response times < 100ms",
  "No new external dependencies"
]'
```

### 7. Provide Context
Help the agent understand the task:
```
--context '{
  "files": ["src/auth/oauth.ts", "docs/oauth-spec.md"],
  "related_tasks": ["API-40", "API-41"],
  "prior_attempts": ["Attempt 1: token refresh had race condition"]
}'
```

## Next Steps

- **[Agent Routing](/guide/agent-routing.md)** — How agents find and claim tasks
- **[Validation](/guide/validation.md)** — How success criteria are executed
- **[Execution & Replay](/guide/execution-replay.md)** — Viewing what happened
