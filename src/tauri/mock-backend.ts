/**
 * Mock Tauri backend for browser development/testing.
 * Provides realistic demo data when running outside Tauri.
 */
import type {
  Project,
  Status,
  Issue,
  Label,
  Member,
  Comment,
  ActivityLogEntry,
  AuditLogEntry,
  IssueHistoryEntry,
  MentionEntry,
  Notification,
  Agent,
  AgentMetrics,
  ProjectMetrics,
  Hook,
  ProjectAgentConfig,
  ExecutionLog,
  TaskGraph,
  IssueWithLabels,
  Epic,
  Milestone,
  MilestoneWithProgress,
  SavedView,
  GitLink,
  AutomationRule,
  AutomationLogEntry,
  GithubConfig,
  GithubEvent,
  CIStatus,
  PRStatus,
  ConnectionTestResult,
  BranchNamePreview,
  TriageSuggestion,
  DecomposedTask,
  ParsedIssue,
  IssueFileLink,
  FileHeatEntry,
  DirectoryHeatEntry,
  TaskContext,
  RecurringIssue,
  RecurringPreview,
  DependencyGraph,
  DependencyNode,
  DependencyEdge,
  IssueRelation,
  AgentPerformance,
  ProjectAgentSummary,
  AgentRanking,
  AgentRegistryEntry,
  AgentCapability,
  AgentMatch,
  HandoffNote,
  TaskLearning,
  SimilarTaskResult,
  WsjfScore,
  AutoScoreResult,
  Pipeline,
  PipelineRun,
  AgentPermission,
  PermissionPreset,
  PermissionCheckResult,
  TaskCost, TaskCostSummary, ProjectCostSummary, BudgetStatus,
  CostBudget, SlaPolicy, SlaStatus, SlaEvent, SlaDashboard,
  GitStatus, GitCommit, GitBranch, GitWorktree,
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
];

const projects: Project[] = [];

const statuses: Record<number, Status[]> = {};

const labels: Record<number, Label[]> = {};

const issues: Issue[] = [];

const issueLabels: Record<number, number[]> = {};

const customFieldValues: Record<number, { id: number; issue_id: number; field_id: number; value: string | null }[]> = {};

const comments: Record<number, Comment[]> = {};

const agents: Agent[] = [];

const epics: Epic[] = [];

const milestones: Milestone[] = [];

const savedViews: SavedView[] = [];

const starredIssues: { issue_id: number; member_id: number }[] = [];

const recentlyViewed: { issue_id: number; member_id: number; viewed_at: string }[] = [];

const gitLinks: Record<number, GitLink[]> = {};

const fileLinks: Record<number, IssueFileLink[]> = {};

const pipelines: Pipeline[] = [];

const pipelineRuns: PipelineRun[] = [];

const agentPermissions: AgentPermission[] = [];

const permissionPresets: PermissionPreset[] = [];

const notifications: Notification[] = [];

const automationRules: AutomationRule[] = [];

const automationLog: AutomationLogEntry[] = [];

const recurringIssues: RecurringIssue[] = [];

const issueRelations: IssueRelation[] = [];

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
      const p: Project = { id: id(), ...args!.input, status: "active", issue_counter: 0, deleted_at: null, path: args!.input.path ?? null, stale_days: null, stale_close_status_id: null, created_at: now, updated_at: now };
      projects.push(p);
      statuses[p.id] = [
        { id: id(), project_id: p.id, name: "Backlog", category: "unstarted", color: "#94a3b8", icon: null, position: 0 },
        { id: id(), project_id: p.id, name: "Todo", category: "unstarted", color: "#60a5fa", icon: null, position: 1 },
        { id: id(), project_id: p.id, name: "In Progress", category: "started", color: "#fbbf24", icon: null, position: 2 },
        { id: id(), project_id: p.id, name: "In Review", category: "started", color: "#a78bfa", icon: null, position: 3 },
        { id: id(), project_id: p.id, name: "Done", category: "completed", color: "#34d399", icon: null, position: 4 },
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
        epic_id: args!.input.epic_id ?? null,
        milestone_id: args!.input.milestone_id ?? null,
        business_value: args!.input.business_value ?? null,
        time_criticality: args!.input.time_criticality ?? null,
        risk_reduction: args!.input.risk_reduction ?? null,
        job_size: args!.input.job_size ?? null,
        wsjf_score: null,
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
    case "get_activity_log": return [
      { id: 1, issue_id: args?.issueId ?? 0, field_changed: "status_id", old_value: "1", new_value: "3", actor_id: 1, actor_type: "user", timestamp: ago(100) },
      { id: 2, issue_id: args?.issueId ?? 0, field_changed: "priority", old_value: "medium", new_value: "high", actor_id: 2, actor_type: "agent", timestamp: ago(200) },
    ] as ActivityLogEntry[];
    case "get_audit_log": return [
      { id: 1, issue_id: 6, issue_identifier: "KAN-6", issue_title: "Fix drag-drop position calculation", field_changed: "status_id", old_value: "1", new_value: "3", actor_id: 2, actor_type: "agent", actor_name: "Claude", actor_avatar_color: "#f59e0b", timestamp: ago(100) },
      { id: 2, issue_id: 7, issue_identifier: "KAN-7", issue_title: "Improve issue detail panel UX", field_changed: "priority", old_value: "medium", new_value: "high", actor_id: 1, actor_type: "user", actor_name: "Arjun", actor_avatar_color: "#6366f1", timestamp: ago(200) },
    ] as AuditLogEntry[];
    case "get_issue_history": return [
      { id: 1, issue_id: args?.issueId ?? 0, field_changed: "status_id", old_value: "1", new_value: "3", actor_id: 1, actor_type: "user", actor_name: "Arjun", actor_avatar_color: "#6366f1", timestamp: ago(100) },
      { id: 2, issue_id: args?.issueId ?? 0, field_changed: "priority", old_value: "medium", new_value: "high", actor_id: 2, actor_type: "agent", actor_name: "Claude", actor_avatar_color: "#f59e0b", timestamp: ago(200) },
      { id: 3, issue_id: args?.issueId ?? 0, field_changed: "title", old_value: "Old title", new_value: "New title", actor_id: 1, actor_type: "user", actor_name: "Arjun", actor_avatar_color: "#6366f1", timestamp: ago(300) },
    ] as IssueHistoryEntry[];
    case "list_mentions": return [] as MentionEntry[];
    case "search_members_for_mention": {
      const q = (args?.query ?? "").toLowerCase();
      return members.filter(m => m.name.toLowerCase().includes(q) || (m.display_name ?? "").toLowerCase().includes(q));
    }

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
    case "list_relations": return issueRelations.filter(r => r.source_issue_id === args?.issueId || r.target_issue_id === args?.issueId);
    case "create_relation": { const r = { id: id(), ...args!.input } as IssueRelation; issueRelations.push(r); return r; }
    case "delete_relation": { const idx = issueRelations.findIndex(r => r.id === args?.id); if (idx >= 0) issueRelations.splice(idx, 1); return; }
    case "dependency_graph": {
      const pid = args?.projectId;
      const projectIssues = issues.filter(i => i.project_id === pid);
      const projectIssueIds = new Set(projectIssues.map(i => i.id));
      const nodes: DependencyNode[] = projectIssues.map(i => {
        const s = (statuses[pid] ?? []).find(st => st.id === i.status_id);
        const m = members.find(mm => mm.id === i.assignee_id);
        return { id: i.id, identifier: i.identifier, title: i.title, status_category: s?.category ?? "unstarted", priority: i.priority, assignee_name: m?.display_name ?? m?.name ?? null };
      });
      const edges: DependencyEdge[] = issueRelations
        .filter(r => projectIssueIds.has(r.source_issue_id) && projectIssueIds.has(r.target_issue_id))
        .map(r => ({ source_id: r.source_issue_id, target_id: r.target_issue_id, relation_type: r.relation_type }));
      return { nodes, edges } as DependencyGraph;
    }

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
    case "mark_notification_read": { const n = notifications.find(x => x.id === args?.id); if (n) n.read = 1; return; }
    case "mark_all_notifications_read": { notifications.forEach(n => n.read = 1); return; }
    case "clear_notifications": { notifications.length = 0; return; }

    // Comments
    case "list_comments": return comments[args?.issueId] ?? [];
    case "create_comment": { const c: Comment = { id: id(), ...args!.input, created_at: now, updated_at: now }; (comments[args!.input.issue_id] ??= []).push(c); return c; }
    case "update_comment": return args?.input;
    case "delete_comment": return;
    case "comment_count": return (comments[args?.issueId] ?? []).length;

    // Custom Fields
    case "list_custom_fields": {
      const pid = args?.projectId;
      // Return default custom fields for any project that exists
      const proj = projects.find(p => p.id === pid);
      if (proj) return [
        { id: 1, project_id: pid, name: "Sprint", field_type: "select", options: '["Sprint 1","Sprint 2","Sprint 3"]', required: false, position: 0 },
        { id: 2, project_id: pid, name: "Complexity", field_type: "select", options: '["Simple","Medium","Complex"]', required: false, position: 1 },
        { id: 3, project_id: pid, name: "Notes", field_type: "text", options: null, required: false, position: 2 },
      ];
      return [];
    }
    case "get_issue_custom_values": return customFieldValues[args?.issueId] ?? [];
    case "set_issue_custom_value": {
      const iid = args?.issueId as number;
      const fid = args?.fieldId as number;
      const val = args?.value as string | null;
      if (!customFieldValues[iid]) customFieldValues[iid] = [];
      const existing = customFieldValues[iid].find((v: { field_id: number }) => v.field_id === fid);
      if (existing) { existing.value = val; } else { customFieldValues[iid].push({ id: id(), issue_id: iid, field_id: fid, value: val }); }
      return;
    }

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
    case "get_project_agent_config": return { project_id: args?.projectId, auto_accept_threshold: 0.95, human_review_threshold: 0.7, max_attempts: 3, heartbeat_interval_seconds: 30, missed_heartbeats_before_offline: 3, use_wsjf_scoring: false } as ProjectAgentConfig;
    case "update_project_agent_config": return args?.input;

    // Hooks
    case "list_hooks": return [] as Hook[];
    case "create_hook": return { id: id(), ...args!.input };
    case "delete_hook": return;

    // WSJF Scoring
    case "set_wsjf_scores": {
      const i = issues.find(x => x.id === args?.input?.issue_id);
      if (i) {
        const bv = Math.max(1, Math.min(10, args!.input.business_value));
        const tc = Math.max(1, Math.min(10, args!.input.time_criticality));
        const rr = Math.max(1, Math.min(10, args!.input.risk_reduction));
        const size = Math.max(1, Math.min(10, args!.input.job_size));
        i.business_value = bv; i.time_criticality = tc; i.risk_reduction = rr; i.job_size = size;
        i.wsjf_score = (bv + tc + rr) / size;
        return { issue_id: i.id, identifier: i.identifier, title: i.title, business_value: bv, time_criticality: tc, risk_reduction: rr, job_size: size, wsjf_score: i.wsjf_score, priority: i.priority } as WsjfScore;
      }
      return null;
    }
    case "auto_score_issue": {
      const i = issues.find(x => x.id === args?.issueId);
      if (i) {
        const bv = i.priority === "urgent" ? 10 : i.priority === "high" ? 8 : i.priority === "medium" ? 5 : i.priority === "low" ? 2 : 1;
        const tc = 3; const rr = 3; const size = 5;
        i.business_value = bv; i.time_criticality = tc; i.risk_reduction = rr; i.job_size = size;
        i.wsjf_score = (bv + tc + rr) / size;
        return { issue_id: i.id, business_value: bv, time_criticality: tc, risk_reduction: rr, job_size: size, wsjf_score: i.wsjf_score, reasoning: `business_value=${bv} (priority=${i.priority})` } as AutoScoreResult;
      }
      return null;
    }
    case "get_ranked_backlog": {
      return issues
        .filter(i => i.project_id === args?.projectId && i.wsjf_score != null)
        .sort((a, b) => (b.wsjf_score ?? 0) - (a.wsjf_score ?? 0))
        .map(i => ({ issue_id: i.id, identifier: i.identifier, title: i.title, business_value: i.business_value ?? 0, time_criticality: i.time_criticality ?? 0, risk_reduction: i.risk_reduction ?? 0, job_size: i.job_size ?? 0, wsjf_score: i.wsjf_score ?? 0, priority: i.priority })) as WsjfScore[];
    }
    case "auto_score_project": return [] as AutoScoreResult[];
    case "recalculate_scores": return [] as WsjfScore[];

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

    // Epics
    case "list_epics": return epics.filter(e => e.project_id === args?.projectId);
    case "get_epic": return epics.find(e => e.id === args?.id) ?? null;
    case "create_epic": { const e: Epic = { id: id(), ...args!.input, color: args!.input.color ?? "#6366f1", status: "active", created_at: now, updated_at: now }; epics.push(e); return e; }
    case "update_epic": { const e = epics.find(x => x.id === args?.id); if (e) Object.assign(e, args!.input, { updated_at: now }); return e; }
    case "delete_epic": { const idx = epics.findIndex(x => x.id === args?.id); if (idx >= 0) { issues.forEach(i => { if (i.epic_id === args?.id) i.epic_id = null; }); epics.splice(idx, 1); } return; }

    // Milestones
    case "list_milestones": {
      const ms = milestones.filter(m => m.project_id === args?.projectId);
      return ms.map(m => {
        const total = issues.filter(i => i.milestone_id === m.id).length;
        const completed = issues.filter(i => i.milestone_id === m.id && [5, 9].includes(i.status_id)).length;
        return { ...m, total_issues: total, completed_issues: completed } as MilestoneWithProgress;
      });
    }
    case "get_milestone": {
      const m = milestones.find(x => x.id === args?.id);
      if (!m) return null;
      const total = issues.filter(i => i.milestone_id === m.id).length;
      const completed = issues.filter(i => i.milestone_id === m.id && [5, 9].includes(i.status_id)).length;
      return { ...m, total_issues: total, completed_issues: completed } as MilestoneWithProgress;
    }
    case "create_milestone": { const m: Milestone = { id: id(), ...args!.input, status: "open", created_at: now, updated_at: now }; milestones.push(m); return m; }
    case "update_milestone": { const m = milestones.find(x => x.id === args?.id); if (m) Object.assign(m, args!.input, { updated_at: now }); return m; }
    case "delete_milestone": { const idx = milestones.findIndex(x => x.id === args?.id); if (idx >= 0) { issues.forEach(i => { if (i.milestone_id === args?.id) i.milestone_id = null; }); milestones.splice(idx, 1); } return; }

    // Saved Views
    case "list_saved_views": return savedViews.filter(v => v.project_id === args?.projectId);
    case "create_saved_view": {
      const sv: SavedView = { id: id(), ...args!.input, filters: args!.input.filters ?? '{}', sort_direction: args!.input.sort_direction ?? 'asc', view_mode: args!.input.view_mode ?? 'board', created_at: now, updated_at: now };
      savedViews.push(sv);
      return sv;
    }
    case "update_saved_view": {
      const sv = savedViews.find(v => v.id === args?.id);
      if (sv) Object.assign(sv, args!.input, { updated_at: now });
      return sv;
    }
    case "delete_saved_view": {
      const idx = savedViews.findIndex(v => v.id === args?.id);
      if (idx >= 0) savedViews.splice(idx, 1);
      return;
    }

    // Starred Issues
    case "star_issue": {
      const exists = starredIssues.find(s => s.issue_id === args?.issueId && s.member_id === args?.memberId);
      if (!exists) starredIssues.push({ issue_id: args!.issueId, member_id: args!.memberId });
      return;
    }
    case "unstar_issue": {
      const idx = starredIssues.findIndex(s => s.issue_id === args?.issueId && s.member_id === args?.memberId);
      if (idx >= 0) starredIssues.splice(idx, 1);
      return;
    }
    case "list_starred": {
      const starred = starredIssues.filter(s => s.member_id === args?.memberId);
      return starred.map(s => issues.find(i => i.id === s.issue_id)).filter(Boolean);
    }
    case "is_starred": {
      return starredIssues.some(s => s.issue_id === args?.issueId && s.member_id === args?.memberId);
    }

    // Recently Viewed
    case "record_view": {
      const existing = recentlyViewed.find(r => r.issue_id === args?.issueId && r.member_id === args?.memberId);
      if (existing) { existing.viewed_at = now; }
      else { recentlyViewed.push({ issue_id: args!.issueId, member_id: args!.memberId, viewed_at: now }); }
      return;
    }
    case "list_recently_viewed": {
      const limit = args?.limit ?? 10;
      const entries = recentlyViewed
        .filter(r => r.member_id === args?.memberId)
        .sort((a, b) => new Date(b.viewed_at).getTime() - new Date(a.viewed_at).getTime())
        .slice(0, limit);
      return entries.map(r => issues.find(i => i.id === r.issue_id)).filter(Boolean);
    }

    // Advanced Search
    case "advanced_search": {
      const q = (args?.queryString ?? "").toLowerCase();
      // Simple mock: just match title/identifier
      return issues.filter(i => i.project_id === args?.projectId && (i.title.toLowerCase().includes(q) || i.identifier.toLowerCase().includes(q)));
    }
    // Git Links
    case "list_git_links": return gitLinks[args?.issueId] ?? [];
    case "create_git_link": {
      const gl: GitLink = { id: id(), ...args!.input, url: args!.input.url ?? null, pr_number: args!.input.pr_number ?? null, pr_state: args!.input.pr_state ?? null, pr_merged: args!.input.pr_merged ?? false, ci_status: args!.input.ci_status ?? null, review_status: args!.input.review_status ?? null, created_at: now, updated_at: now };
      (gitLinks[args!.input.issue_id] ??= []).push(gl);
      return gl;
    }
    case "update_git_link": {
      for (const arr of Object.values(gitLinks)) {
        const gl = arr.find(x => x.id === args?.id);
        if (gl) { Object.assign(gl, args!.input, { updated_at: now }); return gl; }
      }
      return null;
    }
    case "delete_git_link": {
      for (const arr of Object.values(gitLinks)) {
        const idx = arr.findIndex(x => x.id === args?.id);
        if (idx >= 0) { arr.splice(idx, 1); return; }
      }
      return;
    }
    case "git_link_count": return (gitLinks[args?.issueId] ?? []).length;

    // Stale Issues
    case "update_stale_config": {
      const p = projects.find(x => x.id === args?.projectId);
      if (p) {
        p.stale_days = args!.input.stale_days;
        p.stale_close_status_id = args!.input.stale_close_status_id;
      }
      return;
    }
    case "check_stale_issues": return [];

    // Automations
    case "list_automation_rules": return automationRules.filter(r => r.project_id === args?.projectId);
    case "create_automation_rule": {
      const r: AutomationRule = {
        id: id(), ...args!.input,
        enabled: true, execution_count: 0, last_executed_at: null,
        trigger_config: args!.input.trigger_config ?? "{}",
        conditions: args!.input.conditions ?? "[]",
        actions: args!.input.actions ?? "[]",
        created_at: now, updated_at: now,
      };
      automationRules.push(r);
      return r;
    }
    case "update_automation_rule": {
      const r = automationRules.find(x => x.id === args?.id);
      if (r) Object.assign(r, args!.input, { updated_at: now });
      return r;
    }
    case "delete_automation_rule": {
      const idx = automationRules.findIndex(x => x.id === args?.id);
      if (idx >= 0) automationRules.splice(idx, 1);
      return;
    }
    case "toggle_automation_rule": {
      const r = automationRules.find(x => x.id === args?.id);
      if (r) { r.enabled = args?.enabled; r.updated_at = now; }
      return r;
    }
    case "list_automation_log": return automationLog.filter(l => {
      const rule = automationRules.find(r => r.id === l.rule_id);
      return rule && rule.project_id === args?.projectId;
    }).slice(0, args?.limit ?? 50);

    // GitHub Integration
    case "get_github_config": {
      return { id: 1, project_id: args?.projectId, repo_owner: "akassharjun", repo_name: "kanban", access_token: null, branch_pattern: "{{prefix}}-{{number}}/{{slug}}", auto_link_prs: true, auto_transition_on_merge: true, merge_target_status_id: 5, created_at: ago(1000), updated_at: ago(10) } as GithubConfig;
    }
    case "set_github_config": {
      return { id: 1, project_id: args?.projectId, ...args?.input, created_at: ago(1000), updated_at: now } as GithubConfig;
    }
    case "test_github_connection": {
      return { success: true, message: "Connected to akassharjun/kanban", rate_limit_remaining: 4985 } as ConnectionTestResult;
    }
    case "generate_branch_name": {
      return { branch_name: "KAN-6/fix-drag-drop-position-calculation", pattern: "{{prefix}}-{{number}}/{{slug}}" } as BranchNamePreview;
    }
    case "create_branch_for_issue": {
      const ident = args?.issueIdentifier ?? "KAN-0";
      const iss = issues.find(i => i.identifier === ident);
      const slug = iss ? iss.title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "") : "branch";
      const refName = `${ident.toLowerCase()}/${slug}`;
      const gl: GitLink = { id: id(), issue_id: iss?.id ?? 0, link_type: "branch", url: `https://github.com/akassharjun/kanban/tree/${refName}`, ref_name: refName, pr_number: null, pr_state: null, pr_merged: 0, ci_status: null, review_status: null, created_at: now, updated_at: now };
      (gitLinks[gl.issue_id] ??= []).push(gl);
      return gl;
    }
    case "sync_github_prs": {
      return [
        { id: id(), issue_id: 9, link_type: "pr", url: "https://github.com/akassharjun/kanban/pull/42", ref_name: "KAN-9/implement-undo-redo", pr_number: 42, pr_state: "open", pr_merged: 0, ci_status: "success", review_status: "approved", created_at: ago(60), updated_at: ago(5) },
      ] as GitLink[];
    }
    case "get_pr_status": {
      return { number: 42, title: "Implement undo/redo for issue edits", state: "open", merged: false, review_status: "approved", ci_status: "success", url: "https://github.com/akassharjun/kanban/pull/42", author: "claude-agent" } as PRStatus;
    }
    case "get_ci_status": {
      return { status: "success", checks: [{ name: "build", status: "completed", conclusion: "success", url: "https://github.com/akassharjun/kanban/actions/runs/1" }, { name: "test", status: "completed", conclusion: "success", url: "https://github.com/akassharjun/kanban/actions/runs/2" }] } as CIStatus;
    }
    case "list_github_events": {
      return [
        { id: 1, project_id: args?.projectId, event_type: "pr_opened", issue_id: 6, payload: JSON.stringify({ pr_number: 15, pr_title: "Fix drag-drop position calculation" }), processed: true, created_at: ago(50) },
      ] as GithubEvent[];
    }

    // AI Agent Intelligence
    case "triage_issue": {
      return {
        suggested_priority: "medium",
        suggested_label_ids: [],
        suggested_assignee_id: null,
        suggested_epic_id: null,
        confidence: 0.5,
        reasoning: "Mock triage suggestion",
      } as TriageSuggestion;
    }
    case "auto_triage_and_apply": {
      return {
        suggested_priority: "medium",
        suggested_label_ids: [],
        suggested_assignee_id: null,
        suggested_epic_id: null,
        confidence: 0.5,
        reasoning: "Mock auto-triage applied",
      } as TriageSuggestion;
    }
    case "decompose_issue": {
      return [
        { title: "Sub-task 1", description: null, suggested_priority: null, suggested_labels: [] },
        { title: "Sub-task 2", description: null, suggested_priority: null, suggested_labels: [] },
      ] as DecomposedTask[];
    }
    case "apply_decomposition": {
      return [] as Issue[];
    }
    case "parse_natural_language": {
      return {
        title: args?.text ?? "Parsed issue",
        description: args?.text ?? "",
        suggested_priority: "medium",
        suggested_label_ids: [],
        suggested_assignee_id: null,
      } as ParsedIssue;
    }
    case "create_from_natural_language": {
      const proj = projects.find(p => p.id === args?.projectId);
      const counter = proj ? ++proj.issue_counter : id();
      const prefix = proj?.prefix ?? "ISS";
      const i: Issue = {
        id: id(), project_id: args?.projectId, identifier: `${prefix}-${counter}`,
        title: args?.text ?? "New issue", description: args?.text ?? null,
        status_id: args?.statusId, priority: "medium", assignee_id: null,
        parent_id: null, position: 0, estimate: null, due_date: null,
        epic_id: null, milestone_id: null,
        business_value: null, time_criticality: null, risk_reduction: null, job_size: null, wsjf_score: null,
        created_at: now, updated_at: now,
      };
      issues.push(i);
      return i;
    }

    // Context Assembly
    case "get_task_context": {
      const i = issues.find(x => x.identifier === args?.identifier);
      if (!i) return null;
      const il = (issueLabels[i.id] ?? []).map(lid => {
        for (const arr of Object.values(labels)) {
          const l = arr.find(x => x.id === lid);
          if (l) return l;
        }
        return null;
      }).filter(Boolean);
      return {
        issue: i,
        labels: il,
        parent_issue: i.parent_id ? issues.find(x => x.id === i.parent_id) ?? null : null,
        sub_issues: issues.filter(x => x.parent_id === i.id),
        related_issues: [],
        blocking_issues: [],
        blocked_issues: [],
        comments: comments[i.id] ?? [],
        activity_log: [],
        prior_attempts: [],
        similar_completed_issues: [],
        project_path: null,
        context_files: [],
      } as TaskContext;
    }
    case "get_similar_issues": return [];

    // Code Analysis
    case "link_file_to_issue": {
      const link: IssueFileLink = { id: id(), issue_id: args!.input.issue_id, file_path: args!.input.file_path, link_type: args!.input.link_type ?? "related", created_at: now };
      (fileLinks[args!.input.issue_id] ??= []).push(link);
      return link;
    }
    case "unlink_file_from_issue": {
      const arr = fileLinks[args?.issueId];
      if (arr) {
        const idx = arr.findIndex(l => l.file_path === args?.filePath);
        if (idx >= 0) arr.splice(idx, 1);
      }
      return;
    }
    case "list_file_links": return fileLinks[args?.issueId] ?? [];
    case "get_file_heat_map": {
      const allLinks: IssueFileLink[] = Object.values(fileLinks).flat();
      const pathMap: Record<string, { count: number; bugCount: number; last: string }> = {};
      for (const link of allLinks) {
        const i = issues.find(x => x.id === link.issue_id);
        if (!i || i.project_id !== args?.projectId) continue;
        if (!pathMap[link.file_path]) pathMap[link.file_path] = { count: 0, bugCount: 0, last: link.created_at };
        pathMap[link.file_path].count++;
        const iLabels = issueLabels[i.id] ?? [];
        const bugLabel = (labels[i.project_id] ?? []).find(l => l.name === "bug");
        if (bugLabel && iLabels.includes(bugLabel.id)) pathMap[link.file_path].bugCount++;
        if (link.created_at > pathMap[link.file_path].last) pathMap[link.file_path].last = link.created_at;
      }
      return Object.entries(pathMap)
        .map(([file_path, v]) => ({ file_path, issue_count: v.count, bug_count: v.bugCount, last_issue_at: v.last } as FileHeatEntry))
        .sort((a, b) => b.issue_count - a.issue_count)
        .slice(0, args?.limit ?? 20);
    }
    case "get_directory_heat_map": {
      const allLinks2: IssueFileLink[] = Object.values(fileLinks).flat();
      const dirMap: Record<string, Set<string>> = {};
      for (const link of allLinks2) {
        const i = issues.find(x => x.id === link.issue_id);
        if (!i || i.project_id !== args?.projectId) continue;
        const parts = link.file_path.split("/");
        const dir = parts.slice(0, Math.min(args?.depth ?? 2, parts.length - 1)).join("/") || ".";
        if (!dirMap[dir]) dirMap[dir] = new Set();
        dirMap[dir].add(link.file_path);
      }
      return Object.entries(dirMap)
        .map(([directory, files]) => ({ directory, issue_count: files.size, file_count: files.size } as DirectoryHeatEntry))
        .sort((a, b) => b.issue_count - a.issue_count);
    }
    case "get_issues_for_file": {
      const linkedIssueIds = (Object.values(fileLinks).flat())
        .filter(l => l.file_path === args?.filePath)
        .map(l => l.issue_id);
      return issues.filter(i => linkedIssueIds.includes(i.id) && i.project_id === args?.projectId);
    }

    // Diff Issues
    case "create_issue_from_diff": {
      const proj = projects.find(p => p.id === args!.input.project_id);
      const counter = proj ? ++proj.issue_counter : id();
      const prefix = proj?.prefix ?? "ISS";
      const i: Issue = {
        id: id(),
        project_id: args!.input.project_id,
        identifier: `${prefix}-${counter}`,
        title: args!.input.title,
        description: `**File:** \`${args!.input.file_path}\`\n${args!.input.line_range ? `**Lines:** ${args!.input.line_range}\n` : ""}**Severity:** ${args!.input.severity}\n\n---\n\n${args!.input.description ?? ""}`,
        status_id: args!.input.status_id ?? 1,
        priority: args!.input.severity === "bug" ? "high" : args!.input.severity === "improvement" ? "medium" : "low",
        assignee_id: args!.input.assignee_id ?? null,
        parent_id: null,
        position: issues.filter(x => x.project_id === args!.input.project_id).length,
        estimate: null,
        due_date: null,
        epic_id: null,
        milestone_id: null,
        business_value: null, time_criticality: null, risk_reduction: null, job_size: null, wsjf_score: null,
        created_at: now,
        updated_at: now,
      };
      issues.push(i);
      (fileLinks[i.id] ??= []).push({
        id: id(),
        issue_id: i.id,
        file_path: args!.input.file_path,
        link_type: args!.input.severity === "bug" ? "cause" : "related",
        created_at: now,
      });
      return i;
    }

    // Recurring Issues
    case "list_recurring": return recurringIssues.filter(r => r.project_id === args?.projectId);
    case "create_recurring": {
      const r: RecurringIssue = { id: id(), ...args!.input, label_ids: JSON.stringify(args!.input.label_ids ?? []), recurrence_config: args!.input.recurrence_config ?? "{}", priority: args!.input.priority ?? "medium", assignee_id: args!.input.assignee_id ?? null, description_template: args!.input.description_template ?? null, last_run_at: null, enabled: true, total_created: 0, created_at: now, updated_at: now };
      recurringIssues.push(r);
      return r;
    }
    case "update_recurring": {
      const r = recurringIssues.find(x => x.id === args?.id);
      if (r) Object.assign(r, args!.input, { updated_at: now });
      return r;
    }
    case "delete_recurring": {
      const idx = recurringIssues.findIndex(x => x.id === args?.id);
      if (idx >= 0) recurringIssues.splice(idx, 1);
      return;
    }
    case "toggle_recurring": {
      const r = recurringIssues.find(x => x.id === args?.id);
      if (r) { r.enabled = args!.enabled; r.updated_at = now; }
      return r;
    }
    case "check_recurring": return [] as Issue[];
    case "preview_recurring": {
      const r = recurringIssues.find(x => x.id === args?.id);
      if (!r) return null;
      const today = new Date().toISOString().slice(0, 10);
      const dayName = new Date().toLocaleDateString("en-US", { weekday: "long" });
      const count = r.total_created + 1;
      return {
        title: r.title_template.replace("{{date}}", today).replace("{{count}}", String(count)).replace("{{day}}", dayName),
        description: r.description_template?.replace("{{date}}", today).replace("{{count}}", String(count)).replace("{{day}}", dayName) ?? null,
        next_dates: [r.next_run_at, new Date(Date.now() + 86400000 * 2).toISOString(), new Date(Date.now() + 86400000 * 3).toISOString()],
      } as RecurringPreview;
    }
    // Agent Analytics
    case "record_task_metric": return;
    case "get_agent_performance": {
      const a = agents.find(x => x.id === args?.agentId);
      return {
        agent_id: a?.id ?? "", agent_name: a?.name ?? "Unknown",
        total_tasks: 48, completed: 42, failed: 3, rejected: 2, timeout: 1,
        success_rate: 0.875, avg_confidence: 0.89, avg_duration_minutes: 14.5,
        total_lines_changed: 2847,
        tasks_by_type: { implementation: 28, review: 10, testing: 7, research: 3 },
        tasks_by_complexity: { small: 15, medium: 22, large: 11 },
        daily_completions: Array.from({ length: 14 }, (_, i) => ({
          date: new Date(Date.now() - (13 - i) * 86400000).toISOString().slice(0, 10),
          completed: Math.floor(Math.random() * 5) + 1,
          failed: Math.random() > 0.7 ? 1 : 0,
        })),
      } as AgentPerformance;
    }
    case "get_project_agent_summary": return {
      total_agent_tasks: 96, total_completed: 82, total_failed: 8,
      avg_completion_time_minutes: 16.3, agents_active: 2,
      top_performers: [
        { agent_id: "claude-opus-1", agent_name: "Claude Opus", score: 0.91, success_rate: 0.93, avg_confidence: 0.89, tasks_completed: 42 },
        { agent_id: "review-bot-1", agent_name: "Review Bot", score: 0.78, success_rate: 0.85, avg_confidence: 0.82, tasks_completed: 28 },
        { agent_id: "research-agent-1", agent_name: "Research Agent", score: 0.62, success_rate: 0.75, avg_confidence: 0.71, tasks_completed: 12 },
      ] as AgentRanking[],
      task_type_distribution: { implementation: 45, review: 25, testing: 18, research: 8 },
      completion_trend: Array.from({ length: 14 }, (_, i) => ({
        date: new Date(Date.now() - (13 - i) * 86400000).toISOString().slice(0, 10),
        completed: Math.floor(Math.random() * 8) + 2,
        failed: Math.random() > 0.6 ? Math.floor(Math.random() * 2) + 1 : 0,
      })),
    } as ProjectAgentSummary;
    case "get_agent_leaderboard": return [
      { agent_id: "claude-opus-1", agent_name: "Claude Opus", score: 0.91, success_rate: 0.93, avg_confidence: 0.89, tasks_completed: 42 },
      { agent_id: "review-bot-1", agent_name: "Review Bot", score: 0.78, success_rate: 0.85, avg_confidence: 0.82, tasks_completed: 28 },
      { agent_id: "research-agent-1", agent_name: "Research Agent", score: 0.62, success_rate: 0.75, avg_confidence: 0.71, tasks_completed: 12 },
    ] as AgentRanking[];

    // Marketplace
    case "marketplace_register": return { id: id(), ...args!.input, capabilities: JSON.stringify(args!.input.capabilities), total_tasks: 0, rating: null, registered_at: now, last_seen_at: now } as AgentRegistryEntry;
    case "marketplace_update": return args?.input;
    case "marketplace_deregister": return;
    case "marketplace_list": return [
      { id: 1, agent_id: "claude-opus-1", name: "Claude Opus", description: "High-performance implementation agent with broad language support", provider: "claude", version: "4.0", endpoint: "mcp://localhost:3100", capabilities: JSON.stringify(["rust", "typescript", "react", "sql", "python"]), max_concurrent: 3, max_complexity: "large", hourly_rate: null, rating: 0.91, total_tasks: 42, registered_at: ago(5000), last_seen_at: ago(1) },
      { id: 2, agent_id: "review-bot-1", name: "Review Bot", description: "Specialized code review and testing agent", provider: "claude", version: "3.5", endpoint: "mcp://localhost:3101", capabilities: JSON.stringify(["code-review", "testing", "security-review"]), max_concurrent: 5, max_complexity: "medium", hourly_rate: 0.02, rating: 0.82, total_tasks: 28, registered_at: ago(3000), last_seen_at: ago(5) },
      { id: 3, agent_id: "research-agent-1", name: "Research Agent", description: "Documentation and analysis specialist", provider: "gpt", version: "4.1", endpoint: null, capabilities: JSON.stringify(["analysis", "documentation", "research"]), max_concurrent: 2, max_complexity: "medium", hourly_rate: 0.01, rating: 0.71, total_tasks: 12, registered_at: ago(2000), last_seen_at: ago(600) },
    ] as AgentRegistryEntry[];
    case "marketplace_search": return [] as AgentRegistryEntry[];
    case "marketplace_get": return null;
    case "update_agent_proficiency": return;
    case "get_agent_capabilities": return [
      { id: 1, agent_id: args?.agentId ?? "", capability: "rust", proficiency: 0.92, tasks_completed: 18, tasks_failed: 1, avg_confidence: 0.91, updated_at: ago(10) },
      { id: 2, agent_id: args?.agentId ?? "", capability: "typescript", proficiency: 0.88, tasks_completed: 15, tasks_failed: 2, avg_confidence: 0.87, updated_at: ago(20) },
      { id: 3, agent_id: args?.agentId ?? "", capability: "react", proficiency: 0.85, tasks_completed: 12, tasks_failed: 1, avg_confidence: 0.85, updated_at: ago(30) },
      { id: 4, agent_id: args?.agentId ?? "", capability: "sql", proficiency: 0.78, tasks_completed: 8, tasks_failed: 2, avg_confidence: 0.80, updated_at: ago(50) },
    ] as AgentCapability[];
    case "find_best_agent": return [
      { agent_id: "claude-opus-1", name: "Claude Opus", score: 0.92, matched_skills: args?.taskSkills ?? [], avg_proficiency: 0.88, rating: 0.91, status: "idle" },
      { agent_id: "review-bot-1", name: "Review Bot", score: 0.65, matched_skills: ["testing"], avg_proficiency: 0.75, rating: 0.82, status: "busy" },
    ] as AgentMatch[];

    // Handoff Notes
    case "create_handoff_note": {
      const input = args!.input;
      return {
        id: id(), task_identifier: input.task_identifier, from_agent_id: input.from_agent_id,
        to_agent_id: input.to_agent_id ?? null, note_type: input.note_type, summary: input.summary,
        details: input.details ?? null, files_changed: input.files_changed ?? [],
        risks: input.risks ?? [], test_results: input.test_results ?? null,
        metadata: input.metadata ?? {}, created_at: now,
      } as HandoffNote;
    }
    case "list_handoff_notes": return [] as HandoffNote[];
    case "get_handoff_for_agent": return [] as HandoffNote[];

    // Learnings
    case "record_learning": {
      const input = args!.input;
      return {
        id: id(), task_identifier: input.task_identifier, agent_id: input.agent_id,
        outcome: input.outcome, approach_summary: input.approach_summary,
        key_insight: input.key_insight ?? null, pitfalls: input.pitfalls ?? [],
        effective_patterns: input.effective_patterns ?? [], relevant_files: input.relevant_files ?? [],
        tags: input.tags ?? [], created_at: now,
      } as TaskLearning;
    }
    case "find_similar_learnings": return [] as SimilarTaskResult[];
    case "list_learnings": return [] as TaskLearning[];
    case "get_learnings_for_task": return [] as TaskLearning[];
    // Pipelines
    case "list_pipelines": return pipelines.filter(p => p.project_id === args?.projectId);
    case "get_pipeline": return pipelines.find(p => p.id === args?.id) ?? null;
    case "create_pipeline": {
      const p: Pipeline = {
        id: id(), ...args!.input,
        stages: JSON.stringify(args!.input.stages),
        enabled: true, total_runs: 0, created_at: now, updated_at: now,
      };
      pipelines.push(p);
      return p;
    }
    case "update_pipeline": {
      const p = pipelines.find(x => x.id === args?.id);
      if (p) {
        if (args!.input.stages) args!.input.stages = JSON.stringify(args!.input.stages);
        Object.assign(p, args!.input, { updated_at: now });
      }
      return p;
    }
    case "delete_pipeline": {
      const idx = pipelines.findIndex(p => p.id === args?.id);
      if (idx >= 0) pipelines.splice(idx, 1);
      return;
    }
    case "trigger_pipeline": {
      const run: PipelineRun = {
        id: id(), pipeline_id: args?.pipelineId, trigger_issue_id: args?.triggerIssueId ?? null,
        status: "running", current_stage: 0, stage_tasks: JSON.stringify([{ stage_index: 0, task_identifier: `KAN-${id()}`, status: "queued" }]),
        context: args?.context ?? "{}", started_at: now, completed_at: null, error_message: null,
      };
      pipelineRuns.push(run);
      const p = pipelines.find(x => x.id === args?.pipelineId);
      if (p) p.total_runs++;
      return run;
    }
    case "advance_pipeline": {
      const r = pipelineRuns.find(x => x.id === args?.runId);
      if (r) { r.current_stage++; }
      return r;
    }
    case "cancel_pipeline": {
      const r = pipelineRuns.find(x => x.id === args?.runId);
      if (r) { r.status = "cancelled"; r.completed_at = now; }
      return r;
    }
    case "get_pipeline_run": return pipelineRuns.find(r => r.id === args?.runId) ?? null;
    case "list_pipeline_runs": return pipelineRuns.filter(r => r.pipeline_id === args?.pipelineId);
    // Agent Permissions
    case "list_agent_permissions":
      return agentPermissions.filter(p => p.agent_id === args?.agentId);
    case "set_agent_permission": {
      const existing = agentPermissions.find(p => p.agent_id === args?.agentId && p.permission_type === args?.permissionType && p.scope === args?.scope);
      if (existing) {
        existing.allowed = args?.allowed;
        return existing;
      }
      const newPerm: AgentPermission = { id: id(), agent_id: args!.agentId, permission_type: args!.permissionType, scope: args!.scope, allowed: args!.allowed, created_at: now };
      agentPermissions.push(newPerm);
      return newPerm;
    }
    case "remove_agent_permission": {
      const idx = agentPermissions.findIndex(p => p.id === args?.id);
      if (idx >= 0) agentPermissions.splice(idx, 1);
      return;
    }
    case "clear_agent_permissions": {
      const toRemove = agentPermissions.filter(p => p.agent_id === args?.agentId).map(p => p.id);
      for (const rid of toRemove) {
        const idx = agentPermissions.findIndex(p => p.id === rid);
        if (idx >= 0) agentPermissions.splice(idx, 1);
      }
      return;
    }
    case "list_permission_presets":
      return [...permissionPresets];
    case "create_permission_preset": {
      const preset: PermissionPreset = { id: id(), name: args!.name, description: args!.description, permissions: args!.permissions, created_at: now };
      permissionPresets.push(preset);
      return preset;
    }
    case "apply_preset_to_agent": {
      // Clear existing, apply preset perms
      const toRemove2 = agentPermissions.filter(p => p.agent_id === args?.agentId).map(p => p.id);
      for (const rid of toRemove2) {
        const idx2 = agentPermissions.findIndex(p => p.id === rid);
        if (idx2 >= 0) agentPermissions.splice(idx2, 1);
      }
      const preset2 = permissionPresets.find(p => p.id === args?.presetId);
      if (preset2) {
        const entries = JSON.parse(preset2.permissions);
        for (const e of entries) {
          agentPermissions.push({ id: id(), agent_id: args!.agentId, permission_type: e.permission_type, scope: e.scope, allowed: e.allowed, created_at: now });
        }
      }
      return agentPermissions.filter(p => p.agent_id === args?.agentId);
    }
    case "delete_permission_preset": {
      const idx3 = permissionPresets.findIndex(p => p.id === args?.id);
      if (idx3 >= 0) permissionPresets.splice(idx3, 1);
      return;
    }
    case "check_permission": {
      const perms = agentPermissions.filter(p => p.agent_id === args?.agentId && p.permission_type === args?.permissionType);
      if (perms.length === 0) return { allowed: true, reason: "No rules configured, default allow", matched_rule: null } as PermissionCheckResult;
      const deny = perms.find(p => !p.allowed && (p.scope === args?.scope || p.scope === "*"));
      if (deny) return { allowed: false, reason: `Denied by rule: ${deny.permission_type} scope=${deny.scope}`, matched_rule: deny } as PermissionCheckResult;
      const allow = perms.find(p => p.allowed && (p.scope === args?.scope || p.scope === "*"));
      if (allow) return { allowed: true, reason: `Allowed by rule: ${allow.permission_type} scope=${allow.scope}`, matched_rule: allow } as PermissionCheckResult;
      return { allowed: false, reason: "No matching rule (whitelist mode)", matched_rule: null } as PermissionCheckResult;
    }
    case "check_file_access":
      return { allowed: true, reason: "Mock: default allow", matched_rule: null } as PermissionCheckResult;
    case "check_task_claim":
      return { allowed: true, reason: "Mock: default allow", matched_rule: null } as PermissionCheckResult;
    // Cost Tracking
    case "record_cost": {
      const c: TaskCost = { id: id(), ...args!.input, recorded_at: now };
      return c;
    }
    case "get_task_cost_summary": return {
      task_identifier: args?.taskIdentifier ?? "",
      total_compute_minutes: 45.5,
      total_tokens: 125000,
      total_cost_dollars: 3.25,
      cost_breakdown: [
        { id: 1, task_identifier: args?.taskIdentifier ?? "", agent_id: "claude-opus-1", cost_type: "compute_time", amount: 45.5, unit: "minutes", description: "Task execution", recorded_at: ago(60) },
        { id: 2, task_identifier: args?.taskIdentifier ?? "", agent_id: "claude-opus-1", cost_type: "api_tokens", amount: 125000, unit: "tokens", description: null, recorded_at: ago(60) },
        { id: 3, task_identifier: args?.taskIdentifier ?? "", agent_id: "claude-opus-1", cost_type: "custom", amount: 3.25, unit: "dollars", description: "API cost", recorded_at: ago(60) },
      ],
    } as TaskCostSummary;
    case "get_project_cost_summary": return {
      project_id: args?.projectId ?? 1,
      total_cost: 47.80,
      cost_by_agent: [
        { agent_id: "claude-opus-1", agent_name: "Claude Opus", total_cost: 35.20, task_count: 8, avg_cost_per_task: 4.40 },
        { agent_id: "review-bot-1", agent_name: "Review Bot", total_cost: 12.60, task_count: 5, avg_cost_per_task: 2.52 },
      ],
      daily_costs: [
        { date: new Date(Date.now() - 6 * 86400000).toISOString().split("T")[0], cost: 5.20, task_count: 2 },
        { date: new Date(Date.now() - 5 * 86400000).toISOString().split("T")[0], cost: 8.40, task_count: 3 },
        { date: new Date(Date.now() - 4 * 86400000).toISOString().split("T")[0], cost: 3.10, task_count: 1 },
        { date: new Date(Date.now() - 3 * 86400000).toISOString().split("T")[0], cost: 12.50, task_count: 4 },
        { date: new Date(Date.now() - 2 * 86400000).toISOString().split("T")[0], cost: 7.80, task_count: 2 },
        { date: new Date(Date.now() - 1 * 86400000).toISOString().split("T")[0], cost: 6.30, task_count: 2 },
        { date: new Date().toISOString().split("T")[0], cost: 4.50, task_count: 1 },
      ],
      budget_status: [
        { budget_id: 1, budget_type: "monthly", amount: 100, unit: "dollars", spent: 47.80, percentage: 0.478, alert: false, alert_threshold: 0.8 },
        { budget_id: 2, budget_type: "daily", amount: 15, unit: "dollars", spent: 12.50, percentage: 0.833, alert: true, alert_threshold: 0.8 },
      ],
    } as ProjectCostSummary;
    case "set_budget": return { id: id(), ...args!.input, spent: 0, period_start: null, period_end: null, alert_threshold: 0.8, created_at: now } as CostBudget;
    case "list_budgets": return [
      { budget_id: 1, budget_type: "monthly", amount: 100, unit: "dollars", spent: 47.80, percentage: 0.478, alert: false, alert_threshold: 0.8 },
      { budget_id: 2, budget_type: "daily", amount: 15, unit: "dollars", spent: 12.50, percentage: 0.833, alert: true, alert_threshold: 0.8 },
    ] as BudgetStatus[];
    case "check_budget": return [
      { budget_id: 2, budget_type: "daily", amount: 15, unit: "dollars", spent: 12.50, percentage: 0.833, alert: true, alert_threshold: 0.8 },
    ] as BudgetStatus[];
    case "delete_budget": return;

    // SLA
    case "list_sla_policies": return [
      { id: 1, project_id: args?.projectId ?? 1, name: "Urgent Response", target_type: "response_time", priority_filter: "urgent", warning_minutes: 15, breach_minutes: 30, escalation_action: '{"type":"change_priority"}', enabled: 1, created_at: ago(5000) },
      { id: 2, project_id: args?.projectId ?? 1, name: "Standard Resolution", target_type: "resolution_time", priority_filter: null, warning_minutes: 120, breach_minutes: 240, escalation_action: '{"type":"notify"}', enabled: 1, created_at: ago(4000) },
    ] as SlaPolicy[];
    case "create_sla_policy": return { id: id(), ...args!.input, enabled: 1, escalation_action: args!.input.escalation_action ?? "{}", created_at: now } as SlaPolicy;
    case "update_sla_policy": return args?.input;
    case "delete_sla_policy": return;
    case "check_sla_compliance": return [
      { issue_id: 6, issue_identifier: "KAN-6", issue_title: "Fix drag-drop position calculation", policy_id: 1, policy_name: "Urgent Response", status: "warning", elapsed_minutes: 22, remaining_minutes: 8, breach_at: ago(-8) },
      { issue_id: 7, issue_identifier: "KAN-7", issue_title: "Improve issue detail panel UX", policy_id: 2, policy_name: "Standard Resolution", status: "ok", elapsed_minutes: 45, remaining_minutes: 195, breach_at: ago(-195) },
      { issue_id: 8, issue_identifier: "KAN-8", issue_title: "Add comment mentions", policy_id: 2, policy_name: "Standard Resolution", status: "breached", elapsed_minutes: 260, remaining_minutes: 0, breach_at: ago(20) },
    ] as SlaStatus[];
    case "enforce_sla": return [] as SlaEvent[];
    case "get_sla_events": return [
      { id: 1, sla_policy_id: 1, issue_id: args?.issueId ?? 6, event_type: "warning", message: "SLA warning for KAN-6: 22m elapsed", metadata: "{}", created_at: ago(10) },
    ] as SlaEvent[];
    case "get_sla_dashboard": return {
      total_tracked: 3,
      total_ok: 1,
      total_warning: 1,
      total_breached: 1,
      policies: [
        { id: 1, project_id: args?.projectId ?? 1, name: "Urgent Response", target_type: "response_time", priority_filter: "urgent", warning_minutes: 15, breach_minutes: 30, escalation_action: '{"type":"change_priority"}', enabled: 1, created_at: ago(5000) },
        { id: 2, project_id: args?.projectId ?? 1, name: "Standard Resolution", target_type: "resolution_time", priority_filter: null, warning_minutes: 120, breach_minutes: 240, escalation_action: '{"type":"notify"}', enabled: 1, created_at: ago(4000) },
      ],
      statuses: [
        { issue_id: 6, issue_identifier: "KAN-6", issue_title: "Fix drag-drop", policy_id: 1, policy_name: "Urgent Response", status: "warning", elapsed_minutes: 22, remaining_minutes: 8, breach_at: ago(-8) },
        { issue_id: 8, issue_identifier: "KAN-8", issue_title: "Add comment mentions", policy_id: 2, policy_name: "Standard Resolution", status: "breached", elapsed_minutes: 260, remaining_minutes: 0, breach_at: ago(20) },
      ],
      recent_events: [
        { id: 1, sla_policy_id: 2, issue_id: 8, event_type: "breach", message: "SLA breach for KAN-8", metadata: "{}", created_at: ago(20) },
        { id: 2, sla_policy_id: 1, issue_id: 6, event_type: "warning", message: "SLA warning for KAN-6", metadata: "{}", created_at: ago(10) },
      ],
    } as SlaDashboard;

    case "list_project_files": {
      return [
        { path: "src", type: "dir", children: [
          { path: "src/App.tsx", type: "file", size: 15000 },
          { path: "src/main.tsx", type: "file", size: 500 },
          { path: "src/components", type: "dir", children: [
            { path: "src/components/BoardView.tsx", type: "file", size: 8000 },
            { path: "src/components/IssueCard.tsx", type: "file", size: 4000 },
            { path: "src/components/CodeHeatMap.tsx", type: "file", size: 6000 },
            { path: "src/components/IssueDetailPanel.tsx", type: "file", size: 12000 },
            { path: "src/components/Sidebar.tsx", type: "file", size: 6000 },
            { path: "src/components/AgentDashboard.tsx", type: "file", size: 9000 },
            { path: "src/components/ReplayViewer.tsx", type: "file", size: 5000 },
          ]},
          { path: "src/hooks", type: "dir", children: [
            { path: "src/hooks/use-issues.ts", type: "file", size: 1200 },
            { path: "src/hooks/use-projects.ts", type: "file", size: 900 },
            { path: "src/hooks/use-members.ts", type: "file", size: 700 },
          ]},
          { path: "src/tauri", type: "dir", children: [
            { path: "src/tauri/commands.ts", type: "file", size: 12000 },
            { path: "src/tauri/mock-backend.ts", type: "file", size: 28000 },
            { path: "src/tauri/events.ts", type: "file", size: 400 },
          ]},
          { path: "src/lib", type: "dir", children: [
            { path: "src/lib/utils.ts", type: "file", size: 800 },
          ]},
          { path: "src/types", type: "dir", children: [
            { path: "src/types/index.ts", type: "file", size: 6000 },
          ]},
        ]},
        { path: "src-tauri", type: "dir", children: [
          { path: "src-tauri/src", type: "dir", children: [
            { path: "src-tauri/src/main.rs", type: "file", size: 3000 },
            { path: "src-tauri/src/lib.rs", type: "file", size: 4000 },
            { path: "src-tauri/src/mcp.rs", type: "file", size: 45000 },
            { path: "src-tauri/src/cli.rs", type: "file", size: 8000 },
            { path: "src-tauri/src/state.rs", type: "file", size: 2000 },
          ]},
          { path: "src-tauri/migrations_sqlite", type: "dir", children: [
            { path: "src-tauri/migrations_sqlite/schema.sql", type: "file", size: 12000 },
          ]},
        ]},
        { path: "e2e", type: "dir", children: [
          { path: "e2e/playwright.config.ts", type: "file", size: 800 },
        ]},
        { path: "CLAUDE.md", type: "file", size: 5000 },
        { path: "AGENTS.md", type: "file", size: 2000 },
        { path: "package.json", type: "file", size: 2000 },
        { path: "tsconfig.json", type: "file", size: 400 },
        { path: "vite.config.ts", type: "file", size: 600 },
      ];
    }
    case "read_project_file": {
      const filePath = args?.filePath ?? "";
      if (filePath === "CLAUDE.md") return { content: `# Kanban — Project Rules

## Pre-Push Verification (MANDATORY)

Always verify locally before pushing or tagging releases:

\`\`\`bash
# 1. Rust compilation
source "$HOME/.cargo/env" && cd src-tauri && cargo check --lib
# 2. TypeScript compilation
npx tsc --noEmit
# 3. Tests
npm run test:run
\`\`\`

## Tech Stack

- **Backend:** Tauri v2, Rust, SQLite (sqlx with AnyPool), tokio
- **Frontend:** React 18, TypeScript, Vite, Tailwind CSS, shadcn/ui
- **Database:** \`~/.kanban/data.db\` (SQLite with WAL mode)
- **CLI:** \`kanban-cli\` (clap-based)
- **Testing:** vitest + @testing-library/react + happy-dom

## Workflow

All work MUST be tracked on the Kanban board (project ID: 2, prefix: KAN).
` };
      if (filePath === "AGENTS.md") return { content: `# Agent Guidelines

## Overview

This project uses AI agents for automated task execution via the MCP protocol.

## Agent Roles

- **code-reviewer**: QA and code review agent
- **yume--guardian**: Security and compliance agent

## Task Lifecycle

1. Agent calls \`next_task\` to claim work
2. Agent starts task with \`start_task\`
3. Agent completes with \`complete_task\` or fails with \`fail_task\`
4. Human approves with \`approve_task\`

## Heartbeat

Agents must send a heartbeat every 30 seconds or they will be marked offline.
` };
      if (filePath === ".claude/settings.json") return { content: JSON.stringify({
        permissions: {
          allow: ["Bash", "Read", "Write", "Edit", "Glob", "Grep"],
          deny: []
        },
        model: "claude-sonnet-4-6",
        version: "1.0.0"
      }, null, 2) };
      return { content: null };
    }

    case "get_git_status": {
      return {
        branch: "dev",
        ahead: 3,
        behind: 0,
        uncommitted: 2,
        untracked: 1,
      } as GitStatus;
    }

    case "list_git_commits": {
      return [
        { hash: "0c138a3f", short_hash: "0c138a3", author: "Arjun", message: "fix: handle object values in activity log", timestamp: ago(5), issue_refs: [] },
        { hash: "a61fb57e", short_hash: "a61fb57", author: "Arjun", message: "fix: font paths for production builds", timestamp: ago(30), issue_refs: [] },
        { hash: "bd9b502a", short_hash: "bd9b502", author: "Arjun", message: "chore: bump version to 0.6.1", timestamp: ago(60), issue_refs: [] },
        { hash: "5429658b", short_hash: "5429658", author: "Claude", message: "fix: resolve TypeScript errors for KAN-9", timestamp: ago(120), issue_refs: ["KAN-9"] },
        { hash: "1bb390cb", short_hash: "1bb390c", author: "Claude", message: "feat: review workflow in issue detail panel (KAN-7)", timestamp: ago(180), issue_refs: ["KAN-7"] },
        { hash: "3ac66a3d", short_hash: "3ac66a3", author: "Claude", message: "feat: wire ActivityTicker into main layout", timestamp: ago(240), issue_refs: [] },
        { hash: "93daa11e", short_hash: "93daa11", author: "Arjun", message: "feat: integrate presence, badges into IssueCard", timestamp: ago(300), issue_refs: [] },
        { hash: "d571527f", short_hash: "d571527", author: "Claude", message: "feat: Framer Motion card animations", timestamp: ago(360), issue_refs: [] },
        { hash: "6cc936da", short_hash: "6cc936d", author: "Claude", message: "feat: review workflow for KAN-6", timestamp: ago(420), issue_refs: ["KAN-6"] },
        { hash: "35c6dd8b", short_hash: "35c6dd8", author: "Arjun", message: "feat: PredictiveStatus late-risk indicator", timestamp: ago(500), issue_refs: [] },
      ] as GitCommit[];
    }

    case "list_git_branches": {
      return [
        { name: "main", is_current: false, last_commit_hash: "abc0000", last_commit_message: "chore: release v0.6.0", issue_ref: null },
        { name: "dev", is_current: true, last_commit_hash: "0c138a3", last_commit_message: "fix: handle object values", issue_ref: null },
        { name: "kan-3/add-keyboard-shortcuts", is_current: false, last_commit_hash: "abc1234", last_commit_message: "wip: shortcuts panel", issue_ref: "KAN-3" },
        { name: "kan-6/fix-drag-drop", is_current: false, last_commit_hash: "def5678", last_commit_message: "fix: position NaN", issue_ref: "KAN-6" },
        { name: "kan-9/implement-undo-redo", is_current: false, last_commit_hash: "ghi9012", last_commit_message: "feat: undo stack", issue_ref: "KAN-9" },
      ] as GitBranch[];
    }

    case "get_issue_commits": {
      const ident = ((args?.issueIdentifier ?? "") as string).toUpperCase();
      const allCommits = [
        { hash: "5429658b", short_hash: "5429658", author: "Claude", message: "fix: resolve TypeScript errors for KAN-9", timestamp: ago(120), issue_refs: ["KAN-9"] },
        { hash: "1bb390cb", short_hash: "1bb390c", author: "Claude", message: "feat: review workflow in issue detail panel (KAN-7)", timestamp: ago(180), issue_refs: ["KAN-7"] },
        { hash: "6cc936da", short_hash: "6cc936d", author: "Claude", message: "feat: review workflow for KAN-6", timestamp: ago(420), issue_refs: ["KAN-6"] },
        { hash: "abc12345", short_hash: "abc1234", author: "Arjun", message: "wip: keyboard shortcuts panel for KAN-3", timestamp: ago(500), issue_refs: ["KAN-3"] },
      ] as GitCommit[];
      return allCommits.filter(c => c.issue_refs.includes(ident));
    }

    case "get_issue_branches": {
      const ident = ((args?.issueIdentifier ?? "") as string).toLowerCase();
      const allBranches = [
        { name: "kan-3/add-keyboard-shortcuts", is_current: false, last_commit_hash: "abc1234", last_commit_message: "wip: shortcuts panel", issue_ref: "KAN-3" },
        { name: "kan-6/fix-drag-drop", is_current: false, last_commit_hash: "def5678", last_commit_message: "fix: position NaN", issue_ref: "KAN-6" },
        { name: "kan-9/implement-undo-redo", is_current: false, last_commit_hash: "ghi9012", last_commit_message: "feat: undo stack", issue_ref: "KAN-9" },
      ] as GitBranch[];
      return allBranches.filter(b => b.issue_ref?.toUpperCase() === ident.toUpperCase());
    }

    case "list_git_worktrees": {
      return [
        { path: "/home/user/kanban", branch: "dev", head_hash: "0c138a3", is_main: true, agent_id: null, agent_name: null, task_identifier: null },
        { path: "/home/user/kanban/.worktrees/kan-6-fix", branch: "kan-6/fix-drag-drop", head_hash: "def5678", is_main: false, agent_id: "claude-opus-1", agent_name: "Claude Opus", task_identifier: "KAN-6" },
        { path: "/home/user/kanban/.worktrees/kan-9-undo", branch: "kan-9/implement-undo-redo", head_hash: "ghi9012", is_main: false, agent_id: "review-bot-1", agent_name: "Review Bot", task_identifier: "KAN-9" },
      ] as GitWorktree[];
    }

    case "execute_shell_command": {
      const cmd2 = args?.command ?? "";
      if (cmd2 === "clear") return "";
      if (cmd2 === "pwd") return "/home/user/kanban\n";
      if (cmd2 === "ls") return "src/  e2e/  docs/  package.json  tsconfig.json\n";
      if (cmd2 === "whoami") return "kanban-user\n";
      if (cmd2.startsWith("echo ")) return cmd2.slice(5) + "\n";
      if (cmd2 === "help") return "Available commands: help, clear, pwd, ls, echo, whoami\n";
      return `command not found: ${cmd2.split(" ")[0]}\n`;
    }
    case "list_directories": {
      const p = args?.path ?? "/";
      if (p === "/" || p === "/home") return ["/home/user"];
      if ((p as string).includes("/home/user")) return ["/home/user/kanban", "/home/user/documents", "/home/user/projects"];
      return ["/home", "/tmp", "/usr"];
    }

    default:
      throw new Error(`Unhandled mock command: ${cmd}`);
  }
}
