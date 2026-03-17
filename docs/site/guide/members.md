# Members

Members represent people and agents in your workspace. Members are **workspace-scoped** (shared across all projects) but can be assigned to issues in any project.

## Creating Members

### Via CLI

```bash
# Create a member
kanban cli member add "alice@example.com" \
  --display-name "Alice Chen" \
  --avatar-color "#3b82f6"

# Create another member
kanban cli member add "bob@example.com" \
  --display-name "Bob Smith" \
  --avatar-color "#8b5cf6"
```

### Via GUI
1. Click "Team" or "Members" in settings
2. Click "Add Member"
3. Enter email and display name
4. Pick avatar color
5. Click "Create"

## Listing Members

### Via CLI

```bash
kanban cli member list

# Output:
# ID | Name                    | Email               | Avatar Color
# 1  | Alice Chen              | alice@example.com   | #3b82f6
# 2  | Bob Smith               | bob@example.com     | #8b5cf6
# 3  | [claude] Claude Agent   | (none)              | #f97316
# 4  | [codex] Codex Agent     | (none)              | #22c55e
```

## Getting Member Details

### Via CLI

```bash
kanban cli member get 1

# Output:
# {
#   "id": 1,
#   "name": "alice@example.com",
#   "display_name": "Alice Chen",
#   "email": "alice@example.com",
#   "avatar_color": "#3b82f6",
#   "created_at": "2025-03-15T10:00:00Z"
# }
```

## Updating Members

### Via CLI

```bash
# Update display name
kanban cli member update 1 --display-name "Alice C."

# Update avatar color
kanban cli member update 1 --avatar-color "#ec4899"
```

### Via GUI
Click a member → Edit → Save

## Assigning Members to Issues

Members can be assigned to any issue.

### Via CLI

```bash
# Assign to issue
kanban cli issue update API-42 --assignee 1

# Check assignment
kanban cli issue get API-42
# assignee_id: 1
# assignee_name: "Alice Chen"
```

### Via GUI
1. Open an issue
2. Click "Assignee" field
3. Select a member from dropdown
4. Changes save automatically

## Member Types

### Human Team Members
Created manually via the Add Member interface:

```bash
kanban cli member add "engineer@example.com" \
  --display-name "Full Name"
```

These represent people on your team who can:
- Be assigned to issues
- Receive notifications
- Leave comments
- Collaborate

### Agents (Auto-created Members)
When an agent registers, it auto-creates a member:

```bash
kanban cli agent register \
  --name "claude-code" \
  --agent-type claude \
  --skills "coding,testing" \
  --max-concurrent 2
```

Creates a member: `[claude] claude-code` with:
- Avatar color based on agent type:
  - `claude` → Orange (#f97316)
  - `codex` → Green (#22c55e)
  - `gemini` → Blue (#3b82f6)
  - Custom → Purple (#8b5cf6)

Agent members:
- Can be assigned to task contracts
- Log execution activities
- Have stats (tasks completed, success rate, etc.)
- Have last heartbeat and last activity timestamps

## Agent Type Badges

Agents are visually distinguished by their type in the UI:

| Agent Type | Color | Meaning |
|-----------|-------|---------|
| `claude` | Orange | Claude AI (Anthropic) |
| `codex` | Green | OpenAI Codex |
| `gemini` | Blue | Google Gemini |
| Custom | Purple | Custom/other agent |

Example member list with agents:

```
1. Alice Chen              (human, blue dot)
2. [claude] code-reviewer  (agent, orange dot)
3. [codex] feature-gen     (agent, green dot)
4. [gemini] validator      (agent, blue dot)
5. Bob Smith               (human, purple dot)
```

## Working with Agent Members

### Register an Agent

```bash
kanban cli agent register \
  --name "code-reviewer" \
  --agent-type claude \
  --skills "code-review,testing,documentation" \
  --task-types "review,decomposition" \
  --max-concurrent 3 \
  --max-complexity large \
  --worktree-path "/tmp/code-reviewer"
```

This creates:
1. Agent record in `agents` table
2. Member record with name `[claude] code-reviewer`
3. Agent stats record

### View Agent Stats

```bash
kanban cli agent stats --agent-id "550e8400-e29b-41d4-a716-446655440000"

# Output:
# {
#   "agent_id": "550e8400-e29b-41d4-a716-446655440000",
#   "tasks_completed": 15,
#   "tasks_failed": 1,
#   "total_confidence": 13.8,
#   "avg_confidence": 0.92,
#   "total_completion_time_seconds": 1800,
#   "avg_completion_time_minutes": 2.0,
#   "skills_breakdown": {
#     "code-review": 10,
#     "testing": 3,
#     "documentation": 2
#   }
# }
```

### List Active Agents

```bash
kanban cli agent list

# Output:
# ID                                   | Name             | Type   | Status | Active Tasks | Skills
# 550e8400-e29b-41d4-a716-446655440000 | code-reviewer    | claude | idle   | 0            | code-review, testing, docs
# 660e8400-e29b-41d4-a716-446655440001 | feature-impl     | codex  | busy   | 2            | coding, testing
```

### Deregister an Agent

```bash
kanban cli agent deregister --agent-id "550e8400-e29b-41d4-a716-446655440000"
```

This:
1. Marks agent as offline
2. Unassigns all active tasks
3. Moves tasks back to `queued`
4. Keeps member record and history

## Member Activity

View what a member is assigned to:

### Via CLI

```bash
# List issues assigned to a member
kanban cli member issues --member-id 1

# Output:
# ID | Identifier | Title                      | Status      | Priority
# 42 | API-42     | Implement password reset   | In Progress | medium
# 51 | API-51     | Code review PR #123        | In Review   | high
```

### Via GUI
Click member profile → see assigned issues

## Removing Members

### Soft Deletion

Members can't be hard-deleted (for audit purposes). Instead, deactivate them:

```bash
# For human members: remove from active roster
# They remain in history for audit trails

# For agents: deregister (see above)
kanban cli agent deregister --agent-id "..."
```

## Member Permissions

Currently, Kanban has simple membership:
- All members can see all projects and issues
- All agents have equal access to task contracts

Future enhancements may add:
- Role-based access control (RBAC)
- Project-specific permissions
- Agent skill-based filtering
- Approval workflows

## Best Practices

### For Team Members

1. **Use real names** in display names for clarity
2. **Use consistent emails** to track who did what
3. **Assign issues when starting work** — Updates are tracked
4. **Unassign when delegating** — Keeps active assignments current

### For Agents

1. **Give agents descriptive names**
   - Good: `code-reviewer`, `feature-impl`, `test-validator`
   - Bad: `agent1`, `bot`, `automation`

2. **Set appropriate skills**
   - Only list skills agent truly has
   - See **[Agent Routing](/guide/agent-routing.md)** for skill matching
   - Example: `["rust", "async", "testing"]`

3. **Set realistic max_concurrent**
   - 1-2: Single-task agents (careful, conservative)
   - 2-3: Normal agents (good default)
   - 4+: High-throughput agents (use carefully)

4. **Set appropriate max_complexity**
   - `small`: Can't handle anything complex
   - `medium`: Default; handles most tasks
   - `large`: Can handle everything

5. **Use consistent agent types**
   - Helps team understand agent capabilities
   - Enables filtering by agent type

### Member Organization

Keep member lists clean:

```
Recommended Structure:
├── Humans
│   ├── Alice Chen (lead)
│   ├── Bob Smith (backend)
│   └── Carol Davis (frontend)
├── Agents
│   ├── [claude] code-reviewer
│   ├── [claude] feature-impl
│   ├── [codex] code-gen
│   └── [gemini] validator
```

Document your agents:
- What each does
- Which projects they're used in
- Their capabilities and limits

## Next Steps

- **[Issues](/guide/issues.md)** — Assign members to work
- **[Task Contracts](/guide/task-contracts.md)** — Assign agents to executable tasks
- **[Agent Routing](/guide/agent-routing.md)** — How agents are matched to work
