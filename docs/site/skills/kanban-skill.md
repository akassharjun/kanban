# Kanban Board Skill

The core skill that teaches agents how to manage work through the Kanban board. This is the primary skill most agents should load.

## What It Does

When an agent has this skill loaded, it will:

- **Create issues before starting work** — every task gets tracked
- **Triage correctly** — Backlog for unscoped work, Todo for ready items, In Progress when working
- **Log status changes** — comments on every transition explaining why
- **Follow the review workflow** — move to In Review when done, spawn QA agents
- **Coordinate with the board** — use the CLI or MCP to read/write issues

## The Skill (Copy This)

Copy the markdown block below into your agent's instruction file. For Claude Code, add it to `CLAUDE.md` at the project root. For Codex, add it to `AGENTS.md`. For other agents, include it in the system prompt.

:::tip
The skill references specific project IDs and status IDs. Update these to match your board configuration. Run `kanban-cli issue list --project <id>` to see your project's statuses.
:::

### Embeddable Skill Markdown

````markdown
## Kanban Board Integration

All work MUST be tracked on the Kanban board before implementation begins.

### Setup

The CLI binary is at: `src-tauri/target/release/kanban-cli`

```bash
# Verify the CLI works
kanban-cli issue list --project <PROJECT_ID>
```

### Workflow

When the user requests a feature, bug fix, or improvement:

1. **Create an issue first** before writing any code:
   ```bash
   kanban-cli issue create --project <PROJECT_ID> \
     --title "Short description of the work" \
     --status <STATUS_ID> \
     --priority <urgent|high|medium|low> \
     --assignee <MEMBER_ID> \
     --description "Detailed description with acceptance criteria"
   ```

2. **Triage into the correct status:**
   - **Backlog**: Needs scoping, design, or breakdown. Vague requests go here.
   - **Todo**: Scoped with clear outcome. Ready to work.
   - **In Progress**: Actively being worked on.
   - **In Review**: Implementation complete, needs QA.
   - **Done**: QA passed, verified complete.
   - **Blocked**: Cannot proceed due to dependency.

3. **On every status change, leave a comment** explaining why:
   ```bash
   kanban-cli issue update <IDENTIFIER> --status <NEW_STATUS_ID>
   # Log a comment via activity_log:
   sqlite3 ~/.kanban/data.db "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES (<issue_id>, 'comment', NULL, '<message>', datetime('now'));"
   ```

4. **When starting work:**
   - Move issue to In Progress
   - Log: "Starting implementation"

5. **When work is complete:**
   - Move to In Review
   - Log: "Implementation complete, spawning QA review"
   - If QA passes → move to Done with "QA passed"
   - If QA fails → move back to In Progress with description of what needs fixing

### CLI Quick Reference

```bash
# Create issue
kanban-cli issue create --project <ID> --title "..." --status <ID> --priority high --assignee <ID>

# Update status
kanban-cli issue update <IDENTIFIER> --status <ID>

# List issues
kanban-cli issue list --project <ID>
kanban-cli issue list --project <ID> --status <ID>  # Filter by status

# Search
kanban-cli issue search --project <ID> "query"
```

### Status IDs

| Status | Category | Use |
|--------|----------|-----|
| Backlog | unstarted | Needs scoping |
| Todo | unstarted | Ready to work |
| In Progress | started | Actively working |
| In Review | started | QA pending |
| Blocked | blocked | Waiting on dependency |
| Done | completed | Verified complete |
| Discarded | discarded | Abandoned |

> **Note:** Run `kanban-cli issue list --project <ID>` to see the actual status IDs for your project.
````

## Configuration

The skill above uses placeholder IDs. Here's how to find your actual values:

### Find Your Project ID

```bash
kanban-cli issue list --project 1  # Try project 1
kanban-cli issue list --project 2  # Try project 2
```

### Find Status IDs

Status IDs are visible in the board's Settings page under the "Statuses" tab, or query the database:

```bash
sqlite3 ~/.kanban/data.db "SELECT id, name, category FROM statuses WHERE project_id = <YOUR_PROJECT_ID>;"
```

### Find Member IDs

```bash
sqlite3 ~/.kanban/data.db "SELECT id, name, display_name FROM members;"
```

## Example: Claude Code Setup

Add to your project's `CLAUDE.md`:

```markdown
# My Project

## Kanban Board Integration

All work MUST be tracked on the Kanban board (project ID: 2, prefix: KAN).

[Paste the skill markdown from above, replacing placeholder IDs]

## Status IDs (Project: KAN)

| Status | ID |
|--------|----|
| Backlog | 8 |
| Todo | 9 |
| In Progress | 10 |
| In Review | 11 |
| Blocked | 12 |
| Done | 13 |

Assign all issues to Claude (member_id: 3).
```

## Example: Codex Setup

Add to your project's `AGENTS.md`:

```markdown
## Task Tracking

Before starting any work, create an issue on the Kanban board:

[Paste the skill markdown from above]
```

## Example: Generic Agent (System Prompt)

For agents that accept system prompts:

```
You are an AI coding agent. All work must be tracked on the Kanban board.

[Paste the skill markdown from above]

Always create an issue before writing code. Move it through the workflow as you progress.
```
