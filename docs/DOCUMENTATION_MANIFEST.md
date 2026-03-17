# Kanban Documentation Manifest

Complete VitePress documentation for the Kanban project management system. All files have been created and are ready for deployment.

## File Index

### Homepage & Navigation
- **`/docs/site/index.md`** — Hero landing page with features grid
- **`/docs/site/guide/index.md`** — Overview: What is Kanban, architecture, three interfaces

### Core Guides
1. **`/docs/site/guide/getting-started.md`** — Installation, first run, 5-minute tutorial
2. **`/docs/site/guide/concepts.md`** — Core concepts: Projects, Issues, Statuses, Labels, Members, Agents, Task Contracts, Execution Logs
3. **`/docs/site/guide/projects.md`** — Creating, managing, deleting projects; project prefix and issue IDs
4. **`/docs/site/guide/issues.md`** — Full issue lifecycle: create, list, update, search, delete, bulk operations, sub-tasks, relations
5. **`/docs/site/guide/statuses.md`** — Status categories, default workflows, custom statuses, agent behavior based on categories
6. **`/docs/site/guide/labels.md`** — Project-scoped labels, label sets, best practices
7. **`/docs/site/guide/members.md`** — Team members and agents, agent type badges, registration, stats

### Agent Orchestration (Core Features)
8. **`/docs/site/guide/task-contracts.md`** — THE KEY FEATURE
   - Task contract structure (type, objective, context, constraints, success criteria, skills, complexity, timeout)
   - State machine (queued → claimed → executing → validating → completed/blocked/cancelled)
   - Full lifecycle examples
   - Decomposition tasks

9. **`/docs/site/guide/agent-routing.md`** — Task matching algorithm
   - Capacity checks
   - Candidate filtering (blocked tasks, complexity, skills)
   - Sorting by priority
   - Atomic claiming (race condition prevention)
   - Heartbeat management
   - Agent registration and deregistration
   - Per-project config

10. **`/docs/site/guide/validation.md`** — Success criteria execution
    - Validation pipeline overview
    - Running executable checks (shell commands)
    - Safety: preventing shell injection
    - Confidence thresholds (auto-accept, human review, auto-reject)
    - Manual review workflows
    - Timeout handling (5 minutes per command)

### Visibility & Operations
11. **`/docs/site/guide/execution-replay.md`** — Full audit trail
    - Execution log entry types (claim, start, file_read, file_edit, command, error, result, complete, timeout, etc.)
    - Replay viewer
    - Task attempts grouped by retry
    - Logging patterns for agent implementations
    - Metrics and analytics
    - Export for analysis

12. **`/docs/site/guide/import-export.md`** — Data management
    - Export all data to JSON
    - Import from JSON
    - Format specification
    - Backup and recovery
    - Data migration between machines
    - Bulk data cleanup scripts
    - Conflict resolution

## Coverage Summary

| Feature | File(s) | Status |
|---------|---------|--------|
| Installation | getting-started.md | ✓ Complete |
| Projects | projects.md, concepts.md | ✓ Complete |
| Issues (basic) | issues.md, concepts.md | ✓ Complete |
| Issues (advanced) | issues.md | ✓ Complete (sub-tasks, relations, bulk ops) |
| Statuses | statuses.md, concepts.md | ✓ Complete |
| Labels | labels.md, concepts.md | ✓ Complete |
| Members | members.md, concepts.md | ✓ Complete |
| Task Contracts | task-contracts.md, concepts.md | ✓ Complete (state machine, lifecycle) |
| Agent Routing | agent-routing.md | ✓ Complete (matching, capacity, skills, complexity) |
| Validation | validation.md | ✓ Complete (criteria, thresholds, shell safety) |
| Execution Logs | execution-replay.md | ✓ Complete (entry types, replay, metrics) |
| Import/Export | import-export.md | ✓ Complete (JSON format, migration, recovery) |

## Documentation Features

### Each Guide Includes

- **Clear structure** — Table of contents, headings, logical flow
- **Real examples** — CLI commands that actually work
- **Code snippets** — Copy-paste ready for all major features
- **Best practices** — Production-ready patterns
- **Common gotchas** — Warnings and tips (:::warning, :::tip blocks)
- **Navigation** — Links to related guides
- **Scenarios** — Real-world use cases with step-by-step walkthrough

### VitePress Markdown Features Used

- **Frontmatter** — Hero layout on homepage
- **Code blocks** — With language syntax highlighting (bash, json, python, etc.)
- **Admonitions** — :::warning, :::tip blocks for emphasis
- **Tables** — Reference material in organized tables
- **Lists** — Ordered and unordered for clarity
- **Links** — Internal navigation between guides

## Usage

### To Deploy
```bash
# Install VitePress
npm install -g vitepress

# Build static site
cd docs/site
vitepress build

# Or serve locally for preview
vitepress dev
```

### File Organization
```
docs/site/
├── index.md                     # Homepage
└── guide/
    ├── index.md                 # Introduction
    ├── getting-started.md       # Installation & first run
    ├── concepts.md              # Core concepts
    ├── projects.md              # Projects guide
    ├── issues.md                # Issues guide
    ├── statuses.md              # Statuses & workflow
    ├── labels.md                # Labels guide
    ├── members.md               # Members & agents
    ├── task-contracts.md        # Task contracts (KEY)
    ├── agent-routing.md         # Agent routing (KEY)
    ├── validation.md            # Validation pipeline (KEY)
    ├── execution-replay.md      # Execution logs & replay
    └── import-export.md         # Backup & migration
```

## Key Topics Documented

### User Features
- ✓ Creating projects with custom prefixes
- ✓ Creating and managing issues with sub-tasks
- ✓ Custom statuses per project
- ✓ Status categories affecting agent behavior
- ✓ Project-scoped labels
- ✓ Team members and assignment
- ✓ Comments and activity logs
- ✓ Undo/redo support
- ✓ Bulk operations

### Agent Features
- ✓ Agent registration with skills and complexity
- ✓ Task contract creation (executable issues)
- ✓ Task state machine
- ✓ Task routing algorithm (capacity, skills, complexity, priority)
- ✓ Blocking relations and dependency resolution
- ✓ Decomposition tasks
- ✓ Execution logging (11 entry types)
- ✓ Task replay and audit trails

### Validation
- ✓ Success criteria definition
- ✓ Shell command execution with timeout
- ✓ Shell injection prevention
- ✓ Confidence-based routing (auto-accept/review/reject)
- ✓ Thresholds (per-project configuration)
- ✓ Manual review workflows

### Operations
- ✓ CLI commands for all features
- ✓ Export to JSON format
- ✓ Import from JSON format
- ✓ Backup and recovery
- ✓ Data migration between machines
- ✓ Analysis and querying
- ✓ Performance metrics

## Command Coverage

All major CLI commands are documented with examples:

```
Project:
  ✓ create, list, get, update, delete

Issue:
  ✓ create, list, get, update, search, delete, duplicate
  ✓ block, relate, move (set parent)
  ✓ bulk-update
  ✓ activity log

Status:
  ✓ list, create, update, delete

Label:
  ✓ create, list, update, delete

Member:
  ✓ add, list, get, update
  ✓ issues (list assigned)

Task Contract:
  ✓ create, list, get, update, complete
  ✓ log-activity
  ✓ replay, attempts
  ✓ validation (run, show, accept, reject)

Agent:
  ✓ register, deregister, heartbeat
  ✓ list, get, stats
  ✓ next-task

Metrics:
  ✓ metrics (project-level and agent-level)

Export/Import:
  ✓ export (full, by project, by date range)
  ✓ import
```

## Next Steps for Deployment

1. **Add VitePress config** — Create `.vitepress/config.ts` with navigation sidebar
2. **Add themes/styling** — Customize colors, fonts, logo
3. **Add favicon/logo** — Place in `/public` folder
4. **Build & deploy** — To GitHub Pages, Netlify, or your host
5. **Set up search** — Add Algolia DocSearch or local search

## Notes

- All documentation is accurate as of current codebase
- Examples are production-ready and tested against schema
- CLI commands use actual command names from source code
- JSON examples match actual database schema
- Status categories and task states match orchestration engine
- Validation rules match implementation (5-min timeout, shell safety, etc.)

## Maintenance

When updating Kanban features:
1. Update relevant guide file
2. Check for CLI command changes
3. Update examples and schemas
4. Add new feature guides as needed
5. Update table of contents in index.md

All files are in Markdown and can be edited directly.
