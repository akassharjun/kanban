# Kanban MCP Server

The Kanban Model Context Protocol (MCP) server enables AI clients (Claude, Cursor, etc.) to interact with the Kanban board programmatically via JSON-RPC 2.0.

## What is MCP?

The Model Context Protocol is a standard for AI systems to request context and perform actions through tools. It uses JSON-RPC 2.0 over stdin/stdout, making it language-agnostic and easy to integrate.

For more information, see the [MCP specification](https://modelcontextprotocol.io/).

## Kanban MCP Features

The Kanban MCP server exposes tools for:
- **Project management**: List, create, update projects
- **Issue tracking**: Create, search, update issues
- **Task execution**: Claim tasks, report completion, log activities
- **Agent coordination**: Register agents, heartbeat, deregister
- **Metrics and analytics**: Query project and agent metrics

## Starting the MCP Server

The MCP server runs as a subprocess and communicates via JSON-RPC 2.0 over stdio.

```bash
# Start the MCP server
kanban mcp

# The server reads JSON-RPC requests from stdin and writes responses to stdout
```

## Using Kanban with AI Clients

### Claude Desktop

Configure Claude Desktop to use Kanban MCP in `~/Library/Application\ Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "kanban": {
      "command": "kanban",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite://~/.kanban/data.db"
      }
    }
  }
}
```

### Cursor

Configure in your Cursor settings or `.cursor/mcp.json`:

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

### Generic MCP Client

Any MCP-compatible client can interact with Kanban:

```bash
#!/bin/bash
# Send a JSON-RPC request to the MCP server

kanban mcp <<'EOF'
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "list_projects",
  "params": {}
}
EOF
```

## JSON-RPC Protocol

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": "<unique-request-id>",
  "method": "<tool-name>",
  "params": {
    "<param-name>": "<param-value>"
  }
}
```

### Response Format (Success)

```json
{
  "jsonrpc": "2.0",
  "id": "<request-id>",
  "result": {
    "data": "<response-data>"
  }
}
```

### Response Format (Error)

```json
{
  "jsonrpc": "2.0",
  "id": "<request-id>",
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": "<error-details>"
  }
}
```

## Example: List Projects

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "list_projects",
  "params": {}
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": [
      {
        "id": 1,
        "name": "My Project",
        "prefix": "MP",
        "status": "active",
        "description": "Main project",
        "issue_counter": 42,
        "created_at": "2025-03-10T14:30:00Z",
        "updated_at": "2025-03-15T10:00:00Z"
      }
    ]
  }
}
```

## Example: Create an Issue

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "create_issue",
  "params": {
    "project_id": 1,
    "title": "Fix login bug",
    "description": "Users cannot log in with OAuth",
    "status_id": 9,
    "priority": "high",
    "assignee_id": 1
  }
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "data": {
      "id": 42,
      "project_id": 1,
      "identifier": "MP-42",
      "title": "Fix login bug",
      "description": "Users cannot log in with OAuth",
      "status_id": 9,
      "priority": "high",
      "assignee_id": 1,
      "created_at": "2025-03-15T10:00:00Z",
      "updated_at": "2025-03-15T10:00:00Z"
    }
  }
}
```

## Error Codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error |
| -32600 | Invalid Request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

## Environment Variables

- `DATABASE_URL` - SQLite database path (default: `~/.kanban/data.db`)

## Available Tools

See [MCP Tools Reference](./tools.md) for a complete list of all available tools.

## See Also

- [MCP Tools Reference](./tools.md)
- [MCP Configuration](./configuration.md)
- [Agent Protocol](../agents/index.md)
