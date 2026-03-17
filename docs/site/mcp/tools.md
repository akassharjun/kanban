# MCP Tools Reference

Complete reference of all tools exposed by the Kanban MCP server.

## Projects

### list_projects

List all active projects.

**Method:** `list_projects`

**Parameters:**
- None

**Response:**

```json
{
  "data": [
    {
      "id": 1,
      "name": "My Project",
      "prefix": "MP",
      "status": "active",
      "description": "Main project",
      "icon": "🚀",
      "issue_counter": 42,
      "path": "/path/to/project",
      "created_at": "2025-03-10T14:30:00Z",
      "updated_at": "2025-03-15T10:00:00Z"
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "list_projects",
  "params": {}
}
```

### create_project

Create a new project.

**Method:** `create_project`

**Parameters:**
- `name` (string, required) - Project name
- `prefix` (string, required) - Issue prefix
- `description` (string, optional) - Project description
- `icon` (string, optional) - Project icon

**Response:**

```json
{
  "data": {
    "id": 3,
    "name": "New Project",
    "prefix": "NP",
    "status": "active",
    "description": null,
    "icon": null,
    "issue_counter": 0,
    "path": null,
    "created_at": "2025-03-15T10:00:00Z",
    "updated_at": "2025-03-15T10:00:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "create_project",
  "params": {
    "name": "New Project",
    "prefix": "NP",
    "description": "A new project"
  }
}
```

## Issues

### list_issues

List issues in a project with optional filters.

**Method:** `list_issues`

**Parameters:**
- `project_id` (integer, required) - Project ID
- `status_id` (integer, optional) - Filter by status
- `priority` (string, optional) - Filter by priority
- `assignee_id` (integer, optional) - Filter by assignee

**Response:**

```json
{
  "data": [
    {
      "id": 42,
      "project_id": 1,
      "identifier": "MP-42",
      "title": "Fix login bug",
      "description": "Users cannot log in",
      "status_id": 10,
      "priority": "high",
      "assignee_id": 1,
      "parent_id": null,
      "position": 0.0,
      "estimate": 8.0,
      "due_date": "2025-03-20",
      "created_at": "2025-03-10T14:30:00Z",
      "updated_at": "2025-03-15T10:00:00Z"
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "list_issues",
  "params": {
    "project_id": 1,
    "priority": "high"
  }
}
```

### create_issue

Create a new issue.

**Method:** `create_issue`

**Parameters:**
- `project_id` (integer, required)
- `title` (string, required)
- `description` (string, optional)
- `status_id` (integer, required)
- `priority` (string, optional)
- `assignee_id` (integer, optional)
- `parent_id` (integer, optional) - Parent issue ID for subtasks

**Response:**

```json
{
  "data": {
    "id": 45,
    "project_id": 1,
    "identifier": "MP-45",
    "title": "New Issue",
    "description": null,
    "status_id": 9,
    "priority": "none",
    "assignee_id": null,
    "parent_id": null,
    "position": 0.0,
    "estimate": null,
    "due_date": null,
    "created_at": "2025-03-15T10:00:00Z",
    "updated_at": "2025-03-15T10:00:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "create_issue",
  "params": {
    "project_id": 1,
    "title": "Fix bug",
    "status_id": 9,
    "priority": "high"
  }
}
```

### update_issue

Update an issue by identifier.

**Method:** `update_issue`

**Parameters:**
- `identifier` (string, required) - Issue identifier (e.g., MP-42)
- `title` (string, optional)
- `description` (string, optional)
- `status_id` (integer, optional)
- `priority` (string, optional)
- `assignee_id` (integer, optional)

**Response:**

```json
{
  "data": {
    "id": 42,
    "identifier": "MP-42",
    "title": "Updated Title",
    "status_id": 10,
    "priority": "urgent",
    "assignee_id": 2
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "update_issue",
  "params": {
    "identifier": "MP-42",
    "status_id": 10,
    "priority": "urgent"
  }
}
```

### search_issues

Search for issues by query.

**Method:** `search_issues`

**Parameters:**
- `project_id` (integer, required)
- `query` (string, required) - Search query (title and description)

**Response:**

```json
{
  "data": [
    {
      "id": 42,
      "project_id": 1,
      "identifier": "MP-42",
      "title": "Fix login bug",
      "description": "Users cannot log in",
      "status_id": 10,
      "priority": "high",
      "assignee_id": 1
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "search_issues",
  "params": {
    "project_id": 1,
    "query": "login"
  }
}
```

## Agents

### register_agent

Register a new AI agent.

**Method:** `register_agent`

**Parameters:**
- `name` (string, optional) - Agent name (auto-generated if not provided)
- `agent_type` (string, optional) - Type: claude, claude-code, codex, gemini, custom
- `skills` (array, required) - List of skill strings
- `task_types` (array, optional) - Task types
- `max_concurrent` (integer, optional, default: 1)
- `max_complexity` (string, optional, default: large)
- `worktree_path` (string, optional) - Working directory

**Response:**

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "code-analyzer",
    "agent_type": "claude-code",
    "skills": ["python", "rust", "analysis"],
    "task_types": [],
    "max_concurrent": 1,
    "max_complexity": "large",
    "member_id": 3,
    "status": "idle",
    "registered_at": "2025-03-15T10:00:00Z",
    "last_heartbeat": "2025-03-15T10:00:00Z",
    "last_activity_at": null,
    "worktree_path": null
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "register_agent",
  "params": {
    "name": "code-analyzer",
    "agent_type": "claude-code",
    "skills": ["python", "rust", "analysis"]
  }
}
```

### list_agents

List all registered agents.

**Method:** `list_agents`

**Parameters:**
- None

**Response:**

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "code-analyzer",
      "agent_type": "claude-code",
      "skills": ["python", "rust"],
      "status": "idle",
      "registered_at": "2025-03-15T10:00:00Z",
      "last_heartbeat": "2025-03-15T10:00:00Z"
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "list_agents",
  "params": {}
}
```

### agent_heartbeat

Send a heartbeat for an agent.

**Method:** `agent_heartbeat`

**Parameters:**
- `agent_id` (string, required) - Agent UUID

**Response:**

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "code-analyzer",
    "status": "idle",
    "last_heartbeat": "2025-03-15T10:05:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "agent_heartbeat",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### deregister_agent

Deregister an agent and reclaim its tasks.

**Method:** `deregister_agent`

**Parameters:**
- `agent_id` (string, required) - Agent UUID

**Response:**

```json
{
  "data": {
    "status": "deregistered",
    "reclaimed_tasks": 2
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "deregister_agent",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## Tasks

### create_task

Create a new task contract.

**Method:** `create_task`

**Parameters:**
- `project_id` (integer, required)
- `title` (string, required)
- `objective` (string, required)
- `status_id` (integer, required)
- `type` (string, optional) - Task type
- `priority` (string, optional)
- `skills` (array, optional) - Required skills
- `complexity` (string, optional) - small, medium, large
- `constraints` (array, optional)
- `success_criteria` (array, optional)
- `context_files` (array, optional)
- `timeout_minutes` (integer, optional, default: 30)
- `depends_on` (array, optional) - Task identifiers

**Response:**

```json
{
  "data": {
    "id": 45,
    "project_id": 1,
    "identifier": "MP-45",
    "title": "Add password reset",
    "objective": "Implement email-based password reset",
    "type": "feature",
    "status_id": 9,
    "task_state": "unclaimed",
    "skills": ["python", "email"],
    "complexity": "medium",
    "constraints": [],
    "success_criteria": [],
    "timeout_minutes": 30,
    "claimed_by": null,
    "created_at": "2025-03-15T10:00:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "method": "create_task",
  "params": {
    "project_id": 1,
    "title": "Add password reset",
    "objective": "Implement email-based password reset",
    "status_id": 9,
    "skills": ["python", "email"],
    "complexity": "medium"
  }
}
```

### next_task

Get the next available task for an agent.

**Method:** `next_task`

**Parameters:**
- `agent_id` (string, required) - Agent UUID
- `skills` (array, optional) - Override agent skills

**Response:**

```json
{
  "data": {
    "id": 45,
    "identifier": "MP-45",
    "title": "Add password reset",
    "objective": "Implement email-based password reset",
    "type": "task",
    "complexity": "medium",
    "required_skills": ["python"],
    "timeout_minutes": 30,
    "task_state": "claimed",
    "claimed_by": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 12,
  "method": "next_task",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### start_task

Start execution of a claimed task.

**Method:** `start_task`

**Parameters:**
- `identifier` (string, required) - Task identifier
- `agent_id` (string, required) - Agent UUID

**Response:**

```json
{
  "data": {
    "identifier": "MP-45",
    "task_state": "executing",
    "started_at": "2025-03-15T10:01:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 13,
  "method": "start_task",
  "params": {
    "identifier": "MP-45",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### complete_task

Mark a task as completed.

**Method:** `complete_task`

**Parameters:**
- `identifier` (string, required)
- `agent_id` (string, required)
- `confidence` (number, required) - 0.0 to 1.0
- `summary` (string, required) - Completion summary
- `artifacts` (object, optional) - Additional artifacts

**Response:**

```json
{
  "data": {
    "identifier": "MP-45",
    "task_state": "validating",
    "confidence": 0.95,
    "summary": "Implemented feature",
    "completed_at": "2025-03-15T10:30:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 14,
  "method": "complete_task",
  "params": {
    "identifier": "MP-45",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "confidence": 0.95,
    "summary": "Implemented password reset feature with tests"
  }
}
```

### fail_task

Mark a task as failed.

**Method:** `fail_task`

**Parameters:**
- `identifier` (string, required)
- `agent_id` (string, required)
- `reason` (string, required)

**Response:**

```json
{
  "data": {
    "identifier": "MP-45",
    "task_state": "failed",
    "reason": "Database migration failed",
    "failed_at": "2025-03-15T10:45:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 15,
  "method": "fail_task",
  "params": {
    "identifier": "MP-45",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "reason": "Database migration failed"
  }
}
```

### unclaim_task

Return a claimed task without completion.

**Method:** `unclaim_task`

**Parameters:**
- `identifier` (string, required)
- `agent_id` (string, required)
- `reason` (string, optional)

**Response:**

```json
{
  "data": {
    "identifier": "MP-45",
    "task_state": "unclaimed",
    "unclai_at": "2025-03-15T10:50:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 16,
  "method": "unclaim_task",
  "params": {
    "identifier": "MP-45",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### log_task_activity

Log an execution entry for a task.

**Method:** `log_task_activity`

**Parameters:**
- `identifier` (string, required)
- `agent_id` (string, required)
- `entry_type` (string, required) - info, warning, error, progress
- `message` (string, required)
- `metadata` (object, optional)

**Response:**

```json
{
  "data": {
    "identifier": "MP-45",
    "entry_type": "progress",
    "message": "Completed unit tests",
    "logged_at": "2025-03-15T10:15:00Z"
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 17,
  "method": "log_task_activity",
  "params": {
    "identifier": "MP-45",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "entry_type": "progress",
    "message": "Completed unit tests"
  }
}
```

### task_replay

Get execution logs for a task.

**Method:** `task_replay`

**Parameters:**
- `identifier` (string, required)

**Response:**

```json
{
  "data": [
    {
      "timestamp": "2025-03-15T10:01:00Z",
      "entry_type": "progress",
      "message": "Starting implementation"
    },
    {
      "timestamp": "2025-03-15T10:15:00Z",
      "entry_type": "progress",
      "message": "Completed unit tests"
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 18,
  "method": "task_replay",
  "params": {
    "identifier": "MP-45"
  }
}
```

### task_attempts

Get all execution attempts for a task.

**Method:** `task_attempts`

**Parameters:**
- `identifier` (string, required)

**Response:**

```json
{
  "data": [
    {
      "attempt": 1,
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "claimed_at": "2025-03-15T10:00:00Z",
      "started_at": "2025-03-15T10:01:00Z",
      "ended_at": "2025-03-15T10:30:00Z",
      "state": "failed",
      "reason": "Dependency not available"
    },
    {
      "attempt": 2,
      "agent_id": "660e8400-e29b-41d4-a716-446655440001",
      "claimed_at": "2025-03-15T11:00:00Z",
      "started_at": "2025-03-15T11:01:00Z",
      "ended_at": "2025-03-15T11:45:00Z",
      "state": "completed",
      "confidence": 0.95
    }
  ]
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 19,
  "method": "task_attempts",
  "params": {
    "identifier": "MP-45"
  }
}
```

## Metrics

### project_metrics

Get metrics for a project.

**Method:** `project_metrics`

**Parameters:**
- `project_id` (integer, required)

**Response:**

```json
{
  "data": {
    "project_id": 1,
    "project_name": "My Project",
    "issues": {
      "total": 87,
      "open": 32,
      "completed": 53,
      "blocked": 2
    },
    "tasks": {
      "total": 35,
      "completed": 28,
      "failed": 4,
      "success_rate": 0.875
    }
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 20,
  "method": "project_metrics",
  "params": {
    "project_id": 1
  }
}
```

### agent_metrics

Get metrics for an agent.

**Method:** `agent_metrics`

**Parameters:**
- `agent_id` (string, required)

**Response:**

```json
{
  "data": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "agent_name": "code-analyzer",
    "tasks": {
      "total": 42,
      "completed": 39,
      "failed": 3,
      "success_rate": 0.929
    },
    "performance": {
      "average_confidence": 0.91,
      "average_time_per_task_hours": 5.8
    }
  }
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 21,
  "method": "agent_metrics",
  "params": {
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## Error Responses

All error responses follow the JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": "Project not found"
  }
}
```

Common error codes:
- `-32700` - Parse error
- `-32600` - Invalid Request
- `-32601` - Method not found
- `-32602` - Invalid params
- `-32603` - Internal error (including business logic errors)

## See Also

- [MCP Index](./index.md)
- [MCP Configuration](./configuration.md)
