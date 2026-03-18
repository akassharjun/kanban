# E2E Testing with Playwright — Design Spec

**Date:** 2026-03-19
**Status:** Approved

## Overview

Add end-to-end tests using Playwright against the Vite dev server (no Tauri runtime). The app already has a complete mock backend (`src/tauri/mock-backend.ts`, ~660 lines) that activates automatically when `window.__TAURI_INTERNALS__` is absent. Tests simply navigate to the Vite dev server and interact with the fully functional browser-mode app. Work is phased into 6 increments, each producing a runnable test suite.

## Architecture

```
e2e/
├── playwright.config.ts          # Config: baseURL localhost:1420, webServer auto-start
├── fixtures/
│   └── test-base.ts              # Extended test fixture (fresh page per test)
├── helpers/
│   └── actions.ts                # Reusable actions (createIssue, openSearch, switchView)
├── phase-1-navigation/
│   └── navigation.spec.ts
├── phase-2-board/
│   └── board.spec.ts
├── phase-3-issue-lifecycle/
│   └── issue-lifecycle.spec.ts
├── phase-4-search-filtering/
│   └── search-filtering.spec.ts
├── phase-5-keyboard-shortcuts/
│   └── keyboard-shortcuts.spec.ts
└── phase-6-settings-members/
    └── settings-members.spec.ts
```

### Key Decisions

- **No custom mocking needed** — the app's existing `mock-backend.ts` provides full in-memory data and command handling when running outside Tauri. Tests run against the real app code with the real mock backend.
- **One spec file per phase** — can split later if tests grow large
- **Shared `test-base.ts`** — extends Playwright's `test` with a fresh page per test for natural state isolation (each page load re-evaluates all modules, resetting the mock backend's in-memory store)
- **`webServer`** in Playwright config auto-starts `npm run dev` so tests are self-contained
- **Chromium only** — single browser target (Tauri uses WebView, not Firefox/Safari)

## Mock Backend (Existing)

The app already has a dual-path architecture:

- `src/tauri/commands.ts` — checks `isTauri` and routes to either real Tauri invoke or `mockInvoke`
- `src/tauri/mock-backend.ts` — complete in-memory backend with seed data and CRUD operations
- `src/tauri/events.ts` — returns no-op in browser mode (events are not needed for e2e tests)

### Seed Data Available (from mock-backend.ts)

- **Projects:** Kanban Core (id: 1, prefix: KAN), Agent Platform (id: 2, prefix: AGT)
- **Statuses for KAN:** Backlog (1), Todo (2), In Progress (3), In Review (4), Done (5)
- **Statuses for AGT:** Backlog (6), Todo (7), In Progress (8), Done (9)
- **Members:** akassharjun/Arjun (1), claude-agent/Claude (2), review-bot/Review Bot (3)
- **Issues:** 14 issues across both projects, spread across statuses with varying priorities, assignees, epics, milestones
- **Labels:** 7 labels (bug, feature, ui, backend, performance, agent, orchestration)
- **Epics, Milestones, Comments, Activity Log** — all seeded

### Test Isolation

Each Playwright test gets a fresh page. A fresh page means a fresh module evaluation, which re-initializes the mock-backend's in-memory store from scratch. No explicit reset logic is needed. Tests MUST NOT share pages across `test()` blocks.

### Event Limitations & Mutation Refresh

The `listen()` function returns a no-op in browser mode — there is no browser-mode event bus. This means `db-changed` events cannot be simulated.

**How mutations reflect in the UI without events:** The React hooks (useIssues, useStatuses, etc.) call commands via `invoke()` which routes to `mockInvoke`. The mock backend updates its in-memory store synchronously. However, React components won't automatically know about the change — they need to re-fetch. The app's hooks typically re-fetch after mutations (optimistic updates or explicit refetch). If a test's mutation does NOT reflect in the UI, the test should trigger a re-fetch by:
1. Closing and reopening a panel/dialog
2. Navigating away and back
3. Using `page.reload()` as a last resort

All Phase 2 and Phase 3 board tests assume the KAN project (id: 1) is selected, which is the default.

**Mock command coverage:** Before implementing each phase, verify that all commands exercised by the tests are handled in `mockInvoke` (line ~384 of mock-backend.ts). If a command returns undefined, it means the mock doesn't handle it yet — add it to the mock before writing the test.

## Playwright Configuration

```typescript
// playwright.config.ts
{
  testDir: './e2e',
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
}
```

**Package.json scripts:**
- `"test:e2e": "npx playwright test"`
- `"test:e2e:ui": "npx playwright test --ui"`
- `"test:e2e:headed": "npx playwright test --headed"`

## Helper Functions (`actions.ts`)

Reusable multi-step interactions used across phases:

```typescript
// Navigate and wait for app to load
appReady(page): Promise<void>

// Sidebar navigation
navigateTo(page, target: 'project' | 'members' | 'settings' | 'agents'): Promise<void>

// View switching
switchView(page, view: 'board' | 'list' | 'tree'): Promise<void>

// Issue creation via dialog
createIssue(page, opts: { title: string; priority?: string; status?: string }): Promise<void>

// Open issue detail panel by clicking a card
openIssue(page, identifier: string): Promise<void>

// Search via Cmd+K
openSearch(page): Promise<void>
searchFor(page, query: string): Promise<void>
```

## Phases

### Phase 1 — Foundation + Navigation

**Scope:** Infrastructure setup (config, fixtures, helpers) + basic navigation tests.

**Tests:**
- App renders without errors, sidebar visible with project name "Kanban Core"
- Click sidebar to navigate to Members page → member list visible
- Click sidebar to navigate to Settings page → settings content visible
- Click sidebar to navigate to Agents page → agents content visible
- Click view switcher → Board/List/Tree views each render correctly
- Cmd+B toggles sidebar visibility

### Phase 2 — Board Interactions

**Scope:** Board view rendering and card interactions.

**Tests:**
- Board shows 5 status columns for KAN project (Backlog, Todo, In Progress, In Review, Done)
- Issue cards appear in their correct status columns (e.g., KAN-6 in "In Progress")
- Cards display priority indicator, assignee avatar, identifier
- Click card → IssueDetailPanel opens with correct title and description
- Close detail panel via Escape key
- Close detail panel via clicking overlay/outside

### Phase 3 — Issue Lifecycle

**Scope:** Full create/edit/transition flow.

**Tests:**
- Open create issue dialog, fill title, submit → new card appears in board
- Open existing issue, edit title → title change reflected after save
- Open existing issue, edit description → description saved
- Change status via dropdown in detail panel → card moves to new column
- Assign member via detail panel → assignee avatar appears on card
- Activity log section shows recorded changes

### Phase 4 — Search & Filtering

**Scope:** Search dialog and filter controls.

**Tests:**
- Cmd+K opens search/command dialog
- Type "drag" → KAN-6 "Fix drag-drop position calculation" appears in results
- Select search result → detail panel opens for that issue
- Apply status filter → only cards with matching status visible
- Apply priority filter → only cards with matching priority visible
- Clear filters → all cards return to view

### Phase 5 — Keyboard Shortcuts

**Scope:** Global keyboard shortcuts (no duplicate of Phase 1's Cmd+B test).

**Tests:**
- C key opens create issue dialog
- 1 key switches to board view
- 2 key switches to list view
- 3 key switches to tree view
- Cmd+Z triggers undo (verify via UI feedback)
- Shift+Cmd+Z triggers redo (verify via UI feedback)
- Shortcuts do NOT fire when focus is in an input/textarea

### Phase 6 — Settings & Members

**Scope:** Administration pages.

**Tests:**
- Settings page shows current project name "Kanban Core"
- Edit project name → change reflected in sidebar
- Members page shows all 3 members (Arjun, Claude, Review Bot)
- Members display name and colored avatar
- Status management: create a new status → appears in board columns
- Status reordering works (reorder and verify column order changes)

## Testing Approach

- Tests use `test.describe()` blocks per feature area
- Each test uses a fresh page (state isolation via module re-initialization)
- Prefer user-visible text and ARIA roles for selectors over implementation details
- Use `data-testid` attributes where semantic selectors aren't sufficient (add to components as needed)
- Helper functions in `actions.ts` encapsulate multi-step interactions
- Tests reference the actual mock-backend seed data (not production DB IDs)

## Not In Scope

- Drag-and-drop testing (dnd-kit interactions are complex to simulate in Playwright; defer to later)
- Tauri-specific features (file system, native menus, window management)
- Cross-browser testing (Chromium only)
- Visual regression testing
- Performance testing
- Real-time event simulation (browser mode has no event bus)
- Calendar, Gantt, and Roadmap view testing (exist in app but excluded from this test suite)
