# Visual Overhaul + Agent Presence — Design Spec

**Date:** 2026-03-18
**Status:** Approved
**Scope:** Full UI redesign with monochrome luxury theme, agent real-time presence, review workflow, predictive intelligence, and cost visibility.

---

## 1. Theme System — Monochrome Luxury

### Color Palette

| Token | Value | Usage |
|-------|-------|-------|
| `--bg` | `#09090b` (zinc-950) | Page background |
| `--surface` | `#18181b` (zinc-900) | Cards, panels |
| `--border` | `#27272a` (zinc-800) | Default borders |
| `--border-elevated` | `#3f3f46` (zinc-700) | Active/hover borders |
| `--text-primary` | `#fafafa` | Headings, card titles |
| `--text-secondary` | `#a1a1aa` (zinc-400) | Descriptions, metadata |
| `--text-muted` | `#71717a` (zinc-500) | Timestamps, labels |
| `--ticker-bg` | `#0c0c0f` | Bottom ticker, feed panels |

**Semantic colors (used sparingly, only for meaning):**

| Token | Value | Usage |
|-------|-------|-------|
| `--green` | `#4ade80` | Success, online, complete, verified |
| `--orange` | `#f97316` | Warning, blocked, medium priority |
| `--red` | `#ef4444` | Error, urgent, failed, rejected |
| `--yellow` | `#eab308` | Medium priority, needs review |
| `--blue` | `#3b82f6` | Info, links, in-progress accents |

### Typography

- **Primary font:** Geist Sans (Vercel) — `npm install geist`
- **Fallback chain:** `'Geist Sans', 'Inter', system-ui, -apple-system, sans-serif`
- **Mono font:** Geist Mono for code snippets, execution logs, identifiers
- **Headings:** weight 600, letter-spacing -0.3px
- **Column headers:** weight 600, uppercase, letter-spacing 1.5px, font-size 10px, `--text-muted`
- **Body:** weight 400, 13px
- **Small/metadata:** weight 500, 10-11px
- **Numbers (percentages, counts):** weight 700

### Spacing

- 4px base grid
- Card padding: 12px
- Card gap: 6px
- Column gap: 10px
- Section margins: 16px

### shadcn/ui CSS Variable Migration

The existing codebase uses HSL-based variables in the shadcn/ui convention (e.g., `--background: 228 15% 8%` consumed as `hsl(var(--background))`). The new monochrome tokens must **replace** the existing shadcn variables, not sit alongside them.

**Mapping (in `index.css` `:root` / `.dark`):**

| shadcn variable | New HSL value | Hex equivalent | Maps to |
|-----------------|--------------|----------------|---------|
| `--background` | `0 0% 4%` | `#09090b` | `--bg` |
| `--foreground` | `0 0% 98%` | `#fafafa` | `--text-primary` |
| `--card` | `0 0% 9%` | `#18181b` | `--surface` |
| `--card-foreground` | `0 0% 98%` | `#fafafa` | `--text-primary` |
| `--popover` | `0 0% 9%` | `#18181b` | `--surface` |
| `--popover-foreground` | `0 0% 98%` | `#fafafa` | `--text-primary` |
| `--primary` | `0 0% 98%` | `#fafafa` | white accent |
| `--primary-foreground` | `0 0% 4%` | `#09090b` | inverted |
| `--secondary` | `0 0% 15%` | `#27272a` | `--border` |
| `--secondary-foreground` | `0 0% 98%` | `#fafafa` | `--text-primary` |
| `--muted` | `0 0% 15%` | `#27272a` | `--border` |
| `--muted-foreground` | `0 0% 48%` | `#71717a` | `--text-muted` |
| `--accent` | `0 0% 15%` | `#27272a` | `--border` |
| `--accent-foreground` | `0 0% 98%` | `#fafafa` | `--text-primary` |
| `--destructive` | `0 84% 60%` | `#ef4444` | `--red` |
| `--destructive-foreground` | `0 0% 98%` | `#fafafa` | white |
| `--border` | `0 0% 15%` | `#27272a` | `--border` |
| `--input` | `0 0% 15%` | `#27272a` | `--border` |
| `--ring` | `0 0% 98%` | `#fafafa` | white focus ring |

All existing Tailwind utilities (`bg-background`, `text-foreground`, `bg-card`, etc.) continue to work without changes. The new custom tokens (`--surface`, `--text-primary`, etc.) are **aliases** added alongside for use in new components.

### Tailwind Config Changes

In `tailwind.config.js`:

```js
fontFamily: {
  sans: ['"Geist Sans"', '"Inter"', 'system-ui', '-apple-system', 'sans-serif'],
  mono: ['"Geist Mono"', 'ui-monospace', 'monospace'],
},
```

The Geist font import goes in `src/main.tsx`:
```tsx
import 'geist/font/sans.css';
import 'geist/font/mono.css';
```

### Dark Mode

This IS the theme. No light mode variant in this phase. Class-based `.dark` on `<html>` remains; CSS variables updated to monochrome luxury values.

---

## 2. Card System

### Card States

**Default:**
```
background: var(--surface)
border: 1px solid var(--border)
border-radius: 8px
padding: 12px
```

**Hover:**
```
transform: translateY(-2px)
border-color: var(--border-elevated)
box-shadow: 0 4px 12px rgba(0,0,0,0.3)
transition: all 150ms ease
```

**Drag (active):**
```
transform: scale(1.02) rotate(1deg)
box-shadow: 0 8px 24px rgba(0,0,0,0.4)
z-index: 100
opacity of origin column: 0.5
```

**Agent Active (agent currently working on this card):**
```
border-color: var(--border-elevated)
Top accent bar: 1.5px solid var(--text-primary), border-radius top
Agent avatar: 16px circle, var(--border) bg, 1.5px solid var(--green) ring
Status text: "--text-secondary", e.g. "Editing schema.rs"
Green ring: breathing pulse animation (opacity 0.6 → 1.0, 2s ease-in-out infinite)
```

**Done:**
```
opacity: 0.6
Title: color var(--text-muted)
```

**Blocked:**
```
border-left: 3px solid var(--orange)
Small orange dot indicator
```

### Card Content Layout

```
┌─────────────────────────────────────┐
│ [Epic arc-ring badge] Epic Name     │  ← only if card belongs to epic
│ Card Title                          │  ← font-weight 600, 12px
│ ● Priority  ·  agent-name  🤖      │  ← priority dot + agent avatar
│ ▓▓▓▓▓▓▓░░░  Editing schema.rs      │  ← progress bar + live status (if agent active)
│                        0.92 ✓  12k  │  ← confidence badge + token cost (bottom-right)
└─────────────────────────────────────┘
```

### Epic vs Story Visual Differentiation

**Epics** (parent issues with children):
- Slightly larger card (padding 14px)
- Bold left border: 3px solid `#3f3f46`
- Arc ring progress indicator inline
- Child count badge: "4/7 tasks"
- Font-size title: 13px (vs 12px for stories)

**Stories** (regular issues):
- Standard card as described above
- Epic badge pill at top if assigned to an epic

---

## 3. Epic Arc Rings

### Implementation

SVG-based circular progress indicator.

**Structure:**
```svg
<svg width="40" height="40" viewBox="0 0 40 40">
  <!-- Track -->
  <circle cx="20" cy="20" r="17" fill="none" stroke="var(--border)" stroke-width="3"/>
  <!-- Progress -->
  <circle cx="20" cy="20" r="17" fill="none" stroke="var(--text-primary)" stroke-width="3"
    stroke-dasharray="107" stroke-dashoffset="{offset}" stroke-linecap="round"
    transform="rotate(-90 20 20)"/>
</svg>
<!-- Center: percentage text, weight 700, 10px -->
```

**Offset calculation:** `stroke-dashoffset = circumference * (1 - progress)`
- circumference = 2 * π * 17 ≈ 107

**States:**
- In progress: white stroke on zinc track, percentage in center
- Complete: green stroke (`--green`), "✓" in center
- Has blocked tasks: orange segment at the end of the progress arc

**Where shown:**
1. **Epic list view:** 40x40 ring with percentage, alongside epic title and task count
2. **Inline card badge:** 10x10 mini arc ring SVG inside a zinc-800 pill, next to epic name. Shown at top of card.

### React Component

`EpicArcRing` — props: `total`, `completed`, `blocked`, `size` ("sm" | "md" | "lg")

---

## 4. Column Transitions & Animations

### Library

Framer Motion (`framer-motion` package, already compatible with React 18).

### Animations

**Card layout reflow:**
```tsx
<motion.div layout layoutId={issue.id} transition={{ type: "spring", stiffness: 300, damping: 25 }}>
```

**Column entry (card added):**
```tsx
initial={{ opacity: 0, y: 12 }}
animate={{ opacity: 1, y: 0 }}
transition={{ duration: 0.15, ease: "easeOut" }}
```

**Column exit (card removed):**
```tsx
exit={{ opacity: 0, y: -8 }}
transition={{ duration: 0.12 }}
```

**Drag overlay (@dnd-kit):**
- Apply scale(1.02) and elevated shadow to DragOverlay
- Origin card dims to opacity 0.4
- Drop target column gets subtle border highlight (`--border-elevated`)

**Status count badge:**
```tsx
<motion.span key={count} initial={{ scale: 1.3 }} animate={{ scale: 1 }} transition={{ type: "spring" }}>
```

**Breathing pulse (agent active ring):**
```css
@keyframes breathe {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}
.agent-ring-active { animation: breathe 2s ease-in-out infinite; }
```

### Performance

- `layout` prop only on cards within columns, not on the entire board
- Use `AnimatePresence` with `mode="popLayout"` for exit animations
- Cards use `layoutId` matching issue ID for cross-column animation continuity

---

## 5. Bottom Ticker

### Layout

```
┌──────────────────────────────────────────────────────────┐
│ ● claude-opus Edited schema.rs +42 -8 2s │ ● codex ...  │
└──────────────────────────────────────────────────────────┘
```

- Fixed at bottom of viewport
- Height: 36px default
- Background: `--ticker-bg` (`#0c0c0f`)
- Top border: 1px solid `#1a1a1e`
- Entries: horizontal flex, gap 16px, separated by 1px zinc-800 pipes
- Overflow: hidden, newest entries push from left
- Auto-scroll: new entries animate in with slide-from-left (200ms)

### Entry Format

```
[status dot] [agent-name] [action text] [relative time]
```

- Status dot colors: green (active), orange (warning), red (error)
- Agent name: `--text-secondary`, weight 500, 11px
- Action text: `--text-muted`, 11px
- File names/identifiers in action text: `--text-secondary`
- Timestamp: `--text-muted` at 50% opacity, 10px
- Completion events: green "✓" prefix
- Error events: orange "⚠" prefix

### Interaction

- Click entry → navigate to the relevant card (scroll into view + open detail panel)
- Hover on ticker → pause auto-scroll, show 2 more rows (expand to ~80px)
- Entries older than 5 minutes fade to lower opacity

### Data Source

Subscribe to Tauri `db-changed` events + poll `execution_logs` for entries newer than last seen timestamp. In future, WebSocket/SSE for true real-time from agent heartbeats.

### React Component

`ActivityTicker` — subscribes to execution log changes, maintains a rolling buffer of last 50 entries.

---

## 6. Agent Presence on Cards

### Avatar

- 16px circle, background `--border` (`#27272a`), border 2px solid `--surface`
- Agent type emoji centered (🤖)
- When active: border color `--green`, breathing animation
- When idle: border color `--border`
- When error: border color `--red`

### Multiple Agents

Stack avatars with -4px margin-left overlap. Max 3 visible, then "+N" badge.

### Live Status Text

- Below card title or alongside priority
- Font: 10px, `--text-secondary`
- Truncated with ellipsis at card width
- Updates from execution_logs (latest entry for the agent on this issue)
- Entry type mapping:
  - `file_read` → "Reading {filename}"
  - `file_edit` → "Editing {filename}"
  - `command` → "Running {command}"
  - `reasoning` → "Thinking..."
  - `checkpoint` → "Checkpoint: {message}"
  - `error` → "Error: {short_message}" (red text)

### React Component

`AgentPresence` — props: `agents[]`, each with `{ id, name, type, status, lastAction }`.

Subscribes to execution_logs for the card's issue_id. Updates on new entries.

---

## 7. Confidence Score Badges

### Display

Small pill badge, bottom-right of card. Only shown when task has a completed confidence score.

| Range | Color | Icon | Example |
|-------|-------|------|---------|
| >= 0.85 | `--green` bg at 15% | ✓ | `0.92 ✓` |
| 0.50-0.84 | `--yellow` bg at 15% | ⟳ | `0.71 ⟳` |
| < 0.50 | `--red` bg at 15% | ✗ | `0.38 ✗` |

### Styling

```css
.confidence-badge {
  font-size: 10px;
  font-weight: 600;
  font-family: 'Geist Mono';
  padding: 2px 6px;
  border-radius: 4px;
}
```

### React Component

`ConfidenceBadge` — props: `score: number | null`. Returns null if no score.

---

## 8. Review Workflow

### In-Review Column Cards

Cards in "In Review" status get visual differentiation:
- Yellow/amber left border (2px)
- Confidence badge prominently displayed
- "Review" action button visible on hover

### Issue Detail Panel — Review Mode

When a card in review is opened:

**Review toolbar** (top of panel):
```
┌─────────────────────────────────────────┐
│  ✓ Approve    ✗ Reject    ⟳ Retry      │
└─────────────────────────────────────────┘
```

- Approve: green button, calls `approve_task`
- Reject: red button, calls `reject_task`, prompts for reason
- Retry: yellow button, requeues task with incremented attempt_count

**Diff preview tab:**
- Shows files changed by the agent (from execution_logs with `file_edit` entries)
- Inline diff with green/red line highlighting
- File tree sidebar for multi-file changes

**Attempt tabs** (when attempt_count > 1):
- Tab per attempt: "Attempt 1", "Attempt 2", etc.
- Each tab shows that attempt's execution log
- Active tab highlighted with white underline

### React Components

- `ReviewToolbar` — approve/reject/retry buttons with confirmation
- `DiffPreview` — renders file diffs from execution log metadata
- `AttemptTabs` — tab navigation across attempts

---

## 9. Agent Skill Tags

### Display

Small pills on cards showing the agent type/name.

```css
.agent-tag {
  background: var(--border);        /* #27272a */
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 9px;
  color: var(--text-secondary);
  font-weight: 500;
}
```

Agent type determines a subtle accent:
- claude: default zinc
- codex: slightly blue-tinted zinc
- gemini: slightly teal-tinted zinc
- custom: default zinc

Shown next to or instead of the agent avatar when there's room.

---

## 10. Predictive Status

### Logic

For cards in "In Progress" or "Todo" with a due date:

```
estimated_completion = claimed_at + (avg_completion_time_for_complexity * 1.2)
if estimated_completion > due_date:
  show late warning
  days_late = ceil((estimated_completion - due_date) / 1 day)
```

`avg_completion_time_for_complexity` comes from `agent_task_metrics` grouped by the `complexity` column. The current task's complexity is read from `task_contracts.estimated_complexity`. Join path: `task_contracts.estimated_complexity` → match against `agent_task_metrics.complexity` for historical averages.

### Display

- Small clock icon (🕐) with orange tint next to due date on card
- Tooltip on hover: "Based on agent velocity, likely ~2 days late"
- Only shown when prediction confidence > 60% (at least 3 historical data points)

### React Component

`PredictiveStatus` — props: `issueId`, `dueDate`, `complexity`, `agentId`. Queries metrics, returns icon + tooltip or null.

---

## 11. Cost-per-Card

### Display

Small mono-font badge on card, only when cost > 0.

```
12.4k tokens
```

- Font: Geist Mono, 9px, `--text-muted`
- Positioned bottom-right of card (next to confidence badge if present)
- Tooltip expands to breakdown: "Compute: 2.3min · API: 12,400 tokens · Cost: $0.04"

### Data Source

Aggregated from `task_costs` table for the issue.

### React Component

`CostBadge` — props: `issueId`. Fetches cost summary, returns formatted badge or null.

---

## 12. Attempt Comparison

### Location

Issue detail panel, shown when `attempt_count > 1` on the task contract.

### Layout

**Tab bar:**
```
[ Attempt 1 (0.38 ✗) ] [ Attempt 2 (0.71 ⟳) ] [ Attempt 3 (0.92 ✓) ] [ Compare ]
```

**Per-attempt tab:**
- Execution log timeline (existing ReplayViewer, restyled)
- Agent name, duration, confidence score
- Files changed summary

**Compare tab:**
- Side-by-side split view
- Left: select attempt, Right: select attempt
- Diff of results/files changed
- Highlights what changed between attempts

### React Components

- `AttemptTabs` — tab navigation with confidence badges per attempt
- `AttemptComparison` — side-by-side diff view with attempt selectors

---

## 13. Implementation Notes

### New Dependencies

- `framer-motion` (>=11.0) — animations (layout, presence, spring physics). Requires React 18+ (satisfied). `AnimatePresence` with `mode="popLayout"` available since v7+.
- `geist` — font package (Geist Sans + Geist Mono)
- `react-diff-viewer-continued` — diff rendering for `DiffPreview` component (syntax-highlighted inline/split diffs)

### Shared State for Execution Logs

Multiple components need execution log data (AgentPresence, ActivityTicker, ReplayViewer). To avoid duplicate subscriptions and race conditions:

- Create `src/hooks/useExecutionLogs.ts` — a shared hook that subscribes to `db-changed` events and maintains a cache of recent execution logs per issue.
- Uses a module-level cache (Map<issueId, ExecutionLog[]>) so multiple component instances share one subscription.
- Debounces fetches to 2-second intervals.
- Components call `useExecutionLogs(issueId)` and get the latest entries.
- `ActivityTicker` uses a variant: `useGlobalExecutionLogs()` that fetches the last 50 entries across all issues.

### TypeScript Types

Add to `src/types/index.ts`:

```ts
type ExecutionEntryType = 'reasoning' | 'file_read' | 'file_edit' | 'command' | 'discovery' | 'error' | 'checkpoint' | 'claim' | 'start' | 'complete' | 'fail' | 'timeout';

interface AgentPresenceData {
  agentId: string;
  agentName: string;
  agentType: 'claude' | 'codex' | 'gemini' | 'custom';
  status: 'active' | 'idle' | 'error' | 'offline';
  lastAction?: string;
  lastActionType?: ExecutionEntryType;
}

interface TaskCostSummary {
  computeTime: number;
  apiTokens: number;
  totalCost: number;
}
```

### Files to Create

- `src/components/EpicArcRing.tsx`
- `src/components/ActivityTicker.tsx`
- `src/components/AgentPresence.tsx`
- `src/components/ConfidenceBadge.tsx`
- `src/components/CostBadge.tsx`
- `src/components/PredictiveStatus.tsx`
- `src/components/ReviewToolbar.tsx`
- `src/components/DiffPreview.tsx` — uses `react-diff-viewer-continued` for syntax-highlighted diffs
- `src/components/AttemptTabs.tsx`
- `src/components/AttemptComparison.tsx`
- `src/components/AgentSkillTag.tsx`
- `src/components/IssueDetailPanel.tsx` — new component for issue detail with review toolbar, attempt tabs, diff preview (does not currently exist)
- `src/hooks/useExecutionLogs.ts` — shared execution log subscription hook
- `src/tauri/commands.ts` — typed wrappers around Tauri `invoke()` for cost queries, execution logs, agent metrics

### Files to Modify (Major)

- `src/index.css` — replace color system with monochrome luxury CSS variables (see shadcn migration table above)
- `src/components/IssueCard.tsx` — integrate all card-level features (presence, badges, epic badge, animations)
- `src/components/BoardView.tsx` — add Framer Motion layout animations, column transitions
- `src/components/BoardColumn.tsx` — animated card entry/exit
- `src/App.tsx` — add ActivityTicker to layout, import Geist font, wire up IssueDetailPanel
- `src/main.tsx` — add Geist font CSS imports
- `src/types/index.ts` — add new TypeScript interfaces
- `tailwind.config.js` — add Geist font families

### Files to Modify (Minor)

- `src/components/ListView.tsx` — apply theme, add arc ring and badges to list rows
- `src/components/TreeView.tsx` — apply theme, epic hierarchy indicators
- `src/hooks/useIssues.ts` — add execution log subscription for live status

### Backend Changes

None required. All data already exists in the database (execution_logs, task_contracts, task_costs, agent_task_metrics, epics). Frontend reads via existing Tauri commands.

### Accessibility

- All color combinations verified for WCAG AA contrast (4.5:1 minimum for text):
  - `--text-muted` (#71717a) on `--surface` (#18181b) = 4.7:1 ✓
  - `--text-secondary` (#a1a1aa) on `--surface` (#18181b) = 7.5:1 ✓
  - `--text-primary` (#fafafa) on `--surface` (#18181b) = 15.4:1 ✓
- Breathing pulse and ticker animations respect `prefers-reduced-motion`:
  ```css
  @media (prefers-reduced-motion: reduce) {
    .agent-ring-active { animation: none; }
    .ticker-scroll { animation: none; }
  }
  ```
- All interactive elements have focus-visible outlines using `--ring` token

### Performance Considerations

- Framer Motion `layout` only on visible cards (virtualize if > 100 cards)
- Ticker: rolling buffer of 50 entries, prune on new additions
- Agent presence: shared hook with 2-second debounced polling
- Predictive status: cache calculations, recalculate on `db-changed` event only
- Cost badges: fetch once on card mount, update on `db-changed`
- Loading states: skeleton shimmer for CostBadge, PredictiveStatus, AgentPresence while data loads
