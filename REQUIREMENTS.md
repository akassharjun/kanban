# Kanban - Desktop Project Management Tool

A fast, offline-first desktop project management tool inspired by Linear, built in Rust. Designed for developers and AI agents alike.

---

## Vision

A lightweight, blazing-fast alternative to Linear that runs entirely on your desktop with no cloud dependency. Zero rate limits, zero latency to a server, full data ownership. First-class support for AI agent interaction via CLI and MCP.

---

## Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop Shell | **Tauri v2** | Rust backend + web frontend, tiny binaries (~2MB), native OS integration |
| Frontend | **React 18 + TypeScript** | Rich drag-and-drop kanban UI, mature markdown ecosystem |
| Styling | **Tailwind CSS + shadcn/ui** | Clean, modern design system matching Linear's aesthetic |
| Markdown | **react-markdown + remark-gfm** (render), **pulldown-cmark** (Rust-side parsing) | Full GFM support with syntax highlighting |
| Database | **SQLite via sqlx** | Async, compile-time checked queries, zero-config embedded DB |
| Drag & Drop | **@dnd-kit/core** | Modern, accessible drag-and-drop for kanban columns |

---

## Core Features

### 1. Workspace

- Single local workspace per installation
- Workspace-level settings (theme, default views, etc.)
- All data stored in a local SQLite database file

### 2. Team Members

- Workspace-level list of team members
- Each member has: name, display name, email (optional), avatar color/initials
- Add, edit, remove members from workspace settings
- Members are available as assignees across all projects
- Member avatars shown on issue cards and detail views
- Filter issues by assignee across projects

### 3. Projects

- Create, rename, archive, delete projects
- Each project has its own kanban board, issue list, and settings
- Project-level properties:
  - Name
  - Description (markdown)
  - Icon/emoji
  - Status: `Active`, `Paused`, `Completed`, `Archived`
  - Created/updated timestamps
- Project list view in the sidebar

### 4. Issues

#### Properties

| Property | Type | Required | Notes |
|----------|------|----------|-------|
| Title | string | Yes | Short summary |
| Description | markdown | No | Full GFM with syntax highlighting |
| Status | enum | Yes | Default: Todo, Backlog, In Progress, In Review, Blocked, Discarded, Done |
| Priority | enum | No | None, Urgent, High, Medium, Low |
| Labels | string[] | No | Project-scoped, color-coded |
| Assignee | member_id | No | Select from workspace team members |
| Identifier | auto | Yes | Project prefix + auto-increment (e.g., `KAN-42`) |
| Created at | datetime | Yes | Auto-set |
| Updated at | datetime | Yes | Auto-set |
| Due date | date | No | Optional deadline |
| Estimate | number | No | Story points or hours |

#### Operations

- Create, edit, delete issues
- Move between statuses (drag-and-drop on kanban, or manual)
- Bulk status updates (select multiple, change status/priority/assignee)
- Duplicate an issue
- Full-text search across title and description
- Filter by: status, priority, label, assignee, due date
- Sort by: priority, created date, updated date, due date, manual order

### 5. Kanban Board

- Default view for each project
- Columns represent statuses (configurable order)
- Drag-and-drop issues between columns
- Drag-and-drop to reorder within a column
- Column headers show issue count
- Collapsible columns
- Group by: Status (default), Priority, Assignee, Label
- Swimlane support (group rows by a second dimension)
- Visual priority indicators (colored left border or icon)
- Blocked indicator on cards (icon/badge when issue has unresolved blockers)
- Assignee avatar on issue cards
- Quick-add issue from any column (inline input at top/bottom of column)

### 6. Issue Detail View

- Slide-over panel or full-page view
- Markdown editor with live preview for description
- Property editing (status, priority, labels, assignee, due date, estimate)
- Activity log showing status changes and edits with timestamps
- Sub-issues list with inline creation
- Parent issue breadcrumb
- Issue relations section (related, blocks, blocked by, duplicate)
- Visual "blocked" indicator when issue has unresolved blockers
- Keyboard shortcut to close (`Esc`)

### 7. Markdown Support

- Full GitHub-Flavored Markdown rendering:
  - Headings, bold, italic, strikethrough
  - Ordered/unordered lists, task lists (checkboxes)
  - Code blocks with syntax highlighting (common languages)
  - Tables
  - Links and images (local file paths or URLs)
  - Blockquotes
  - Horizontal rules
- WYSIWYG-style toolbar for formatting
- Issue reference linking (`KAN-42` auto-links to the issue)

### 8. Parent/Child Relationships (Epics)

- Any issue can be a parent (epic) with child sub-issues
- Sub-issues inherit project but have independent status/priority
- Parent issue shows aggregated progress (e.g., "3/7 sub-issues done")
- Auto-close parent: when all child issues reach `completed` or `discarded` status, parent automatically transitions to `Done`
- Tree view for hierarchical browsing
- Max nesting depth: 2 levels (Epic -> Issue -> Sub-issue)
- Drag issue to become child of another issue

### 9. Labels

- Project-scoped labels
- Each label has: name, color
- Create/edit/delete labels from project settings
- Filter board and list views by label
- Multi-label assignment per issue

### 10. Views

- **Board view**: Kanban (default)
- **List view**: Sortable table with all issue properties
- **Tree view**: Hierarchical view showing epic/issue/sub-issue structure
- Saved filters per project (persist filter + sort + group-by combinations)
- Toggle to show/hide completed issues

### 11. Issue Templates

- Project-scoped templates for common issue types (e.g., "Bug Report", "Feature Request", "Task")
- Each template defines: default title prefix, description (markdown boilerplate), default status, priority, labels
- Create/edit/delete templates from project settings
- Select template when creating a new issue (pre-fills fields)
- CLI: `kanban issue create --project "Auth" --template "Bug Report" --title "Login crash"`
- MCP: `create_issue` accepts optional `template` parameter

### 12. Custom Statuses

- Each project can define its own status workflow
- Default statuses: `Todo`, `Backlog`, `In Progress`, `In Review`, `Blocked`, `Discarded`, `Done`
- Status categories: `unstarted` (Todo, Backlog), `started` (In Progress, In Review), `blocked` (Blocked), `completed` (Done), `discarded` (Discarded)
- Add, rename, reorder, delete statuses per project
- Status icon and color customization

---

## AI Agent Features

### 13. CLI Interface

A built-in CLI (`kanban`) that exposes all core operations for scripting and agent use:

```
kanban project list
kanban project create "Auth Service"
kanban issue create --project "Auth Service" --title "Add OAuth2" --priority high
kanban issue list --project "Auth Service" --status "In Progress"
kanban issue update KAN-42 --status "Done"
kanban issue search "authentication"
kanban issue move KAN-42 --parent KAN-10
kanban issue block KAN-42 --by KAN-10
kanban issue relate KAN-42 --to KAN-15
kanban member add "Sarah" --email sarah@example.com
kanban member list
kanban label create --project "Auth Service" --name "backend" --color "#3b82f6"
```

- JSON output mode (`--json`) for machine consumption
- Batch operations via stdin (pipe newline-delimited JSON commands)
- Tab completion for shells (bash, zsh, fish)

### 14. MCP Server

Built-in Model Context Protocol server so AI agents (Claude Code, Cursor, etc.) can interact directly:

**Tools exposed:**
- `list_projects` - List all projects
- `create_issue` - Create an issue with all properties
- `update_issue` - Update any issue property
- `search_issues` - Full-text search with filters
- `get_issue` - Get issue details including description and sub-issues
- `list_issues` - List issues with filtering and sorting
- `move_issue` - Change status or parent
- `bulk_update` - Update multiple issues at once
- `get_board` - Get kanban board state for a project
- `create_label` - Create a project label
- `add_blocker` - Mark an issue as blocked by another
- `list_members` - List workspace members
- `add_member` - Add a team member

**Resources exposed:**
- `kanban://projects` - All projects
- `kanban://project/{id}/board` - Board state
- `kanban://issue/{identifier}` - Issue details

### 15. Agent-Friendly Data Model

- All entities have stable, predictable identifiers (`KAN-42`, not UUIDs in the UI)
- Enum values are documented and constrained (agents can validate before calling)
- Timestamps in ISO 8601
- Markdown stored as raw text (agents write markdown naturally)
- Issue references in descriptions auto-resolve (`KAN-42` -> clickable link)

### 16. Notifications

- Native OS notifications via Tauri's notification API
- Notification triggers:
  - Issue assigned to you (requires setting "me" in workspace settings)
  - Issue you're assigned to was moved to Blocked
  - Due date approaching (configurable: 1 day, 3 days before)
  - Due date overdue
  - Sub-issue completed on an epic you're assigned to
  - Status change on issues you created
- In-app notification center (bell icon) with unread count
- Notification history (dismissible, mark as read)
- Per-trigger enable/disable in settings
- CLI: `kanban notifications list`, `kanban notifications clear`

### 17. Automation Hooks

- **On status change**: Run a shell command when an issue transitions (e.g., notify, trigger build)
- **On creation**: Run a command when a new issue is created
- Configurable per project via a `hooks.toml` file

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New issue | `C` |
| Search | `Cmd + K` |
| Switch project | `Cmd + P` |
| Toggle sidebar | `Cmd + B` |
| Board view | `1` |
| List view | `2` |
| Tree view | `3` |
| Close detail panel | `Esc` |
| Set priority (in detail) | `P` then `1-4` |
| Set status | `S` |
| Assign | `A` |
| Add label | `L` |
| Delete issue | `Cmd + Backspace` |
| Undo | `Cmd + Z` |
| Redo | `Cmd + Shift + Z` |

---

## Database Schema (High-Level)

```
projects
  id, name, description, icon, status, created_at, updated_at

statuses
  id, project_id, name, category, color, icon, position

issues
  id, project_id, identifier, title, description, status_id,
  priority, assignee_id (FK members), parent_id, position, estimate,
  due_date, created_at, updated_at

issue_templates
  id, project_id, name, description_template (markdown), default_status_id,
  default_priority, default_label_ids (JSON), created_at, updated_at

labels
  id, project_id, name, color

issue_labels
  issue_id, label_id

members
  id, name, display_name, email, avatar_color, created_at

issue_relations
  id, source_issue_id, target_issue_id, relation_type (related, blocks, blocked_by, duplicate)

activity_log
  id, issue_id, field_changed, old_value, new_value, timestamp

undo_log
  id, operation_type (create, update, delete), entity_type (issue, label, project, member, relation),
  entity_id, snapshot_before (JSON), snapshot_after (JSON), undone (bool), timestamp

notifications
  id, type, issue_id, message, read (bool), created_at

hooks
  id, project_id, event_type, command
```

---

## Non-Functional Requirements

- **Performance**: All UI interactions under 50ms. Database queries under 10ms for typical datasets (< 10k issues).
- **Offline-first**: No network required. Everything local.
- **Data portability**: Export/import to JSON for backup and migration.
- **Binary size**: Under 10MB for the final distributable.
- **Platform**: macOS only.
- **Unlimited Undo/Redo**: Full undo/redo history for all operations (create, edit, delete, move, status change). Backed by an operation log in the database. Persists across app restarts. `Cmd+Z` to undo, `Cmd+Shift+Z` to redo.
- **Theme**: Dark mode default, light mode available. Follow system preference.

---

## Out of Scope (for v1)

Features present in Linear, Jira, and/or GitHub Projects but excluded from v1:

### Planning & Agile
- Cycles/sprints with time-boxing, capacity planning, and automatic rollover *(Linear, Jira, GitHub)*
- Roadmap / timeline view with milestone tracking *(Linear, Jira)*
- Burndown / burn-up charts *(Linear, Jira, GitHub)*
- Velocity tracking and estimation from historical data *(Linear, Jira)*
- Project dependency graphs (visual cross-project blocking) *(Linear)*
- Portfolio management, budget allocation, resource planning *(Jira)*
- Backlog ranking view (dedicated backlog separate from board) *(Jira)*

### Workflow & Automation
- Recurring issues on a schedule (daily/weekly/monthly) *(Linear)*
- Form templates with custom field types and validation rules *(Linear, Jira)*
- Conditional workflow rules (if X then Y) beyond simple hooks *(Jira, GitHub)*
- Auto-add issues to project based on filter criteria *(GitHub)*

### Collaboration
- Real-time multi-user collaboration / sync
- User authentication or login accounts
- Threaded comments on issues *(Linear, Jira)*
- @mentions for team members in descriptions/comments *(Linear, Jira, GitHub)*
- Activity feed (workspace-wide stream of changes) *(Linear)*
- Guest access with limited permissions *(Linear, Jira)*
- Role-based access control (owner/admin/member/guest) *(Linear, Jira)*

### Reporting & Analytics
- Customizable dashboards with widgets/gadgets *(Jira)*
- DORA metrics (deployment frequency, lead time) *(GitHub)*
- AI-powered insights and discussion summaries *(Linear)*
- Custom charts and data visualizations *(GitHub, Jira)*

### Search & Query
- Advanced query language (JQL-style) for complex filtering *(Jira)*

### Time & Estimation
- Time tracking / work logging with remaining estimate adjustment *(Jira)*
- Time spent reports (original vs actual vs remaining) *(Jira)*

### Development Integration
- Git integration (branch/PR linking, auto-close on merge) *(Linear, Jira, GitHub)*
- CI/CD status on issues *(GitHub, Jira)*

### Service Management
- Helpdesk / ticketing queues *(Jira Service Management)*
- SLA management with escalation rules *(Jira)*
- Customer-facing self-service portal *(Jira)*

### Platform & Distribution
- Cloud storage or remote database
- Mobile app
- Windows / Linux support
- Calendar view
- Plugin / extension system
- Marketplace for third-party add-ons *(Jira)*

---

## Decisions

1. **UI Framework**: Tauri v2 with React/TypeScript frontend.
2. **Database location**: `~/.kanban/data.db`, configurable via env var or settings.
3. **Issue identifier format**: 3-letter project prefix + auto-increment (e.g., `KAN-1`, `AUT-42`). User sets prefix on project creation.
