# Installation Guide

Step-by-step setup for adding the Kanban skill to your AI agent. Includes complete sample configuration files.

## Claude Code (CLAUDE.md)

Claude Code reads project instructions from `CLAUDE.md` at the repository root. Add the Kanban skill there.

### Sample CLAUDE.md

Create or update `CLAUDE.md` in your project root:

````markdown
# My Project — Agent Instructions

## Kanban Board Integration

All work MUST be tracked on the Kanban board (project ID: 2, prefix: KAN).

### Workflow

When the user requests a feature, bug fix, or improvement:

1. **Create an issue on the board first** using the CLI before doing any work:
   ```bash
   kanban-cli issue create --project 2 \
     --title "Short description" \
     --status 9 \
     --priority medium \
     --assignee 3 \
     --description "Detailed description"
   ```

2. **Triage into the correct status:**
   - **Backlog** (status_id: 8): Needs scoping or breakdown
   - **Todo** (status_id: 9): Scoped and ready to work
   - **In Progress** (status_id: 10): Actively being worked on
   - **In Review** (status_id: 11): Implementation complete, QA pending
   - **Done** (status_id: 13): Verified complete
   - **Blocked** (status_id: 12): Waiting on dependency

3. **On every status change, leave a comment** explaining why it moved.

4. **When starting work:** Move to In Progress, log "Starting implementation"

5. **When complete:** Move to In Review, log "Implementation complete"

### CLI Reference

```bash
# Create issue
kanban-cli issue create --project 2 --title "..." --status 9 --priority high --assignee 3

# Update status
kanban-cli issue update KAN-42 --status 10

# List issues
kanban-cli issue list --project 2

# Search
kanban-cli issue search --project 2 "query"
```

### Assign all issues to Claude (member_id: 3).

## Commands

```bash
npm run dev        # Frontend dev server
npm run test:run   # Run tests
npm run build      # Production build
```
````

### MCP Server Setup (Optional)

If your agent supports MCP, add the Kanban MCP server to your Claude Code settings:

```json
{
  "mcpServers": {
    "kanban": {
      "command": "src-tauri/target/release/kanban-mcp",
      "args": []
    }
  }
}
```

This gives the agent direct access to all board operations via MCP tools instead of CLI commands.

## Codex (AGENTS.md)

OpenAI Codex reads agent instructions from `AGENTS.md`.

### Sample AGENTS.md

````markdown
# Agent Instructions

## Task Tracking

Before starting any work, create an issue on the Kanban board:

```bash
kanban-cli issue create --project 2 \
  --title "Description of work" \
  --status 9 \
  --priority medium \
  --assignee 3
```

## Workflow

1. Create issue → move to In Progress when starting
2. Implement the feature or fix
3. Run tests: `npm run test:run`
4. Move issue to In Review
5. If tests pass, move to Done

## Status IDs

| Status | ID | When to use |
|--------|----|-------------|
| Backlog | 8 | Needs scoping |
| Todo | 9 | Ready to work |
| In Progress | 10 | Working on it |
| In Review | 11 | Done, needs QA |
| Done | 13 | Verified |
| Blocked | 12 | Stuck |

## CLI Quick Reference

```bash
kanban-cli issue list --project 2              # See all issues
kanban-cli issue update KAN-42 --status 10     # Move to In Progress
kanban-cli issue search --project 2 "bug"      # Search issues
```
````

## Generic Agent (System Prompt)

For agents that accept system prompts (custom setups, LangChain, etc.):

```
You are an AI coding agent. All work must be tracked on the Kanban board.

Before writing any code:
1. Create an issue: kanban-cli issue create --project 2 --title "..." --status 9 --priority medium
2. Move to In Progress: kanban-cli issue update <ID> --status 10

After completing work:
3. Move to In Review: kanban-cli issue update <ID> --status 11
4. If verified, move to Done: kanban-cli issue update <ID> --status 13

Always create the issue BEFORE starting implementation. Never skip this step.
```

## Ralph TUI / Ralph Loop

If using Ralph TUI or the Ralph Loop technique for autonomous task execution:

```bash
# Ralph Loop that works through Kanban issues
/ralph-loop "Pick the next Todo issue from the Kanban board (kanban-cli issue list --project 2 --status 9), move it to In Progress, implement it, run tests, and move to Done. Repeat until no Todo issues remain." \
  --completion-promise "ALL_ISSUES_DONE" \
  --max-iterations 20
```

The key integration: Ralph Loop reads issues from the board, works on them, and updates their status — creating a fully autonomous development pipeline.

## Verifying Setup

After adding the skill, verify the agent can access the board:

```bash
# Test CLI access
kanban-cli issue list --project 2

# Test MCP access (if configured)
echo '{"jsonrpc":"2.0","id":1,"method":"list_projects","params":{}}' | kanban-mcp
```

If the CLI returns issues, the agent is ready to use the board.
