# MCP Integration Skill

For agents that support the Model Context Protocol (MCP), the Kanban MCP server provides direct tool access to all board operations — no CLI needed.

## Overview

The MCP server exposes 60+ tools for managing projects, issues, statuses, labels, members, agents, and task contracts. Agents call these tools directly through the MCP protocol instead of shelling out to the CLI.

## Setup

Add to your agent's MCP configuration:

```json
{
  "mcpServers": {
    "kanban": {
      "command": "kanban",
      "args": ["mcp"]
    }
  }
}
```

For Claude Code, add this to `.claude/settings.json` or your user settings.

## Key Tools

### Issue Management

| Tool | Description |
|------|-------------|
| `list_issues` | List issues with optional filters (status, priority, assignee) |
| `create_issue` | Create a new issue with title, status, priority, assignee |
| `update_issue` | Update any issue field |
| `search_issues` | Full-text search across issues |
| `get_issue` | Get full issue details by ID |

### Workflow

| Tool | Description |
|------|-------------|
| `list_statuses` | Get available statuses for a project |
| `list_members` | Get team members for assignment |
| `list_labels` | Get available labels |
| `set_issue_labels` | Set labels on an issue |

### Agent Operations

| Tool | Description |
|------|-------------|
| `register_agent` | Register as an agent with skills and capabilities |
| `agent_heartbeat` | Send heartbeat to stay active |
| `next_task` | Claim the next available task matching agent skills |
| `start_task` | Mark a task as started |
| `complete_task` | Mark a task complete with confidence score |
| `fail_task` | Report task failure with reason |

## Embeddable MCP Skill

Add this to your agent's instructions to use MCP instead of CLI:

````markdown
## Kanban Board (MCP)

Use the Kanban MCP tools to track all work. The MCP server is configured as `kanban`.

### Workflow

1. **Before starting work**, create an issue:
   - Call `create_issue` with project_id, title, status_id, priority, assignee_id

2. **When starting work**, update status:
   - Call `update_issue` with status_id for "In Progress"

3. **When complete**, move to review:
   - Call `update_issue` with status_id for "In Review"

4. **List available work**:
   - Call `list_issues` with project_id and status filter for "Todo"

### Tool Examples

Create an issue:
```json
{ "tool": "create_issue", "input": { "project_id": 2, "title": "Fix login bug", "status_id": 9, "priority": "high", "assignee_id": 3 } }
```

Update status:
```json
{ "tool": "update_issue", "input": { "id": 42, "status_id": 10 } }
```

Search:
```json
{ "tool": "search_issues", "input": { "project_id": 2, "query": "login" } }
```
````

## Advantages Over CLI

| Feature | CLI | MCP |
|---------|-----|-----|
| Latency | Shell spawn (~100ms) | Direct IPC (~5ms) |
| Type safety | String parsing | Structured JSON |
| Error handling | Exit codes | JSON-RPC errors |
| Streaming | No | Yes (via MCP protocol) |
| Authentication | None needed | None needed |

MCP is the recommended integration method for agents that support it. Fall back to CLI for agents that don't.

## See Also

- [MCP Server Configuration](/mcp/configuration)
- [MCP Tools Reference](/mcp/tools)
- [Agent Protocol](/agents/)
