# Members Command

Manage team members via the CLI.

## Commands

### List members

List all team members (includes AI agents).

```bash
kanban member list [OPTIONS]
```

**Options:**
- None (shows all members)

**Examples:**

```bash
# List all members
kanban member list

# JSON output
kanban member list --json
```

**Output (default):**
```
ID  NAME                   EMAIL                DISPLAY_NAME     CREATED
1   Alice Johnson          alice@example.com    Alice            2025-03-01T10:00:00Z
2   Bob Smith              bob@example.com      Bob              2025-03-01T10:00:00Z
3   [claude] Claude Agent  (none)               Claude Agent     2025-03-15T09:00:00Z
4   Charlie Brown          charlie@example.com  Charlie          2025-03-05T14:30:00Z
```

**JSON Output:**
```json
[
  {
    "id": 1,
    "name": "Alice Johnson",
    "display_name": "Alice",
    "email": "alice@example.com",
    "avatar_color": "#3b82f6",
    "created_at": "2025-03-01T10:00:00Z"
  },
  {
    "id": 2,
    "name": "Bob Smith",
    "display_name": "Bob",
    "email": "bob@example.com",
    "avatar_color": "#ef4444",
    "created_at": "2025-03-01T10:00:00Z"
  },
  {
    "id": 3,
    "name": "[claude] Claude Agent",
    "display_name": "Claude Agent",
    "email": null,
    "avatar_color": "#f97316",
    "created_at": "2025-03-15T09:00:00Z"
  }
]
```

### Add a member

Add a new team member.

```bash
kanban member add <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Member name (required)

**Options:**
- `--email <EMAIL>` - Email address
- `--display-name <DISPLAY_NAME>` - Display name (shorter name for UI)

**Examples:**

```bash
# Simple member
kanban member add "John Doe"

# With email
kanban member add "Jane Smith" --email jane@example.com

# With display name
kanban member add "David Johnson" \
  --email david@example.com \
  --display-name "David"

# JSON output
kanban member add "Eve Wilson" --email eve@example.com --json
```

**Output:**
```
Created member: John Doe (ID: 5)
```

**JSON Output:**
```json
{
  "id": 5,
  "name": "John Doe",
  "display_name": null,
  "email": null,
  "avatar_color": "#a78bfa",
  "created_at": "2025-03-15T10:00:00Z"
}
```

### Delete a member

Remove a member from the system.

```bash
kanban member delete <ID>
```

**Arguments:**
- `<ID>` - Member ID to delete

**Examples:**

```bash
# Delete member 5
kanban member delete 5

# Output
Member 5 deleted.
```

**Notes:**
- Issues assigned to the deleted member will have their assignee set to null.
- Agent member records (created automatically when agents register) should not be manually deleted.

## Member Types

### Human Members
- Created manually with `kanban member add`
- Have email addresses and display names
- Can be assigned to issues and tasks

### Agent Members
- Created automatically when agents register
- Named in format: `[agent-type] agent-name` (e.g., `[claude] My Agent`)
- Used for tracking agent actions and assignments

## Avatar Colors

Member avatar colors are assigned randomly on creation:

- Team members: Various colors
- claude/claude-code agents: Orange (`#f97316`)
- codex agents: Green (`#22c55e`)
- gemini agents: Blue (`#3b82f6`)
- custom agents: Purple (`#8b5cf6`)

## Examples

### Add team members for a project

```bash
# Add the initial team
kanban member add "Alice Lead" --email alice@company.com --display-name "Alice"
kanban member add "Bob Dev" --email bob@company.com --display-name "Bob"
kanban member add "Charlie Designer" --email charlie@company.com --display-name "Charlie"

# Verify
kanban member list
```

### Remove a team member

```bash
# Find the member ID
kanban member list --json | jq '.[] | select(.email == "bob@company.com") | .id'

# Delete (assuming ID is 2)
kanban member delete 2

# Reassign their issues if needed
kanban issue list --project 1 --assignee 2 --json | \
  jq -r '.[] | .identifier' | \
  while read id; do
    kanban issue update "$id" --assignee 1  # Reassign to Alice
  done
```

### List agents vs humans

```bash
# Show only AI agents (those with [agent-type] prefix)
kanban member list --json | jq '.[] | select(.name | startswith("["))'

# Show only human members
kanban member list --json | jq '.[] | select(.name | startswith("[") | not)'

# Count by type
echo "Humans: $(kanban member list --json | jq '[.[] | select(.name | startswith("[") | not)] | length')"
echo "Agents: $(kanban member list --json | jq '[.[] | select(.name | startswith("["))] | length')"
```

## See Also

- [Issues Command](./issues.md)
- [Agents Command](./agents.md)
- [Tasks Command](./tasks.md)
