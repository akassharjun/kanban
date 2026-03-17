# Issues

Issues are the core unit of work in Kanban. Learn how to create, manage, search, and organize them.

## Creating an Issue

An issue needs:
- **Project** — Which project it belongs to
- **Title** — Short summary (required)
- **Status** — Which column (required)
- **Description** — Detailed context (optional)
- **Priority** — `none`, `low`, `medium`, `high`, `urgent` (default: `none`)
- **Assignee** — Person or agent responsible (optional)
- **Parent ID** — For sub-tasks (optional)
- **Estimate** — Time estimate or story points (optional)
- **Due Date** — YYYY-MM-DD format (optional)
- **Labels** — Tags for grouping (optional)

### Via CLI

```bash
# Simple issue
kanban cli issue create \
  --project 1 \
  --title "Fix login bug" \
  --status 2 \
  --priority high

# Full issue with all fields
kanban cli issue create \
  --project 1 \
  --title "Implement password reset" \
  --status 2 \
  --priority medium \
  --description "
    Add password reset flow:
    1. Email verification
    2. Token generation
    3. Reset form
    4. Success confirmation
  " \
  --assignee 1 \
  --estimate 8 \
  --due-date "2025-03-20" \
  --labels "feature,auth"
```

The system automatically:
1. Increments the project's issue counter
2. Generates an identifier (e.g., `API-42`)
3. Logs the creation in the activity log
4. Records in undo/redo stack

### Via GUI
1. Click the "+" button or right-click a status
2. Fill in the form
3. Click "Create"

## Getting Issue Details

### Via CLI

```bash
kanban cli issue get API-42

# Output:
# {
#   "id": 42,
#   "project_id": 1,
#   "identifier": "API-42",
#   "title": "Implement password reset",
#   "description": "Add password reset flow...",
#   "status_id": 2,
#   "priority": "medium",
#   "assignee_id": 1,
#   "parent_id": null,
#   "estimate": 8.0,
#   "due_date": "2025-03-20",
#   "created_at": "2025-03-15T10:00:00Z",
#   "updated_at": "2025-03-15T10:05:00Z",
#   "labels": [
#     { "id": 1, "name": "feature", "color": "#3b82f6" },
#     { "id": 2, "name": "auth", "color": "#8b5cf6" }
#   ]
# }
```

## Listing Issues

### Via CLI

```bash
# List all issues in a project
kanban cli issue list --project 1

# Filter by status
kanban cli issue list --project 1 --status 2

# Filter by priority
kanban cli issue list --project 1 --priority high

# Filter by assignee
kanban cli issue list --project 1 --assignee 1

# Filter by label
kanban cli issue list --project 1 --label 5
```

### Via GUI
Click a project to see its board. Issues are grouped by status (column).

## Searching Issues

### Via CLI

```bash
# Search by title or description
kanban cli issue search --project 1 "password reset"

# Output:
# ID | Identifier | Title                      | Status    | Priority
# 42 | API-42     | Implement password reset   | Todo      | medium
# 51 | API-51     | Reset password via email   | Backlog   | low
```

Search looks in:
- Issue titles
- Descriptions
- Comments
- Task contract objectives

## Updating an Issue

### Via CLI

Update any field:

```bash
# Update title
kanban cli issue update API-42 --title "New title"

# Update status (move to different column)
kanban cli issue update API-42 --status 3

# Update priority
kanban cli issue update API-42 --priority urgent

# Update assignee
kanban cli issue update API-42 --assignee 2

# Update description
kanban cli issue update API-42 --description "Updated details"

# You can combine multiple fields
kanban cli issue update API-42 \
  --title "Updated" \
  --status 4 \
  --priority high
```

Each update is logged in the activity log.

### Via GUI
1. Click an issue to open the detail panel
2. Edit fields directly
3. Changes save automatically

## Deleting an Issue

### Via CLI

```bash
kanban cli issue delete API-42
```

This performs a soft delete. The issue record stays in the database but is marked as deleted.

:::warning
Deleted issues and their execution logs are still kept for audit purposes. Use with caution.
:::

## Sub-tasks (Parent-Child Issues)

Create hierarchical task structures:

### Via CLI

```bash
# Create a parent issue
kanban cli issue create \
  --project 1 \
  --title "Release v1.0" \
  --status 2

# Create child issues (sub-tasks)
kanban cli issue create \
  --project 1 \
  --title "Finalize API docs" \
  --status 2 \
  --parent 100  # Parent issue ID

kanban cli issue create \
  --project 1 \
  --title "Add e2e tests" \
  --status 2 \
  --parent 100

kanban cli issue create \
  --project 1 \
  --title "Update changelog" \
  --status 2 \
  --parent 100
```

### Via GUI
1. Open a parent issue
2. Click "Add Sub-task"
3. Fill in the sub-task

Sub-tasks:
- Are not visible as separate cards on the board
- Count toward parent progress
- Can be assigned independently
- Inherit some parent properties

## Issue Relations

Link issues together for dependency tracking.

### Block Relation

Mark one issue as blocked by another:

```bash
# API-45 is blocked by API-42
kanban cli issue block API-45 --by API-42

# This creates a relation:
# API-42 (blocks) → API-45 (blocked_by)
```

**Agent Impact:** Agents won't pick up API-45 until API-42 is completed.

### Related Relation

Mark issues as related (no blocking):

```bash
# API-50 is related to API-49
kanban cli issue relate API-50 --to API-49
```

### Move (Change Parent)

```bash
# Make API-44 a sub-task of API-40
kanban cli issue move API-44 --parent API-40
```

## Activity Log

Every change to an issue is tracked:

```bash
kanban cli issue activity API-42

# Output:
# Timestamp              | Field      | Old Value           | New Value
# 2025-03-15 10:00:00Z   | title      | "Fix login bug"     | "Fix login page crash"
# 2025-03-15 10:05:00Z   | status_id  | "2"                 | "3"
# 2025-03-15 10:10:00Z   | priority   | "high"              | "urgent"
# 2025-03-15 14:30:00Z   | assignee   | "1"                 | "2"
```

This provides:
- Full audit trail
- When changes happened
- Who changed them (via comments)
- Impact analysis

## Bulk Operations

Update multiple issues at once:

### Via CLI

```bash
# Move all issues from status 2 to status 3
kanban cli issue bulk-update \
  --project 1 \
  --from-status 2 \
  --to-status 3

# Or via JSON (coming soon)
kanban cli issue bulk-update --file bulk-changes.json
```

### Via GUI
1. Select multiple issues (checkboxes)
2. Bulk action menu appears
3. Choose "Move to", "Set priority", etc.

## Positioning (Drag-and-Drop Order)

Issues have a `position` field that controls their order within a status.

### Via GUI
Click and drag an issue to reorder.

### Via CLI
```bash
# Reposition within status
kanban cli issue update API-42 --position 3.5
```

Positions are floating-point numbers for smooth insertion between items. When you drag issue A between B and C:
- Get position of B: 2.0
- Get position of C: 3.0
- Set A's position to: 2.5

## Duplicating an Issue

Create a copy of an issue:

```bash
kanban cli issue duplicate API-42

# Creates a new issue with:
# - Same title (with "Copy of" prefix)
# - Same description
# - Same priority
# - Blank assignee and due date
# - New ID (e.g., API-99)
```

## Undo/Redo

All issue operations support undo/redo:

### Via GUI
- Press Ctrl+Z (Cmd+Z on macOS) to undo
- Press Ctrl+Y (Cmd+Shift+Z on macOS) to redo

### Via CLI (Tauri commands)
```bash
# Undo the last operation
kanban undo

# Redo the last undone operation
kanban redo
```

## Batch Import

Import issues from JSON:

```bash
# issues.json
[
  {
    "title": "Feature A",
    "description": "Do feature A",
    "status_id": 2,
    "priority": "high"
  },
  {
    "title": "Feature B",
    "description": "Do feature B",
    "status_id": 2,
    "priority": "medium"
  }
]

kanban cli issue batch-create \
  --project 1 \
  --file issues.json
```

## Best Practices

### Keep Titles Concise
Good: "Add OAuth login"
Bad: "User needs to be able to log in with their Google or Facebook account and we need to handle the JWT tokens properly"

### Write Descriptive Descriptions
Use Markdown for clarity:

```markdown
# Acceptance Criteria
- [ ] Supports Google OAuth
- [ ] Supports Facebook OAuth
- [ ] JWT tokens stored securely
- [ ] Logout clears session

# Implementation Notes
- Use google-oauth2-strategy for OAuth
- Store tokens in httpOnly cookies
- Add rate limiting to login endpoint

# Related Issues
- API-40 (blocks this work)
- API-45 (related: rate limiting)
```

### Use Labels Consistently
Define your labels early and stick to them:
- `bug` — Production issue
- `feature` — New capability
- `tech-debt` — Refactoring/improvement
- `spike` — Research task
- `documentation` — Docs only

### Set Realistic Estimates
Use story points or hours that reflect reality:
- `small` = 1-4 hours
- `medium` = 1-3 days
- `large` = 1+ weeks

### Assign Issues
- Assign to yourself when you start work
- Keep only 2-3 active issues per person
- Unassign when delegating

### Use Due Dates for Deadlines
Set due dates for:
- Critical path items
- Customer commitments
- Hard deadlines

### Create Sub-tasks for Complex Issues
If an issue needs 5+ steps, break it into sub-tasks:
- Parent: "Implement payment processing"
- Child 1: "Design API endpoints"
- Child 2: "Implement Stripe integration"
- Child 3: "Add refund handling"
- Child 4: "Write tests"
- Child 5: "Update documentation"

## Next Steps

- **[Statuses](/guide/statuses.md)** — Configure your workflow
- **[Labels](/guide/labels.md)** — Organize with tags
- **[Task Contracts](/guide/task-contracts.md)** — Add execution details for agents
- **[Agent Routing](/guide/agent-routing.md)** — Have agents claim tasks automatically
