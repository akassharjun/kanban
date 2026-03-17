# Projects Command

Manage Kanban projects via the CLI.

## Commands

### List projects

List all active projects.

```bash
kanban project list [OPTIONS]
```

**Output (default):**
```
ID  NAME               PREFIX  STATUS   ISSUES  CREATED
1   My Project         PRJ     active   12      2025-03-10T14:30:00Z
2   Backend Services   BE      active   8       2025-03-12T09:15:00Z
3   Archived Project   AR      archived 0       2025-02-28T11:00:00Z
```

**Output (JSON):**
```bash
kanban project list --json
```

```json
[
  {
    "id": 1,
    "name": "My Project",
    "prefix": "PRJ",
    "status": "active",
    "description": "Our main project",
    "icon": "🚀",
    "issue_counter": 12,
    "path": "/path/to/project",
    "created_at": "2025-03-10T14:30:00Z",
    "updated_at": "2025-03-15T10:00:00Z"
  }
]
```

### Create a project

Create a new project with a name and prefix.

```bash
kanban project create <name> --prefix <PREFIX> [OPTIONS]
```

**Arguments:**
- `<name>` - Project name (required)

**Options:**
- `--prefix <PREFIX>` - Issue prefix for the project (required, e.g., "KAN", "PRJ")
- `--description <DESCRIPTION>` - Project description
- `--icon <ICON>` - Emoji or icon for the project

**Examples:**

```bash
# Simple project
kanban project create "Frontend App" --prefix FE

# With description and icon
kanban project create "Backend Services" \
  --prefix BE \
  --description "API, database, and worker services" \
  --icon "⚙️"

# JSON output
kanban project create "Mobile App" --prefix MB --json
```

**Output:**
```
Created project: My Project (ID: 3, prefix: MP)
```

**JSON Output:**
```json
{
  "id": 3,
  "name": "Mobile App",
  "prefix": "MB",
  "status": "active",
  "description": null,
  "icon": null,
  "issue_counter": 0,
  "path": null,
  "created_at": "2025-03-15T10:00:00Z",
  "updated_at": "2025-03-15T10:00:00Z"
}
```

### Update a project

Update project details.

```bash
kanban project update <id> [OPTIONS]
```

**Arguments:**
- `<id>` - Project ID

**Options:**
- `--name <NAME>` - New project name
- `--description <DESCRIPTION>` - New description
- `--status <STATUS>` - New status (active, paused, completed, archived)
- `--icon <ICON>` - New icon/emoji

**Examples:**

```bash
# Update name
kanban project update 1 --name "Renamed Project"

# Update status
kanban project update 2 --status archived

# Update description
kanban project update 1 --description "Our main product" --icon "🎯"

# All at once
kanban project update 3 \
  --name "New Name" \
  --description "Updated description" \
  --status paused \
  --icon "⏸️"

# JSON output
kanban project update 1 --name "Updated" --json
```

**Output:**
```
Updated project: Renamed Project
```

**JSON Output:**
```json
{
  "id": 1,
  "name": "Renamed Project",
  "prefix": "PRJ",
  "status": "active",
  "description": "Updated description",
  "icon": "🎯",
  "issue_counter": 12,
  "path": null,
  "created_at": "2025-03-10T14:30:00Z",
  "updated_at": "2025-03-15T10:30:00Z"
}
```

### Delete a project

Soft-delete a project (can be restored).

```bash
kanban project delete <id>
```

**Arguments:**
- `<id>` - Project ID to delete

**Examples:**

```bash
# Delete a project
kanban project delete 5

# Confirm deletion
Are you sure? (y/n) y
Project 5 deleted.
```

**Notes:**
- Deletion is soft (archived). The project and its issues remain in the database.
- Issues within the deleted project are also marked as deleted.
- To restore, use the web UI or modify the database directly.

## Status Values

| Status | Description |
|--------|-------------|
| `active` | Project is active and in use |
| `paused` | Project is paused (no new work) |
| `completed` | Project is completed |
| `archived` | Project is archived |

## Prefix Rules

- Must be 1-5 characters (uppercase letters and numbers only)
- Must be unique across all projects
- Used for auto-generating issue identifiers (e.g., PRJ-1, PRJ-2)

## Examples

### Complete workflow

```bash
# 1. Create a project
kanban project create "Q2 Goals" --prefix Q2G --description "Second quarter objectives" --icon "📊"

# 2. List projects to confirm
kanban project list

# 3. Update project status
kanban project update 4 --status active

# 4. Later, pause it
kanban project update 4 --status paused

# 5. Eventually archive it
kanban project update 4 --status archived

# 6. Delete when completely done
kanban project delete 4
```

### Integration with scripts

```bash
#!/bin/bash
# Export all projects as JSON
kanban project list --json > projects.json

# Create a project from script
PROJECT_ID=$(kanban project create "New Project" --prefix NEW --json | jq '.id')
echo "Created project: $PROJECT_ID"

# Update all projects to archived
kanban project list --json | jq -r '.[].id' | while read id; do
  kanban project update "$id" --status archived
done
```

## See Also

- [Issues Command](./issues.md)
- [Labels Command](./labels.md)
