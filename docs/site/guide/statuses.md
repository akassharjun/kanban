# Statuses & Workflow

Statuses define your project's workflow columns. Each project has its own set of statuses, and each status belongs to a semantic category that drives agent behavior.

## Understanding Status Categories

Every status belongs to one of five categories:

| Category | Meaning | Effect on Agents |
|----------|---------|------------------|
| `unstarted` | Work hasn't begun | Agents search here for available tasks |
| `started` | Work is in progress | Agents continue working; don't search for new tasks here |
| `blocked` | Waiting on dependency | Agents skip blocked tasks |
| `completed` | Work is done | Agents won't touch; triggers validation |
| `discarded` | Work abandoned | Agents ignore; task is closed |

The category is what matters to the orchestration engine. The name (e.g., "In Progress", "In Review") is just a label.

## Default Statuses

When you create a project, these seven statuses are created automatically:

```
Position | Name         | Category    | Color   | Meaning
---------|------|----------|---------|------
0        | Backlog      | unstarted   | #6b7280 | Not yet prioritized
1        | Todo         | unstarted   | #6b7280 | Ready to start
2        | In Progress  | started     | #3b82f6 | Someone is working
3        | In Review    | started     | #8b5cf6 | Waiting for code review
4        | Blocked      | blocked     | #ef4444 | Can't proceed
5        | Done         | completed   | #22c55e | Finished
6        | Discarded    | discarded   | #6b7280 | Won't do
```

## Creating Custom Statuses

Add statuses tailored to your workflow.

### Via CLI

```bash
# Add a "QA Testing" status
kanban cli status create \
  --project 1 \
  --name "QA Testing" \
  --category started \
  --color "#9333ea"

# Add a "Deployment" status
kanban cli status create \
  --project 1 \
  --name "Deployment" \
  --category started \
  --color "#ec4899"

# Add a "Ready for Production" status
kanban cli status create \
  --project 1 \
  --name "Ready for Prod" \
  --category completed \
  --color "#06b6d4"
```

### Via GUI
1. Right-click on your project
2. Click "Manage Statuses"
3. Click "Add Status"
4. Fill in name, category, color
5. Click "Create"

### Example Workflows

#### Simple Kanban (Startups)
```
Todo (unstarted)
  ↓
In Progress (started)
  ↓
Done (completed)
```

#### Software Development (Scrum-like)
```
Backlog (unstarted)
  ↓
Todo (unstarted)
  ↓
In Progress (started)
  ↓
In Review (started) ← Code review happens here
  ↓
QA Testing (started) ← QA validation
  ↓
Done (completed)
```

#### DevOps Release
```
Backlog (unstarted)
  ↓
Todo (unstarted)
  ↓
Development (started)
  ↓
Testing (started)
  ↓
Staging (started)
  ↓
Production Ready (completed)
  ↓
Released (completed)
```

#### Customer Support
```
New (unstarted)
  ↓
In Progress (started)
  ↓
Waiting for Customer (blocked)
  ↓
Ready to Close (started)
  ↓
Closed (completed)
```

## Listing Statuses

### Via CLI

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
# 7  | QA Testing   | started     | #9333ea | 6
# 8  | Discarded    | discarded   | #6b7280 | 7
```

## Updating a Status

### Via CLI

```bash
# Change the name
kanban cli status update 7 --name "QA Validation"

# Change the category
kanban cli status update 7 --category "completed"

# Change the color (hex)
kanban cli status update 7 --color "#00d084"

# Change position (reorder)
kanban cli status update 7 --position 5
```

Position controls left-to-right order on the board. Lower numbers appear first.

### Via GUI
1. Right-click a status
2. Click "Edit"
3. Update fields
4. Click "Save"

## Reordering Statuses

Change the order statuses appear on the board:

### Via CLI

```bash
# Move "In Review" from position 3 to position 2
kanban cli status update 4 --position 2

# Shift "In Progress" to the right
kanban cli status update 3 --position 3
```

### Via GUI
Drag statuses left/right in the status management panel.

## Deleting a Status

### Via CLI

```bash
kanban cli status delete 7
```

### Via GUI
Right-click status → Delete

:::warning
You cannot delete a status that has issues in it. Move all issues to another status first, or cascade delete the entire project.

When you delete a status, issues in it must be moved:
```bash
# Move all issues from deleted status (5) to "Done" (6)
kanban cli issue list --project 1 --status 5 | \
  xargs -I {} kanban cli issue update {} --status 6
```
:::

## How Statuses Affect Agents

When agents are looking for work:

1. **Search in `unstarted` statuses** — Agent queries issues in Todo, Backlog
2. **Skip `started` statuses** — Agents don't grab new work from In Progress or In Review (they're already being worked)
3. **Respect `blocked` statuses** — If a task is blocked, agent skips it
4. **Validate `completed` statuses** — When task reaches Done, validation pipeline runs
5. **Ignore `discarded`** — Discarded tasks are never picked up

Example agent routing:

```
Agent calls: next_task()
  ↓
Query: SELECT * FROM task_contracts WHERE status IN ('unstarted')
       AND task_state = 'queued'
  ↓
Filter: Remove blocked tasks (via issue_relations)
  ↓
Filter: Remove tasks exceeding agent's max_complexity
  ↓
Filter: Remove tasks requiring skills agent doesn't have
  ↓
Sort: By priority (urgent → high → medium → low)
  ↓
Claim: First match (atomically, prevent race conditions)
```

See **[Agent Routing](/guide/agent-routing.md)** for full details.

## Automatic Status Transitions

When agents claim and complete tasks, the system automatically moves issues through statuses based on **task state**:

| Task State | Auto-moves to | Explanation |
|-----------|--------|---------|
| `queued` | (stays) | In an `unstarted` status |
| `claimed` | (stays) | Agent has it, waiting to start |
| `executing` | (stays) | Agent is working |
| `validating` | (stays) | Running success criteria |
| `completed` | First `completed` status | Task is done |
| `blocked` | `blocked` status | Dependency missing |
| `cancelled` | `discarded` status | Agent gave up |

When you call `task_update` to change a task's `task_state`:

```bash
kanban cli task update API-42 \
  --task-state executing \
  --confidence 0.85
```

The system automatically:
1. Finds the status category for `executing` (which is `started`)
2. Moves the issue to that status (e.g., "In Progress")
3. Logs the change in activity log

This keeps issue status and task state in sync.

## Status Configuration Best Practices

### 1. Keep It Simple
Start with 3-5 statuses. You can always add more later.

Good:
```
Backlog → Todo → In Progress → Done
```

Complex (but fine if you need it):
```
Backlog → Todo → In Progress → In Review → QA → Staging → Done
```

### 2. Use Consistent Categories
Group statuses by category to match your process:

```
Backlog (unstarted)
Todo (unstarted)
  ↓
In Progress (started)
In Review (started)
  ↓
Testing (started) [QA before done]
  ↓
Done (completed)
```

### 3. Use Colors Consistently
Pick a color scheme and stick to it:
- **Gray** (#6b7280) — Inactive (Backlog, Discarded)
- **Blue** (#3b82f6) — Active work (In Progress)
- **Purple** (#8b5cf6) — Waiting (In Review, QA)
- **Red** (#ef4444) — Blocked
- **Green** (#22c55e) — Done

### 4. Name for Clarity
Use clear, active names:

Good:
- "In Progress"
- "In Review"
- "Ready for Deployment"

Bad:
- "S3" (unclear)
- "INPROG" (abbreviations)
- "Bob's Status" (person-specific)

### 5. Update Category, Not Name
When you need workflow changes, update the **category**, not the name.

Example: You want code review to happen after In Progress but before Done.
- Create new status: "Code Review" (started)
- Insert it between "In Progress" and "Done"
- Don't rename existing statuses

This keeps historical records clean.

## Checking Statuses Before Moves

Always verify issues can be moved:

```bash
# Get an issue
kanban cli issue get API-42
# status_id: 2 (Todo)

# List valid destination statuses
kanban cli status list --project 1
# Can move to: 1 (Backlog), 3 (In Progress), 4 (In Review), 5 (Blocked), 6 (Done), 7 (Discarded)

# Move it
kanban cli issue update API-42 --status 3

# Verify
kanban cli issue get API-42
# status_id: 3 (In Progress)
```

## Multi-Project Workflows

Different projects can have different workflows. Example:

```
Backend API (API project):
  Backlog → Todo → In Progress → In Review → Done

Mobile App (APP project):
  Backlog → Design → Development → QA → App Store → Done

DevOps (OPS project):
  Todo → In Progress → Deployed → Monitoring → Done
```

Each project's agents respect that project's status flow. When routing a task in API project to an agent, it uses API project's statuses and categories.

## Next Steps

- **[Issues](/guide/issues.md)** — Move issues between statuses
- **[Agent Routing](/guide/agent-routing.md)** — How status categories guide agent assignment
- **[Task Contracts](/guide/task-contracts.md)** — Task state vs. issue status
