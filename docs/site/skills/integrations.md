# Plugin Integrations

The Kanban board works alongside popular AI agent plugins and tools. Here's how to combine them for powerful workflows.

## Superpowers

[Superpowers](https://github.com/obra/superpowers) is a structured development workflow plugin for Claude Code. It provides skills for brainstorming, TDD, planning, debugging, and code review.

### How They Work Together

| Superpowers Skill | Kanban Integration |
|---|---|
| **Brainstorming** | Create a Backlog issue first, brainstorm designs, move to Todo when scoped |
| **Writing Plans** | Attach the plan to the issue description, create sub-issues for each task |
| **TDD** | Move issue to In Progress, follow RED-GREEN-REFACTOR, update issue on completion |
| **Systematic Debugging** | Create a bug issue, use the 4-phase debug cycle, log findings as comments |
| **Subagent-Driven Development** | Each subagent task maps to a sub-issue, review status tracked on the board |
| **Code Review** | Move to In Review, dispatch review agent, record findings as comments |

### Sample CLAUDE.md with Both

```markdown
# My Project

## Kanban Board
All work tracked on the board (project: 2, prefix: KAN).
[Include Kanban skill from the installation guide]

## Development Workflow
- Use /brainstorm before implementing features (creates Backlog issue)
- Use /tdd for all implementations (issue moves to In Progress)
- Use /debug for bug fixes (creates bug issue with priority)
- On completion, move issue to In Review and dispatch code reviewer
```

The key insight: **Superpowers defines HOW to work, Kanban tracks WHAT work is being done.** They're complementary — Superpowers provides the methodology, Kanban provides the coordination layer.

## Ralph Loop / Ralph TUI

[Ralph Loop](https://ghuntley.com/ralph/) is an autonomous iteration technique. [Ralph TUI](https://ralph-tui.com/) is a multi-agent terminal orchestrator.

### Ralph Loop + Kanban

Ralph Loop works through tasks autonomously until a completion promise is met. Combined with Kanban, it becomes a task execution engine:

```bash
# Process all Todo issues autonomously
/ralph-loop "
1. Run: kanban-cli issue list --project 2 --status 9
2. Pick the highest priority issue
3. Move it to In Progress: kanban-cli issue update <ID> --status 10
4. Implement the change described in the issue title
5. Run tests: npm run test:run
6. If tests pass, move to Done: kanban-cli issue update <ID> --status 13
7. If tests fail, fix and retry
8. Repeat from step 1
" --completion-promise "NO_MORE_TODO_ISSUES" --max-iterations 30
```

### Ralph TUI + Kanban

Ralph TUI orchestrates multiple agents simultaneously. Your Kanban board becomes the shared work queue:

- **Agent 1** (Claude Code): Claims KAN-15, works in worktree `kan-15-auth`
- **Agent 2** (Codex): Claims KAN-16, works in worktree `kan-16-api`
- **Agent 3** (Review Bot): Monitors In Review column, validates completed work

Ralph TUI reads from the board, dispatches to agents, and the board reflects real-time status. The Agent Dashboard in Kanban shows all active worktrees and which agent is working on what.

## GitHub Plugin

The official GitHub MCP plugin can sync with the Kanban board:

### Bi-directional Workflow

```
GitHub Issue Created → Kanban issue auto-created (via webhook/automation)
Kanban issue Done → GitHub PR auto-linked
GitHub PR Merged → Kanban issue closed
```

### Setup

The Kanban board already has GitHub integration built in (Settings → GitHub tab):

1. Set repo owner and name
2. Add GitHub PAT (or set `GITHUB_PAT` env var)
3. Enable auto-link PRs and auto-transition on merge

When an agent creates a branch via the board's "Create Branch" button, it automatically creates a git link. When the PR is merged, the issue transitions to Done.

## Playwright Plugin

The [Playwright MCP plugin](https://github.com/anthropics/claude-code-playwright) gives agents browser automation capabilities. Combined with Kanban, agents can:

- **Verify UI changes** — after implementing a frontend issue, the agent can open a browser, navigate the app, and screenshot the result before moving to In Review
- **Run acceptance tests** — when an issue has acceptance criteria like "button should be visible," the agent can use Playwright to verify it programmatically
- **Visual QA** — take before/after screenshots and attach them as comments on the issue

### Example: Agent validates a UI fix

```
1. Agent picks up KAN-42 "Fix login button alignment"
2. Implements the CSS fix
3. Uses Playwright to navigate to /login, take a screenshot
4. Attaches screenshot to the issue as a comment
5. Moves to In Review with "Fix verified visually"
```

This turns every agent into a QA tester — they don't just write code, they verify it works.

## Code Review Plugin

The code-review plugin can be triggered from the Kanban board:

1. Agent completes work, moves issue to In Review
2. Code review agent is dispatched (or `/review-pr` is invoked)
3. Review findings are logged as comments on the issue
4. If approved → Done; if changes needed → back to In Progress

## Building Your Pipeline

Combine these tools into a full autonomous pipeline:

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ Kanban Board │────▶│ Ralph Loop   │────▶│ Agent Work  │
│ (Todo queue) │     │ (dispatcher) │     │ (implement) │
└──────┬──────┘     └──────────────┘     └──────┬──────┘
       │                                         │
       │     ┌──────────────┐     ┌─────────────┘
       │     │ Code Review  │◀────│
       │     │ (validate)   │     │
       │     └──────┬───────┘     │
       │            │             │
       ▼            ▼             │
  ┌─────────┐  ┌─────────┐  ┌────┴────┐
  │ Backlog │  │  Done   │  │ Failed  │
  │ (next)  │  │ (ship)  │  │ (retry) │
  └─────────┘  └─────────┘  └─────────┘
```

The Kanban board is the central nervous system. Everything reads from it and writes back to it.
