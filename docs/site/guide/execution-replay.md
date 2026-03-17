# Execution Logs & Replay

Every action an agent takes is logged. Execution logs provide a complete audit trail and can be replayed to see exactly what happened.

## Execution Log Entries

When an agent works on a task, it logs entries:

```bash
kanban cli task log-activity \
  --identifier API-42 \
  --agent-id "agent-xyz" \
  --entry-type start \
  --message "Starting implementation of password reset flow"
```

### Entry Types

| Type | Meaning | Example |
|------|---------|---------|
| `claim` | Agent claimed task | "Agent claimed API-42" |
| `start` | Agent started working | "Starting implementation" |
| `reasoning` | Agent's reasoning | "Analyzed requirements, will use JWT tokens" |
| `file_read` | Agent read a file | "Read src/auth.ts (524 bytes)" |
| `file_edit` | Agent modified file | "Modified src/auth.ts, added OAuth handler" |
| `command` | Agent ran command | "Ran tests, exit code: 0" |
| `discovery` | Agent found something | "Discovered missing error handler in login" |
| `error` | Error occurred | "TypeScript compilation failed" |
| `result` | Agent's result | "Completed implementation with all tests passing" |
| `complete` | Task finished | "Task completed successfully" |
| `timeout` | Task timed out | "Task exceeded 60-minute timeout" |

### Metadata

Additional structured data can be attached:

```bash
kanban cli task log-activity \
  --identifier API-42 \
  --agent-id "agent-xyz" \
  --entry-type file_edit \
  --message "Modified src/auth.ts" \
  --metadata '{
    "file": "src/auth.ts",
    "lines_added": 42,
    "lines_removed": 8,
    "diff_preview": "+"
  }'
```

Each entry stores:
- **issue_id** — Which task
- **agent_id** — Who did it
- **attempt_number** — Which try (for retries)
- **entry_type** — What kind of event
- **message** — Human-readable description
- **metadata** — JSON with structured data
- **timestamp** — When it happened (UTC)

## Viewing Execution Logs

### Task Replay (Ordered Timeline)

View all log entries for a task in order:

```bash
kanban cli task replay API-42

# Output (formatted timeline):
# [2025-03-15 10:00:00Z] claim      | Agent claimed task
# [2025-03-15 10:00:15Z] start      | Starting implementation of password reset
# [2025-03-15 10:01:00Z] reasoning  | Analyzed OAuth providers, will use JWT tokens
# [2025-03-15 10:02:30Z] file_read  | Read src/auth/oauth.ts (1042 bytes)
# [2025-03-15 10:05:00Z] file_edit  | Created src/auth/oauth-handler.ts
# [2025-03-15 10:05:15Z] command    | Ran npm test (exit 0)
# [2025-03-15 10:06:00Z] command    | Ran npm run lint (exit 0)
# [2025-03-15 10:10:30Z] result     | Completed with confidence 0.92
# [2025-03-15 10:10:45Z] complete   | Task finished
```

### Task Attempts (Grouped by Retry)

View attempts grouped by retry number:

```bash
kanban cli task attempts API-42

# Output:
# Attempt 1 (2025-03-15 10:00:00Z to 10:10:45Z):
#   Status: completed
#   Confidence: 0.92
#   Duration: 10m 45s
#   Key Events:
#     - file_edit: Created OAuth handler
#     - command: Tests passed
#
# Attempt 2 (2025-03-15 14:00:00Z to 14:15:00Z):
#   Status: failed
#   Error: TypeScript compilation error
#   Duration: 15m
#   Key Events:
#     - error: Type mismatch in handler signature
#
# Attempt 3 (2025-03-15 15:00:00Z to 15:08:30Z):
#   Status: completed
#   Confidence: 0.85
#   Duration: 8m 30s
```

### GUI Replay Viewer

In the desktop app:
1. Open an issue
2. Click "Execution Log" or "Replay"
3. Timeline shows all events
4. Click events to expand
5. View metadata as JSON

## Complete Execution Logs Example

Here's a realistic execution log for a task:

```
Task: API-42 - Implement password reset
Agent: [claude] code-reviewer
Project: API (api project)

─── Attempt 1 ─────────────────────────────────────

[10:00:00Z] claim
  Status: Agent claimed task
  Agent: agent-xyz
  Confidence: N/A (not started yet)

[10:00:15Z] start
  Message: Starting implementation of password reset flow
  Metadata: { "start_time": "2025-03-15T10:00:15Z" }

[10:01:00Z] reasoning
  Message: Analyzed requirements. Will implement:
    1. Email verification endpoint
    2. Token generation (30-min expiry)
    3. Reset form handler
    4. Success confirmation
  Metadata: {
    "approach": "stateless JWT tokens",
    "tools": ["typescript", "express", "bcrypt"],
    "estimated_time": "45 minutes"
  }

[10:02:30Z] file_read
  Message: Read src/auth/oauth.ts to understand existing auth
  Metadata: {
    "file": "src/auth/oauth.ts",
    "bytes": 1042,
    "lines": 34
  }

[10:02:45Z] file_read
  Message: Read docs/requirements.md for password reset spec
  Metadata: {
    "file": "docs/requirements.md",
    "bytes": 2150,
    "lines": 78
  }

[10:05:00Z] file_edit
  Message: Created password reset endpoint handler
  Metadata: {
    "file": "src/auth/password-reset.ts",
    "lines_added": 120,
    "lines_removed": 0,
    "operation": "create"
  }

[10:08:00Z] file_edit
  Message: Added reset token validation middleware
  Metadata: {
    "file": "src/middleware/reset-validator.ts",
    "lines_added": 35,
    "lines_removed": 0,
    "operation": "create"
  }

[10:09:00Z] command
  Message: Running TypeScript compiler
  Metadata: {
    "command": "npm run type-check",
    "exit_code": 0,
    "duration_seconds": 8,
    "output": "Successfully compiled 120 files"
  }

[10:09:30Z] command
  Message: Running unit tests
  Metadata: {
    "command": "npm test -- --testPathPattern=password-reset",
    "exit_code": 0,
    "duration_seconds": 15,
    "output": "PASS auth/password-reset.test.ts (42 tests, 2.1s)"
  }

[10:10:00Z] command
  Message: Running linter
  Metadata: {
    "command": "npm run lint",
    "exit_code": 0,
    "duration_seconds": 12,
    "output": "All files OK"
  }

[10:10:30Z] result
  Message: Implementation complete. All tests passing, no lint errors.
  Metadata: {
    "confidence": 0.92,
    "summary": "Password reset flow implemented with:
      - Email verification
      - Secure token generation
      - Complete test coverage
      - Zero lint errors",
    "artifacts": {
      "new_files": [
        "src/auth/password-reset.ts",
        "src/middleware/reset-validator.ts"
      ],
      "modified_files": ["src/auth/routes.ts"],
      "tests_added": 42
    }
  }

[10:10:45Z] complete
  Status: Task completed successfully
  Duration: 10m 45s
  Confidence: 0.92
```

## Logging in Agent Code

When implementing an agent, log activities:

### Pseudo-Code Pattern

```python
async def execute_task(task_contract):
    agent_id = "my-agent"
    task_id = task_contract.identifier

    # Log claim (automatic)
    log_activity(task_id, "claim", f"Claimed by {agent_id}")

    # Log start
    log_activity(task_id, "start", "Starting work")

    # Log reasoning
    log_activity(task_id, "reasoning", "Analysis:", metadata={
        "approach": "...",
        "steps": [...]
    })

    try:
        # Log file reads
        content = read_file("path/to/file.ts")
        log_activity(task_id, "file_read", f"Read file.ts", metadata={
            "file": "path/to/file.ts",
            "bytes": len(content)
        })

        # Log file edits
        new_content = generate_code(content)
        write_file("path/to/file.ts", new_content)
        log_activity(task_id, "file_edit", f"Modified file.ts", metadata={
            "file": "path/to/file.ts",
            "lines_added": count_added,
            "lines_removed": count_removed
        })

        # Log commands
        result = run_command("npm test")
        log_activity(task_id, "command", f"Ran tests", metadata={
            "command": "npm test",
            "exit_code": result.exit_code,
            "duration_seconds": result.duration,
            "output": result.stdout
        })

        # Log discovery
        if issue_found:
            log_activity(task_id, "discovery", f"Found missing handler", metadata={
                "issue": "...",
                "severity": "high"
            })

        # Log result
        log_activity(task_id, "result", "Implementation complete", metadata={
            "confidence": 0.92,
            "summary": "...",
            "artifacts": {...}
        })

        # Complete task
        complete_task(task_id, confidence=0.92, summary="...")

    except Exception as e:
        # Log error
        log_activity(task_id, "error", f"Error: {str(e)}", metadata={
            "error_type": type(e).__name__,
            "traceback": traceback.format_exc()
        })
        # Don't complete; task will retry or be marked as failed
```

## Viewing Validation Results

Validation results are also logged in execution logs:

```bash
kanban cli task replay API-42 | grep validat

# Shows:
# [10:10:45Z] validating       | Started validation pipeline
# [10:10:46Z] validation_check | Check 1/3: Tests pass ✓ (42 tests)
# [10:10:47Z] validation_check | Check 2/3: Lint ✓ (no issues)
# [10:10:48Z] validation_check | Check 3/3: Type check ✓
# [10:10:48Z] validation_result| All checks passed (confidence: 0.92)
```

## Querying Logs Programmatically

### List All Logs for Issue

```bash
kanban cli task replay API-42 --json

# Returns:
# [
#   {
#     "id": 1,
#     "issue_id": 42,
#     "agent_id": "agent-xyz",
#     "attempt_number": 1,
#     "entry_type": "claim",
#     "message": "Agent claimed task",
#     "metadata": null,
#     "timestamp": "2025-03-15T10:00:00Z"
#   },
#   {
#     "id": 2,
#     "issue_id": 42,
#     "agent_id": "agent-xyz",
#     "attempt_number": 1,
#     "entry_type": "start",
#     "message": "Starting implementation",
#     "metadata": null,
#     "timestamp": "2025-03-15T10:00:15Z"
#   },
#   ...
# ]
```

### Filter by Agent

```bash
kanban cli task replay API-42 --agent agent-xyz --json
```

### Filter by Entry Type

```bash
kanban cli task replay API-42 --type file_edit --json

# Returns: Only file_edit entries
```

### Get Specific Attempt

```bash
kanban cli task attempts API-42 --attempt 2 --json

# Returns: Only logs from attempt 2
```

## Metrics from Logs

Calculate metrics from execution logs:

```bash
# Agent productivity
kanban cli metrics --agent agent-xyz

# Shows:
# Tasks completed: 23
# Avg duration: 3.2 hours
# Avg confidence: 0.89
# Success rate: 95% (22/23)
# Most common error: timeout (in 1 task)

# Per-skill breakdown:
# - code-review: 15 tasks, 0.92 avg confidence
# - testing: 8 tasks, 0.87 avg confidence
# - documentation: 3 tasks, 0.85 avg confidence
```

## Export & Analysis

### Export Logs for Analysis

```bash
# Export all logs for a project
kanban cli export --project 1 --output project-logs.json

# Returns:
# {
#   "project": {...},
#   "execution_logs": [...all logs...],
#   "task_contracts": [...all tasks...],
#   ...
# }
```

Then analyze with your tools:
- Python pandas for time series
- jq for JSON querying
- Excel for spreadsheets
- Grafana for visualizations

### Common Queries

```bash
# Find all tasks that timed out
kanban cli task replay --type timeout | jq '.[] | select(.entry_type == "timeout")'

# Find tasks with errors
kanban cli export --json | jq '.execution_logs[] | select(.entry_type == "error")'

# Average task duration by agent
kanban cli export --json | jq -r '.execution_logs[] |
  select(.entry_type == "complete") |
  "\(.agent_id) \(.metadata.duration_seconds)"'

# Tasks with low confidence
kanban cli export --json | jq '.execution_logs[] |
  select(.entry_type == "result" and .metadata.confidence < 0.70)'
```

## Retention & Cleanup

Execution logs are permanent for audit purposes. However, you can:

### Archive Old Data

```bash
# Export old logs
kanban cli export --project 1 --before "2025-01-01" --output archive.json

# Keep in database or delete (if needed)
# (Currently no delete command to protect audit trail)
```

### Manage Database Size

Logs are stored in `execution_logs` table. SQLite database grows with time:

```bash
# Check database size
ls -lh ~/.kanban/data.db
# Example: 45M (100k tasks with full logging)

# Optimize database
sqlite3 ~/.kanban/data.db "VACUUM;"
```

## Best Practices

### 1. Log Entry Types Correctly
Use the right type for each event:
- `claim` when agent claims
- `start` when work begins
- `file_read`/`file_edit` for code changes
- `command` for test runs
- `result` for completion

### 2. Include Useful Metadata
Attach structured data:
```json
{
  "file": "src/main.ts",
  "lines_added": 42,
  "operation": "create",
  "purpose": "Added password reset handler"
}
```

### 3. Use Descriptive Messages
Human-readable, but brief:
```
Good: "Modified src/auth.ts to add reset handler"
Bad: "changed file"
```

### 4. Log Errors with Context
Include what failed and why:
```json
{
  "error_type": "TypeScriptError",
  "message": "Type mismatch in handler signature",
  "file": "src/auth.ts",
  "line": 42,
  "expected": "Promise<void>",
  "got": "void"
}
```

### 5. Set Confidence Carefully
Be honest about certainty:
```
0.95 = Very confident (tests all pass, code reviewed)
0.85 = Confident (tests pass, some edge cases unclear)
0.70 = Somewhat confident (working but some concerns)
0.50 = Low confidence (needs review before production)
```

## Next Steps

- **[Task Contracts](/guide/task-contracts.md)** — Define tasks with success criteria
- **[Validation](/guide/validation.md)** — How results are validated
- **[Agent Routing](/guide/agent-routing.md)** — How agents are matched to tasks
