# What is Kanban?

Kanban is a **single-binary desktop application** for managing projects and coordinating AI agent work. It's built on three core principles:

1. **One Database, Three Interfaces** — Your data lives in SQLite. Access it via the desktop GUI (Tauri + React), command-line CLI, or JSON-RPC MCP server.
2. **Agent-First Architecture** — Designed from the ground up to orchestrate AI agents. Register agents with skills, route tasks automatically, validate results.
3. **Task Contracts** — Issues can be extended with executable task contracts: success criteria (shell commands), constraints, complexity ratings, and full execution logs.

## Who Is It For?

- **AI engineers** building multi-agent systems
- **Teams coordinating** bot and human workflows
- **Project managers** wanting full audit trails of what happened
- **Anyone** who wants a portable, single-file Kanban board

## Architecture

```
┌─────────────────────────────────────────────────────┐
│           Kanban Binary (One Application)            │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌────────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │   GUI App  │  │   CLI    │  │   MCP Server     │ │
│  │ (Tauri +   │  │ (clap)   │  │ (JSON-RPC 2.0)   │ │
│  │  React)    │  │          │  │                  │ │
│  └────────────┘  └──────────┘  └──────────────────┘ │
│        ↓              ↓                  ↓            │
├─────────────────────────────────────────────────────┤
│     Tauri Command Bridge (Shared Backend)           │
├─────────────────────────────────────────────────────┤
│                                                       │
│   Orchestration Engine:                             │
│   ├─ Task Routing (agent matching)                  │
│   ├─ Validation Pipeline (shell commands)           │
│   ├─ Dependency Resolution (blocks/blocked_by)      │
│   ├─ Decomposition (large → small tasks)            │
│   └─ Activity Logging (every action tracked)        │
│                                                       │
├─────────────────────────────────────────────────────┤
│              SQLite Database (~/.kanban/data.db)    │
│              WAL mode (fast, concurrent reads)      │
├─────────────────────────────────────────────────────┤
│  Projects | Issues | Statuses | Labels | Members   │
│  Task Contracts | Agents | Execution Logs          │
│  Activity Log | Undo/Redo | Comments               │
└─────────────────────────────────────────────────────┘
```

## The Three Interfaces

### GUI (Kanban App)
Run `kanban` or `kanban app` to launch the desktop application.
- Create and manage projects
- Drag-and-drop issues across statuses
- View agent activity and execution logs
- Monitor task validation
- Undo/redo operations

### CLI (Command Line)
Run `kanban cli <command>` for scripting and automation.
- Bulk operations
- CI/CD integration
- Scheduled task creation
- Data export/import
- Agent registration

### MCP Server (JSON-RPC)
Run `kanban mcp` to expose Kanban as an MCP server.
- Integration with Claude or other MCP clients
- Programmatic task management
- Real-time agent orchestration
- Perfect for agentic workflows

All three interfaces read from and write to the same database, with optimistic locking to prevent conflicts.

## Core Concepts at a Glance

| Concept | Purpose |
|---------|---------|
| **Project** | Container for issues. Has a prefix (e.g., KAN) and auto-incrementing IDs (KAN-1, KAN-2). |
| **Issue** | Work item with title, description, status, priority, assignee. Can have parent (sub-issues). |
| **Status** | Workflow state (Backlog, Todo, In Progress, In Review, Done, etc.). Project-scoped. |
| **Status Category** | Maps statuses to agent behavior: `unstarted`, `started`, `blocked`, `completed`, `discarded`. |
| **Label** | Tag for grouping issues. Project-scoped. Has color. |
| **Member** | Person or agent. Agents auto-create members on registration. |
| **Task Contract** | Extends an issue with contract: type, objective, skills, complexity, success criteria (shell commands), constraints, timeout. |
| **Agent** | Registered AI system with skills, max complexity, max concurrent tasks. Gets assigned to task contracts. |
| **Execution Log** | Records every action: claim, start, reasoning, file_read, file_edit, command, error, complete, timeout. Replayable. |

---

## Next Steps

- **[Getting Started](/guide/getting-started.md)** — Installation and first 5 minutes
- **[Concepts](/guide/concepts.md)** — Deep dive into each core concept
- **[Projects](/guide/projects.md)** — Creating and managing projects
- **[Issues](/guide/issues.md)** — Full issue lifecycle
- **[Task Contracts](/guide/task-contracts.md)** — The centerpiece: executable task definitions
- **[Agent Routing](/guide/agent-routing.md)** — How tasks are matched to agents
- **[Validation](/guide/validation.md)** — Auto-validation with shell commands
- **[Execution & Replay](/guide/execution-replay.md)** — Full audit trail and replay viewer
