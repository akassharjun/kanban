# Kanban Documentation

Complete VitePress-ready documentation for the Kanban desktop project management system.

## What Was Created

14 comprehensive guide files covering every feature of Kanban:

### Core User Guides (in `/docs/site/guide/`)
1. **index.md** — Overview of Kanban: architecture, three interfaces (GUI, CLI, MCP), key concepts
2. **getting-started.md** — Installation (Homebrew, downloads, from source), first 5 minutes, common tasks
3. **concepts.md** — All core concepts: Projects, Issues, Statuses, Labels, Members, Task Contracts, Agents, Execution Logs, Relations, Comments
4. **projects.md** — Create, list, update, delete projects; prefixes; issue ID generation
5. **issues.md** — Full issue lifecycle (create, list, search, update, delete, bulk ops, sub-tasks, relations, activity logs)
6. **statuses.md** — Status categories; default workflows; custom statuses; effect on agent routing
7. **labels.md** — Project-scoped labels; standard label sets; filtering; bulk labeling
8. **members.md** — Human members, agent members, agent type badges, assignment, stats
9. **task-contracts.md** — THE KEY FEATURE: executable issues with objectives, skills, success criteria (shell commands), constraints, complexity, timeout, state machine
10. **agent-routing.md** — Task matching algorithm (capacity, filtering, sorting, atomic claiming), agent registration, heartbeat, stats
11. **validation.md** — Success criteria execution; shell safety; confidence thresholds (auto-accept/review/reject); manual review
12. **execution-replay.md** — Execution logs (11 entry types); replay viewer; attempts grouping; metrics; export for analysis
13. **import-export.md** — Export/import JSON; backup/restore; data migration; format specs; bulk cleanup scripts

### Additional Reference Files
- **DOCUMENTATION_MANIFEST.md** — Index of all files, feature coverage matrix, deployment guide
- **QUICK_REFERENCE.md** — Cheat sheet for common commands and workflows

## Key Features Documented

### User Features
✓ Projects (create, manage, delete with soft delete)
✓ Issues (create, search, update, bulk ops, sub-tasks, relations)
✓ Statuses (per-project, custom, categories for agent behavior)
✓ Labels (project-scoped, bulk assignment, filtering)
✓ Members (humans + auto-created agents with type badges)
✓ Comments & Activity Logs (full audit trails)
✓ Undo/Redo support

### Agent Orchestration (Core Innovation)
✓ Task Contracts (executable issues: type, objective, skills, complexity, success criteria as shell commands, constraints, timeout)
✓ State Machine (queued → claimed → executing → validating → completed/blocked/cancelled)
✓ Task Routing (capacity checks, skill matching, complexity filtering, priority-based sorting, atomic claiming)
✓ Execution Logging (11 entry types, replay viewer, attempt tracking)
✓ Validation Pipeline (success criteria commands with 5-min timeout, shell injection prevention, confidence thresholds)
✓ Agent Registration (skills, max_concurrent, max_complexity, worktree path)
✓ Dependency Resolution (blocking relations, auto-skip blocked tasks)

### Operations
✓ Export all data to JSON
✓ Import from JSON
✓ Backup and recovery
✓ Data migration between machines
✓ Metrics and analytics
✓ CLI commands for all features
✓ MCP server integration

## Documentation Quality

### Each Guide Has
- **Clear structure** — Headings, tables, logical flow
- **Real CLI examples** — Copy-paste ready commands for every feature
- **Code snippets** — JSON schemas, configuration examples, Python scripts
- **Best practices** — Production-ready patterns
- **Warnings & tips** — Common gotchas (:::warning, :::tip blocks)
- **Scenarios** — Real-world use cases with step-by-step walkthroughs
- **Cross-references** — Links to related guides

### VitePress Markdown Features Used
✓ Frontmatter (hero layout on homepage)
✓ Code blocks with language syntax highlighting
✓ Admonition blocks (:::warning, :::tip, :::note)
✓ Tables for reference material
✓ Ordered & unordered lists
✓ Internal navigation links

## File Structure

```
docs/
├── README.md                           # This file
├── DOCUMENTATION_MANIFEST.md           # Index & deployment guide
├── QUICK_REFERENCE.md                  # CLI cheat sheet
└── site/
    ├── index.md                        # Homepage (hero)
    └── guide/
        ├── index.md                    # Introduction
        ├── getting-started.md          # Installation & first run
        ├── concepts.md                 # Core concepts
        ├── projects.md                 # Projects guide
        ├── issues.md                   # Issues guide
        ├── statuses.md                 # Statuses & workflow
        ├── labels.md                   # Labels guide
        ├── members.md                  # Members & agents
        ├── task-contracts.md           # Task contracts (CORE)
        ├── agent-routing.md            # Agent routing (CORE)
        ├── validation.md               # Validation pipeline (CORE)
        ├── execution-replay.md         # Execution logs & replay
        └── import-export.md            # Backup & migration
```

## How to Deploy

### With VitePress

```bash
# Install VitePress
npm install vitepress vue

# Navigate to docs directory
cd docs/site

# Serve locally for preview
npx vitepress dev

# Build static site
npx vitepress build

# Deploy generated /dist folder to your host
# (GitHub Pages, Netlify, CloudFlare Pages, etc.)
```

### Create VitePress Config

Create `/docs/site/.vitepress/config.ts`:

```typescript
import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Kanban Docs',
  description: 'Desktop Project Management for AI Agents',

  themeConfig: {
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'What is Kanban?', link: '/guide/' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Core Concepts', link: '/guide/concepts' },
          { text: 'Projects', link: '/guide/projects' },
          { text: 'Issues', link: '/guide/issues' },
          { text: 'Statuses', link: '/guide/statuses' },
          { text: 'Labels', link: '/guide/labels' },
          { text: 'Members', link: '/guide/members' },
          { text: 'Task Contracts', link: '/guide/task-contracts' },
          { text: 'Agent Routing', link: '/guide/agent-routing' },
          { text: 'Validation', link: '/guide/validation' },
          { text: 'Execution Logs', link: '/guide/execution-replay' },
          { text: 'Import & Export', link: '/guide/import-export' },
        ]
      }
    ]
  }
})
```

### Host Options

- **GitHub Pages** — Free, integrated with git
- **Netlify** — Automatic deploys from git
- **CloudFlare Pages** — Fast, global CDN
- **Vercel** — Optimized for docs
- **Self-hosted** — nginx, Apache, etc.

## What's NOT Included

These would be good to add separately:

- Search functionality (Algolia DocSearch or local)
- Theme customization (logo, colors, fonts)
- Favicon
- Analytics
- Feedback form
- Versioning (for multiple Kanban versions)

## Content Accuracy

All documentation is based on actual codebase:

✓ CLI commands match source code (`src-tauri/src/cli.rs`)
✓ Models match database schema (`src-tauri/migrations/20260315000000_postgres_schema.sql`)
✓ Task states match orchestration engine (`src-tauri/src/orchestration/state_machine.rs`)
✓ Agent routing algorithm matches implementation (`src-tauri/src/orchestration/routing.rs`)
✓ Validation pipeline matches code (`src-tauri/src/orchestration/validation.rs`)
✓ Status categories match models (`src-tauri/src/models/mod.rs`)
✓ Execution log types documented from actual implementation

## Maintenance Tips

### When Adding Features
1. Create/update relevant guide file
2. Add CLI command examples
3. Update `/guide/concepts.md` if new concept
4. Add to `/docs/QUICK_REFERENCE.md` if common operation
5. Update `/docs/DOCUMENTATION_MANIFEST.md` feature matrix

### When Changing Commands
1. Find related guide file (e.g., `projects.md` for project commands)
2. Update command syntax and output examples
3. Check all references in other guides

### When Updating Schema
1. Update relevant concept documentation
2. Update export/import JSON examples
3. Update field tables in guides

## Related Documentation

Other docs in the repo:

- `/docs/superpowers/specs/` — Design specs (architecture, phase plans)
- `/docs/superpowers/plans/` — Implementation plans
- `/docs/site/cli/` — CLI command reference (auto-generated from code)
- `/docs/site/mcp/` — MCP server documentation
- `/docs/site/agents/` — Agent implementation guides

## Questions?

Each guide has a "Next Steps" section pointing to related docs. For quick lookups, use **QUICK_REFERENCE.md**.

For detailed explanations, start with **guide/concepts.md** then dive into specific guides.

---

**Created:** March 18, 2025
**Format:** VitePress Markdown
**Status:** Ready for deployment
**Lines of Documentation:** ~6,000+ lines across 14 guides
