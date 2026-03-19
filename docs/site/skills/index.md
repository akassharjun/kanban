# Agent Skills

Skills are reusable instruction sets that teach AI agents how to interact with the Kanban board. They define workflows, conventions, and best practices that agents follow when managing issues, tracking work, and coordinating with other agents.

## What is a Skill?

A skill is a markdown file that gets loaded into an agent's context, giving it the knowledge and rules to work with the Kanban board effectively. Think of it as a playbook — when an agent has the Kanban skill loaded, it knows how to:

- Create and triage issues
- Move work through status transitions
- Leave comments and activity logs
- Coordinate with other agents via the board
- Follow your project's specific conventions

## Why Skills?

Without a skill, an agent interacts with the Kanban board as a generic API. With the skill loaded, the agent understands your **workflow** — it knows that "Backlog" means "needs scoping," that every status change needs a comment, and that work should be tracked before implementation begins.

## Available Skills

| Skill | Purpose | Best For |
|-------|---------|----------|
| [Kanban Board Skill](/skills/kanban-skill) | Full workflow management | Claude Code, Codex, any coding agent |
| [MCP Integration](/skills/mcp-integration) | Direct MCP server usage | Agents with MCP support |

## Quick Start

The fastest way to give an agent Kanban board awareness:

1. Copy the skill markdown from the [Kanban Board Skill](/skills/kanban-skill) page
2. Add it to your agent's instruction file (CLAUDE.md, AGENTS.md, or system prompt)
3. The agent will now track all work through the board automatically

See [Installation Guide](/skills/installation) for detailed setup instructions per platform.
