# Projects

A project is the top-level container for all your work. Learn how to create, manage, and organize projects.

## Creating a Project

Every project needs:
- **Name** — What you're calling it (e.g., "Backend API")
- **Prefix** — 2-4 character code (e.g., `API`) used in issue IDs
- **Description** — (Optional) Longer explanation
- **Icon** — (Optional) Emoji or icon for UI

### Via CLI

```bash
kanban cli project create "Backend API" \
  --prefix "API" \
  --description "REST API for mobile app" \
  --icon "🚀"
```

### Via GUI
1. Click "New Project"
2. Fill in name, prefix, description
3. Click "Create"

When you create a project, **default statuses are automatically created**:
- Backlog (unstarted)
- Todo (unstarted)
- In Progress (started)
- In Review (started)
- Blocked (blocked)
- Done (completed)
- Discarded (discarded)

## Listing Projects

### Via CLI

```bash
# List all projects
kanban cli project list

# Output:
# ID | Name          | Prefix | Status | Issues
# 1  | Backend API   | API    | active | 12
# 2  | Mobile App    | APP    | active | 8
# 3  | DevOps        | OPS    | paused | 3
```

### Via GUI
Projects appear in the left sidebar.

## Getting a Project

### Via CLI

```bash
kanban cli project get 1

# Output:
# {
#   "id": 1,
#   "name": "Backend API",
#   "description": "REST API for mobile app",
#   "prefix": "API",
#   "issue_counter": 12,
#   "status": "active",
#   "created_at": "2025-03-15T10:00:00Z",
#   "updated_at": "2025-03-15T14:30:00Z"
# }
```

## Updating a Project

### Via CLI

```bash
# Update name
kanban cli project update 1 --name "Backend API v2"

# Update status (pause work)
kanban cli project update 1 --status paused

# Update description
kanban cli project update 1 --description "New description"

# Valid statuses: active, paused, completed, archived
```

### Via GUI
Click a project → Edit settings

## Deleting a Project

Deletion is a soft delete (marked as deleted, not permanently removed).

### Via CLI

```bash
kanban cli project delete 1
```

### Via GUI
Right-click a project → Delete

:::warning
Soft-deleted projects are hidden from UI but remain in the database. You can recover them by directly updating the database if needed.
:::

## Project Path

For agent-based projects, you can set a **project path** — a local directory where the agent operates.

```bash
kanban cli project update 1 --path "/home/user/projects/backend-api"
```

This path is used by agents to:
- Know where to clone/open the repository
- Store worktree state
- Reference files in relative paths

## Project Statuses

| Status | Meaning |
|--------|---------|
| `active` | Project is ongoing |
| `paused` | Project temporarily on hold |
| `completed` | Project is finished |
| `archived` | Project is archived (read-only) |

Agents will not pick up tasks from `paused`, `completed`, or `archived` projects.

## Working with Statuses in a Project

Each project has its own set of statuses (columns on the board).

### List Statuses

```bash
kanban cli status list --project 1

# Output:
# ID | Name         | Category    | Color   | Position
# 1  | Backlog      | unstarted   | #6b7280 | 0
# 2  | Todo         | unstarted   | #6b7280 | 1
# 3  | In Progress  | started     | #3b82f6 | 2
# 4  | In Review    | started     | #8b5cf6 | 3
# 5  | Blocked      | blocked     | #ef4444 | 4
# 6  | Done         | completed   | #22c55e | 5
# 7  | Discarded    | discarded   | #6b7280 | 6
```

### Create a Custom Status

```bash
kanban cli status create \
  --project 1 \
  --name "Testing" \
  --category started \
  --color "#ec4899" \
  --icon "🧪"
```

### Update a Status

```bash
kanban cli status update 3 --name "In Progress (Assigned)"
kanban cli status update 3 --color "#3b82f6"
kanban cli status update 3 --position 2
```

### Delete a Status

```bash
kanban cli status delete 7
```

:::tip
You can't delete a status if issues are in it. Move them first or cascade delete the entire project.
:::

## Working with Members in a Project

Members (team members or agents) are workspace-scoped but can be assigned to issues in any project.

### List Members

```bash
kanban cli member list

# Output:
# ID | Name              | Email                | Avatar Color
# 1  | Alice Chen        | alice@example.com    | #3b82f6
# 2  | [claude] Claude   | (none)               | #f97316
# 3  | Bob Smith         | bob@example.com      | #8b5cf6
```

### Add a Member

```bash
kanban cli member add "alice@example.com" \
  --display-name "Alice Chen" \
  --avatar-color "#3b82f6"
```

### Assign Member to Issue

```bash
kanban cli issue update API-1 --assignee 1
```

## Working with Labels in a Project

Labels are project-scoped for organization and filtering.

### Create a Label

```bash
kanban cli label create \
  --project 1 \
  --name "backend" \
  --color "#3b82f6"

kanban cli label create \
  --project 1 \
  --name "security" \
  --color "#ef4444"
```

### List Labels

```bash
kanban cli label list --project 1
```

### Assign Label to Issue

```bash
kanban cli issue create \
  --project 1 \
  --title "Add CORS headers" \
  --status 2 \
  --labels "backend,security"
```

## Project Metrics

View high-level metrics for a project:

```bash
kanban cli metrics --project 1

# Output:
# Total Issues: 12
# Backlog: 2
# Todo: 3
# In Progress: 4
# In Review: 1
# Blocked: 1
# Done: 1
# Discarded: 0
#
# Active Agents: 2
# Avg Task Completion Time: 2.5 hours
# Avg Confidence: 0.87
```

## Best Practices

### Use Clear Prefixes
Pick prefixes that are:
- 2-4 characters
- Easy to remember
- Unique across projects
- Example: `API`, `WEB`, `CLI`, `OPS`

### Organize Statuses by Category
Group statuses by their category:
- **Unstarted:** Backlog, Todo
- **Started:** In Progress, In Review
- **Blocked:** Blocked, Waiting
- **Completed:** Done
- **Discarded:** Cancelled

This makes agent routing and automation predictable.

### Use Labels Consistently
Define labels early and document them:
- `bug` — Defect to fix
- `feature` — New capability
- `tech-debt` — Maintenance work
- `research` — Investigation needed
- `urgent` — Drops everything else

### Set Project Path for Agent Projects
If you plan to use agents, set the project path:

```bash
kanban cli project update 1 --path "/home/user/projects/my-repo"
```

This helps agents understand where to work.

## Next Steps

- **[Issues](/guide/issues.md)** — Create and manage work items
- **[Statuses](/guide/statuses.md)** — Deep dive into workflow configuration
- **[Labels](/guide/labels.md)** — Organizing with tags
- **[Members](/guide/members.md)** — Team and agent assignment
