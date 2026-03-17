# Kanban Quick Reference

Fast lookup for common operations. For detailed guides, see `/docs/site/guide/`.

## Installation

```bash
brew install kanban                    # Homebrew
kanban                                 # Launch GUI
kanban cli project list                # Run CLI command
kanban mcp                             # Start MCP server
```

## Projects

```bash
kanban cli project create "Name" --prefix "PRE"           # Create
kanban cli project list                                   # List
kanban cli project get 1                                  # Get
kanban cli project update 1 --name "New Name"             # Update
kanban cli project delete 1                               # Delete
```

## Issues

```bash
# Create
kanban cli issue create \
  --project 1 --title "Title" --status 2 --priority high

# List & Search
kanban cli issue list --project 1                         # All
kanban cli issue list --project 1 --status 2              # By status
kanban cli issue list --project 1 --priority high         # By priority
kanban cli issue search --project 1 "keyword"             # Search

# Update
kanban cli issue update PRE-1 --title "New title"         # Title
kanban cli issue update PRE-1 --status 3                  # Status
kanban cli issue update PRE-1 --priority urgent           # Priority
kanban cli issue update PRE-1 --assignee 1                # Assign

# Relations
kanban cli issue block PRE-5 --by PRE-3                   # Block relation
kanban cli issue relate PRE-7 --to PRE-6                  # Related relation
kanban cli issue move PRE-4 --parent PRE-1                # Set parent

# Delete
kanban cli issue delete PRE-1                             # Delete
```

## Statuses

```bash
kanban cli status list --project 1                        # List
kanban cli status create \
  --project 1 --name "Testing" --category started         # Create
kanban cli status update 3 --name "New Name"              # Update
kanban cli status delete 7                                # Delete
```

## Labels

```bash
kanban cli label create --project 1 --name "bug" --color "#ef4444"  # Create
kanban cli label list --project 1                                   # List
kanban cli label update 1 --color "#22c55e"                         # Update
kanban cli label delete 1                                           # Delete
```

## Members

```bash
kanban cli member add "email@example.com" --display-name "Name"  # Add
kanban cli member list                                           # List
kanban cli member get 1                                          # Get
kanban cli member update 1 --display-name "New Name"             # Update
```

## Task Contracts

```bash
# Create
kanban cli task create \
  --project 1 \
  --title "Title" \
  --objective "What to accomplish" \
  --status 2 \
  --type implementation \
  --skills "skill1,skill2" \
  --complexity medium \
  --success-criteria '[
    {
      "check": "Tests pass",
      "command": "npm test",
      "expect": "exit_code == 0"
    }
  ]' \
  --constraints "No breaking changes" \
  --timeout 60

# Get & List
kanban cli task get PRE-1                                 # Get
kanban cli task list --project 1                          # List all
kanban cli task list --project 1 --state queued           # By state

# Complete
kanban cli task complete \
  --identifier PRE-1 \
  --agent-id "agent-xyz" \
  --confidence 0.92 \
  --summary "Completed work"

# Log Activity
kanban cli task log-activity \
  --identifier PRE-1 \
  --agent-id "agent-xyz" \
  --entry-type file_edit \
  --message "Modified file" \
  --metadata '{"file": "src/main.ts"}'

# Replay
kanban cli task replay PRE-1                              # View log
kanban cli task attempts PRE-1                            # By attempt
```

## Agents

```bash
# Register
kanban cli agent register \
  --name "agent-name" \
  --agent-type claude \
  --skills "skill1,skill2" \
  --max-concurrent 2 \
  --max-complexity large

# List & Get
kanban cli agent list                                     # List
kanban cli agent get "agent-id"                           # Get

# Work
kanban cli agent next-task --agent-id "agent-id"         # Get task
kanban cli agent heartbeat --agent-id "agent-id"         # Send heartbeat

# Deregister
kanban cli agent deregister --agent-id "agent-id"        # Deregister

# Stats
kanban cli agent stats --agent-id "agent-id"             # Get stats
```

## Validation

```bash
# Run validation (manual)
kanban cli validation run --identifier PRE-1

# Show validation results
kanban cli validation show --identifier PRE-1

# Accept/Reject (manual review)
kanban cli task validation accept --identifier PRE-1 --confidence 0.80
kanban cli task validation reject --identifier PRE-1 --reason "Needs work"
```

## Export & Import

```bash
kanban cli export --output backup.json                    # Export all
kanban cli export --project 1 --output proj.json          # Export project
kanban cli export --after "2025-03-01" --output rec.json  # Export range

kanban cli import backup.json                             # Import
```

## Metrics

```bash
kanban cli metrics --project 1                            # Project metrics
kanban cli metrics --agent "agent-id"                     # Agent metrics
```

## Status Categories

| Category | Meaning | Use For |
|----------|---------|---------|
| `unstarted` | Work hasn't begun | Backlog, Todo |
| `started` | Work in progress | In Progress, In Review |
| `blocked` | Waiting on dependency | Blocked |
| `completed` | Work is done | Done |
| `discarded` | Work abandoned | Cancelled, Discarded |

## Task States

```
queued → claimed → executing → validating → completed
                                         ↘ blocked
                                         ↘ cancelled
```

## Priority Levels

```
none < low < medium < high < urgent
```

## Complexity Levels

```
small < medium < large
```

## Priority Thresholds (Default)

```
Auto-accept (confidence >= 0.85):
  → task_state = completed
  → issue.status = Done
  → No human review

Human Review (0.50 ≤ confidence < 0.85):
  → task_state = validating
  → issue.status = stayed
  → Need human decision

Auto-reject (confidence < 0.50):
  → task_state = blocked
  → issue.status = Blocked
  → Ready for retry
```

## Database Location

```bash
~/.kanban/data.db                       # Default SQLite

# Use custom location
export DATABASE_URL="sqlite:///path/to/db"
kanban

# Or via CLI
kanban --database-url "sqlite:///path/to/db"
```

## Activity Logging Example

```bash
# Log issue status change (manual)
sqlite3 ~/.kanban/data.db \
  "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) \
   VALUES (1, 'status_id', '2', '3', datetime('now'));"
```

## Common Workflows

### Create Issue & Assign to Agent

```bash
# 1. Create issue with task contract
kanban cli task create \
  --project 1 \
  --title "Implement feature" \
  --objective "Add feature X" \
  --status 2 \
  --type implementation \
  --skills "typescript,testing" \
  --complexity medium

# 2. Agent calls next-task
kanban cli agent next-task --agent-id "agent-xyz"

# 3. Agent does work, logs activity
kanban cli task log-activity \
  --identifier PROJ-1 \
  --agent-id agent-xyz \
  --entry-type file_edit \
  --message "Implemented feature"

# 4. Agent completes task
kanban cli task complete \
  --identifier PROJ-1 \
  --agent-id agent-xyz \
  --confidence 0.92 \
  --summary "Feature implemented and tested"

# 5. System validates & auto-accepts (or routes to human review)
```

### Dependency Chain

```bash
# 1. Create dependent tasks
kanban cli task create --project 1 --title "Task A" --status 2
kanban cli task create --project 1 --title "Task B" --status 2
kanban cli task create --project 1 --title "Task C" --status 2

# 2. Create blocking relations
kanban cli issue block PROJ-2 --by PROJ-1  # B blocked by A
kanban cli issue block PROJ-3 --by PROJ-2  # C blocked by B

# 3. Agent picks up Task A (first available)
kanban cli agent next-task --agent-id agent-1
# Returns: PROJ-1

# 4. Agent completes Task A
kanban cli task complete --identifier PROJ-1 --agent-id agent-1

# 5. Now Task B is available
kanban cli agent next-task --agent-id agent-2
# Returns: PROJ-2

# 6. Complete B, then C becomes available
```

### Bulk Update Issues

```bash
# Move all "Todo" issues to "In Progress"
kanban cli issue list --project 1 --status 2 | \
  jq '.[] | .id' | \
  xargs -I {} kanban cli issue update {} --status 3
```

### Backup & Restore

```bash
# Backup
kanban cli export --output backup-$(date +%Y%m%d).json

# Restore (into fresh database)
rm ~/.kanban/data.db
kanban cli import backup-20250315.json
```

## API Endpoints (Tauri Commands)

Commands can also be called from the frontend via Tauri:

```typescript
// TypeScript
import { invoke } from '@tauri-apps/api/tauri';

// Create issue
await invoke('create_issue', {
  input: {
    project_id: 1,
    title: "Title",
    status_id: 2,
    priority: "high"
  }
});

// Get issue
const issue = await invoke('get_issue', { id: 1 });

// Update issue
await invoke('update_issue', {
  id: 1,
  input: { title: "New Title" }
});
```

## MCP Server

```bash
kanban mcp

# Then connect via MCP client (e.g., Claude)
# The server exposes:
# - project management
# - issue CRUD
# - task contract creation
# - agent registration
# - task execution
```

## Troubleshooting

```bash
# Check database status
sqlite3 ~/.kanban/data.db "SELECT COUNT(*) FROM issues;"

# View last errors (CLI)
kanban cli project list 2>&1 | tail

# Force reset (CAREFUL!)
rm ~/.kanban/data.db
kanban app  # Recreates fresh DB
```

## Key Paths

```
Binary: /usr/local/bin/kanban
Config: ~/.kanban/
Database: ~/.kanban/data.db
Docs: /docs/site/guide/
```

## For More Details

- **Installation** → `/docs/site/guide/getting-started.md`
- **Concepts** → `/docs/site/guide/concepts.md`
- **Issues** → `/docs/site/guide/issues.md`
- **Task Contracts** → `/docs/site/guide/task-contracts.md`
- **Agent Routing** → `/docs/site/guide/agent-routing.md`
- **Validation** → `/docs/site/guide/validation.md`
- **Execution Logs** → `/docs/site/guide/execution-replay.md`
- **Import/Export** → `/docs/site/guide/import-export.md`
