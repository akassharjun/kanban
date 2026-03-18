/**
 * Mock Tauri backend for browser development/testing.
 * Provides realistic demo data when running outside Tauri.
 */
import type {
  Project, Status, Issue, Label, Member, Comment,
  ActivityLogEntry, Notification, Agent, AgentMetrics,
  ProjectMetrics, Hook,
  ProjectAgentConfig, ExecutionLog, TaskGraph,
  IssueWithLabels,
} from "@/types";

// Check if we're running inside Tauri
export const isTauri = typeof window !== "undefined" && !!(window as any).__TAURI_INTERNALS__;

// ---- Seed data ----

let nextId = 100;
const id = () => ++nextId;

const now = new Date().toISOString();
const ago = (minutes: number) => new Date(Date.now() - minutes * 60_000).toISOString();

const members: Member[] = [
  { id: 1, name: "akassharjun", display_name: "Arjun", email: "arjun@kanban.dev", avatar_color: "#6366f1", created_at: ago(10000) },
  { id: 2, name: "claude-agent", display_name: "Claude", email: null, avatar_color: "#f59e0b", created_at: ago(5000) },
  { id: 3, name: "review-bot", display_name: "Review Bot", email: null, avatar_color: "#10b981", created_at: ago(3000) },
];

const projects: Project[] = [
  { id: 1, name: "Kanban Core", description: "Core kanban board features", icon: "📋", status: "active", prefix: "KAN", issue_counter: 24, path: null, created_at: ago(10000), updated_at: ago(10) },
  { id: 2, name: "Agent Platform", description: "AI agent task orchestration", icon: "🤖", status: "active", prefix: "AGT", issue_counter: 12, path: null, created_at: ago(5000), updated_at: ago(30) },
];

const statuses: Record<number, Status[]> = {
  1: [
    { id: 1, project_id: 1, name: "Backlog", category: "unstarted", color: "#94a3b8", icon: null, position: 0 },
    { id: 2, project_id: 1, name: "Todo", category: "unstarted", color: "#60a5fa", icon: null, position: 1 },
    { id: 3, project_id: 1, name: "In Progress", category: "started", color: "#fbbf24", icon: null, position: 2 },
    { id: 4, project_id: 1, name: "In Review", category: "started", color: "#a78bfa", icon: null, position: 3 },
    { id: 5, project_id: 1, name: "Done", category: "completed", color: "#34d399", icon: null, position: 4 },
  ],
  2: [
    { id: 6, project_id: 2, name: "Backlog", category: "unstarted", color: "#94a3b8", icon: null, position: 0 },
    { id: 7, project_id: 2, name: "Todo", category: "unstarted", color: "#60a5fa", icon: null, position: 1 },
    { id: 8, project_id: 2, name: "In Progress", category: "started", color: "#fbbf24", icon: null, position: 2 },
    { id: 9, project_id: 2, name: "Done", category: "completed", color: "#34d399", icon: null, position: 3 },
  ],
};

const labels: Record<number, Label[]> = {
  1: [
    { id: 1, project_id: 1, name: "bug", color: "#ef4444" },
    { id: 2, project_id: 1, name: "feature", color: "#3b82f6" },
    { id: 3, project_id: 1, name: "ui", color: "#8b5cf6" },
    { id: 4, project_id: 1, name: "backend", color: "#f97316" },
    { id: 5, project_id: 1, name: "performance", color: "#14b8a6" },
  ],
  2: [
    { id: 6, project_id: 2, name: "agent", color: "#f59e0b" },
    { id: 7, project_id: 2, name: "orchestration", color: "#6366f1" },
  ],
};

const issues: Issue[] = [
  // Kanban Core - Backlog
  { id: 1, project_id: 1, identifier: "KAN-1", title: "Add dark/light mode toggle animation", description: "Smooth transition between themes", status_id: 1, priority: "low", assignee_id: null, parent_id: null, position: 0, estimate: 2, due_date: "2026-03-28", created_at: ago(5000), updated_at: ago(100) },
  { id: 2, project_id: 1, identifier: "KAN-2", title: "Implement board column resize", description: "Allow users to resize columns by dragging edges", status_id: 1, priority: "medium", assignee_id: null, parent_id: null, position: 1, estimate: 5, due_date: "2026-03-30", created_at: ago(4800), updated_at: ago(200) },
  // Kanban Core - Todo
  { id: 3, project_id: 1, identifier: "KAN-3", title: "Add keyboard shortcuts help panel", description: "Show a modal with all keyboard shortcuts when user presses ?", status_id: 2, priority: "medium", assignee_id: 1, parent_id: null, position: 0, estimate: 3, due_date: "2026-03-25", created_at: ago(3000), updated_at: ago(50) },
  { id: 4, project_id: 1, identifier: "KAN-4", title: "Issue templates for common workflows", description: "Pre-fill issue fields from templates", status_id: 2, priority: "high", assignee_id: 1, parent_id: null, position: 1, estimate: 5, due_date: "2026-03-22", created_at: ago(2800), updated_at: ago(30) },
  { id: 5, project_id: 1, identifier: "KAN-5", title: "Export board to CSV/JSON", description: null, status_id: 2, priority: "low", assignee_id: null, parent_id: null, position: 2, estimate: null, due_date: null, created_at: ago(2700), updated_at: ago(500) },
  // Kanban Core - In Progress
  { id: 6, project_id: 1, identifier: "KAN-6", title: "Fix drag-drop position calculation", description: "Position sometimes becomes NaN when dropping at list boundaries", status_id: 3, priority: "urgent", assignee_id: 2, parent_id: null, position: 0, estimate: 2, due_date: "2026-03-19", created_at: ago(1000), updated_at: ago(5) },
  { id: 7, project_id: 1, identifier: "KAN-7", title: "Improve issue detail panel UX", description: "## Improvements needed\n- Better spacing between sections\n- Collapsible sections\n- Loading states for async ops", status_id: 3, priority: "high", assignee_id: 1, parent_id: null, position: 1, estimate: 8, due_date: "2026-03-20", created_at: ago(800), updated_at: ago(2) },
  { id: 8, project_id: 1, identifier: "KAN-8", title: "Add comment mentions (@user)", description: null, status_id: 3, priority: "medium", assignee_id: 2, parent_id: 7, position: 0, estimate: 3, due_date: null, created_at: ago(600), updated_at: ago(15) },
  // Kanban Core - In Review
  { id: 9, project_id: 1, identifier: "KAN-9", title: "Implement undo/redo for issue edits", description: "Use Cmd+Z to undo last change", status_id: 4, priority: "high", assignee_id: 2, parent_id: null, position: 0, estimate: 5, due_date: "2026-03-19", created_at: ago(2000), updated_at: ago(60) },
  // Kanban Core - Done
  { id: 10, project_id: 1, identifier: "KAN-10", title: "Setup project with Tauri v2", description: null, status_id: 5, priority: "high", assignee_id: 1, parent_id: null, position: 0, estimate: null, due_date: null, created_at: ago(9000), updated_at: ago(8000) },
  { id: 11, project_id: 1, identifier: "KAN-11", title: "Implement board view with drag-drop", description: null, status_id: 5, priority: "high", assignee_id: 1, parent_id: null, position: 1, estimate: null, due_date: null, created_at: ago(8500), updated_at: ago(7000) },
  { id: 12, project_id: 1, identifier: "KAN-12", title: "Add list and tree views", description: null, status_id: 5, priority: "medium", assignee_id: 1, parent_id: null, position: 2, estimate: null, due_date: null, created_at: ago(7000), updated_at: ago(6000) },
  // Agent Platform
  { id: 13, project_id: 2, identifier: "AGT-1", title: "Design task contract schema", description: null, status_id: 9, priority: "high", assignee_id: 2, parent_id: null, position: 0, estimate: null, due_date: null, created_at: ago(4000), updated_at: ago(3000) },
  { id: 14, project_id: 2, identifier: "AGT-2", title: "Implement agent heartbeat", description: null, status_id: 8, priority: "high", assignee_id: 2, parent_id: null, position: 0, estimate: 3, due_date: "2026-03-21", created_at: ago(3500), updated_at: ago(100) },
];

const issueLabels: Record<number, number[]> = {
  1: [3],       // ui
  2: [2, 3],    // feature, ui
  3: [2, 3],    // feature, ui
  6: [1],       // bug
  7: [3],       // ui
  8: [2],       // feature
  9: [2, 4],    // feature, backend
  14: [6, 7],   // agent, orchestration
};

const comments: Record<number, Comment[]> = {
  6: [
    { id: 1, issue_id: 6, member_id: 2, content: "Reproducing this when you drop an issue below the last item in a column. The `nextPosition` becomes undefined.", created_at: ago(100), updated_at: ago(100) },
    { id: 2, issue_id: 6, member_id: 1, content: "Good find. Let's add a fallback: if no next issue, use `lastPosition + 1`.", created_at: ago(80), updated_at: ago(80) },
  ],
  7: [
    { id: 3, issue_id: 7, member_id: 1, content: "Breaking this into sub-tasks. Starting with collapsible sections.", created_at: ago(400), updated_at: ago(400) },
  ],
  9: [
    { id: 4, issue_id: 9, member_id: 2, content: "Implementation complete. Undo stack stores JSON snapshots before/after mutations. Uses Cmd+Z / Shift+Cmd+Z.", created_at: ago(65), updated_at: ago(65) },
    { id: 5, issue_id: 9, member_id: 3, content: "QA review: tested with create, update, delete, and bulk operations. All working correctly. Approving.", created_at: ago(61), updated_at: ago(61) },
  ],
};

const agents: Agent[] = [
  { id: "claude-opus-1", name: "Claude Opus", agent_type: "implementation", skills: ["rust", "typescript", "react", "sql"], task_types: ["implementation", "review"], max_concurrent: 3, max_complexity: "high", member_id: 2, worktree_path: "/tmp/kanban-wt-1", status: "busy", registered_at: ago(5000), last_heartbeat: ago(1), last_activity_at: ago(2) },
  { id: "review-bot-1", name: "Review Bot", agent_type: "review", skills: ["code-review", "testing"], task_types: ["review", "testing"], max_concurrent: 5, max_complexity: "medium", member_id: 3, worktree_path: null, status: "idle", registered_at: ago(3000), last_heartbeat: ago(5), last_activity_at: ago(60) },
  { id: "research-agent-1", name: "Research Agent", agent_type: "research", skills: ["analysis", "documentation"], task_types: ["research", "decomposition"], max_concurrent: 2, max_complexity: "low", member_id: null, worktree_path: null, status: "offline", registered_at: ago(2000), last_heartbeat: ago(600), last_activity_at: ago(500) },
];

const notifications: Notification[] = [
  { id: 1, type: "mention", issue_id: 6, message: "Claude mentioned you in KAN-6", read: false, created_at: ago(80) },
  { id: 2, type: "status_change", issue_id: 9, message: "KAN-9 moved to In Review", read: false, created_at: ago(60) },
  { id: 3, type: "comment", issue_id: 7, message: "New comment on KAN-7", read: true, created_at: ago(400) },
];

// ---- Mock command handler ----

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function mockInvoke(cmd: string, args?: Record<string, any>): Promise<any> {
  // Small delay to simulate async
  await new Promise(r => setTimeout(r, 30));

  switch (cmd) {
    case "health_check": return "ok";

    // Projects
    case "list_projects": return [...projects];
    case "get_project": return projects.find(p => p.id === args?.id) ?? null;
    case "create_project": {
      const p: Project = { id: id(), ...args!.input, status: "active", issue_counter: 0, path: null, created_at: now, updated_at: now };
      projects.push(p);
      statuses[p.id] = [
        { id: id(), project_id: p.id, name: "Backlog", category: "unstarted", color: "#94a3b8", icon: null, position: 0 },
        { id: id(), project_id: p.id, name: "Todo", category: "unstarted", color: "#60a5fa", icon: null, position: 1 },
        { id: id(), project_id: p.id, name: "In Progress", category: "started", color: "#fbbf24", icon: null, position: 2 },
        { id: id(), project_id: p.id, name: "Done", category: "completed", color: "#34d399", icon: null, position: 3 },
      ];
      labels[p.id] = [];
      return p;
    }
    case "update_project": {
      const p = projects.find(x => x.id === args?.id);
      if (p) Object.assign(p, args!.input, { updated_at: now });
      return p;
    }
    case "delete_project": {
      const idx = projects.findIndex(p => p.id === args?.id);
      if (idx >= 0) projects.splice(idx, 1);
      return;
    }

    // Statuses
    case "list_statuses": return statuses[args?.projectId] ?? [];
    case "create_status": {
      const s: Status = { id: id(), ...args!.input };
      (statuses[s.project_id] ??= []).push(s);
      return s;
    }
    case "update_status": {
      for (const arr of Object.values(statuses)) {
        const s = arr.find(x => x.id === args?.id);
        if (s) { Object.assign(s, args!.input); return s; }
      }
      return null;
    }
    case "delete_status": {
      for (const arr of Object.values(statuses)) {
        const idx = arr.findIndex(x => x.id === args?.id);
        if (idx >= 0) { arr.splice(idx, 1); return; }
      }
      return;
    }
    case "reorder_statuses": return;

    // Issues
    case "list_issues": {
      const f = args?.filter;
      return issues.filter(i => {
        if (f.project_id && i.project_id !== f.project_id) return false;
        if (f.status_id && i.status_id !== f.status_id) return false;
        if (f.priority && i.priority !== f.priority) return false;
        if (f.assignee_id && i.assignee_id !== f.assignee_id) return false;
        if (f.parent_id && i.parent_id !== f.parent_id) return false;
        return true;
      });
    }
    case "create_issue": {
      const proj = projects.find(p => p.id === args!.input.project_id);
      const counter = proj ? ++proj.issue_counter : id();
      const prefix = proj?.prefix ?? "ISS";
      const i: Issue = {
        id: id(), ...args!.input,
        identifier: `${prefix}-${counter}`,
        priority: args!.input.priority ?? "none",
        assignee_id: args!.input.assignee_id ?? null,
        parent_id: args!.input.parent_id ?? null,
        position: args!.input.position ?? issues.filter(x => x.project_id === args!.input.project_id && x.status_id === args!.input.status_id).length,
        estimate: args!.input.estimate ?? null,
        due_date: args!.input.due_date ?? null,
        description: args!.input.description ?? null,
        created_at: now, updated_at: now,
      };
      issues.push(i);
      if (args!.input.label_ids) issueLabels[i.id] = args!.input.label_ids;
      return i;
    }
    case "get_issue": {
      const i = issues.find(x => x.id === args?.id);
      if (!i) return null;
      const il = (issueLabels[i.id] ?? []).map(lid => {
        for (const arr of Object.values(labels)) {
          const l = arr.find(x => x.id === lid);
          if (l) return l;
        }
        return null;
      }).filter(Boolean);
      return { ...i, labels: il } as IssueWithLabels;
    }
    case "get_issue_by_identifier": {
      const i = issues.find(x => x.identifier === args?.identifier);
      if (!i) return null;
      return { ...i, labels: [] } as IssueWithLabels;
    }
    case "update_issue": {
      const i = issues.find(x => x.id === args?.id);
      if (i) Object.assign(i, args!.input, { updated_at: now });
      return i;
    }
    case "delete_issue": {
      const idx = issues.findIndex(x => x.id === args?.id);
      if (idx >= 0) issues.splice(idx, 1);
      return;
    }
    case "duplicate_issue": {
      const orig = issues.find(x => x.id === args?.id);
      if (!orig) return null;
      const dup = { ...orig, id: id(), identifier: orig.identifier + "-dup", created_at: now, updated_at: now };
      issues.push(dup);
      return dup;
    }
    case "bulk_update_issues": {
      const { issue_ids, ...fields } = args!.input;
      return issue_ids.map((iid: number) => {
        const i = issues.find(x => x.id === iid);
        if (i) Object.assign(i, fields, { updated_at: now });
        return i;
      }).filter(Boolean);
    }
    case "search_issues": {
      const q = (args?.query ?? "").toLowerCase();
      return issues.filter(i => i.project_id === args?.projectId && (i.title.toLowerCase().includes(q) || i.identifier.toLowerCase().includes(q)));
    }
    case "get_sub_issues": return issues.filter(i => i.parent_id === args?.parentId);
    case "set_issue_labels": { issueLabels[args!.issueId] = args!.labelIds; return; }
    case "list_issue_label_mappings": {
      const projectId = args?.projectId;
      const mappings: { issue_id: number; label_id: number }[] = [];
      for (const [issueId, labelIds] of Object.entries(issueLabels)) {
        const issue = issues.find(i => i.id === Number(issueId));
        if (issue && issue.project_id === projectId) {
          for (const labelId of labelIds) {
            mappings.push({ issue_id: Number(issueId), label_id: labelId });
          }
        }
      }
      return mappings;
    }
    case "get_activity_log": return [] as ActivityLogEntry[];

    // Members
    case "list_members": return [...members];
    case "create_member": { const m: Member = { id: id(), ...args!.input, avatar_color: args!.input.avatar_color ?? "#6366f1", created_at: now }; members.push(m); return m; }
    case "update_member": { const m = members.find(x => x.id === args?.id); if (m) Object.assign(m, args!.input); return m; }
    case "delete_member": { const idx = members.findIndex(x => x.id === args?.id); if (idx >= 0) members.splice(idx, 1); return; }

    // Labels
    case "list_labels": return labels[args?.projectId] ?? [];
    case "create_label": { const l: Label = { id: id(), ...args!.input }; (labels[l.project_id] ??= []).push(l); return l; }
    case "update_label": { for (const arr of Object.values(labels)) { const l = arr.find(x => x.id === args?.id); if (l) { Object.assign(l, args!.input); return l; } } return null; }
    case "delete_label": { for (const arr of Object.values(labels)) { const idx = arr.findIndex(x => x.id === args?.id); if (idx >= 0) { arr.splice(idx, 1); return; } } return; }

    // Relations
    case "list_relations": return [];
    case "create_relation": return { id: id(), ...args!.input };
    case "delete_relation": return;

    // Templates
    case "list_templates": return [];
    case "create_template": return { id: id(), ...args!.input, created_at: now, updated_at: now };
    case "update_template": return args?.input;
    case "delete_template": return;

    // Undo/Redo
    case "undo": return null;
    case "redo": return null;

    // Notifications
    case "list_notifications": return [...notifications];
    case "unread_notification_count": return notifications.filter(n => !n.read).length;
    case "mark_notification_read": { const n = notifications.find(x => x.id === args?.id); if (n) n.read = true; return; }
    case "mark_all_notifications_read": { notifications.forEach(n => n.read = true); return; }
    case "clear_notifications": { notifications.length = 0; return; }

    // Comments
    case "list_comments": return comments[args?.issueId] ?? [];
    case "create_comment": { const c: Comment = { id: id(), ...args!.input, created_at: now, updated_at: now }; (comments[args!.input.issue_id] ??= []).push(c); return c; }
    case "update_comment": return args?.input;
    case "delete_comment": return;
    case "comment_count": return (comments[args?.issueId] ?? []).length;

    // Custom Fields
    case "list_custom_fields": return [];
    case "get_issue_custom_values": return [];
    case "set_issue_custom_value": return;

    // Agents
    case "list_agents": return [...agents];
    case "agent_metrics_cmd": {
      const a = agents.find(x => x.id === args?.agentId);
      return { agent_id: a?.id ?? "", name: a?.name ?? "", status: a?.status ?? "offline", tasks_completed: 42, tasks_failed: 3, success_rate: 0.93, avg_confidence: 0.87, avg_completion_time_minutes: 12, current_tasks: [], skills_success_rate: {} } as AgentMetrics;
    }
    case "deregister_agent": return;

    // Metrics
    case "project_metrics": return { total_tasks: 14, completed: 3, queued: 5, in_progress: 3, blocked: 0, validating: 1, failed_attempts: 2, agents_online: 1, avg_confidence: 0.87, avg_completion_time_minutes: 15, tasks_completed_24h: 2, task_type_breakdown: { implementation: { count: 8 }, review: { count: 3 }, testing: { count: 2 }, research: { count: 1 } } } as ProjectMetrics;
    case "recent_activity": return [] as ExecutionLog[];
    case "task_replay": return [] as ExecutionLog[];
    case "get_task_contract": return null;
    case "task_graph": return { nodes: [], edges: [] } as TaskGraph;

    // Agent Config
    case "get_project_agent_config": return { project_id: args?.projectId, auto_accept_threshold: 0.95, human_review_threshold: 0.7, max_attempts: 3, heartbeat_interval_seconds: 30, missed_heartbeats_before_offline: 3 } as ProjectAgentConfig;
    case "update_project_agent_config": return args?.input;

    // Hooks
    case "list_hooks": return [] as Hook[];
    case "create_hook": return { id: id(), ...args!.input };
    case "delete_hook": return;

    // Task Contracts
    case "create_task_contract": return { id: id(), ...args!.input, identifier: `TC-${id()}`, task_state: "queued", claimed_by: null, claimed_at: null, attempt_count: 0, created_at: now, updated_at: now };
    case "next_task": return null;
    case "start_task": return;
    case "complete_task": return;
    case "fail_task": return;
    case "approve_task": return;
    case "reject_task": return;
    case "unclaim_task": return;
    case "log_task_activity": return;

    default:
      console.warn(`[mock] Unhandled command: ${cmd}`, args);
      return null;
  }
}
