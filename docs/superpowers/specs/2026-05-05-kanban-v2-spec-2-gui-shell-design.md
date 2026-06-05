# Spec #2 — Kanban v2 GUI Shell (Tauri Desktop App)

**Date:** 2026-05-05
**Supersedes:** none (greenfield GUI; v1 GUI was wiped before Spec #1 shipped)
**Builds on:** [Spec #1 — Core + CLI](2026-05-03-kanban-v2-core-cli-design.md)
**Roadmap epics covered:** E01 Workspace & Theme · E02 Projects · E04 Issues (CRUD) · E05 Kanban Board · E06 Detail Panel & Markdown
**Roadmap epics deferred to Spec #3+:** E03 Members · E07 Hierarchy · E08 Labels UI · E09 Multi-views · E10 Templates · E11 Custom Statuses UI · E13 MCP · E14 Notifications · E15 Hooks · E16 Shortcuts (beyond Esc) · E17 Undo/Redo wired to ⌘Z · E18 Import/Export UI · E19 Agents · E20 Audit UI

## Goals

1. Ship a usable macOS desktop kanban built on the v2 core, in a single coherent spec.
2. Validate the Rust ↔ React bridge end to end: a TypeScript caller can apply any `Operation` and receive typed errors.
3. Deliver the visible "kanban board" headline: drag-and-drop columns by status, click-to-open detail panel, markdown body, theme toggle.
4. Preserve the architectural invariants of `kanban-core` (sync, single mutator via `Workspace::apply`, append-only migrations).

## Non-goals

- Cross-platform releases. macOS-only first ship; Linux/Windows are mechanical CI work for Spec #3+.
- Code signing / Apple Developer enrollment. First ship is unsigned with a documented `xattr` workaround.
- Multi-process change detection (the GUI does **not** auto-refresh while a CLI mutates the DB). Refresh on focus is sufficient.
- Labels UI, multi-views, ⌘Z keyboard shortcut, audit log UI, hierarchy display, agent surface. All deferred.
- Postgres path (#30 in E01) — out of scope; SQLite-only.
- Syntax highlighting in markdown bodies. Plain `<pre>` only.

## Stack

- **Tauri** 2.x. Custom title bar (`decorations: false`, `titleBarStyle: Overlay` on macOS).
- **React** 19, **Vite** 5, **Tailwind v4**, **shadcn/ui** (copy-in primitives).
- **react-router** 7 (declarative mode), **TanStack Query** 5, **react-hook-form** + **zod**, **@dnd-kit/core** + **@dnd-kit/sortable**, **react-markdown** + **remark-gfm**, **lucide-react**.
- **tauri-specta** + **specta** for Rust → TS type generation. Generated `bindings.ts` is **git-tracked** so a clean checkout has types without a build step.
- **Tests:** Vitest + React Testing Library (unit + component); Playwright + `tauri-driver` (E2E).
- **Package manager:** pnpm.

## Workspace layout

```
kanban/
├── Cargo.toml                       # workspace
├── crates/
│   ├── kanban-core/                 # unchanged from Spec #1
│   ├── kanban-cli/                  # unchanged from Spec #1
│   └── kanban-tauri/         (new)  # Tauri app crate
│       ├── src/
│       │   ├── main.rs              # tauri::Builder + invoke handlers
│       │   ├── commands.rs          # apply, reads, undo/redo, settings
│       │   ├── settings.rs          # workspace_settings table I/O
│       │   ├── error.rs             # ApiError + From<KanbanError>
│       │   └── state.rs             # AppState holding Mutex<Workspace>
│       ├── tauri.conf.json
│       ├── build.rs                 # tauri_build + specta export
│       ├── icons/                   # generated from a single SVG
│       └── Cargo.toml
├── ui/                       (new)  # Vite root
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── playwright.config.ts
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── routes/
│   │   ├── components/{sidebar,board,detail,chrome,ui}/
│   │   ├── data/{bindings.ts,ops.ts,client.ts,queries.ts,mutations.ts}
│   │   ├── theme/{ThemeProvider.tsx,useSystemTheme.ts}
│   │   ├── lib/{cn.ts,sortKey.ts}
│   │   └── styles.css
│   ├── tests/                       # Vitest unit + RTL component
│   └── e2e/                         # Playwright/tauri-driver
└── docs/superpowers/specs/2026-05-05-kanban-v2-spec-2-gui-shell-design.md
```

`ui/` is **not** a Cargo workspace member; it is an independent pnpm package. Tauri references it via `frontendDist: "../../ui/dist"` in `tauri.conf.json`.

## Architectural invariants preserved

- `kanban-core` stays **sync**. Every Tauri command that touches the DB executes inside `tokio::task::spawn_blocking(move || …)` so the Tauri event loop is never blocked.
- **Single public mutator** for domain data: Tauri's `apply` command is a thin wrapper around `Workspace::apply(Operation)`. The 14 Operation variants are the only path that touches issues/projects/labels/statuses/seq.
- **Migrations are append-only.** Spec #2 adds `0002_workspace_settings.sql`; `0001_init.sql` remains frozen.
- **Race-safe identifier allocation** stays in core; the GUI never computes seqs.
- **ImportSnapshot inverse semantics** stay in core; undo/redo via Tauri commands just call `Workspace::undo()` / `redo()`.

### Documented exception: workspace settings

Theme preference and any future per-workspace UI prefs live in a new `workspace_settings` table. They are exposed via `Workspace::get_setting(key) -> Option<String>` and `Workspace::set_setting(key, value)` — **not** through `Operation`/`apply`. Settings are app-level prefs, not domain state, and have no undo semantics. The single-mutator invariant retains its original meaning (issue/project/label/status writes).

## Schema additions

`crates/kanban-core/migrations/0002_workspace_settings.sql`:

```sql
CREATE TABLE workspace_settings (
  key        TEXT PRIMARY KEY NOT NULL,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO workspace_settings (key, value, updated_at)
  VALUES ('theme', 'system', datetime('now'));
```

That is the only schema change in Spec #2. The board, detail panel, projects, and issues all use existing tables from Spec #1.

## Tauri command surface

Ten commands total. All commands return `Result<T, ApiError>`. All DB-touching handlers run inside `spawn_blocking`.

### Reads

- `list_projects() -> Vec<Project>`
- `get_project(prefix: String) -> Project`
- `list_issues(project: String, filter: ListIssuesFilter) -> Vec<Issue>` — `ListIssuesFilter` is `{ status?: String, label?: String, search?: String }`. Search left for Spec #3 polish; the filter struct exists now to avoid a future signature change.
- `get_issue(key: String) -> IssueDetail` — full issue payload including labels.
- `list_statuses(project: String) -> Vec<Status>` — for board column ordering.
- `list_labels(project: String) -> Vec<Label>` — board cards display label dots even though full label management UI is deferred.
- `get_settings() -> Settings`

### Mutations

- `apply(op: Operation) -> ApplyResult` — the one mutator for domain data.
- `undo() -> Option<Operation>` — exposed now for the bridge test, even though no UI binding lands until Spec #3.
- `redo() -> Option<Operation>` — same.

### Settings

- `update_settings(theme: ThemeChoice) -> Settings` — only theme for now; struct exists to take more fields later.

### Error mapping

```rust
#[derive(serde::Serialize, specta::Type, thiserror::Error, Debug)]
#[serde(tag = "kind")]
pub enum ApiError {
  #[error("{resource} not found: {key}")]
  NotFound { resource: String, key: String },
  #[error("validation: {field}: {message}")]
  Validation { field: String, message: String },
  #[error("conflict: {message}")]
  Conflict { message: String },
  #[error("internal: {message}")]
  Internal { message: String },
}

impl From<KanbanError> for ApiError {
  fn from(e: KanbanError) -> Self { /* exhaustive match — all variants mapped */ }
}
```

Validation errors carry the offending `field` so the React side can route them back into the form (zod resolver) instead of surfacing them as a toast.

### tauri-specta wiring

`build.rs` runs the specta export at compile time and writes `ui/src/data/bindings.ts`. The file contains:
- All Operation/data types (`Operation`, `CreateProject`, `UpdateIssueField`, `ReorderIssue`, …).
- All read/return types (`Project`, `Issue`, `IssueDetail`, `Status`, `Label`, `Settings`, `ListIssuesFilter`, `ApplyResult`, `ThemeChoice`, `ApiError`).
- A typed `commands` object: `commands.apply(op)`, `commands.listProjects()`, etc., each returning `Promise<Result<T, ApiError>>`.

`bindings.ts` is git-tracked. Rationale: a fresh `pnpm install` should yield typed code without first running `cargo build`. The build.rs guards against staleness by erroring if regeneration produces a diff in CI.

## React app structure

### Routing tree (react-router v7)

| Path | Renders |
|---|---|
| `/` | empty state — "select or create a project" |
| `/p/{prefix}` | board for that project |
| `/p/{prefix}/i/{key}` | board (mounted, untouched) + detail panel overlay |

The detail panel route is a **child** of the project route. Closing it (`Esc`, ✕, click on dimmed board area) navigates back to `/p/{prefix}` without remounting the board. There is no separate `/settings` route in Spec #2 — the title-bar `<ThemeToggle>` is the only settings surface; a dedicated settings route lands in Spec #3 when more prefs need a home.

### Component map

- **`<Sidebar>`** — fixed 200px left rail. Header "PROJECTS" label, list of projects from `useProjects()` rendered as `<NavLink>`s. Bottom: `+ New project` button → `<NewProjectDialog>` (react-hook-form + zod schema requiring `name` + `prefix` matching `[A-Z]{2,8}`).
- **`<Board>`** — main pane for `/p/{prefix}`. Loads `useIssues(prefix)`, `useStatuses(prefix)`, `useLabels(prefix)`. Groups issues by status (in `Status.sort_order`). Renders one `<BoardColumn>` per status.
- **`<BoardColumn>`** — header (`Status.name` + count), `<SortableContext strategy={verticalListSortingStrategy}>` containing `<IssueCard>`s in `sort_key` order, dashed `+ Add issue` button at the bottom that swaps to an inline single-line input (Enter = create, Esc = cancel; no modal).
- **`<IssueCard>`** — issue key (mono small) + title + small label dots + priority badge. Click → navigate to `/p/{prefix}/i/{key}`. `cursor: grab`; on drag the card lifts (rotation + shadow).
- **`<IssuePanel>`** — 520px right overlay. Header: `IssueKey`, status `<select>`, priority badge, ⋯ (more menu — empty stub for Spec #3), ✕. Body: click-to-edit `<h3>` title; explicit Edit button on the markdown description. Footer: created/updated timestamps. Esc closes.
- **`<MarkdownView>`** — `react-markdown` + `remark-gfm`, styled with Tailwind prose-tweaks. Fenced code blocks render as plain `<pre>`. Anchor `onClick` calls `tauri-plugin-shell` `open()` so links open in the system browser, not inside the WebView.
- **`<ThemeToggle>`** — title-bar icon button. Cycles `system → light → dark → system` on click; tooltip shows the current state.
- **`<TitleBar>`** — `<header data-tauri-drag-region>` with workspace name on the left, `<ThemeToggle>` on the right. macOS traffic-light buttons live natively at `{x: 16, y: 18}` per `tauri.conf.json`.
- **`<Toaster>`** — single shadcn `<Toaster>` mounted at the App root. `useApply().onError` pushes a toast.

### Data layer

#### Query keys

Single module `data/queries.ts` so invalidation stays consistent:

```ts
export const qk = {
  projects: ()                  => ['projects'] as const,
  project:  (prefix: string)    => ['projects', prefix] as const,
  issues:   (prefix: string)    => ['projects', prefix, 'issues'] as const,
  issue:    (key: string)       => ['issues', key] as const,
  statuses: (prefix: string)    => ['projects', prefix, 'statuses'] as const,
  labels:   (prefix: string)    => ['projects', prefix, 'labels'] as const,
  settings: ()                  => ['settings'] as const,
};
```

`refetchOnWindowFocus: true` is set globally on the `QueryClient`. Tauri fires a window-focus event the React side picks up via TanStack Query's default focus-refetch.

#### Mutation hook — `useApply()`

```ts
export function useApply() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (op: Operation) => commands.apply(op),
    onMutate: async (op) => optimisticPatch(qc, op),
    onError:  (_err, _op, ctx) => ctx?.rollback?.(),
    onSettled:(_res, _err, op) => invalidateFor(qc, op),
  });
}
```

`optimisticPatch` and `invalidateFor` dispatch on `op.type`:

| Operation | Optimistic patch | Invalidate |
|---|---|---|
| `CreateProject` | append to `qk.projects()` | `qk.projects()` |
| `UpdateProject` | patch project in cache | `qk.projects()`, `qk.project(prefix)` |
| `ArchiveProject` / `DeleteProject` | remove from cache | `qk.projects()` |
| `CreateIssue` | push stub into target status column | `qk.issues(prefix)` |
| `UpdateIssueField` | patch issue in `qk.issues(prefix)` and `qk.issue(key)` | both |
| `ReorderIssue` | reorder cards in column | `qk.issues(prefix)` |
| `DeleteIssue` | remove from cache | `qk.issues(prefix)` |
| `CreateLabel` / `UpdateLabel` / `DeleteLabel` | patch labels list | `qk.labels(prefix)`, `qk.issues(prefix)` |
| `AttachLabel` / `DetachLabel` | patch issue's labels | `qk.issue(key)`, `qk.issues(prefix)` |
| `ImportSnapshot` | (skip optimistic — too broad) | invalidate everything |

On `ApiError` return, the rollback context restores prior cache state and a toast renders `error.kind: error.message`. `Validation` errors with a `field` are caught by per-field zod resolvers and never reach the toast.

#### TS Operation builders — `data/ops.ts`

The core's `Operation` enum uses `#[serde(tag = "op", content = "args")]`, so the JSON shape on the wire is `{ op: 'UpdateIssueField', args: { … } }`. `IssueFieldChange` uses `#[serde(tag = "field", content = "value")]` with PascalCase variant names. Hand-written thin builders typed against `bindings.ts`:

```ts
import type { Operation, CreateProject, CreateIssue, UpdateIssueField, ReorderIssue, IssueFieldChange } from './bindings';

export const ops = {
  createProject:    (args: CreateProject)    => ({ op: 'CreateProject', args }) as Operation,
  updateProject:    (args: UpdateProject)    => ({ op: 'UpdateProject', args }) as Operation,
  archiveProject:   (args: ArchiveProject)   => ({ op: 'ArchiveProject', args }) as Operation,
  deleteProject:    (args: DeleteProject)    => ({ op: 'DeleteProject', args }) as Operation,
  createIssue:      (args: CreateIssue)      => ({ op: 'CreateIssue', args }) as Operation,
  updateIssueField: (args: UpdateIssueField) => ({ op: 'UpdateIssueField', args }) as Operation,
  reorderIssue:     (args: ReorderIssue)     => ({ op: 'ReorderIssue', args }) as Operation,
  deleteIssue:      (args: DeleteIssue)      => ({ op: 'DeleteIssue', args }) as Operation,
  createLabel:      (args: CreateLabel)      => ({ op: 'CreateLabel', args }) as Operation,
  updateLabel:      (args: UpdateLabel)      => ({ op: 'UpdateLabel', args }) as Operation,
  deleteLabel:      (args: DeleteLabel)      => ({ op: 'DeleteLabel', args }) as Operation,
  attachLabel:      (args: AttachLabel)      => ({ op: 'AttachLabel', args }) as Operation,
  detachLabel:      (args: DetachLabel)      => ({ op: 'DetachLabel', args }) as Operation,
  importSnapshot:   (args: ImportSnapshot)   => ({ op: 'ImportSnapshot', args }) as Operation,
} as const;

// IssueFieldChange helpers — shorthand for the wire shape `{ field: 'Title', value: '…' }`
export const change = {
  title:       (v: string)         => ({ field: 'Title',       value: v }) as IssueFieldChange,
  description: (v: string | null)  => ({ field: 'Description', value: v }) as IssueFieldChange,
  status:      (v: string /*Uuid*/)=> ({ field: 'Status',      value: v }) as IssueFieldChange,
  priority:    (v: 'none'|'low'|'medium'|'high'|'urgent') => ({ field: 'Priority', value: v }) as IssueFieldChange,
  dueDate:     (v: string | null)  => ({ field: 'DueDate',     value: v }) as IssueFieldChange,
} as const;
```

Operation structs reference UUIDs (e.g. `UpdateIssueField { id: Uuid, change: IssueFieldChange }`, `ReorderIssue { id: Uuid, new_sort_key: f64 }`, `CreateIssue { id: Uuid, project_id: Uuid, status_id: Uuid, … }`). The React side gets UUIDs from already-loaded query results: when the user edits issue `AUTH-12`, `useIssue('AUTH-12')` has already returned the issue with its `id`, and we pass that UUID into the Operation.

## Board mechanics

### Drag and drop

`@dnd-kit/core` + `@dnd-kit/sortable`. One `<DndContext>` at the `<Board>` root. One `<SortableContext>` per column. `useDraggable` on each `<IssueCard>`, the whole card is the activator (`cursor: grab`). On drag start, the card visually lifts (rotation -1deg + soft blue shadow + stronger border).

- **Drop on a different column** → two operations:
  1. `apply(ops.updateIssueField({ id: issue.id, change: change.status(targetStatusId) }))`
  2. `apply(ops.reorderIssue({ id: issue.id, new_sort_key: midpoint(before, after) }))`

  Each lands in its own undo entry by design — the user can undo the position change without undoing the status change.

- **Drop within the same column** → one operation:
  - `apply(ops.reorderIssue({ id: issue.id, new_sort_key: midpoint(before, after) }))`

### Sort key strategy

`Issue.sort_key: f64` already exists in core. Reorder = midpoint between neighbors:

```ts
// ui/src/lib/sortKey.ts
export function midpoint(before?: number, after?: number): number {
  if (before == null && after == null) return 1024;       // empty column
  if (before == null) return after!  - 1024;              // dropped at top
  if (after  == null) return before  + 1024;              // dropped at bottom
  return (before + after) / 2;                            // between
}
```

f64 precision affords ~50 reorders before adjacent cards' sort keys collide. Once a column hits collision risk, the user keeps dragging without visible failure (the core's race-safe row update still wins) but visual ordering becomes ambiguous. **Rebalance** is **out of scope** for Spec #2 — a `RebalanceColumn` operation lands in Spec #3 alongside the labels UI. A code comment in `lib/sortKey.ts` records this debt explicitly.

### "+ Add issue" inline form

Clicking the dashed `+ Add issue` button at the bottom of a column swaps the button for a single-line input. Enter submits via:

```ts
apply(ops.createIssue({
  id: uuidv7(),                      // generated client-side; core asserts uniqueness
  project_id: project.id,
  title,
  description: null,
  status_id: column.status_id,
  priority: 'medium',
  due_date: null,
  label_ids: [],
}))
```

Sort key is **not** caller-supplied — `kanban-core`'s `CreateIssue` handler appends to the bottom of the target status's column automatically, mirroring the CLI behavior shipped in Spec #1. Esc cancels. No modal, no other fields — those edits happen in the detail panel after creation.

## Detail panel mechanics

### Edit-in-place

All edits go through `apply(ops.updateIssueField({ id: issue.id, change: change.<field>(value) }))`:

- **Title** — click anywhere on the `<h3>` swaps it for an `<input autoFocus>` containing the current value. Enter or blur commits `change.title(value)`. Esc reverts. Empty title is rejected client-side (zod) before reaching `apply`.
- **Status** — `<select>` in the panel header. `onChange` commits `change.status(statusId)`.
- **Priority** — same dropdown pattern, commits `change.priority(value)`.
- **Description** — explicit Edit button; click swaps the rendered markdown for a monospace `<textarea>`. Save commits `change.description(value)`, Cancel reverts. No live preview in Spec #2.

### Markdown rendering

`react-markdown@9` + `remark-gfm` (tables, strikethrough, task lists, autolinks). No plugins for syntax highlighting, math, or mermaid. Anchor clicks → `tauri-plugin-shell.open()` so the system browser opens, not an in-WebView navigation. Tailwind prose tweaks live in `ui/src/styles.css` alongside the theme tokens.

### Panel close

`Esc` keypress, ✕ button, or click on the dimmed-board area all call `navigate('/p/${prefix}')`. The dimmed area gets `pointer-events: auto` and an `onClick={close}`. The board never unmounts during this transition; only the overlay route does.

## Theme system

State machine:

```
settings.theme ∈ { 'system', 'light', 'dark' }     # persisted in workspace_settings
                          ↓
                   ThemeProvider
                          ↓
       if 'system' → matchMedia('(prefers-color-scheme: dark)')
                          ↓
              <html class="dark" | "">
                          ↓
              Tailwind v4 dark: variants resolve
```

`useSystemTheme()` subscribes to `matchMedia('(prefers-color-scheme: dark)')` on mount and returns `'light' | 'dark'`. ThemeProvider reads it only when `settings.theme === 'system'`. `<ThemeToggle>` cycles `system → light → dark → system`; on each click it calls `useUpdateSettings({ theme })`.

Tailwind v4 darkMode is `class`-based by default. Tokens in `ui/src/styles.css`:

```css
@import "tailwindcss";

:root {
  --color-accent: oklch(67% 0.18 250);
  --color-accent-fg: white;
  /* … */
}

.dark {
  --color-accent: oklch(72% 0.16 250);
  /* … */
}
```

shadcn primitives have dark variants out of the box; theme tokens above only override accent/highlight colors that need to feel "ours."

## Tauri window + title bar

`tauri.conf.json`:

```json
{
  "app": {
    "windows": [{
      "title": "kanban",
      "width": 1200, "height": 760, "minWidth": 720, "minHeight": 480,
      "decorations": false,
      "titleBarStyle": "Overlay",
      "trafficLightPosition": { "x": 16, "y": 18 }
    }]
  }
}
```

`<TitleBar>` is a styled `<header data-tauri-drag-region>` with the workspace name on the left, theme toggle on the right. macOS traffic-light buttons remain native (Tauri 2 handles them) and sit inside the `padding-left: 76px` region of the bar.

## Multi-process behavior

The GUI does **not** auto-refresh on external DB mutations.

- The GUI's own mutations invalidate query keys per the table above (instant feedback).
- Window focus → TanStack Query refetches every active query (refetchOnWindowFocus).
- CLI mutations from another terminal show up the next time the GUI window receives focus.

A SQLite `update_hook` watcher thread is **explicitly out of scope** for Spec #2 (deferred to Spec #3 alongside labels UI and the rebalance op). Documented in CLAUDE.md after Spec #2 ships.

## Error handling

- `commands.apply(op)` returns `Result<ApplyResult, ApiError>`.
- Mutation `onError` rolls back the optimistic patch and pushes a toast for `NotFound | Conflict | Internal` errors.
- `Validation` errors carrying a `field` get caught upstream by the form's zod resolver — the field gets a red border and inline message; no toast.
- React Query `useQuery` errors render an inline retry placeholder ("couldn't load — retry") in the affected pane (sidebar, board, detail). No global error boundary swallowing things silently.

## Testing

| Layer | Tool | Coverage |
|---|---|---|
| Pure TS logic | Vitest | `lib/sortKey.ts` (empty/top/bottom/between cases), `data/ops.ts` (every builder produces the right `Operation` shape), `useSystemTheme` (matchMedia mock + change events), `client.ts` ApiError handling. |
| Components | Vitest + React Testing Library | `<Sidebar>` (renders projects, opens new-project dialog, submits create, surfaces validation), `<Board>` (column rendering, drag emits the right Operation pair, status change works), `<IssueCard>` (click navigates, drag activates), `<IssuePanel>` (renders markdown, click-to-edit title commits, status change commits, Esc closes), `<ThemeProvider>` (toggles `dark` class on system change, persists override). Tauri commands stubbed via `vi.mock('@/data/bindings')`. |
| End-to-end | Playwright + `tauri-driver` | One **happy-path smoke**: launch → create project → create issue → drag to next status → open detail → edit title → undo → close panel → quit. One **theme persistence**: toggle theme to dark → quit → relaunch → assert `<html class="dark">`. |

CI gate fails on red. The TDD discipline mirrors Spec #1: every Operation builder lands as a failing test first; every component has happy + at least one error path.

If the `tauri-driver` harness ratholes past one task in the implementation plan, fall back to Vitest+RTL only for Spec #2 and file E2E as a Spec #3 prerequisite. The plan budgets that decision point explicitly.

## CI workflow

`.github/workflows/ci.yml` extensions on top of Spec #1's gate:

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `pnpm install --frozen-lockfile` (in `ui/`)
5. `pnpm typecheck` (tsc --noEmit)
6. `pnpm lint` (eslint + prettier check)
7. `pnpm test:unit` (Vitest)
8. `pnpm tauri build` — validates the Rust↔TS bridge compiles end-to-end
9. `pnpm test:e2e` — Playwright/tauri-driver, **macOS-only matrix**, runs on `pull_request` only (not push) to keep inner-loop fast.

## Bundling and release (macOS only)

`crates/kanban-tauri/tauri.conf.json` bundle config:

```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "identifier": "com.kanban.app",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns"],
    "macOS": {
      "minimumSystemVersion": "11.0",
      "exceptionDomain": null,
      "frameworks": null
    }
  }
}
```

**First ship is unsigned.** DEVELOPMENT.md documents the install workaround:

```sh
xattr -dr com.apple.quarantine /Applications/kanban.app
```

Apple Developer signing lands in Spec #3 once the distribution cadence is decided.

`.github/workflows/release.yml` triggers on tag `v2.0.0-alpha+`:
- `pnpm tauri build --target universal-apple-darwin`
- Upload `.dmg` and `.app.tar.gz` to the GitHub Release
- No Homebrew tap automation (deferred to Spec #3).

## Documentation updates

- **README.md** — new "Desktop app" section with a screenshot and `gh release download` instructions. Existing CLI section preserved.
- **DEVELOPMENT.md** — new sections: `pnpm tauri dev`, test commands (`pnpm test:unit`, `pnpm test:e2e`), bundle command (`pnpm tauri build`), unsigned-app `xattr` workaround, tauri-driver setup notes.
- **CLAUDE.md** — append a "Tauri layer" section under architecture invariants documenting the spawn_blocking pattern, the workspace_settings exception, and the ui/ vs crates/ split.

## Acceptance criteria

The spec is shippable when all of the following hold:

1. `pnpm tauri dev` launches the GUI on an empty `~/.kanban/data.db`. The schema (including the new `0002_workspace_settings`) is created automatically. The app shows the empty "select or create a project" state.
2. Sidebar lets you create a project (validating the prefix). Board lets you create issues, drag between columns, drag within a column, click to open the detail panel, edit title via click-to-edit, edit status via dropdown, edit description via the Edit button. All optimistic, all roll back on `ApiError`.
3. Theme toggle cycles system / light / dark; the `<html class="dark">` flips correctly; relaunching the app preserves the choice.
4. Running `kanban project create FOO` from another terminal then refocusing the GUI window shows the new project.
5. `pnpm tauri build` produces a runnable `kanban.app` bundle on macOS that boots from a clean `~/.kanban/`.
6. CI is green: cargo (fmt + clippy + test), pnpm (typecheck + lint + Vitest unit), `pnpm tauri build`, `pnpm test:e2e` (Playwright + tauri-driver smoke).

## Out of scope (deferred to Spec #3+)

- Labels UI — viewing/creating/attaching labels via GUI surface (cards already display label dots).
- Multi-views (list view, calendar) — board is the only view.
- ⌘Z / ⌘⇧Z keyboard shortcuts — the Rust commands exist, no UI binding yet.
- Search bar — `ListIssuesFilter.search` exists in the API but no UI input.
- Cross-platform releases — Linux/Windows.
- Code signing.
- SQLite `update_hook` cross-process watcher.
- Sort-key rebalance operation.
- Hierarchy display, members UI, templates, custom statuses UI, notifications, hooks, agents, audit log UI.
- Syntax highlighting in markdown.
- Postgres path (#30).
- Issue attachments.
