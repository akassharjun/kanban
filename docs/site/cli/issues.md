# Issues Command

Manage Kanban issues via the CLI.

## Commands

### List issues

List issues in a project with optional filters.

```bash
kanban issue list --project <PROJECT_ID> [OPTIONS]
```

**Required:**
- `--project <PROJECT_ID>` - Project ID to list issues from

**Options:**
- `--status <STATUS_ID>` - Filter by status ID (e.g., 9 for Todo, 10 for In Progress)
- `--priority <PRIORITY>` - Filter by priority (none, low, medium, high, urgent)
- `--assignee <MEMBER_ID>` - Filter by assignee member ID

**Examples:**

```bash
# List all issues in project 1
kanban issue list --project 1

# List high-priority issues
kanban issue list --project 1 --priority high

# List all issues assigned to member 3
kanban issue list --project 1 --assignee 3

# In Progress issues only
kanban issue list --project 1 --status 10

# Multiple filters
kanban issue list --project 1 --priority high --status 10 --assignee 3

# JSON output
kanban issue list --project 1 --json
```

**Output (default):**
```
ID   IDENTIFIER  TITLE                    STATUS        PRIORITY  ASSIGNEE
42   KAN-42      Fix login bug            In Progress   high      Alice
43   KAN-43      Add dark mode            Todo          medium    Bob
44   KAN-44      Update docs              In Review     low       (unassigned)
```

**JSON Output:**
```json
[
  {
    "id": 42,
    "project_id": 1,
    "identifier": "KAN-42",
    "title": "Fix login bug",
    "description": "Users cannot log in with OAuth",
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
```

### Create an issue

Create a new issue.

```bash
kanban issue create --project <PROJECT_ID> --title <TITLE> --status <STATUS_ID> [OPTIONS]
```

**Required:**
- `--project <PROJECT_ID>` - Project ID
- `--title <TITLE>` - Issue title
- `--status <STATUS_ID>` - Initial status ID

**Options:**
- `--priority <PRIORITY>` - Priority (none, low, medium, high, urgent) [default: none]
- `--description <DESCRIPTION>` - Issue description (markdown)
- `--assignee <MEMBER_ID>` - Assignee member ID
- `--parent <PARENT_ID>` - Parent issue ID (for subtasks)

**Examples:**

```bash
# Simple issue
kanban issue create --project 1 --title "Fix bug" --status 9

# With priority and assignee
kanban issue create \
  --project 1 \
  --title "Implement dark mode" \
  --status 9 \
  --priority high \
  --assignee 2

# With description
kanban issue create \
  --project 1 \
  --title "Update documentation" \
  --status 9 \
  --description "Update API docs for v2.0 endpoints" \
  --priority medium

# As a subtask
kanban issue create \
  --project 1 \
  --title "Configure TypeScript" \
  --status 9 \
  --parent 42 \
  --priority medium

# JSON output
kanban issue create \
  --project 1 \
  --title "New feature" \
  --status 9 \
  --priority high \
  --json
```

**Output:**
```
Created issue: KAN-45 - Implement dark mode
Assigned to: Alice
Status: Todo
Priority: high
```

**JSON Output:**
```json
{
  "id": 45,
  "project_id": 1,
  "identifier": "KAN-45",
  "title": "Implement dark mode",
  "description": null,
  "status_id": 9,
  "priority": "high",
  "assignee_id": 2,
  "parent_id": null,
  "position": 0.0,
  "estimate": null,
  "due_date": null,
  "created_at": "2025-03-15T10:00:00Z",
  "updated_at": "2025-03-15T10:00:00Z"
}
```

### Update an issue

Update issue fields.

```bash
kanban issue update <IDENTIFIER> [OPTIONS]
```

**Arguments:**
- `<IDENTIFIER>` - Issue identifier (e.g., KAN-42)

**Options:**
- `--title <TITLE>` - New title
- `--status <STATUS_ID>` - New status ID
- `--priority <PRIORITY>` - New priority
- `--assignee <MEMBER_ID>` - New assignee
- `--description <DESCRIPTION>` - New description

**Examples:**

```bash
# Update title
kanban issue update KAN-42 --title "Update login flow"

# Move to In Progress
kanban issue update KAN-42 --status 10

# Change priority
kanban issue update KAN-42 --priority urgent

# Assign to someone
kanban issue update KAN-42 --assignee 3

# Multiple fields
kanban issue update KAN-42 \
  --title "Critical fix needed" \
  --priority urgent \
  --status 10 \
  --assignee 1

# JSON output
kanban issue update KAN-42 --priority high --json
```

**Output:**
```
Updated KAN-42: Fix login bug
Title: Update login flow
Priority: urgent
Status: In Progress (10)
Assignee: Charlie (3)
```

### Search issues

Search for issues by query string.

```bash
kanban issue search --project <PROJECT_ID> <QUERY>
```

**Required:**
- `--project <PROJECT_ID>` - Project to search in
- `<QUERY>` - Search query (searches title and description)

**Examples:**

```bash
# Find all issues mentioning "login"
kanban issue search --project 1 "login"

# Search for "database"
kanban issue search --project 1 "database"

# JSON output
kanban issue search --project 1 "bug" --json
```

**Output:**
```
Found 3 issues:

KAN-42  Fix login bug              In Progress  high
KAN-51  Login timeout issue        Todo         medium
KAN-63  Session handling on login  Blocked      urgent
```

### Move issue (set parent)

Create a parent-child relationship (make an issue a subtask).

```bash
kanban issue move <IDENTIFIER> --parent <PARENT_IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Issue to move
- `--parent <PARENT_IDENTIFIER>` - Parent issue identifier

**Examples:**

```bash
# Make KAN-50 a subtask of KAN-42
kanban issue move KAN-50 --parent KAN-42

# Output
KAN-50 is now a subtask of KAN-42
```

### Block issue

Add a "blocks" relationship (issue A blocks issue B).

```bash
kanban issue block <IDENTIFIER> --by <BLOCKING_IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Issue being blocked
- `--by <BLOCKING_IDENTIFIER>` - Issue that blocks it

**Examples:**

```bash
# KAN-43 is blocked by KAN-42
kanban issue block KAN-43 --by KAN-42

# Output
KAN-43 is blocked by KAN-42
```

### Relate issues

Add a "related" relationship between two issues.

```bash
kanban issue relate <IDENTIFIER> --to <RELATED_IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - First issue
- `--to <RELATED_IDENTIFIER>` - Second issue

**Examples:**

```bash
# KAN-42 is related to KAN-51
kanban issue relate KAN-42 --to KAN-51

# Output
KAN-42 is related to KAN-51
```

### Delete an issue

Soft-delete an issue.

```bash
kanban issue delete <IDENTIFIER>
```

**Arguments:**
- `<IDENTIFIER>` - Issue identifier to delete

**Examples:**

```bash
kanban issue delete KAN-99
Issue KAN-99 deleted.
```

## Priority Values

| Priority | Usage |
|----------|-------|
| `none` | No priority (default) |
| `low` | Low priority |
| `medium` | Medium priority |
| `high` | High priority |
| `urgent` | Urgent/critical |

## Status IDs

Status IDs vary per project. Common values:

| Status ID | Category | Example |
|-----------|----------|---------|
| 8 | unstarted | Backlog |
| 9 | unstarted | Todo |
| 10 | started | In Progress |
| 11 | started | In Review |
| 12 | blocked | Blocked |
| 13 | completed | Done |
| 14 | discarded | Discarded |

Get actual status IDs for your project via the web UI or by examining the database.

## Examples

### Typical workflow

```bash
# 1. Create an issue
kanban issue create \
  --project 1 \
  --title "Implement user authentication" \
  --status 9 \
  --priority high \
  --description "Add OAuth2 support"

# 2. Assign it
kanban issue update KAN-42 --assignee 2

# 3. Move to In Progress
kanban issue update KAN-42 --status 10

# 4. Create subtasks
kanban issue create --project 1 --title "Setup OAuth provider" --status 9 --parent 42
kanban issue create --project 1 --title "Implement callback handler" --status 9 --parent 42
kanban issue create --project 1 --title "Add tests" --status 9 --parent 42

# 5. Transition through workflow
kanban issue update KAN-43 --status 10  # In Progress
kanban issue update KAN-44 --status 10
kanban issue update KAN-45 --status 10

# 6. Mark as done
kanban issue update KAN-43 --status 13
kanban issue update KAN-44 --status 13
kanban issue update KAN-45 --status 13
kanban issue update KAN-42 --status 13
```

### Search and batch update

```bash
#!/bin/bash
# Find all high-priority bugs and assign to Alice (member 1)
kanban issue search --project 1 "bug" --json | \
  jq -r '.[] | select(.priority == "high") | .identifier' | \
  while read id; do
    kanban issue update "$id" --assignee 1 --priority urgent
  done
```

### Create from template

```bash
#!/bin/bash
# Create multiple related issues
PROJECT=1
PARENT=$(kanban issue create --project $PROJECT --title "Q2 Sprint" --status 9 --json | jq -r '.id')

for task in "Design" "Implement" "Test" "Deploy"; do
  kanban issue create --project $PROJECT --title "$task" --status 9 --parent $PARENT
done
```

## See Also

- [Projects Command](./projects.md)
- [Members Command](./members.md)
- [Tasks Command](./tasks.md)
