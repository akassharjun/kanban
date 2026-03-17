# Validation Pipeline

When an agent completes a task, its result is validated against success criteria. This validation pipeline determines whether the task truly succeeded or needs rework.

## Overview

The validation process:

1. **Agent completes task** → calls `task_complete()`
2. **Extract success criteria** → from task_contract.success_criteria
3. **Run executable checks** → Shell commands with 5-minute timeout
4. **Collect results** → Output, exit codes, errors
5. **Evaluate confidence** → Calculate based on check results
6. **Route to decision** → Auto-accept, auto-reject, or human review

```
Agent calls: task_complete(confidence=0.92)
      ↓
task_state → 'validating'
issue.status → (stays in started)
      ↓
Extract success_criteria from contract
      ↓
For each criterion with "command":
  ├─ Execute command (5-min timeout)
  ├─ Capture stdout/stderr
  ├─ Check exit code vs "expect"
  └─ Log result
      ↓
Evaluate decision:
  ├─ confidence >= auto_accept_threshold?
  │  ├─ Yes → task_state='completed'
  │  │        issue.status → 'Done'
  │  └─ No → continue
  ├─ confidence < human_review_threshold?
  │  ├─ Yes → task_state='blocked'
  │  │        issue.status → 'Blocked'
  │  │        notify human reviewer
  │  └─ No → continue
  └─ Between thresholds → wait for human review
      ↓
Done
```

## Success Criteria

Success criteria are defined when creating a task contract:

```bash
kanban cli task create \
  --project 1 \
  --title "Implement feature" \
  --type implementation \
  --success-criteria '[
    {
      "check": "Tests pass",
      "command": "npm test",
      "expect": "exit_code == 0"
    },
    {
      "check": "Linting succeeds",
      "command": "npm run lint",
      "expect": "exit_code == 0"
    },
    {
      "check": "Type checking",
      "command": "npm run type-check",
      "expect": "exit_code == 0"
    },
    {
      "check": "Code review completed",
      "description": "At least one other dev must approve"
    }
  ]'
```

Each criterion can be:

### Non-Executable (Description Only)
```json
{
  "check": "Code follows style guide",
  "description": "Ensure proper formatting and naming conventions"
}
```
These are informational and don't block validation.

### Executable (Shell Command)
```json
{
  "check": "Tests pass",
  "command": "npm test",
  "expect": "exit_code == 0"
}
```

The system will:
1. Execute `npm test` in a shell
2. Capture stdout and stderr
3. Check if exit code == 0
4. Pass/fail based on expectation

### Complex Executable
```json
{
  "check": "Performance benchmark",
  "command": "npm run benchmark | grep 'p99' | awk '{print $2}' | awk -F'ms' '{print $1}'",
  "expect": "exit_code == 0"
}
```

Commands can be complex pipelines, but they must:
- Return exit code 0 for success
- Complete in < 5 minutes
- Not contain shell injection characters (;, &&, ||, |, etc.)

## Running Validation

### Automatic (Agent Triggers)

When agent completes a task with confidence:

```bash
kanban cli task complete \
  --identifier API-42 \
  --agent-id "agent-xyz" \
  --confidence 0.92 \
  --summary "Implemented feature with all tests passing"
```

The system automatically:
1. Extracts success_criteria from API-42's contract
2. Executes all `command` criteria
3. Logs results
4. Routes to decision based on confidence + thresholds

### Manual (Testing/Debugging)

You can run validation on a task manually:

```bash
kanban cli validation run --identifier API-42

# Output:
# {
#   "all_passed": true,
#   "checks": [
#     {
#       "name": "Tests pass",
#       "passed": true,
#       "output": "PASS: 42 tests\n"
#     },
#     {
#       "name": "Linting succeeds",
#       "passed": true,
#       "output": "No issues found\n"
#     },
#     {
#       "name": "Type checking",
#       "passed": true,
#       "output": "No errors\n"
#     }
#   ]
# }
```

## Validation Result

After running validation, you get a `ValidationResult`:

```json
{
  "all_passed": true,
  "checks": [
    {
      "name": "Tests pass",
      "passed": true,
      "output": "PASS: 42 tests passed in 3.2s",
      "error": null
    },
    {
      "name": "Linting succeeds",
      "passed": true,
      "output": "No linting issues",
      "error": null
    },
    {
      "name": "Type checking",
      "passed": true,
      "output": "Type checking passed",
      "error": null
    }
  ]
}
```

Each check has:
- **name** — The "check" field from criteria
- **passed** — true/false
- **output** — stdout from command (first 2000 chars)
- **error** — stderr or error message (first 2000 chars)

## Decision Thresholds

Project-level configuration controls how confidence is evaluated:

### Configuration

```bash
# View project's validation thresholds
kanban cli project get 1

# Shows:
# {
#   ...
#   "agent_config": {
#     "auto_accept_threshold": 0.85,
#     "human_review_threshold": 0.50,
#     ...
#   }
# }
```

### Auto-Accept (confidence >= 0.85)
If agent's confidence is >= 85%, result is automatically accepted:
- task_state → `completed`
- issue.status → `Done`
- No human review needed

### Human Review (0.50 to 0.85)
If confidence is between 50% and 85%, task is flagged for human review:
- task_state → (stays `validating`)
- issue.status → (stays `started`)
- Notification sent to reviewer
- Awaiting human decision

### Auto-Reject (confidence < 0.50)
If confidence is below 50%, result is automatically rejected:
- task_state → `blocked` or moved back to `queued`
- issue.status → `Blocked` (or `Backlog` for retry)
- Task is not marked complete
- Can be retried if `attempt_count < max_attempts`

### Example Scenarios

```
Scenario 1: High confidence (0.95)
┌─ Auto-accept threshold: 0.85
├─ Agent confidence: 0.95
├─ Result: 0.95 >= 0.85 ✓
└─ Action: Auto-accepted → task_state=completed

Scenario 2: Medium confidence (0.72)
┌─ Auto-accept: 0.85
├─ Human review: 0.50
├─ Agent confidence: 0.72
├─ Result: 0.50 <= 0.72 < 0.85
└─ Action: Human review required → blocked, awaiting decision

Scenario 3: Low confidence (0.35)
┌─ Human review threshold: 0.50
├─ Agent confidence: 0.35
├─ Result: 0.35 < 0.50
└─ Action: Auto-rejected → blocked, wait for retry or manual fix
```

## Editing Thresholds

Adjust confidence thresholds per project:

```bash
kanban cli project config set 1 \
  --auto-accept-threshold 0.90 \
  --human-review-threshold 0.60 \
  --max-attempts 5 \
  --heartbeat-interval 30
```

| Setting | Default | Range | Effect |
|---------|---------|-------|--------|
| `auto_accept_threshold` | 0.85 | 0.0-1.0 | Confidence needed for auto-accept |
| `human_review_threshold` | 0.50 | 0.0-1.0 | Min confidence before auto-reject |
| `max_attempts` | 3 | 1+ | Retries allowed per task |
| `heartbeat_interval_seconds` | 60 | 10+ | Agent heartbeat frequency |

## Command Safety

The system prevents **shell injection** by rejecting commands with dangerous characters:

Dangerous patterns (rejected):
- `;` — Command separator
- `&&` — AND operator
- `||` — OR operator
- `|` — Pipe
- `` ` `` — Backtick execution
- `$(...)` — Command substitution
- `${...}` — Variable substitution
- `>`, `<` — Redirection
- Newlines

Example:

```bash
# REJECTED: Contains ; and $(...)
"command": "npm test; $(rm -rf /)",
"error": "Command rejected: contains unsafe shell characters"

# REJECTED: Contains pipe
"command": "grep test results.txt | wc -l"
"error": "Command rejected: contains unsafe shell characters"

# ACCEPTED: Simple command, no dangerous chars
"command": "npm test",
```

This protects against agents injecting malicious commands.

### Safe Commands

Good examples:

```bash
# Single command
"command": "npm test"

# Simple arguments
"command": "npm test -- --coverage"

# Path references
"command": "/usr/local/bin/golangci-lint run"

# Variable expansion (before command execution)
"command": "echo $HOME"  # Shell expands $HOME before execution
```

## Timeout

Each command has a **5-minute timeout**. If not complete by then:

```
Command: npm test
Status: Running...
Time: 4:50
Status: Running...
Time: 5:00
Status: TIMEOUT
Error: "Command timed out (5 min limit)"
```

For tasks that might take longer, break them into smaller checks or increase the overall timeout via task definition:

```bash
kanban cli task create \
  --timeout 120 \  # 2 hours for whole task
  --success-criteria '[
    {
      "check": "Integration tests",
      "command": "npm run test:integration",
      "expect": "exit_code == 0"
    }
  ]'
```

## Manual Review

For results requiring human review:

### Get Validation Details

```bash
kanban cli validation show --identifier API-42

# Output:
# {
#   "all_passed": false,
#   "checks": [
#     {
#       "name": "Tests pass",
#       "passed": true,
#       "output": "PASS: 42 tests"
#     },
#     {
#       "name": "No regressions",
#       "passed": false,
#       "error": "Performance regression detected: API response time +15%"
#     }
#   ]
# }
```

### Accept or Reject Manually

```bash
# Accept the result despite validation failures
kanban cli task validation accept \
  --identifier API-42 \
  --confidence 0.80 \
  --reason "Performance regression is acceptable for this iteration"

# Or reject and send back
kanban cli task validation reject \
  --identifier API-42 \
  --reason "Performance regression too large, needs optimization"
```

This:
- Sets task_state → `completed` (if accepted)
- task_state → `queued` (if rejected, for retry)
- Logs human decision in activity log

## Validation Logging

All validation runs are logged in execution_logs:

```bash
kanban cli task replay API-42

# Shows:
# Entry Type         | Message                          | Timestamp
# validating         | Started validation               | 2025-03-15 10:30:00Z
# validation_check   | Tests pass: PASS (42 tests)     | 2025-03-15 10:30:05Z
# validation_check   | Linting: PASS                   | 2025-03-15 10:30:10Z
# validation_check   | Type check: PASS                | 2025-03-15 10:30:15Z
# validation_complete| All checks passed (confidence:0.95)| 2025-03-15 10:30:15Z
```

## Example Task with Comprehensive Validation

```bash
kanban cli task create \
  --project 1 \
  --title "Add authentication module" \
  --type implementation \
  --objective "Implement JWT-based auth for API" \
  --skills "typescript,auth,testing" \
  --complexity medium \
  --timeout 60 \
  --success-criteria '[
    {
      "check": "Unit tests pass",
      "command": "npm test -- --testPathPattern=auth",
      "expect": "exit_code == 0"
    },
    {
      "check": "Integration tests pass",
      "command": "npm run test:integration -- --testPathPattern=auth",
      "expect": "exit_code == 0"
    },
    {
      "check": "No TypeScript errors",
      "command": "npm run type-check",
      "expect": "exit_code == 0"
    },
    {
      "check": "No linting issues",
      "command": "npm run lint",
      "expect": "exit_code == 0"
    },
    {
      "check": "Code review approved",
      "description": "At least one other engineer must review and approve"
    },
    {
      "check": "Security audit passed",
      "command": "npm audit --audit-level=moderate",
      "expect": "exit_code == 0"
    },
    {
      "check": "Build succeeds",
      "command": "npm run build",
      "expect": "exit_code == 0"
    }
  ]'

# When agent completes:
kanban cli task complete \
  --identifier API-99 \
  --agent-id "agent-coder" \
  --confidence 0.88 \
  --summary "Implemented JWT auth with comprehensive tests"

# System runs all executable checks in parallel (6 commands)
# If all pass: accepted (0.88 >= 0.85 threshold)
# If 1-2 fail: human review needed (0.50-0.85 range)
# If most fail: auto-rejected (< 0.50)
```

## Best Practices

### 1. Define Clear Success Criteria
Make criteria objective and measurable:

Good:
```json
{
  "check": "Tests pass",
  "command": "npm test -- --testPathPattern=feature",
  "expect": "exit_code == 0"
}
```

Bad:
```json
{
  "check": "Code is good",
  "description": "Code should be good"
}
```

### 2. Mix Automated and Manual Checks
Use both where appropriate:

```json
[
  {
    "check": "Tests pass",
    "command": "npm test",
    "expect": "exit_code == 0"
  },
  {
    "check": "Code review approved",
    "description": "At least one senior engineer approval"
  },
  {
    "check": "Performance acceptable",
    "command": "npm run benchmark | grep p99 | grep -v '>100ms'",
    "expect": "exit_code == 0"
  }
]
```

### 3. Keep Commands Simple
Avoid complex pipes. Test locally first:

Good:
```bash
npm test
npm run lint
go build -o app ./cmd
```

Bad:
```bash
npm test && npm run lint && npm run build && git commit
```

### 4. Set Realistic Confidence
Don't demand 99% confidence on exploratory work:

```bash
# Conservative (high bar)
--confidence 0.95

# Balanced (standard)
--confidence 0.85

# Exploratory (low bar, needs review)
--confidence 0.65
```

### 5. Set Appropriate Thresholds
Match your team's risk tolerance:

```bash
# Conservative team (high quality bar)
kanban cli project config set 1 \
  --auto-accept-threshold 0.90 \
  --human-review-threshold 0.70

# Balanced team (standard)
kanban cli project config set 1 \
  --auto-accept-threshold 0.85 \
  --human-review-threshold 0.50

# Experimental team (low friction)
kanban cli project config set 1 \
  --auto-accept-threshold 0.75 \
  --human-review-threshold 0.40
```

## Next Steps

- **[Task Contracts](/guide/task-contracts.md)** — Define tasks with criteria
- **[Execution & Replay](/guide/execution-replay.md)** — View what happened
- **[Agent Routing](/guide/agent-routing.md)** — How agents get assigned
