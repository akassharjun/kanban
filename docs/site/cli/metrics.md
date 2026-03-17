# Metrics Command

View system metrics and analytics via the CLI.

## Commands

### View system metrics

Display metrics for the system, optionally filtered by project or agent.

```bash
kanban metrics [OPTIONS]
```

**Options:**
- `--project <PROJECT_ID>` - Show metrics for a specific project
- `--agent <AGENT_ID>` - Show metrics for a specific agent

**Examples:**

```bash
# System-wide metrics
kanban metrics

# Project metrics
kanban metrics --project 1

# Agent metrics
kanban metrics --agent 550e8400-e29b-41d4-a716-446655440000

# JSON output
kanban metrics --json
```

## System-Wide Metrics

Display overall system statistics.

**Example:**

```bash
kanban metrics
```

**Output:**

```
=== Kanban System Metrics ===

Projects:
  Total: 3
  Active: 2
  Paused: 1

Issues:
  Total: 127
  Open: 45
  Completed: 78
  Blocked: 4

Agents:
  Total registered: 5
  Currently idle: 3
  Currently busy: 2
  Online: 5
  Offline: 0

Task Execution:
  Total tasks: 127
  Completed: 89
  Failed: 8
  Success rate: 91.7%
  Average confidence: 0.91

Activity (Last 7 days):
  Issues created: 23
  Issues closed: 18
  Tasks completed: 34
  Agent heartbeats: 1,250
```

**JSON Output:**

```bash
kanban metrics --json
```

```json
{
  "timestamp": "2025-03-15T10:00:00Z",
  "projects": {
    "total": 3,
    "active": 2,
    "paused": 1,
    "completed": 0,
    "archived": 0
  },
  "issues": {
    "total": 127,
    "by_status": {
      "unstarted": 45,
      "started": 32,
      "blocked": 4,
      "completed": 78,
      "discarded": 0
    },
    "by_priority": {
      "none": 50,
      "low": 30,
      "medium": 35,
      "high": 10,
      "urgent": 2
    }
  },
  "agents": {
    "total": 5,
    "by_status": {
      "idle": 3,
      "busy": 2,
      "offline": 0
    },
    "by_type": {
      "claude": 2,
      "claude-code": 2,
      "codex": 1
    }
  },
  "tasks": {
    "total": 127,
    "by_state": {
      "unclaimed": 15,
      "claimed": 5,
      "executing": 3,
      "validating": 2,
      "completed": 89,
      "failed": 8,
      "blocked": 5
    },
    "completion_stats": {
      "total_completed": 89,
      "total_failed": 8,
      "success_rate": 0.917,
      "average_confidence": 0.91,
      "total_execution_time_seconds": 532800,
      "average_time_per_task_seconds": 5988
    }
  },
  "activity": {
    "period_days": 7,
    "issues_created": 23,
    "issues_closed": 18,
    "tasks_completed": 34,
    "agent_heartbeats": 1250
  }
}
```

## Project Metrics

Display metrics for a specific project.

**Example:**

```bash
kanban metrics --project 1
```

**Output:**

```
=== Metrics for Project: My Project (KAN) ===

Issues:
  Total: 87
  Open: 32
  Completed: 53
  Blocked: 2

By Priority:
  Urgent: 1
  High: 8
  Medium: 24
  Low: 20
  None: 34

By Status:
  Backlog: 5
  Todo: 18
  In Progress: 6
  In Review: 3
  Blocked: 2
  Done: 53

Issues Created: 87
Issues Closed: 53
Closure Rate: 60.9%
Average Estimate: 5.2 hours

Related Tasks:
  Task Contracts: 35
  Completed: 28
  Success Rate: 80%
  In Progress: 3
  Failed: 4

Team:
  Assigned Members: 4
  Top Contributor: Alice (18 issues)
  Current Assignee Count: 32 issues
```

**JSON Output:**

```bash
kanban metrics --project 1 --json
```

```json
{
  "project_id": 1,
  "project_name": "My Project",
  "project_prefix": "KAN",
  "issues": {
    "total": 87,
    "by_status": {
      "backlog": 5,
      "todo": 18,
      "in_progress": 6,
      "in_review": 3,
      "blocked": 2,
      "done": 53,
      "discarded": 0
    },
    "by_priority": {
      "none": 34,
      "low": 20,
      "medium": 24,
      "high": 8,
      "urgent": 1
    }
  },
  "tasks": {
    "total": 35,
    "completed": 28,
    "failed": 4,
    "in_progress": 3,
    "success_rate": 0.875,
    "average_confidence": 0.89
  },
  "team": {
    "total_members": 4,
    "issues_by_assignee": {
      "alice": 18,
      "bob": 12,
      "charlie": 10,
      "unassigned": 47
    }
  },
  "timeline": {
    "first_issue": "2025-01-10T08:00:00Z",
    "last_issue": "2025-03-15T09:30:00Z",
    "duration_days": 64
  }
}
```

## Agent Metrics

Display metrics for a specific agent.

**Example:**

```bash
kanban metrics --agent 550e8400-e29b-41d4-a716-446655440000
```

**Output:**

```
=== Agent Metrics ===

Agent: code-analyzer
Type: claude-code
Status: idle
Registered: 2025-03-10T14:00:00Z
Last Active: 2025-03-15T09:50:00Z

Tasks:
  Total: 42
  Completed: 39
  Failed: 3
  Success Rate: 92.9%

Performance:
  Average Confidence: 0.91
  Total Execution Time: 245 hours
  Average Time per Task: 5.8 hours
  Min Time: 0.5 hours
  Max Time: 18 hours

Skills Usage:
  python: 25 tasks (59.5%)
  rust: 12 tasks (28.6%)
  testing: 18 tasks (42.9%)
  refactoring: 8 tasks (19%)

Recent Activity:
  Last 24h: 3 tasks completed
  Last 7d: 18 tasks completed
  Last 30d: 35 tasks completed

Quality Metrics:
  Average time to completion: 5.8h
  Tasks with >0.95 confidence: 28
  Tasks with <0.80 confidence: 2
  Revision requests: 1
```

**JSON Output:**

```bash
kanban metrics --agent 550e8400-e29b-41d4-a716-446655440000 --json
```

```json
{
  "agent_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_name": "code-analyzer",
  "agent_type": "claude-code",
  "status": "idle",
  "registered_at": "2025-03-10T14:00:00Z",
  "last_heartbeat": "2025-03-15T09:50:00Z",
  "tasks": {
    "total": 42,
    "completed": 39,
    "failed": 3,
    "success_rate": 0.929
  },
  "performance": {
    "average_confidence": 0.91,
    "total_execution_time_seconds": 882000,
    "average_time_per_task_seconds": 20909,
    "min_time_seconds": 1800,
    "max_time_seconds": 64800
  },
  "skills": {
    "python": 25,
    "rust": 12,
    "testing": 18,
    "refactoring": 8
  },
  "activity": {
    "last_24h": 3,
    "last_7d": 18,
    "last_30d": 35
  },
  "quality": {
    "high_confidence_tasks": 28,
    "low_confidence_tasks": 2,
    "revision_requests": 1
  }
}
```

## Interpreting Metrics

### Success Rate
- Percentage of tasks completed successfully
- Higher is better
- Target: >90%

### Average Confidence
- Agent's average confidence in their solutions
- Range: 0.0 (no confidence) to 1.0 (complete confidence)
- Target: >0.85

### Average Execution Time
- Average time per task
- Useful for capacity planning
- Compare across agents for performance

### Skills Breakdown
- Shows which skills the agent uses most
- Useful for matching agents to tasks

## Examples

### Dashboard view

```bash
#!/bin/bash
# Display a simple dashboard

echo "=== Kanban Dashboard ==="
echo

# System metrics
echo "--- System Stats ---"
kanban metrics | head -20
echo

# Project metrics
echo "--- Project: My Project ---"
kanban metrics --project 1 | head -15
echo

# Agent status
echo "--- Agent Health ---"
kanban agent list --json | jq '.[] | {name, status, last_heartbeat}'
```

### Export metrics to CSV

```bash
#!/bin/bash
# Export agent metrics to CSV

echo "agent_id,name,type,total_tasks,completed,failed,success_rate,avg_confidence" > agents.csv

kanban agent list --json | jq -r '.[] | .id' | while read agent_id; do
  kanban metrics --agent "$agent_id" --json | jq -r \
    '.agent_id + "," + .agent_name + "," + .agent_type + "," + \
     .tasks.total + "," + .tasks.completed + "," + .tasks.failed + "," + \
     .tasks.success_rate + "," + .performance.average_confidence' >> agents.csv
done

echo "Exported to agents.csv"
```

### Monitor agent performance

```bash
#!/bin/bash
# Script to monitor agent performance every 5 minutes

AGENT_ID="550e8400-e29b-41d4-a716-446655440000"

while true; do
  METRICS=$(kanban metrics --agent "$AGENT_ID" --json)
  SUCCESS=$(echo "$METRICS" | jq '.tasks.success_rate')
  AVG_CONF=$(echo "$METRICS" | jq '.performance.average_confidence')
  TASKS=$(echo "$METRICS" | jq '.tasks.total')

  echo "$(date) - Tasks: $TASKS | Success: $SUCCESS | Confidence: $AVG_CONF"

  sleep 300
done
```

## See Also

- [Agents Command](./agents.md)
- [Tasks Command](./tasks.md)
- [Projects Command](./projects.md)
