# Import & Export

Kanban can export all data to JSON and import it back. This is useful for:
- **Backups** — Save your data to disk
- **Migration** — Move between machines or databases
- **Integration** — Export to external systems
- **Analysis** — Work with data in Python, Excel, etc.
- **Snapshots** — Version control your project state

## Exporting Data

### Export Everything

Export all projects, issues, agents, logs, and more:

```bash
kanban cli export --output backup.json

# Creates: backup.json (complete snapshot)
```

### Export a Specific Project

```bash
kanban cli export --project 1 --output project-1.json

# Creates: JSON with only project 1 data
```

### Export by Date Range

```bash
kanban cli export \
  --after "2025-03-01" \
  --before "2025-03-15" \
  --output recent-work.json
```

### Export to stdout

```bash
kanban cli export --json > backup.json

# Or for a specific project
kanban cli export --project 1 --json | jq . > formatted.json
```

## Export Format

The exported JSON has this structure:

```json
{
  "version": "1.0",
  "exported_at": "2025-03-15T15:30:00Z",
  "database_backend": "sqlite",
  "projects": [
    {
      "id": 1,
      "name": "Backend API",
      "prefix": "API",
      "description": "REST API backend",
      "status": "active",
      "created_at": "2025-03-10T10:00:00Z",
      "updated_at": "2025-03-15T14:30:00Z"
    }
  ],
  "statuses": [
    {
      "id": 1,
      "project_id": 1,
      "name": "Backlog",
      "category": "unstarted",
      "color": "#6b7280",
      "position": 0
    }
  ],
  "issues": [
    {
      "id": 1,
      "project_id": 1,
      "identifier": "API-1",
      "title": "Implement authentication",
      "description": "Add OAuth support",
      "status_id": 2,
      "priority": "high",
      "assignee_id": null,
      "parent_id": null,
      "estimate": 8.0,
      "due_date": "2025-03-20",
      "created_at": "2025-03-10T10:00:00Z",
      "updated_at": "2025-03-15T14:30:00Z"
    }
  ],
  "labels": [
    {
      "id": 1,
      "project_id": 1,
      "name": "bug",
      "color": "#ef4444"
    }
  ],
  "members": [
    {
      "id": 1,
      "name": "alice@example.com",
      "display_name": "Alice Chen",
      "email": "alice@example.com",
      "avatar_color": "#3b82f6",
      "created_at": "2025-03-10T10:00:00Z"
    }
  ],
  "issue_labels": [
    {
      "issue_id": 1,
      "label_id": 1
    }
  ],
  "issue_relations": [
    {
      "id": 1,
      "source_issue_id": 1,
      "target_issue_id": 2,
      "relation_type": "blocks"
    }
  ],
  "task_contracts": [
    {
      "issue_id": 1,
      "type": "implementation",
      "task_state": "completed",
      "objective": "Add OAuth login",
      "context": {
        "files": ["src/auth/oauth.ts"],
        "related_tasks": []
      },
      "constraints": ["No breaking changes"],
      "success_criteria": [
        {
          "check": "Tests pass",
          "command": "npm test",
          "expect": "exit_code == 0"
        }
      ],
      "required_skills": ["typescript", "auth"],
      "estimated_complexity": "medium",
      "timeout_minutes": 60,
      "claimed_by": "agent-xyz",
      "claimed_at": "2025-03-15T10:00:00Z",
      "result": {
        "confidence": 0.92,
        "summary": "OAuth implemented with tests"
      },
      "attempt_count": 1
    }
  ],
  "agents": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "code-reviewer",
      "agent_type": "claude",
      "skills": ["code-review", "testing"],
      "task_types": ["review"],
      "max_concurrent": 2,
      "max_complexity": "large",
      "member_id": 2,
      "status": "idle",
      "registered_at": "2025-03-10T09:00:00Z",
      "last_heartbeat": "2025-03-15T15:00:00Z"
    }
  ],
  "agent_stats": [
    {
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "tasks_completed": 15,
      "tasks_failed": 1,
      "total_confidence": 13.8,
      "total_completion_time_seconds": 1800,
      "skills_breakdown": {
        "code-review": 10,
        "testing": 5
      }
    }
  ],
  "execution_logs": [
    {
      "id": 1,
      "issue_id": 1,
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "attempt_number": 1,
      "entry_type": "claim",
      "message": "Agent claimed task",
      "metadata": null,
      "timestamp": "2025-03-15T10:00:00Z"
    }
  ],
  "activity_log": [
    {
      "id": 1,
      "issue_id": 1,
      "field_changed": "status_id",
      "old_value": "2",
      "new_value": "3",
      "timestamp": "2025-03-15T10:05:00Z"
    }
  ],
  "comments": [
    {
      "id": 1,
      "issue_id": 1,
      "member_id": 1,
      "content": "Great work!",
      "created_at": "2025-03-15T10:10:00Z",
      "updated_at": "2025-03-15T10:10:00Z"
    }
  ]
}
```

## Importing Data

### Import Full Backup

```bash
kanban cli import backup.json

# Restores:
# - All projects
# - All issues and sub-issues
# - All statuses, labels, members
# - All task contracts with results
# - All execution logs
# - All activity and comments
```

:::warning
Import will **merge** with existing data. If you import a project with the same name, it will create a duplicate. Use caution when importing to an existing database.
:::

### Import into Clean Database

Start fresh:

```bash
# Delete the old database (CAREFUL!)
rm ~/.kanban/data.db

# Initialize new database (created on first run)
kanban app  # Start app, let it create fresh database, exit

# Import your data
kanban cli import backup.json
```

### Merge from Another Machine

Transfer data from one machine to another:

```bash
# On machine A: export
kanban cli export --output my-projects.json

# Transfer file to machine B

# On machine B: import
kanban cli import my-projects.json
```

## Use Cases

### Backup Before Major Changes

```bash
# Before making big changes, backup
kanban cli export --output backup-$(date +%Y%m%d).json

# Make changes...

# If something breaks, restore
rm ~/.kanban/data.db
kanban cli import backup-20250315.json
```

### Export for Analysis

```bash
# Export all data
kanban cli export --output analysis.json

# Analyze with Python
python3 << 'EOF'
import json

with open('analysis.json') as f:
    data = json.load(f)

# Count issues by status
status_map = {s['id']: s['name'] for s in data['statuses']}
issues_by_status = {}
for issue in data['issues']:
    status = status_map[issue['status_id']]
    issues_by_status[status] = issues_by_status.get(status, 0) + 1

print("Issues by status:")
for status, count in issues_by_status.items():
    print(f"  {status}: {count}")

# Agent productivity
for agent in data['agents']:
    stats = next(s for s in data['agent_stats'] if s['agent_id'] == agent['id'])
    print(f"\n{agent['name']} ({agent['agent_type']}):")
    print(f"  Completed: {stats['tasks_completed']}")
    print(f"  Failed: {stats['tasks_failed']}")
    print(f"  Avg confidence: {stats['total_confidence'] / stats['tasks_completed']:.2%}")
EOF
```

### Export Project for Sharing

```bash
# Export just one project
kanban cli export --project 1 --output share-project.json

# Send to colleague or archive
```

### Version Control Your Board

```bash
# Regular snapshots in git
kanban cli export --output .snapshots/board-$(date +%Y%m%d-%H%M%S).json

# Track in git for history
git add .snapshots/
git commit -m "Board snapshot $(date)"
```

### Bulk Data Cleanup

```bash
# Export
kanban cli export --output data.json

# Edit with jq or Python to remove/modify data
jq 'del(.execution_logs[] | select(.timestamp < "2025-01-01"))' data.json > cleaned.json

# Re-import (into fresh database)
rm ~/.kanban/data.db
kanban cli import cleaned.json
```

## Format Specifications

### DateTime Format
All dates are ISO 8601 UTC:
```
2025-03-15T15:30:45Z
```

### IDs
- **Project/Issue/Label/Member IDs** — BigInt (1, 2, 3...)
- **Agent IDs** — UUID v4 strings
- **Attempt Numbers** — BigInt (1, 2, 3...)

### Enums

**Priority:**
```
"none", "low", "medium", "high", "urgent"
```

**Status Category:**
```
"unstarted", "started", "blocked", "completed", "discarded"
```

**Project Status:**
```
"active", "paused", "completed", "archived"
```

**Task Type:**
```
"implementation", "review", "decomposition"
```

**Task State:**
```
"queued", "claimed", "executing", "validating", "completed", "blocked", "cancelled"
```

**Relation Type:**
```
"related", "blocks", "blocked_by", "duplicate"
```

## Importing Considerations

### 1. Database Conflicts

If you import while data exists, you may get conflicts:

```bash
# If issue identifier already exists, import will fail
# Error: UNIQUE constraint failed: issues.identifier

# Solution: Import into clean database or modify identifiers
```

### 2. Foreign Keys

Import respects foreign key constraints:
- Issues reference valid status_ids
- Task contracts reference valid issue_ids
- Members must be created before assigning to issues

If import fails due to FK violations, check that:
- All referenced projects exist
- All referenced statuses exist
- All referenced members exist

### 3. Agent References

Agent UUIDs are preserved during import. If you import the same agent twice:

```bash
# First import: agent "550e8400..." created
# Second import: conflict on agent.id UNIQUE constraint

# Solution: Use different database or delete agent first
```

## Advanced: Custom Import/Export Scripts

### Export Specific Queries

```bash
# Export only completed tasks
kanban cli export --json | \
  jq '.task_contracts[] | select(.task_state == "completed")' \
  > completed-tasks.json

# Export only failed attempts
kanban cli export --json | \
  jq '.execution_logs[] | select(.entry_type == "error")' \
  > errors.json

# Export agent stats only
kanban cli export --json | jq '.agent_stats' > agent-performance.json
```

### Transform During Import

```bash
# Change all project prefixes
jq '.projects[] |= .prefix |= "NEW-" + .' backup.json > renamed.json
kanban cli import renamed.json

# Reassign all issues to a different user
jq '.issues[] |= .assignee_id = 5' backup.json > reassigned.json
kanban cli import reassigned.json

# Zero out all agent stats
jq '.agent_stats[] |= {agent_id, tasks_completed: 0, tasks_failed: 0, total_confidence: 0.0}' backup.json > reset-stats.json
kanban cli import reset-stats.json
```

## Troubleshooting

### Import Fails with "Unknown field"

Your import file might be from a newer version. Download the latest Kanban.

### Import Fails with UNIQUE constraint

Identifier or agent ID already exists:

```bash
# Option 1: Use different database
export DATABASE_URL="sqlite:///fresh.db"
kanban cli import backup.json

# Option 2: Modify identifiers in file
jq '.issues[] |= .identifier |= . + "-import"' backup.json > renamed.json
kanban cli import renamed.json

# Option 3: Clear database
rm ~/.kanban/data.db
kanban cli import backup.json
```

### Import Fails with foreign key violation

Referenced data doesn't exist. Check:

```bash
# Export current state
kanban cli export --json > current.json

# Check what projects exist
jq '.projects[].id' current.json

# Check what statuses exist
jq '.statuses[] | {id, name}' current.json

# Your import references non-existent status_id
```

### Restore is Slow

Large imports (millions of rows) can take time:

```bash
# Progress indication (if available)
# On large databases (>100MB), restore might take minutes

# For faster import, disable sync:
# (This is automatic; just be patient)

# Monitor with:
ls -lh ~/.kanban/data.db  # Watch file size grow
```

## Best Practices

### 1. Regular Backups
```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d)
kanban cli export --output ~/backups/kanban-$DATE.json
```

### 2. Name Backups by Purpose
```bash
kanban cli export --output backup-before-refactor.json
kanban cli export --output backup-release-v1.0.json
kanban cli export --output backup-2025-03-15.json
```

### 3. Test Imports Before Production
```bash
# Test import in temporary database
export DATABASE_URL="sqlite:///test.db"
kanban cli import backup.json
kanban cli metrics --project 1

# If OK, then use in production
unset DATABASE_URL
kanban cli import backup.json
```

### 4. Document Your Backups
```
backups/
├── README.md
│   "Backups of Kanban database
│    - Daily: kanban-YYYYMMDD.json
│    - Before major changes: backup-{reason}.json
│    - Restore: kanban cli import {file}.json"
├── kanban-20250310.json
├── kanban-20250311.json
├── backup-before-refactor.json
└── backup-release-v1.0.json
```

## Next Steps

- **[Projects](/guide/projects.md)** — Manage projects
- **[Issues](/guide/issues.md)** — Manage issues
- **[Task Contracts](/guide/task-contracts.md)** — Create executable tasks
