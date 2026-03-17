# Core Concepts

A guide to the fundamental building blocks of Kanban.

## Projects

A **project** is a container for all related work. Each project has:
- **Name** — Human-readable title
- **Prefix** — 2-4 letter code (e.g., `KAN`, `PROJ`, `API`)
- **Issue Counter** — Auto-increments per project
- **Status** — `active`, `paused`, `completed`, or `archived`
- **Path** (Optional) — Worktree path for agent-based projects

Issue identifiers are generated as `{prefix}-{counter}`. For example, a project with prefix `KAN` creates issues: `KAN-1`, `KAN-2`, `KAN-3`.

```bash
# Create a project
kanban cli project create "Engineering" \
  --prefix "ENG" \
  --description "Core platform work"

# List all projects
kanban cli project list

# Get a specific project
kanban cli project get 1
```

---

## Issues

An **issue** is a unit of work. Each issue has:
- **Identifier** — Auto-generated (e.g., `KAN-42`)
- **Title** — Short summary
- **Description** — Detailed context (Markdown)
- **Status** — Which column (backlog, in progress, etc.)
- **Priority** — `none`, `low`, `medium`, `high`, `urgent`
- **Assignee** — Person or agent responsible
- **Parent ID** — (Optional) For sub-tasks
- **Position** — Order within a status (for drag-and-drop)
- **Estimate** — Time or story points
- **Due Date** — When it should be done
- **Labels** — Tags for grouping

```bash
# Create an issue
kanban cli issue create \
  --project 1 \
  --title "Implement user authentication" \
  --status 2 \
  --priority high \
  --description "Add login/logout with OAuth"

# Update an issue
kanban cli issue update KAN-1 \
  --title "Updated title" \
  --priority urgent

# Move to a different status
kanban cli issue update KAN-1 --status 4

# Delete an issue
kanban cli issue delete KAN-1
```

### Activity Log
Every change to an issue is logged:
- Who changed it
- When
- Old value → new value

This gives you full audit trails without needing to dig through version history.

---

## Statuses & Workflow

A **status** is a column on your Kanban board. Each project defines its own statuses with:
- **Name** — Display name (e.g., "In Progress")
- **Category** — Semantic meaning for agents
- **Color** — Visual indication
- **Icon** — (Optional) For UI
- **Position** — Order on the board

### Status Categories

Categories drive agent behavior:

| Category | Meaning | Use |
|----------|---------|-----|
| `unstarted` | Work hasn't begun | Backlog, Todo |
| `started` | Work is in progress | In Progress, In Review |
| `blocked` | Waiting on dependency | Blocked |
| `completed` | Work is done | Done |
| `discarded` | Work won't happen | Cancelled, Discarded |

When an agent completes a task, its status automatically moves to the first `completed` status in the project.

### Default Statuses
When you create a project, these statuses are created automatically:

```
Backlog (unstarted, gray)
Todo (unstarted, gray)
In Progress (started, blue)
In Review (started, purple)
Blocked (blocked, red)
Done (completed, green)
Discarded (discarded, gray)
```

### Custom Statuses
Add your own:

```bash
# Create a custom status
kanban cli status create \
  --project 1 \
  --name "QA Testing" \
  --category "started" \
  --color "#9333ea"
```

---

## Labels

A **label** is a tag for grouping issues. Labels are **project-scoped** and have a color.

```bash
# Create labels
kanban cli label create \
  --project 1 \
  --name "bug" \
  --color "#ef4444"

kanban cli label create \
  --project 1 \
  --name "feature" \
  --color "#3b82f6"

# List labels in a project
kanban cli label list --project 1

# Attach a label to an issue when creating
kanban cli issue create \
  --project 1 \
  --title "Login crash" \
  --status 2 \
  --labels "bug,urgent"
```

---

## Members

A **member** represents a person or agent in your workspace. Members are **workspace-scoped** (shared across all projects).

Each member has:
- **Name** — Unique identifier
- **Display Name** — Friendly name
- **Email** — (Optional)
- **Avatar Color** — For UI display

When an agent registers, it automatically creates a member:

```bash
kanban cli agent register \
  --name "Claude Code" \
  --agent-type claude \
  --skills "coding,testing"
```

This creates a member named `[claude] Claude Code` with an orange avatar.

You can also create members manually:

```bash
kanban cli member add "alice@example.com" \
  --display-name "Alice" \
  --avatar-color "#3b82f6"
```

Then assign issues to them:

```bash
kanban cli issue update KAN-1 --assignee 1
```

---

## Task Contracts

A **task contract** is an extended issue with execution details. Instead of just a title and description, you define:
- **Type** — `implementation`, `review`, or `decomposition`
- **Objective** — What the agent should accomplish
- **Context** — JSON with files, related tasks, prior attempts
- **Constraints** — Things the agent must follow
- **Success Criteria** — Shell commands that prove completion
- **Required Skills** — Skills the agent must have
- **Estimated Complexity** — `small`, `medium`, or `large`
- **Timeout** — Minutes before task auto-fails
- **Task State** — `queued` → `claimed` → `executing` → `validating` → `completed`

Example:

```bash
kanban cli task create \
  --project 1 \
  --title "Add rate limiting" \
  --objective "Implement token bucket rate limiter in API" \
  --status 2 \
  --type implementation \
  --skills "go,apis,databases" \
  --complexity medium \
  --success-criteria '[
    {
      "check": "Tests pass",
      "command": "go test ./... -v",
      "expect": "exit_code == 0"
    },
    {
      "check": "No lint errors",
      "command": "golangci-lint run",
      "expect": "exit_code == 0"
    }
  ]' \
  --constraints "Must not break existing API" \
  --timeout 60
```

See **[Task Contracts](/guide/task-contracts.md)** for full details.

---

## Agents

An **agent** is a registered AI system that can claim and execute task contracts. Each agent has:
- **ID** — UUID
- **Name** — Friendly name (auto-generated if not provided)
- **Agent Type** — `claude`, `codex`, `gemini`, or custom
- **Skills** — List of capabilities (e.g., `["coding", "testing", "documentation"]`)
- **Task Types** — Types it handles (e.g., `["implementation", "review"]`)
- **Max Concurrent** — How many tasks it can work on simultaneously
- **Max Complexity** — Highest complexity it will take (`small`, `medium`, `large`)
- **Status** — `idle`, `busy`, or `offline`
- **Last Heartbeat** — When it last reported in
- **Last Activity** — When it last made progress on a task
- **Worktree Path** — (Optional) Where it operates

When an agent registers:

```bash
kanban cli agent register \
  --name "code-reviewer" \
  --agent-type claude \
  --skills "code-review,architecture,documentation" \
  --task-types "review,decomposition" \
  --max-concurrent 3 \
  --max-complexity large \
  --worktree-path "/tmp/code-reviewer-work"
```

The system:
1. Creates an agent record
2. Auto-creates a member (e.g., `[claude] code-reviewer`)
3. Initializes agent stats

The agent then calls `next_task` to get work:

```bash
kanban cli agent next-task \
  --agent-id "550e8400-e29b-41d4-a716-446655440000"
```

See **[Agent Routing](/guide/agent-routing.md)** for the matching algorithm.

---

## Execution Logs

Every action an agent takes is logged as an **execution log entry**. Types include:
- `claim` — Agent claimed the task
- `start` — Agent started working
- `reasoning` — Agent's thought process
- `file_read` — Agent read a file
- `file_edit` — Agent modified a file
- `command` — Agent ran a command
- `discovery` — Agent found something (e.g., a bug)
- `error` — Something went wrong
- `result` — Agent's output
- `complete` — Task finished
- `timeout` — Task exceeded time limit

Each entry has:
- **Issue ID** — Which task
- **Agent ID** — Who did it
- **Attempt Number** — Which try (for retries)
- **Entry Type** — From the list above
- **Message** — Human-readable text
- **Metadata** — Structured data (JSON)
- **Timestamp** — When it happened

```bash
# View execution log for a task
kanban cli task replay KAN-42

# Shows:
# 1. [2025-03-15 10:00:00Z] claim - Agent claimed task
# 2. [2025-03-15 10:00:15Z] start - Agent started work
# 3. [2025-03-15 10:02:30Z] file_read - Read main.go (424 bytes)
# 4. [2025-03-15 10:05:00Z] file_edit - Modified main.go
# 5. [2025-03-15 10:10:00Z] command - Ran tests (exit 0)
# 6. [2025-03-15 10:10:30Z] complete - Task finished (confidence: 0.92)
```

See **[Execution & Replay](/guide/execution-replay.md)** for full details.

---

## Relations

Issues can be related in several ways:

| Relation | Meaning |
|----------|---------|
| `related` | Loosely connected |
| `blocks` | This task blocks another |
| `blocked_by` | This task is blocked by another |
| `duplicate` | This is a duplicate of another |

```bash
# Mark KAN-5 as blocked by KAN-3
kanban cli issue block KAN-5 --by KAN-3

# Mark KAN-7 as related to KAN-6
kanban cli issue relate KAN-7 --to KAN-6
```

When routing tasks to agents, the system respects blocking relations:
- Blocked tasks won't be assigned until their blockers complete
- This enables task dependencies and critical path management

---

## Comments

Issues can have comments. Comments are created automatically by agents or manually by users.

```bash
kanban cli comment add KAN-1 "Ready for review"
```

Comments are useful for:
- Communication between team members
- Agent reasoning logs (auto-commented)
- Status updates
- Decision tracking

---

## Summary

| Concept | Scope | Purpose |
|---------|-------|---------|
| **Project** | Workspace | Container for related work |
| **Issue** | Project | Unit of work with status and assignee |
| **Status** | Project | Workflow state (column on board) |
| **Label** | Project | Tag for grouping |
| **Member** | Workspace | Person or agent |
| **Task Contract** | Project | Extended issue with execution details |
| **Agent** | Workspace | AI system that claims and executes tasks |
| **Execution Log** | Task | Record of actions taken |
| **Comment** | Issue | Discussion or reasoning |
| **Relation** | Workspace | Connection between issues (blocking, etc.) |

Next, dive into how to use each:
- **[Issues](/guide/issues.md)** — Full lifecycle and operations
- **[Task Contracts](/guide/task-contracts.md)** — Executable work definitions
- **[Agent Routing](/guide/agent-routing.md)** — How agents find work
