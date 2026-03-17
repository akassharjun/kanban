# MCP Configuration

Configure Kanban MCP for use with various AI clients and editors.

## Claude Desktop

Claude Desktop is the recommended MCP client for Kanban.

### Installation

1. Make sure the `kanban` binary is in your PATH
2. Edit `~/Library/Application\ Support/Claude/claude_desktop_config.json`
3. Add the following configuration

**macOS:**

```json
{
  "mcpServers": {
    "kanban": {
      "command": "/usr/local/bin/kanban",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite://~/.kanban/data.db"
      }
    }
  }
}
```

**Linux:**

```json
{
  "mcpServers": {
    "kanban": {
      "command": "/home/user/.local/bin/kanban",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite://~/.kanban/data.db"
      }
    }
  }
}
```

**Windows:**

```json
{
  "mcpServers": {
    "kanban": {
      "command": "C:\\Users\\User\\AppData\\Local\\kanban\\kanban.exe",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite://C:\\Users\\User\\.kanban\\data.db"
      }
    }
  }
}
```

### Verification

Once configured, you should see Kanban tools available in Claude. Test with:

"List my projects"

Claude should use the `list_projects` tool.

## Cursor

Cursor IDE supports MCP via its configuration system.

### Installation

1. Ensure the `kanban` binary is in your PATH
2. Create or edit `.cursor/mcp.json` in your workspace
3. Add Kanban configuration

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

### Environment Variables

If needed, set the database URL in your shell profile before starting Cursor:

```bash
export DATABASE_URL="sqlite://$HOME/.kanban/data.db"
cursor
```

### Verification

In Cursor, press Cmd+Shift+P and search for "MCP servers". You should see Kanban listed.

## Generic MCP Client

Any MCP-compatible client can use Kanban. Here's a generic example:

```python
#!/usr/bin/env python3
import subprocess
import json
import sys

def send_request(method, params=None):
    """Send a JSON-RPC request to Kanban MCP server."""
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params or {}
    }

    proc = subprocess.Popen(
        ["kanban", "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    stdout, stderr = proc.communicate(input=json.dumps(request))

    if proc.returncode != 0:
        print(f"Error: {stderr}", file=sys.stderr)
        return None

    return json.loads(stdout)

# Example usage
result = send_request("list_projects")
print(json.dumps(result, indent=2))
```

## Docker

Run Kanban MCP in a container:

```dockerfile
FROM rust:latest

RUN git clone https://github.com/your-org/kanban.git /app
WORKDIR /app
RUN cargo build --release

ENTRYPOINT ["/app/target/release/kanban", "mcp"]
```

Usage:

```bash
docker run \
  -e DATABASE_URL="sqlite:///data/kanban.db" \
  -v ~/.kanban:/data \
  kanban-mcp
```

## Systemd Service

Run Kanban MCP as a service:

```ini
# /etc/systemd/user/kanban-mcp.service

[Unit]
Description=Kanban MCP Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/kanban mcp
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment="DATABASE_URL=sqlite://%h/.kanban/data.db"

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user enable kanban-mcp
systemctl --user start kanban-mcp
systemctl --user status kanban-mcp
```

## Environment Variables

### DATABASE_URL

Path to the SQLite database.

```bash
# Default
export DATABASE_URL="sqlite://$HOME/.kanban/data.db"

# Custom location
export DATABASE_URL="sqlite:///var/lib/kanban/data.db"

# Start server with custom database
DATABASE_URL="sqlite:///data/kanban.db" kanban mcp
```

### LOG_LEVEL

Set logging level (optional):

```bash
export LOG_LEVEL="debug"
kanban mcp
```

## Testing Your Configuration

### Test with curl (if MCP is exposed over HTTP)

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "list_projects", "params": {}}'
```

### Test with the CLI tool

```bash
# If you have a test script
cat <<'EOF' | kanban mcp
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "list_projects",
  "params": {}
}
EOF
```

### Test with Claude Desktop

In Claude:
- Type "What projects do I have?"
- Claude should invoke the Kanban MCP tool
- You should see project information in the response

## Troubleshooting

### "Command not found: kanban"

Make sure the `kanban` binary is in your PATH:

```bash
# Check if kanban is in PATH
which kanban

# Add to PATH if needed
export PATH="$PATH:/path/to/kanban/target/release"
```

### Database not found

Ensure the database exists and the path is correct:

```bash
# Check database file
ls -la ~/.kanban/data.db

# Create directory if missing
mkdir -p ~/.kanban
```

### Permission denied

Ensure the database and directory are readable/writable:

```bash
chmod 755 ~/.kanban
chmod 644 ~/.kanban/data.db
```

### MCP server not responding

Check logs (for systemd):

```bash
journalctl --user -u kanban-mcp -f
```

Or run directly to see errors:

```bash
kanban mcp
# Press Ctrl+C after a few seconds
```

## Multiple Projects/Instances

If you have multiple Kanban instances, configure each separately:

```json
{
  "mcpServers": {
    "kanban-main": {
      "command": "kanban",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite:///home/user/.kanban/main.db"
      }
    },
    "kanban-test": {
      "command": "kanban",
      "args": ["mcp"],
      "env": {
        "DATABASE_URL": "sqlite:///home/user/.kanban/test.db"
      }
    }
  }
}
```

## See Also

- [MCP Index](./index.md)
- [MCP Tools Reference](./tools.md)
