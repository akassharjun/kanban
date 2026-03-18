# Visual Overhaul + Agent Presence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the kanban board with a monochrome luxury theme, real-time agent presence, review workflow, and predictive intelligence — making it the best-looking agent-driven kanban board.

**Architecture:** Replace the existing HSL color system with zinc-based monochrome variables, add Framer Motion for spring-physics animations, create a bottom activity ticker for live agent events, and layer confidence/cost/prediction badges onto cards. All data already exists in the backend — this is purely frontend.

**Tech Stack:** React 18, TypeScript, Tailwind CSS, Framer Motion, Geist font, @dnd-kit, react-diff-viewer-continued, Tauri v2

**Spec:** `docs/superpowers/specs/2026-03-18-visual-overhaul-agent-presence-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/components/EpicArcRing.tsx` | SVG circular progress for epic completion |
| `src/components/ActivityTicker.tsx` | Bottom bar showing live agent events |
| `src/components/AgentPresence.tsx` | Agent avatar + live status on cards |
| `src/components/ConfidenceBadge.tsx` | Score pill badge (green/yellow/red) |
| `src/components/CostBadge.tsx` | Token usage badge on cards |
| `src/components/PredictiveStatus.tsx` | Late-risk clock icon with tooltip |
| `src/components/ReviewToolbar.tsx` | Approve/reject/retry buttons |
| `src/components/DiffPreview.tsx` | File diff viewer for review |
| `src/components/AttemptTabs.tsx` | Tab navigation across task attempts |
| `src/components/AttemptComparison.tsx` | Side-by-side attempt diff |
| `src/components/AgentSkillTag.tsx` | Agent type pill badge |
| `src/hooks/use-execution-logs.ts` | Shared execution log subscription |
| `src/hooks/use-task-contracts.ts` | Task contract data hook |
| `src/hooks/use-task-costs.ts` | Cost data hook |

### Modified Files
| File | Changes |
|------|---------|
| `src/index.css` | Replace color variables with monochrome luxury palette |
| `src/main.tsx` | Add Geist font CSS imports |
| `tailwind.config.ts` | Add Geist font families |
| `src/types/index.ts` | Add ExecutionLog, AgentPresenceData, TaskContract types |
| `src/components/IssueCard.tsx` | Integrate presence, badges, epic badge, animations |
| `src/components/BoardView.tsx` | Add Framer Motion layout animations |
| `src/components/BoardColumn.tsx` | Animated card entry/exit, column styling |
| `src/components/IssueDetailPanel.tsx` | Add review toolbar, attempt tabs |
| `src/App.tsx` | Add ActivityTicker, wire new hooks |
| `src/components/ListView.tsx` | Apply monochrome theme |
| `src/components/TreeView.tsx` | Apply monochrome theme |

---

## Task 1: Install Dependencies & Font Setup

**Files:**
- Modify: `package.json`
- Modify: `src/main.tsx`
- Modify: `tailwind.config.ts`

- [ ] **Step 1: Install new packages**

```bash
cd /home/s7on/Developer/kanban
npm install framer-motion geist react-diff-viewer-continued
```

- [ ] **Step 2: Add Geist font imports to main.tsx**

In `src/main.tsx`, add these imports before `./index.css`:

```tsx
import "geist/font/sans.css";
import "geist/font/mono.css";
```

Full file becomes:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ToastProvider } from "./components/Toast";
import "geist/font/sans.css";
import "geist/font/mono.css";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <App />
    </ToastProvider>
  </React.StrictMode>,
);
```

- [ ] **Step 3: Add Geist font families to Tailwind config**

In `tailwind.config.ts`, add `fontFamily` inside `theme.extend`:

```ts
fontFamily: {
  sans: ['"Geist Sans"', '"Inter"', "system-ui", "-apple-system", "sans-serif"],
  mono: ['"Geist Mono"', "ui-monospace", "monospace"],
},
```

- [ ] **Step 4: Verify dev server starts**

```bash
npm run dev
```

Expected: Vite starts on :1420 without errors.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json src/main.tsx tailwind.config.ts
git commit -m "chore: add framer-motion, geist font, react-diff-viewer"
```

---

## Task 2: Monochrome Luxury Theme

**Files:**
- Modify: `src/index.css`

- [ ] **Step 1: Replace CSS variables in index.css**

Replace the entire `.dark { ... }` block in `src/index.css` with:

```css
.dark {
  --background: 0 0% 4%;
  --foreground: 0 0% 98%;
  --card: 0 0% 9%;
  --card-foreground: 0 0% 98%;
  --popover: 0 0% 9%;
  --popover-foreground: 0 0% 98%;
  --primary: 0 0% 98%;
  --primary-foreground: 0 0% 4%;
  --secondary: 0 0% 15%;
  --secondary-foreground: 0 0% 98%;
  --muted: 0 0% 15%;
  --muted-foreground: 0 0% 48%;
  --accent: 0 0% 15%;
  --accent-foreground: 0 0% 98%;
  --destructive: 0 84% 60%;
  --destructive-foreground: 0 0% 98%;
  --border: 0 0% 15%;
  --input: 0 0% 15%;
  --ring: 0 0% 98%;
  --sidebar: 0 0% 4%;
  --column: 0 0% 6%;

  /* Semantic tokens for new components */
  --surface: 0 0% 9%;
  --border-elevated: 0 0% 25%;
  --text-primary: 0 0% 98%;
  --text-secondary: 0 0% 63%;
  --text-muted: 0 0% 48%;
  --ticker-bg: 0 0% 5%;
  --green: 142 71% 65%;
  --orange: 25 95% 53%;
  --red: 0 84% 60%;
  --yellow: 48 96% 53%;
  --blue: 217 91% 60%;
}
```

- [ ] **Step 2: Add reduced-motion media query**

Append to the `@layer base` block in `src/index.css`:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

- [ ] **Step 3: Add ticker-bg and surface utilities**

Add after the `@layer base` block:

```css
@layer utilities {
  .bg-ticker { background: hsl(var(--ticker-bg)); }
  .bg-surface { background: hsl(var(--surface)); }
  .border-elevated { border-color: hsl(var(--border-elevated)); }
  .text-primary-custom { color: hsl(var(--text-primary)); }
  .text-secondary-custom { color: hsl(var(--text-secondary)); }
  .text-muted-custom { color: hsl(var(--text-muted)); }
}
```

- [ ] **Step 4: Verify the theme renders**

```bash
npm run dev
```

Open http://localhost:1420 — the board should now show the monochrome black/zinc palette. All shadcn components should still work.

- [ ] **Step 5: Commit**

```bash
git add src/index.css
git commit -m "feat: monochrome luxury theme — zinc color system"
```

---

## Task 3: TypeScript Types

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Add new types to src/types/index.ts**

**Important:** `ExecutionLog`, `FullTaskContract`, and `TaskCostSummary` already exist in this file. Do NOT redeclare them. Only append the new types below at the end of the file:

```ts
// Agent presence types (for card UI)
export type ExecutionEntryType =
  | "reasoning"
  | "file_read"
  | "file_edit"
  | "command"
  | "discovery"
  | "error"
  | "checkpoint"
  | "claim"
  | "start"
  | "complete"
  | "fail"
  | "timeout";

export interface AgentPresenceData {
  agentId: string;
  agentName: string;
  agentType: "claude" | "codex" | "gemini" | "custom";
  status: "active" | "idle" | "error" | "offline";
  lastAction?: string;
  lastActionType?: ExecutionEntryType;
}

export interface TickerEntry {
  id: number;
  agentName: string;
  agentId: string;
  action: string;
  entryType: string;
  issueIdentifier: string | null;
  issueId: number;
  timestamp: string;
}
```

**Note on existing types used by later tasks:**
- `ExecutionLog` (line ~307) — use as-is, `entry_type` is `string` not the union type
- `FullTaskContract` (line ~212) — use this instead of creating a new `TaskContract`. Key fields: `complexity` (not `estimated_complexity`), `confidence`, `claimed_by`, `attempt_count`
- `TaskCostSummary` (line ~869) — use as-is with fields: `total_compute_minutes`, `total_tokens`, `total_cost_dollars`

- [ ] **Step 2: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add execution log, task contract, and presence types"
```

---

## Task 4: Shared Execution Log Hook

**Files:**
- Create: `src/hooks/use-execution-logs.ts`

- [ ] **Step 1: Create the shared hook**

**Important:** Use `@/tauri/commands` and `@/tauri/events` wrappers (NOT raw `@tauri-apps/api`). The existing commands are `taskReplay(identifier)` for per-issue logs and `recentActivity(projectId, limit)` for global logs.

```ts
import { useState, useEffect, useCallback } from "react";
import * as api from "@/tauri/commands";
import { listen } from "@/tauri/events";
import type { ExecutionLog, TickerEntry } from "@/types";

// Module-level cache for deduplication
const logCache = new Map<string, ExecutionLog[]>();
const subscribers = new Map<string, Set<() => void>>();

function notifySubscribers(identifier: string) {
  const subs = subscribers.get(identifier);
  if (subs) subs.forEach((cb) => cb());
}

async function fetchLogs(identifier: string): Promise<ExecutionLog[]> {
  try {
    const logs = await api.taskReplay(identifier);
    logCache.set(identifier, logs);
    notifySubscribers(identifier);
    return logs;
  } catch {
    return logCache.get(identifier) || [];
  }
}

export function useExecutionLogs(identifier: string | null) {
  const [logs, setLogs] = useState<ExecutionLog[]>([]);

  useEffect(() => {
    if (identifier === null) return;

    // Initialize from cache
    const cached = logCache.get(identifier);
    if (cached) setLogs(cached);

    // Subscribe
    const cb = () => setLogs(logCache.get(identifier) || []);
    if (!subscribers.has(identifier)) subscribers.set(identifier, new Set());
    subscribers.get(identifier)!.add(cb);

    // Fetch fresh
    fetchLogs(identifier);

    // Listen for DB changes (debounced)
    let timeout: ReturnType<typeof setTimeout>;
    const unlisten = listen("db-changed", () => {
      clearTimeout(timeout);
      timeout = setTimeout(() => fetchLogs(identifier), 2000);
    });

    return () => {
      subscribers.get(identifier)?.delete(cb);
      clearTimeout(timeout);
      unlisten.then((fn) => fn());
    };
  }, [identifier]);

  return logs;
}

export function useGlobalExecutionLogs(projectId: number | null, limit = 50) {
  const [entries, setEntries] = useState<TickerEntry[]>([]);

  const fetchGlobal = useCallback(async () => {
    if (projectId === null) return;
    try {
      const logs = await api.recentActivity(projectId, limit);
      // Map ExecutionLog[] to TickerEntry[]
      const mapped: TickerEntry[] = logs.map((log) => ({
        id: log.id,
        agentName: log.agent_id || "unknown",
        agentId: log.agent_id || "",
        action: log.message,
        entryType: log.entry_type,
        issueIdentifier: null,
        issueId: log.issue_id,
        timestamp: log.timestamp,
      }));
      setEntries(mapped);
    } catch {
      // Silently fail — ticker is non-critical
    }
  }, [projectId, limit]);

  useEffect(() => {
    fetchGlobal();

    let timeout: ReturnType<typeof setTimeout>;
    const unlisten = listen("db-changed", () => {
      clearTimeout(timeout);
      timeout = setTimeout(fetchGlobal, 2000);
    });

    return () => {
      clearTimeout(timeout);
      unlisten.then((fn) => fn());
    };
  }, [fetchGlobal]);

  return entries;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/use-execution-logs.ts
git commit -m "feat: shared execution log hook with dedup and debounce"
```

---

## Task 5: EpicArcRing Component

**Files:**
- Create: `src/components/EpicArcRing.tsx`
- Create: `src/components/__tests__/EpicArcRing.test.tsx`

- [ ] **Step 1: Write the test**

```tsx
// src/components/__tests__/EpicArcRing.test.tsx
import { render, screen } from "@testing-library/react";
import { EpicArcRing } from "../EpicArcRing";

describe("EpicArcRing", () => {
  it("shows percentage for in-progress epic", () => {
    render(<EpicArcRing total={10} completed={7} />);
    expect(screen.getByText("70%")).toBeInTheDocument();
  });

  it("shows checkmark for completed epic", () => {
    render(<EpicArcRing total={4} completed={4} />);
    expect(screen.getByText("✓")).toBeInTheDocument();
  });

  it("shows 0% when no tasks completed", () => {
    render(<EpicArcRing total={5} completed={0} />);
    expect(screen.getByText("0%")).toBeInTheDocument();
  });

  it("renders at small size for inline badge", () => {
    const { container } = render(
      <EpicArcRing total={10} completed={3} size="sm" />,
    );
    const svg = container.querySelector("svg");
    expect(svg).toHaveAttribute("width", "16");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
npm run test:run -- --reporter=verbose src/components/__tests__/EpicArcRing.test.tsx
```

Expected: FAIL — module not found.

- [ ] **Step 3: Create the component**

```tsx
// src/components/EpicArcRing.tsx

interface EpicArcRingProps {
  total: number;
  completed: number;
  blocked?: number;
  size?: "sm" | "md" | "lg";
}

const sizeConfig = {
  sm: { width: 16, radius: 6, stroke: 2, fontSize: "4px", fontWeight: 700 },
  md: { width: 32, radius: 13, stroke: 2.5, fontSize: "8px", fontWeight: 700 },
  lg: { width: 40, radius: 17, stroke: 3, fontSize: "10px", fontWeight: 700 },
};

export function EpicArcRing({
  total,
  completed,
  blocked = 0,
  size = "lg",
}: EpicArcRingProps) {
  const config = sizeConfig[size];
  const cx = config.width / 2;
  const circumference = 2 * Math.PI * config.radius;
  const progress = total > 0 ? completed / total : 0;
  const isComplete = total > 0 && completed === total;
  const offset = circumference * (1 - progress);

  return (
    <div
      style={{
        position: "relative",
        width: config.width,
        height: config.width,
        flexShrink: 0,
      }}
    >
      <svg
        width={config.width}
        height={config.width}
        viewBox={`0 0 ${config.width} ${config.width}`}
      >
        {/* Track */}
        <circle
          cx={cx}
          cy={cx}
          r={config.radius}
          fill="none"
          stroke="hsl(0 0% 15%)"
          strokeWidth={config.stroke}
        />
        {/* Progress */}
        <circle
          cx={cx}
          cy={cx}
          r={config.radius}
          fill="none"
          stroke={isComplete ? "hsl(142 71% 65%)" : "hsl(0 0% 98%)"}
          strokeWidth={config.stroke}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
          transform={`rotate(-90 ${cx} ${cx})`}
          style={{ transition: "stroke-dashoffset 0.5s ease" }}
        />
      </svg>
      {size !== "sm" && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: config.fontSize,
            fontWeight: config.fontWeight,
            color: isComplete ? "hsl(142 71% 65%)" : "hsl(0 0% 98%)",
            fontFamily: '"Geist Mono", monospace',
          }}
        >
          {isComplete ? "✓" : `${Math.round(progress * 100)}%`}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Run tests**

```bash
npm run test:run -- --reporter=verbose src/components/__tests__/EpicArcRing.test.tsx
```

Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/EpicArcRing.tsx src/components/__tests__/EpicArcRing.test.tsx
git commit -m "feat: EpicArcRing component with SVG circular progress"
```

---

## Task 6: ConfidenceBadge Component

**Files:**
- Create: `src/components/ConfidenceBadge.tsx`
- Create: `src/components/__tests__/ConfidenceBadge.test.tsx`

- [ ] **Step 1: Write the test**

```tsx
// src/components/__tests__/ConfidenceBadge.test.tsx
import { render, screen } from "@testing-library/react";
import { ConfidenceBadge } from "../ConfidenceBadge";

describe("ConfidenceBadge", () => {
  it("returns null when score is null", () => {
    const { container } = render(<ConfidenceBadge score={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows green badge for high confidence", () => {
    render(<ConfidenceBadge score={0.92} />);
    expect(screen.getByText(/0\.92/)).toBeInTheDocument();
    expect(screen.getByText(/✓/)).toBeInTheDocument();
  });

  it("shows yellow badge for medium confidence", () => {
    render(<ConfidenceBadge score={0.71} />);
    expect(screen.getByText(/0\.71/)).toBeInTheDocument();
    expect(screen.getByText(/⟳/)).toBeInTheDocument();
  });

  it("shows red badge for low confidence", () => {
    render(<ConfidenceBadge score={0.38} />);
    expect(screen.getByText(/0\.38/)).toBeInTheDocument();
    expect(screen.getByText(/✗/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
npm run test:run -- --reporter=verbose src/components/__tests__/ConfidenceBadge.test.tsx
```

- [ ] **Step 3: Create the component**

```tsx
// src/components/ConfidenceBadge.tsx
import { cn } from "@/lib/utils";

interface ConfidenceBadgeProps {
  score: number | null;
}

export function ConfidenceBadge({ score }: ConfidenceBadgeProps) {
  if (score === null) return null;

  const isHigh = score >= 0.85;
  const isMedium = score >= 0.5 && score < 0.85;

  const colorClass = isHigh
    ? "text-green-400 bg-green-400/15"
    : isMedium
      ? "text-yellow-400 bg-yellow-400/15"
      : "text-red-400 bg-red-400/15";

  const icon = isHigh ? "✓" : isMedium ? "⟳" : "✗";

  return (
    <span
      className={cn(
        "inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 font-mono text-[10px] font-semibold",
        colorClass,
      )}
    >
      {score.toFixed(2)} <span>{icon}</span>
    </span>
  );
}
```

- [ ] **Step 4: Run tests**

```bash
npm run test:run -- --reporter=verbose src/components/__tests__/ConfidenceBadge.test.tsx
```

Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/ConfidenceBadge.tsx src/components/__tests__/ConfidenceBadge.test.tsx
git commit -m "feat: ConfidenceBadge with color-coded score display"
```

---

## Task 7: CostBadge Component

**Files:**
- Create: `src/components/CostBadge.tsx`
- Create: `src/hooks/use-task-costs.ts`

- [ ] **Step 1: Create the cost data hook**

**Important:** Uses `@/tauri/commands` wrapper. The existing `getTaskCostSummary` takes a task identifier string (e.g. "KAN-42"), not an issue ID. The existing `TaskCostSummary` type has fields: `total_compute_minutes`, `total_tokens`, `total_cost_dollars`.

```ts
// src/hooks/use-task-costs.ts
import { useState, useEffect } from "react";
import * as api from "@/tauri/commands";
import { listen } from "@/tauri/events";
import type { TaskCostSummary } from "@/types";

export function useTaskCosts(taskIdentifier: string | null): TaskCostSummary | null {
  const [costs, setCosts] = useState<TaskCostSummary | null>(null);

  useEffect(() => {
    if (taskIdentifier === null) return;

    const fetchCosts = async () => {
      try {
        const result = await api.getTaskCostSummary(taskIdentifier);
        setCosts(result);
      } catch {
        setCosts(null);
      }
    };

    fetchCosts();
    const unlisten = listen("db-changed", () => fetchCosts());
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [taskIdentifier]);

  return costs;
}
```

- [ ] **Step 2: Create the badge component**

**Note:** Uses the existing `TaskCostSummary` type fields (`total_tokens`, `total_cost_dollars`, `total_compute_minutes`).

```tsx
// src/components/CostBadge.tsx
import type { TaskCostSummary } from "@/types";

interface CostBadgeProps {
  costs: TaskCostSummary;
}

function formatTokens(tokens: number): string {
  if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`;
  if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}k`;
  return String(tokens);
}

export function CostBadge({ costs }: CostBadgeProps) {
  if (costs.total_tokens === 0 && costs.total_cost_dollars === 0) return null;

  return (
    <span
      className="inline-flex items-center rounded px-1.5 py-0.5 font-mono text-[9px] text-muted-foreground"
      title={`Compute: ${costs.total_compute_minutes.toFixed(1)}min · Tokens: ${costs.total_tokens.toLocaleString()} · Cost: $${costs.total_cost_dollars.toFixed(2)}`}
    >
      {formatTokens(costs.total_tokens)} tok
    </span>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add src/components/CostBadge.tsx src/hooks/use-task-costs.ts
git commit -m "feat: CostBadge and useTaskCosts hook"
```

---

## Task 8: AgentPresence Component

**Files:**
- Create: `src/components/AgentPresence.tsx`
- Create: `src/components/AgentSkillTag.tsx`

- [ ] **Step 1: Create AgentPresence**

```tsx
// src/components/AgentPresence.tsx
import { cn } from "@/lib/utils";
import type { AgentPresenceData, ExecutionEntryType } from "@/types";

interface AgentPresenceProps {
  agents: AgentPresenceData[];
  maxVisible?: number;
}

function actionLabel(type?: ExecutionEntryType, action?: string): string {
  if (!type || !action) return "";
  switch (type) {
    case "file_read": return `Reading ${action}`;
    case "file_edit": return `Editing ${action}`;
    case "command": return `Running ${action}`;
    case "reasoning": return "Thinking...";
    case "checkpoint": return action;
    case "error": return `Error: ${action}`;
    default: return action;
  }
}

export function AgentPresence({ agents, maxVisible = 3 }: AgentPresenceProps) {
  if (agents.length === 0) return null;

  const visible = agents.slice(0, maxVisible);
  const overflow = agents.length - maxVisible;
  const activeAgent = agents.find((a) => a.status === "active");

  return (
    <div className="flex items-center gap-1.5 mt-1">
      <div className="flex items-center -space-x-1">
        {visible.map((agent) => (
          <div
            key={agent.agentId}
            className={cn(
              "flex h-4 w-4 items-center justify-center rounded-full bg-secondary text-[6px] border-2 border-card",
              agent.status === "active" && "border-green-400 agent-ring-active",
              agent.status === "error" && "border-red-400",
              agent.status === "idle" && "border-border",
            )}
            title={agent.agentName}
          >
            🤖
          </div>
        ))}
        {overflow > 0 && (
          <span className="text-[9px] text-muted-foreground ml-1">
            +{overflow}
          </span>
        )}
      </div>
      {activeAgent?.lastAction && (
        <span className="text-[10px] text-muted-foreground truncate max-w-[120px]">
          {actionLabel(activeAgent.lastActionType, activeAgent.lastAction)}
        </span>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Create AgentSkillTag**

```tsx
// src/components/AgentSkillTag.tsx
import { cn } from "@/lib/utils";

interface AgentSkillTagProps {
  agentType: "claude" | "codex" | "gemini" | "custom";
  name?: string;
}

export function AgentSkillTag({ agentType, name }: AgentSkillTagProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded px-1.5 py-0.5 text-[9px] font-medium bg-secondary text-muted-foreground",
        agentType === "codex" && "text-blue-400/80",
        agentType === "gemini" && "text-teal-400/80",
      )}
    >
      {name || agentType}
    </span>
  );
}
```

- [ ] **Step 3: Add breathing animation to index.css**

Append to the `@layer utilities` block in `src/index.css`:

```css
@keyframes breathe {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}
.agent-ring-active {
  animation: breathe 2s ease-in-out infinite;
}
```

- [ ] **Step 4: Commit**

```bash
git add src/components/AgentPresence.tsx src/components/AgentSkillTag.tsx src/index.css
git commit -m "feat: AgentPresence with breathing ring and AgentSkillTag"
```

---

## Task 9: ActivityTicker Component

**Files:**
- Create: `src/components/ActivityTicker.tsx`

- [ ] **Step 1: Create the ticker**

```tsx
// src/components/ActivityTicker.tsx
import { useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { useGlobalExecutionLogs } from "@/hooks/use-execution-logs";
import type { TickerEntry } from "@/types";
import { motion, AnimatePresence } from "framer-motion";

function dotColor(type: string): string {
  switch (type) {
    case "error":
    case "fail":
      return "bg-red-400";
    case "complete":
      return "bg-green-400";
    case "timeout":
      return "bg-orange-400";
    default:
      return "bg-green-400";
  }
}

function actionText(entry: TickerEntry): string {
  switch (entry.entryType) {
    case "file_edit":
      return `Edited ${entry.action}`;
    case "file_read":
      return `Read ${entry.action}`;
    case "command":
      return `Running ${entry.action}`;
    case "complete":
      return `Completed ${entry.issueIdentifier || "task"}`;
    case "fail":
      return `Failed ${entry.issueIdentifier || "task"}`;
    case "claim":
      return `Claimed ${entry.issueIdentifier || "task"}`;
    case "start":
      return `Started ${entry.issueIdentifier || "task"}`;
    case "reasoning":
      return "Thinking...";
    default:
      return entry.action;
  }
}

function relativeTime(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  return `${Math.floor(mins / 60)}h`;
}

interface ActivityTickerProps {
  projectId: number | null;
  onClickEntry?: (issueId: number) => void;
}

export function ActivityTicker({ projectId, onClickEntry }: ActivityTickerProps) {
  const entries = useGlobalExecutionLogs(projectId, 50);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to left on new entries
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollLeft = 0;
    }
  }, [entries.length]);

  if (entries.length === 0) {
    return (
      <div className="h-9 border-t border-border bg-ticker flex items-center justify-center">
        <span className="text-[11px] text-muted-foreground">
          No agent activity yet
        </span>
      </div>
    );
  }

  return (
    <div className="h-9 border-t border-border bg-ticker flex items-center overflow-hidden group hover:h-20 transition-[height] duration-200">
      <div
        ref={scrollRef}
        className="flex items-center gap-4 px-3 overflow-x-auto scrollbar-none w-full"
      >
        <AnimatePresence mode="popLayout">
          {entries.map((entry) => (
            <motion.div
              key={entry.id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className={cn(
                "flex items-center gap-1.5 shrink-0 cursor-pointer",
                "hover:opacity-100",
              )}
              onClick={() => onClickEntry?.(entry.issueId)}
            >
              <div className={cn("w-1 h-1 rounded-full", dotColor(entry.entryType))} />
              <span className="text-[11px] font-medium text-muted-foreground">
                {entry.agentName}
              </span>
              <span className="text-[11px] text-muted-foreground/60">
                {actionText(entry)}
              </span>
              <span className="text-[10px] text-muted-foreground/30">
                {relativeTime(entry.timestamp)}
              </span>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/ActivityTicker.tsx
git commit -m "feat: ActivityTicker with animated live agent event stream"
```

---

## Task 10: PredictiveStatus Component

**Files:**
- Create: `src/components/PredictiveStatus.tsx`

- [ ] **Step 1: Create the component**

**Important:** Uses `@/tauri/commands` wrapper. The existing command is `getAgentPerformance(agentId)` which returns `AgentPerformance` with `avg_duration_minutes` and `tasks_completed`.

```tsx
// src/components/PredictiveStatus.tsx
import { useState, useEffect } from "react";
import * as api from "@/tauri/commands";
import { Clock } from "lucide-react";

interface PredictiveStatusProps {
  issueId: number;
  dueDate: string | null;
  agentId: string | null;
}

export function PredictiveStatus({
  dueDate,
  agentId,
}: PredictiveStatusProps) {
  const [daysLate, setDaysLate] = useState<number | null>(null);

  useEffect(() => {
    if (!dueDate || !agentId) return;

    const estimate = async () => {
      try {
        const perf = await api.getAgentPerformance(agentId);
        if (perf.tasks_completed < 3) return; // Not enough data

        const due = new Date(dueDate).getTime();
        const now = Date.now();
        const estimatedMs = perf.avg_duration_minutes * 60 * 1000 * 1.2;
        const estimatedCompletion = now + estimatedMs;

        if (estimatedCompletion > due) {
          setDaysLate(
            Math.ceil((estimatedCompletion - due) / (1000 * 60 * 60 * 24)),
          );
        }
      } catch {
        // Silently fail
      }
    };

    estimate();
  }, [dueDate, agentId]);

  if (daysLate === null) return null;

  return (
    <span
      className="inline-flex items-center gap-0.5 text-orange-400"
      title={`Based on agent velocity, likely ~${daysLate} day${daysLate > 1 ? "s" : ""} late`}
    >
      <Clock className="h-3 w-3" />
    </span>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/PredictiveStatus.tsx
git commit -m "feat: PredictiveStatus late-risk indicator"
```

---

## Task 11: Review Workflow Components

**Files:**
- Create: `src/components/ReviewToolbar.tsx`
- Create: `src/components/DiffPreview.tsx`
- Create: `src/components/AttemptTabs.tsx`
- Create: `src/components/AttemptComparison.tsx`

- [ ] **Step 1: Create ReviewToolbar**

```tsx
// src/components/ReviewToolbar.tsx
import { useState } from "react";
import * as api from "@/tauri/commands";
import { Button } from "@/components/ui/button";
import { Check, X, RotateCcw } from "lucide-react";

interface ReviewToolbarProps {
  issueIdentifier: string;
  onActionComplete: () => void;
}

export function ReviewToolbar({
  issueIdentifier,
  onActionComplete,
}: ReviewToolbarProps) {
  const [loading, setLoading] = useState<string | null>(null);

  const handleAction = async (
    action: "approve" | "reject" | "retry",
  ) => {
    setLoading(action);
    try {
      if (action === "approve") {
        await api.approveTask(issueIdentifier);
      } else if (action === "reject") {
        await api.rejectTask(issueIdentifier, "Rejected via review toolbar");
      } else {
        await api.rejectTask(issueIdentifier, "Retry requested");
      }
      onActionComplete();
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="flex items-center gap-2 p-3 border-b border-border bg-card rounded-t-lg">
      <Button
        size="sm"
        variant="outline"
        className="text-green-400 border-green-400/30 hover:bg-green-400/10"
        onClick={() => handleAction("approve")}
        disabled={loading !== null}
      >
        <Check className="h-3.5 w-3.5 mr-1" />
        Approve
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="text-red-400 border-red-400/30 hover:bg-red-400/10"
        onClick={() => handleAction("reject")}
        disabled={loading !== null}
      >
        <X className="h-3.5 w-3.5 mr-1" />
        Reject
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="text-yellow-400 border-yellow-400/30 hover:bg-yellow-400/10"
        onClick={() => handleAction("retry")}
        disabled={loading !== null}
      >
        <RotateCcw className="h-3.5 w-3.5 mr-1" />
        Retry
      </Button>
    </div>
  );
}
```

- [ ] **Step 2: Create AttemptTabs**

```tsx
// src/components/AttemptTabs.tsx
import { cn } from "@/lib/utils";
import { ConfidenceBadge } from "./ConfidenceBadge";

interface Attempt {
  number: number;
  confidence: number | null;
  agentName: string;
}

interface AttemptTabsProps {
  attempts: Attempt[];
  activeAttempt: number;
  onSelect: (attempt: number) => void;
  onCompare?: () => void;
}

export function AttemptTabs({
  attempts,
  activeAttempt,
  onSelect,
  onCompare,
}: AttemptTabsProps) {
  return (
    <div className="flex items-center gap-1 border-b border-border px-3 overflow-x-auto">
      {attempts.map((attempt) => (
        <button
          key={attempt.number}
          onClick={() => onSelect(attempt.number)}
          className={cn(
            "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-colors",
            activeAttempt === attempt.number
              ? "border-foreground text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground",
          )}
        >
          Attempt {attempt.number}
          <ConfidenceBadge score={attempt.confidence} />
        </button>
      ))}
      {attempts.length > 1 && onCompare && (
        <button
          onClick={onCompare}
          className="px-3 py-2 text-xs font-medium text-muted-foreground hover:text-foreground border-b-2 border-transparent"
        >
          Compare
        </button>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Create DiffPreview**

```tsx
// src/components/DiffPreview.tsx
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer-continued";
import type { ExecutionLog } from "@/types";

interface DiffPreviewProps {
  logs: ExecutionLog[];
}

export function DiffPreview({ logs }: DiffPreviewProps) {
  const edits = logs.filter((l) => l.entry_type === "file_edit" && l.metadata);

  if (edits.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        No file changes in this attempt.
      </div>
    );
  }

  return (
    <div className="divide-y divide-border">
      {edits.map((edit) => {
        let meta: { file?: string; old_content?: string; new_content?: string } = {};
        try {
          meta = JSON.parse(edit.metadata || "{}");
        } catch {
          /* ignore */
        }

        return (
          <div key={edit.id} className="p-3">
            <div className="text-xs font-mono text-muted-foreground mb-2">
              {meta.file || "Unknown file"}
            </div>
            <div className="rounded overflow-hidden text-xs">
              <ReactDiffViewer
                oldValue={meta.old_content || ""}
                newValue={meta.new_content || ""}
                splitView={false}
                useDarkTheme
                compareMethod={DiffMethod.LINES}
                styles={{
                  variables: {
                    dark: {
                      diffViewerBackground: "#18181b",
                      addedBackground: "#052e16",
                      removedBackground: "#450a0a",
                      addedColor: "#4ade80",
                      removedColor: "#f87171",
                      wordAddedBackground: "#065f46",
                      wordRemovedBackground: "#7f1d1d",
                    },
                  },
                }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}
```

- [ ] **Step 4: Create AttemptComparison**

```tsx
// src/components/AttemptComparison.tsx
import { useState } from "react";
import { DiffPreview } from "./DiffPreview";
import type { ExecutionLog } from "@/types";

interface AttemptComparisonProps {
  logsByAttempt: Map<number, ExecutionLog[]>;
}

export function AttemptComparison({ logsByAttempt }: AttemptComparisonProps) {
  const attemptNumbers = Array.from(logsByAttempt.keys()).sort();
  const [left, setLeft] = useState(attemptNumbers[0] || 1);
  const [right, setRight] = useState(attemptNumbers[attemptNumbers.length - 1] || 2);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-4 p-3 border-b border-border">
        <label className="text-xs text-muted-foreground">
          Left:
          <select
            value={left}
            onChange={(e) => setLeft(Number(e.target.value))}
            className="ml-1 bg-secondary text-foreground border border-border rounded px-2 py-1 text-xs"
          >
            {attemptNumbers.map((n) => (
              <option key={n} value={n}>
                Attempt {n}
              </option>
            ))}
          </select>
        </label>
        <label className="text-xs text-muted-foreground">
          Right:
          <select
            value={right}
            onChange={(e) => setRight(Number(e.target.value))}
            className="ml-1 bg-secondary text-foreground border border-border rounded px-2 py-1 text-xs"
          >
            {attemptNumbers.map((n) => (
              <option key={n} value={n}>
                Attempt {n}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="flex flex-1 divide-x divide-border overflow-auto">
        <div className="flex-1 overflow-auto">
          <div className="text-xs font-medium text-muted-foreground p-2 border-b border-border">
            Attempt {left}
          </div>
          <DiffPreview logs={logsByAttempt.get(left) || []} />
        </div>
        <div className="flex-1 overflow-auto">
          <div className="text-xs font-medium text-muted-foreground p-2 border-b border-border">
            Attempt {right}
          </div>
          <DiffPreview logs={logsByAttempt.get(right) || []} />
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Commit**

```bash
git add src/components/ReviewToolbar.tsx src/components/AttemptTabs.tsx src/components/DiffPreview.tsx src/components/AttemptComparison.tsx
git commit -m "feat: review workflow — toolbar, attempt tabs, diff preview, comparison"
```

---

## Task 12: Integrate Framer Motion into BoardView

**Files:**
- Modify: `src/components/BoardView.tsx`
- Modify: `src/components/BoardColumn.tsx`
- Modify: `src/components/SortableIssueCard.tsx`

- [ ] **Step 1: Add Framer Motion to BoardColumn**

In `src/components/BoardColumn.tsx`, add import:

```tsx
import { motion, AnimatePresence } from "framer-motion";
```

Wrap each card in the column's render loop with:

```tsx
<AnimatePresence mode="popLayout">
  {issues.map((issue) => (
    <motion.div
      key={issue.id}
      layout
      layoutId={`card-${issue.id}`}
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ type: "spring", stiffness: 300, damping: 25 }}
    >
      {/* existing SortableIssueCard */}
    </motion.div>
  ))}
</AnimatePresence>
```

- [ ] **Step 2: Add animated status count badge**

In the column header where the issue count is shown, wrap it with:

```tsx
<motion.span
  key={issues.length}
  initial={{ scale: 1.3 }}
  animate={{ scale: 1 }}
  transition={{ type: "spring", stiffness: 400, damping: 15 }}
  className="text-xs text-muted-foreground"
>
  {issues.length}
</motion.span>
```

- [ ] **Step 3: Verify board animations work**

```bash
npm run dev
```

Drag a card between columns — it should animate smoothly with spring physics. Column counts should bounce on change.

- [ ] **Step 4: Commit**

```bash
git add src/components/BoardColumn.tsx
git commit -m "feat: Framer Motion card animations and spring physics"
```

---

## Task 13: Integrate New Components into IssueCard

**Files:**
- Modify: `src/components/IssueCard.tsx`

- [ ] **Step 1: Add imports to IssueCard.tsx**

Add these imports at the top:

```tsx
import { EpicArcRing } from "./EpicArcRing";
import { AgentPresence } from "./AgentPresence";
import { ConfidenceBadge } from "./ConfidenceBadge";
import { CostBadge } from "./CostBadge";
import { PredictiveStatus } from "./PredictiveStatus";
import type { AgentPresenceData, FullTaskContract, TaskCostSummary, Epic } from "@/types";
```

Update the `IssueCardProps` interface to accept:

```tsx
interface IssueCardProps {
  issue: Issue;
  member?: Member;
  labels: Label[];
  issues?: Issue[];
  onClick: () => void;
  isDragging?: boolean;
  slaStatus?: SlaStatus;
  epic?: Epic;
  epicProgress?: { total: number; completed: number };
  agentPresence?: AgentPresenceData[];
  taskContract?: FullTaskContract;
  costSummary?: TaskCostSummary | null;
}
```

- [ ] **Step 2: Add epic badge to card top**

After the parent issue breadcrumb, before the title, add:

```tsx
{epic && epicProgress && (
  <div className="flex items-center gap-1.5 mb-1.5">
    <EpicArcRing
      total={epicProgress.total}
      completed={epicProgress.completed}
      size="sm"
    />
    <span className="text-[9px] font-medium text-muted-foreground truncate">
      {epic.title}
    </span>
  </div>
)}
```

- [ ] **Step 3: Add agent presence below title**

After the title `<p>` tag, add:

```tsx
{agentPresence && agentPresence.length > 0 && (
  <AgentPresence agents={agentPresence} />
)}
```

- [ ] **Step 4: Add confidence and cost badges at card bottom**

At the bottom of the card, add:

```tsx
<div className="flex items-center justify-end gap-1.5 mt-2">
  {taskContract?.confidence != null && (
    <ConfidenceBadge score={taskContract.confidence} />
  )}
  {costSummary && costSummary.total_tokens > 0 && (
    <CostBadge costs={costSummary} />
  )}
  {issue.due_date && taskContract && (
    <PredictiveStatus
      issueId={issue.id}
      dueDate={issue.due_date}
      agentId={taskContract.claimed_by}
    />
  )}
</div>
```

- [ ] **Step 5: Add hover and drag styles**

Update the card's className to use monochrome styling:

```tsx
className={cn(
  "group cursor-pointer rounded-lg border border-border bg-card p-3 transition-all duration-150",
  "hover:border-border/80 hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(0,0,0,0.3)]",
  isDragging && "scale-[1.02] rotate-1 shadow-[0_8px_24px_rgba(0,0,0,0.4)] ring-2 ring-foreground/10",
  agentPresence?.some(a => a.status === "active") && "border-[hsl(var(--border-elevated))]",
  slaStatus?.status === "breached" && "border-red-500/40",
  slaStatus?.status === "warning" && "border-yellow-500/40"
)}
```

Add a top accent bar for active agent cards:

```tsx
{agentPresence?.some(a => a.status === "active") && (
  <div className="absolute top-0 left-0 right-0 h-[1.5px] bg-foreground rounded-t-lg" />
)}
```

- [ ] **Step 6: Run tests**

```bash
npm run test:run
```

Expected: All existing tests pass (new props are optional).

- [ ] **Step 7: Commit**

```bash
git add src/components/IssueCard.tsx
git commit -m "feat: integrate presence, badges, epic ring into IssueCard"
```

---

## Task 14: Wire ActivityTicker into App.tsx

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Import ActivityTicker**

Add to App.tsx imports:

```tsx
import { ActivityTicker } from "./components/ActivityTicker";
```

- [ ] **Step 2: Add ticker to layout**

At the very end of the App component's return JSX, just before the closing `</div>`, add:

```tsx
<ActivityTicker
  projectId={selectedProjectId}
  onClickEntry={(issueId) => {
    const issue = issues.find((i) => i.id === issueId);
    if (issue) {
      setSelectedIssueId(issue.id);
    }
  }}
/>
```

- [ ] **Step 3: Verify ticker renders**

```bash
npm run dev
```

A bottom bar should appear showing "No agent activity yet" (or live entries if agents have logged activity).

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat: wire ActivityTicker into main layout"
```

---

## Task 15: Wire Review Components into IssueDetailPanel

**Files:**
- Modify: `src/components/IssueDetailPanel.tsx`

- [ ] **Step 1: Add imports**

Add to IssueDetailPanel.tsx:

```tsx
import { ReviewToolbar } from "./ReviewToolbar";
import { AttemptTabs } from "./AttemptTabs";
import { DiffPreview } from "./DiffPreview";
import { AttemptComparison } from "./AttemptComparison";
import { useExecutionLogs } from "@/hooks/use-execution-logs";
```

- [ ] **Step 2: Add review toolbar for issues in review status**

Inside the panel, check if the issue's status category is "started" and has a task contract in "validating" state. If so, render `<ReviewToolbar>` at the top of the panel.

- [ ] **Step 3: Add attempt tabs when attempt_count > 1**

Below the review toolbar (or below the description), add:

```tsx
{taskContract && taskContract.attempt_count > 1 && (
  <AttemptTabs
    attempts={/* build from execution logs grouped by attempt_number */}
    activeAttempt={activeAttempt}
    onSelect={setActiveAttempt}
    onCompare={() => setShowComparison(true)}
  />
)}
```

- [ ] **Step 4: Add diff preview tab**

Below the attempt tabs, show `<DiffPreview>` for the active attempt's logs.

- [ ] **Step 5: Run the app and verify**

```bash
npm run dev
```

Open an issue that has task contract data. Review toolbar and attempt tabs should render when applicable.

- [ ] **Step 6: Commit**

```bash
git add src/components/IssueDetailPanel.tsx
git commit -m "feat: review workflow in issue detail panel"
```

---

## Task 16: Final Integration & Polish

**Files:**
- Modify: `src/components/ListView.tsx`
- Modify: `src/components/TreeView.tsx`

- [ ] **Step 1: Apply monochrome theme to ListView**

In `ListView.tsx`, update table row styling to use the monochrome palette — `bg-card` for rows, `text-foreground` for text, `border-border` for cell borders. Update hover states to use `hover:bg-secondary`.

- [ ] **Step 2: Apply monochrome theme to TreeView**

Same approach — update TreeView colors to use the monochrome variables.

- [ ] **Step 3: Run full test suite**

```bash
npm run test:run
```

Expected: All tests pass.

- [ ] **Step 4: Run TypeScript check**

```bash
npx tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 5: Visual QA**

```bash
npm run dev
```

Check:
- Board view: monochrome theme, card hover/drag animations
- Cards: epic badges, agent presence indicators
- Bottom ticker: live or placeholder state
- Issue detail: review toolbar (if applicable)
- List view: theme applied
- Tree view: theme applied

- [ ] **Step 6: Commit**

```bash
git add src/components/ListView.tsx src/components/TreeView.tsx
git commit -m "feat: apply monochrome luxury theme to list and tree views"
```

---

## Task 17: Final Commit & Push

- [ ] **Step 1: Run all verification**

```bash
npx tsc --noEmit && npm run test:run
```

Expected: Zero errors, all tests pass.

- [ ] **Step 2: Push**

```bash
source .env
git push https://${GITHUB_PAT}@github.com/akassharjun/kanban.git dev
```
