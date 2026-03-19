import { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import { Trash2, Activity, Bot, Cpu, CheckCircle2, Clock, AlertTriangle, Wifi, Shield, ShieldCheck, ShieldX, Plus, X, ChevronDown, Search, DollarSign, GitBranch } from "lucide-react";
import type { AgentMetrics, ExecutionLog, AgentPermission, PermissionPreset, PermissionCheckResult, Agent as AgentType, GitWorktree } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats, recentActivity, getIssue, deregisterAgent, listAgentPermissions, setAgentPermission, removeAgentPermission, clearAgentPermissions, listPermissionPresets, applyPresetToAgent, checkPermission, listGitWorktrees } from "@/tauri/commands";
import { cn } from "@/lib/utils";
import { CostDashboard } from "@/components/CostDashboard";

export interface AgentDashboardProps {
  projectId: number | null;
  projectName?: string | null;
  projectPrefix?: string | null;
  onViewReplay?: (identifier: string) => void;
}

const ENTRY_TYPE_STYLES: Record<string, string> = {
  claim: "bg-blue-500/15 text-blue-400",
  start: "bg-blue-500/15 text-blue-400",
  reasoning: "bg-purple-500/15 text-purple-400",
  file_read: "bg-cyan-500/15 text-cyan-400",
  file_edit: "bg-yellow-500/15 text-yellow-400",
  command: "bg-orange-500/15 text-orange-400",
  discovery: "bg-emerald-500/15 text-emerald-400",
  error: "bg-red-500/15 text-red-400",
  result: "bg-green-500/15 text-green-400",
  complete: "bg-green-500/15 text-green-400",
  fail: "bg-red-500/15 text-red-400",
  checkpoint: "bg-muted text-muted-foreground",
  timeout: "bg-red-500/15 text-red-400",
  unblocked: "bg-emerald-500/15 text-emerald-400",
  approve: "bg-green-500/15 text-green-400",
  reject: "bg-red-500/15 text-red-400",
  reclaim: "bg-yellow-500/15 text-yellow-400",
  unclaim: "bg-muted text-muted-foreground",
};

const AGENT_TYPE_COLORS: Record<string, string> = {
  claude: "bg-orange-500/15 text-orange-400",
  "claude-code": "bg-orange-500/15 text-orange-400",
  implementation: "bg-orange-500/15 text-orange-400",
  codex: "bg-green-500/15 text-green-400",
  gemini: "bg-blue-500/15 text-blue-400",
  review: "bg-purple-500/15 text-purple-400",
  research: "bg-cyan-500/15 text-cyan-400",
  custom: "bg-muted text-muted-foreground",
};

const STATUS_COLORS: Record<string, string> = {
  busy: "bg-amber-500",
  online: "bg-green-500",
  idle: "bg-green-500",
  offline: "bg-muted-foreground/30",
};

function MetricCard({ label, value, colorClass, icon: Icon }: { label: string; value: number | string; colorClass?: string; icon?: React.ElementType }) {
  return (
    <div className="rounded-xl border border-border/50 bg-card px-4 py-3.5">
      <div className="flex items-center gap-1.5 text-[11px] uppercase tracking-wider text-muted-foreground/50 font-semibold">
        {Icon && <Icon className="h-3 w-3" />}
        {label}
      </div>
      <div className={`text-2xl font-mono font-bold mt-1.5 ${colorClass ?? "text-foreground"}`}>{value}</div>
    </div>
  );
}

function formatTime(timestamp: string): string {
  if (timestamp.length >= 19) return timestamp.slice(11, 19);
  return timestamp;
}

const PERMISSION_TYPES = ["project_access", "file_access", "action", "task_type", "max_cost"] as const;

function AgentPermissionsPanel({ agent }: { agent: AgentType }) {
  const [permissions, setPermissions] = useState<AgentPermission[]>([]);
  const [presets, setPresets] = useState<PermissionPreset[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newType, setNewType] = useState<string>("project_access");
  const [newScope, setNewScope] = useState("");
  const [newAllowed, setNewAllowed] = useState(true);
  const [showPresets, setShowPresets] = useState(false);
  // Permission test
  const [testType, setTestType] = useState<string>("project_access");
  const [testScope, setTestScope] = useState("");
  const [testResult, setTestResult] = useState<PermissionCheckResult | null>(null);
  const [showTest, setShowTest] = useState(false);

  const fetchPermissions = useCallback(async () => {
    try {
      const [perms, presetList] = await Promise.all([
        listAgentPermissions(agent.id),
        listPermissionPresets(),
      ]);
      setPermissions(perms);
      setPresets(presetList);
    } catch (e) {
      console.error("Failed to load permissions", e);
    } finally {
      setLoading(false);
    }
  }, [agent.id]);

  useEffect(() => { fetchPermissions(); }, [fetchPermissions]);

  const handleAdd = async () => {
    if (!newScope.trim()) return;
    try {
      await setAgentPermission(agent.id, newType, newScope.trim(), newAllowed);
      setNewScope("");
      setShowAddForm(false);
      fetchPermissions();
    } catch (e) {
      console.error("Failed to set permission", e);
    }
  };

  const handleRemove = async (id: number) => {
    try {
      await removeAgentPermission(id);
      fetchPermissions();
    } catch (e) {
      console.error("Failed to remove permission", e);
    }
  };

  const handleClear = async () => {
    if (!window.confirm(`Remove all permissions for "${agent.name}"? This will reset to full access.`)) return;
    try {
      await clearAgentPermissions(agent.id);
      fetchPermissions();
    } catch (e) {
      console.error("Failed to clear permissions", e);
    }
  };

  const handleApplyPreset = async (presetId: number) => {
    try {
      await applyPresetToAgent(agent.id, presetId);
      setShowPresets(false);
      fetchPermissions();
    } catch (e) {
      console.error("Failed to apply preset", e);
    }
  };

  const handleTest = async () => {
    if (!testScope.trim()) return;
    try {
      const result = await checkPermission(agent.id, testType, testScope.trim());
      setTestResult(result);
    } catch (e) {
      console.error("Failed to check permission", e);
    }
  };

  if (loading) return <div className="text-[10px] text-muted-foreground/40 py-1">Loading permissions...</div>;

  return (
    <div className="space-y-2 border-t border-border/30 pt-2 mt-2">
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 flex items-center gap-1">
          <Shield className="h-3 w-3" /> Permissions
        </span>
        <div className="flex items-center gap-1">
          <button onClick={() => setShowTest(!showTest)} className="text-[10px] text-muted-foreground/50 hover:text-muted-foreground px-1.5 py-0.5 rounded border border-border/50 hover:border-border transition-colors" title="Test permissions">
            <Search className="h-3 w-3" />
          </button>
          <button onClick={() => setShowPresets(!showPresets)} className="text-[10px] text-muted-foreground/50 hover:text-muted-foreground px-1.5 py-0.5 rounded border border-border/50 hover:border-border transition-colors" title="Apply preset">
            <ChevronDown className="h-3 w-3" />
          </button>
          <button onClick={() => setShowAddForm(!showAddForm)} className="text-[10px] text-muted-foreground/50 hover:text-muted-foreground px-1.5 py-0.5 rounded border border-border/50 hover:border-border transition-colors" title="Add rule">
            <Plus className="h-3 w-3" />
          </button>
          {permissions.length > 0 && (
            <button onClick={handleClear} className="text-[10px] text-muted-foreground/30 hover:text-red-400 px-1.5 py-0.5 rounded border border-border/50 hover:border-red-500/30 transition-colors" title="Clear all">
              <Trash2 className="h-3 w-3" />
            </button>
          )}
        </div>
      </div>

      {/* Preset dropdown */}
      {showPresets && (
        <div className="bg-muted/50 rounded-lg p-2 space-y-1">
          <div className="text-[10px] font-semibold text-muted-foreground/50 mb-1">Apply Preset</div>
          {presets.map(preset => (
            <button key={preset.id} onClick={() => handleApplyPreset(preset.id)} className="w-full text-left text-[11px] px-2 py-1 rounded hover:bg-muted transition-colors">
              <span className="font-medium text-foreground">{preset.name}</span>
              {preset.description && <span className="text-muted-foreground/50 ml-1">- {preset.description}</span>}
            </button>
          ))}
        </div>
      )}

      {/* Add form */}
      {showAddForm && (
        <div className="bg-muted/50 rounded-lg p-2 space-y-1.5">
          <div className="flex gap-1.5">
            <select value={newType} onChange={e => setNewType(e.target.value)} className="text-[11px] bg-background border border-border/50 rounded px-1.5 py-1 flex-shrink-0">
              {PERMISSION_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
            </select>
            <input type="text" value={newScope} onChange={e => setNewScope(e.target.value)} placeholder="Scope (e.g. src/**/*.ts)" className="text-[11px] bg-background border border-border/50 rounded px-1.5 py-1 flex-1 min-w-0" onKeyDown={e => e.key === "Enter" && handleAdd()} />
          </div>
          <div className="flex items-center justify-between">
            <label className="flex items-center gap-1.5 text-[11px] text-muted-foreground cursor-pointer">
              <input type="checkbox" checked={newAllowed} onChange={e => setNewAllowed(e.target.checked)} className="rounded" />
              {newAllowed ? <span className="text-green-500">Allow</span> : <span className="text-red-500">Deny</span>}
            </label>
            <div className="flex gap-1">
              <button onClick={() => setShowAddForm(false)} className="text-[10px] px-2 py-0.5 rounded text-muted-foreground hover:bg-muted transition-colors">Cancel</button>
              <button onClick={handleAdd} className="text-[10px] px-2 py-0.5 rounded bg-primary text-primary-foreground hover:bg-primary/90 transition-colors">Add</button>
            </div>
          </div>
        </div>
      )}

      {/* Test tool */}
      {showTest && (
        <div className="bg-muted/50 rounded-lg p-2 space-y-1.5">
          <div className="text-[10px] font-semibold text-muted-foreground/50">Permission Test</div>
          <div className="flex gap-1.5">
            <select value={testType} onChange={e => setTestType(e.target.value)} className="text-[11px] bg-background border border-border/50 rounded px-1.5 py-1 flex-shrink-0">
              {PERMISSION_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
            </select>
            <input type="text" value={testScope} onChange={e => setTestScope(e.target.value)} placeholder="Scope to test" className="text-[11px] bg-background border border-border/50 rounded px-1.5 py-1 flex-1 min-w-0" onKeyDown={e => e.key === "Enter" && handleTest()} />
            <button onClick={handleTest} className="text-[10px] px-2 py-0.5 rounded bg-primary text-primary-foreground hover:bg-primary/90 transition-colors flex-shrink-0">Test</button>
          </div>
          {testResult && (
            <div className={`text-[11px] font-mono px-2 py-1 rounded ${testResult.allowed ? "bg-green-500/15 text-green-400" : "bg-red-500/15 text-red-400"}`}>
              {testResult.allowed ? <ShieldCheck className="h-3 w-3 inline mr-1" /> : <ShieldX className="h-3 w-3 inline mr-1" />}
              {testResult.allowed ? "ALLOWED" : "DENIED"}: {testResult.reason}
            </div>
          )}
        </div>
      )}

      {/* Permissions table */}
      {permissions.length === 0 ? (
        <div className="text-[10px] text-muted-foreground/40 font-mono">No rules (full access)</div>
      ) : (
        <div className="space-y-0.5">
          {permissions.map(perm => (
            <div key={perm.id} className="flex items-center justify-between text-[11px] font-mono group">
              <div className="flex items-center gap-1.5 min-w-0">
                {perm.allowed ? (
                  <ShieldCheck className="h-3 w-3 text-green-500 flex-shrink-0" />
                ) : (
                  <ShieldX className="h-3 w-3 text-red-500 flex-shrink-0" />
                )}
                <span className="text-muted-foreground/60">{perm.permission_type}</span>
                <span className="text-foreground truncate">{perm.scope}</span>
              </div>
              <button onClick={() => handleRemove(perm.id)} className="text-muted-foreground/20 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0 p-0.5">
                <X className="h-3 w-3" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
type DashboardTab = "agents" | "costs";

export function AgentDashboard({ projectId, onViewReplay }: AgentDashboardProps) {
  const [dashboardTab, setDashboardTab] = useState<DashboardTab>("agents");
  const { agents, loading: agentsLoading, refresh: refreshAgents } = useAgents();
  const { metrics, loading: metricsLoading, refresh: refreshMetrics } = useProjectMetrics(projectId);
  const [agentStats, setAgentStats] = useState<Record<string, AgentMetrics>>({});
  const [activityLogs, setActivityLogs] = useState<ExecutionLog[]>([]);
  const [activityLoading, setActivityLoading] = useState(true);
  const [issueIdentifiers, setIssueIdentifiers] = useState<Record<number, string>>({});
  const [showInactive, setShowInactive] = useState(false);
  const [expandedAgent, setExpandedAgent] = useState<string | null>(null);
  const [worktrees, setWorktrees] = useState<GitWorktree[]>([]);
  const [worktreesLoading, setWorktreesLoading] = useState(false);

  useEffect(() => {
    if (agents.length === 0) return;
    let cancelled = false;
    async function fetchStats() {
      const results: Record<string, AgentMetrics> = {};
      for (const agent of agents) {
        if (cancelled) return;
        try {
          const stats = await getAgentStats(agent.id);
          if (!cancelled) results[agent.id] = stats;
        } catch { /* stats unavailable */ }
      }
      if (!cancelled) setAgentStats(results);
    }
    fetchStats();
    return () => { cancelled = true; };
  }, [agents]);

  const refreshActivity = useCallback(async () => {
    if (!projectId) { setActivityLoading(false); return; }
    try {
      const logs = await recentActivity(projectId, 100);
      setActivityLogs(logs);
    } catch (e) {
      console.error("Failed to load activity", e);
    } finally {
      setActivityLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refreshActivity(); }, [refreshActivity]);

  useEffect(() => {
    if (!projectId) { setWorktrees([]); return; }
    let cancelled = false;
    setWorktreesLoading(true);
    listGitWorktrees(projectId)
      .then(wt => { if (!cancelled) setWorktrees(wt); })
      .catch(() => { if (!cancelled) setWorktrees([]); })
      .finally(() => { if (!cancelled) setWorktreesLoading(false); });
    return () => { cancelled = true; };
  }, [projectId]);

  useEffect(() => {
    if (activityLogs.length === 0) return;
    const uniqueIds = [...new Set(activityLogs.map(l => l.issue_id))];
    const missingIds = uniqueIds.filter(id => !(id in issueIdentifiers));
    if (missingIds.length === 0) return;
    let cancelled = false;
    async function fetchIdentifiers() {
      const results: Record<number, string> = {};
      for (const id of missingIds) {
        try {
          const issue = await getIssue(id);
          if (!cancelled) results[id] = issue.identifier;
        } catch { /* deleted */ }
      }
      if (!cancelled) setIssueIdentifiers(prev => ({ ...prev, ...results }));
    }
    fetchIdentifiers();
    return () => { cancelled = true; };
  }, [activityLogs]);

  useEffect(() => {
    const unlisten = listen("db-changed", () => {
      refreshAgents();
      refreshMetrics();
      refreshActivity();
    });
    return () => { unlisten.then(fn => fn()); };
  }, [refreshAgents, refreshMetrics, refreshActivity]);

  const activeAgents = agents.filter(a => a.status !== "offline");
  const inactiveAgents = agents.filter(a => a.status === "offline");

  return (
    <div className="p-6 space-y-6 h-full overflow-y-auto">
      {/* Tab Switcher */}
      <div className="flex gap-1 border-b border-border -mt-2 mb-2">
        <button
          onClick={() => setDashboardTab("agents")}
          className={cn(
            "px-4 py-2 text-sm border-b-2 transition-colors -mb-[1px] flex items-center gap-1.5",
            dashboardTab === "agents" ? "border-primary text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"
          )}
        >
          <Bot className="h-3.5 w-3.5" />
          Agents
        </button>
        <button
          onClick={() => setDashboardTab("costs")}
          className={cn(
            "px-4 py-2 text-sm border-b-2 transition-colors -mb-[1px] flex items-center gap-1.5",
            dashboardTab === "costs" ? "border-primary text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"
          )}
        >
          <DollarSign className="h-3.5 w-3.5" />
          Costs
        </button>
      </div>

      {dashboardTab === "costs" && projectId && (
        <CostDashboard projectId={projectId} />
      )}
      {dashboardTab === "costs" && !projectId && (
        <div className="flex items-center justify-center h-64 text-muted-foreground">Select a project to view cost data</div>
      )}

      {dashboardTab === "agents" && <>
      {/* Metrics Overview */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <MetricCard icon={Cpu} label="Total Tasks" value={metricsLoading ? "--" : (metrics?.total_tasks ?? 0)} />
        <MetricCard icon={CheckCircle2} label="Completed" value={metricsLoading ? "--" : (metrics?.completed ?? 0)} colorClass="text-green-500" />
        <MetricCard icon={Clock} label="Queued" value={metricsLoading ? "--" : (metrics?.queued ?? 0)} />
        <MetricCard icon={Activity} label="In Progress" value={metricsLoading ? "--" : (metrics?.in_progress ?? 0)} colorClass="text-amber-500" />
        <MetricCard icon={AlertTriangle} label="Blocked" value={metricsLoading ? "--" : (metrics?.blocked ?? 0)} colorClass="text-red-500" />
        <MetricCard icon={Wifi} label="Agents Online" value={metricsLoading ? "--" : (metrics?.agents_online ?? 0)} colorClass="text-primary" />
      </div>

      {/* Agents */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Agents</h2>
          {inactiveAgents.length > 0 && (
            <button
              onClick={() => setShowInactive(!showInactive)}
              className="text-[11px] text-muted-foreground/50 hover:text-muted-foreground transition-colors px-2.5 py-1 rounded-lg border border-border/50 hover:border-border"
            >
              {showInactive ? "Hide inactive" : `Show inactive (${inactiveAgents.length})`}
            </button>
          )}
        </div>
        {agentsLoading ? (
          <div className="text-sm text-muted-foreground/50">Loading agents...</div>
        ) : agents.length === 0 ? (
          <div className="rounded-xl border border-border/50 bg-card p-8 text-center">
            <Bot className="h-8 w-8 mx-auto text-muted-foreground/30 mb-2" />
            <p className="text-sm text-muted-foreground/50">No agents registered</p>
            <p className="text-xs text-muted-foreground/30 mt-1">Agents will appear here when they connect via the MCP server</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {[...activeAgents, ...(showInactive ? inactiveAgents : [])].map((agent) => {
              const isOnline = agent.status !== "offline";
              const stats = agentStats[agent.id];
              const agentType = agent.agent_type || "custom";
              const typeStyle = AGENT_TYPE_COLORS[agentType] || AGENT_TYPE_COLORS.custom;
              const skills: string[] = Array.isArray(agent.skills) ? agent.skills : [];
              const statusColor = STATUS_COLORS[agent.status] || STATUS_COLORS.offline;

              const handleDelete = async () => {
                if (!window.confirm(`Remove agent "${agent.name}"? Its active tasks will be requeued.`)) return;
                try {
                  await deregisterAgent(agent.id);
                  refreshAgents();
                } catch (e) {
                  console.error("Failed to deregister agent", e);
                }
              };

              return (
                <div
                  key={agent.id}
                  className={`rounded-xl border bg-card p-4 space-y-2.5 transition-all ${
                    isOnline ? "border-border/50 shadow-sm" : "border-border/30 opacity-50"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-semibold text-sm truncate">{agent.name}</span>
                      <span className={`text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded-md ${typeStyle}`}>
                        {agentType}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="flex items-center gap-1.5 text-xs">
                        <span className={`inline-block h-2 w-2 rounded-full ${statusColor} ${isOnline ? "animate-pulse" : ""}`} />
                        <span className="text-muted-foreground font-mono text-[11px]">{agent.status}</span>
                      </span>
                      <button
                        onClick={handleDelete}
                        className="text-muted-foreground/30 hover:text-red-400 transition-colors p-0.5 rounded"
                        title="Remove agent"
                      >
                        <Trash2 className="h-3 w-3" />
                      </button>
                    </div>
                  </div>

                  <div className="text-[10px] font-mono text-muted-foreground/50">
                    model: <span className="text-muted-foreground">{agentType}</span>
                    {" · "}concurrency: <span className="text-muted-foreground">{agent.max_concurrent}</span>
                    {" · "}max: <span className="text-muted-foreground">{agent.max_complexity}</span>
                  </div>
                  {agent.worktree_path && (
                    <div className="text-[10px] font-mono text-muted-foreground/50 truncate">
                      worktree: <span className="text-muted-foreground">{agent.worktree_path}</span>
                    </div>
                  )}

                  {skills.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {skills.map((skill: string) => (
                        <span key={skill} className="bg-muted text-muted-foreground rounded-md px-2 py-0.5 text-[11px] font-mono">{skill}</span>
                      ))}
                    </div>
                  )}

                  {stats && (
                    <div className="text-[11px] font-mono text-muted-foreground">
                      completed: <span className="text-green-500">{stats.tasks_completed}</span>
                      {" | "}failed: <span className="text-red-500">{stats.tasks_failed}</span>
                      {" | "}confidence: <span className="text-foreground">{stats.avg_confidence.toFixed(2)}</span>
                    </div>
                  )}

                  {stats && stats.current_tasks && stats.current_tasks.length > 0 && (
                    <div className="space-y-0.5">
                      {stats.current_tasks.map((task: string) => (
                        <div key={task} className="text-xs font-mono text-amber-500">▶ {task}</div>
                      ))}
                    </div>
                  )}

                  <div className="flex items-center justify-between">
                    <div className="text-[10px] text-muted-foreground/30 font-mono">
                      last active: {formatTime(agent.last_activity_at || agent.last_heartbeat)}
                    </div>
                    <button
                      onClick={(e) => { e.stopPropagation(); setExpandedAgent(expandedAgent === agent.id ? null : agent.id); }}
                      className="text-[10px] text-muted-foreground/40 hover:text-muted-foreground flex items-center gap-0.5 transition-colors"
                    >
                      <Shield className="h-3 w-3" />
                      {expandedAgent === agent.id ? "Hide" : "Permissions"}
                    </button>
                  </div>

                  {expandedAgent === agent.id && (
                    <AgentPermissionsPanel agent={agent} />
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Worktrees */}
      {projectId && (
        <div>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3 flex items-center gap-1.5">
            <GitBranch className="h-3.5 w-3.5" />
            Worktrees
          </h2>
          {worktreesLoading ? (
            <div className="text-sm text-muted-foreground/50">Loading worktrees...</div>
          ) : worktrees.length === 0 ? (
            <div className="rounded-xl border border-border/50 bg-card p-6 text-center">
              <GitBranch className="h-6 w-6 mx-auto text-muted-foreground/20 mb-2" />
              <p className="text-sm text-muted-foreground/40 font-mono">No worktrees found for this project.</p>
            </div>
          ) : (
            <div className="rounded-xl border border-border/50 bg-card overflow-hidden">
              <div className="grid grid-cols-[1fr_6rem_8rem_5rem] gap-2 px-3 py-2 border-b border-border/50 text-[10px] uppercase tracking-wider text-muted-foreground/40 font-semibold">
                <span>Path</span>
                <span>Branch</span>
                <span>Agent</span>
                <span>Task</span>
              </div>
              <div className="divide-y divide-border/30">
                {worktrees.map((wt) => {
                  const linkedAgent = wt.agent_id ? agents.find(a => a.id === wt.agent_id) : null;
                  const agentStatus = linkedAgent?.status ?? null;
                  const statusColor = agentStatus ? (STATUS_COLORS[agentStatus] || STATUS_COLORS.offline) : null;
                  return (
                    <div key={wt.path} className="grid grid-cols-[1fr_6rem_8rem_5rem] gap-2 px-3 py-2 items-center hover:bg-muted/30 transition-colors">
                      <div className="min-w-0">
                        <div className="text-[11px] font-mono text-muted-foreground/70 truncate" title={wt.path}>
                          {wt.path.split("/").slice(-2).join("/")}
                        </div>
                        {wt.is_main && (
                          <span className="text-[9px] font-semibold uppercase tracking-wider text-primary/60">main</span>
                        )}
                      </div>
                      <div className="text-[11px] font-mono text-muted-foreground truncate" title={wt.branch}>
                        {wt.branch}
                      </div>
                      <div className="flex items-center gap-1.5 min-w-0">
                        {linkedAgent ? (
                          <>
                            {statusColor && (
                              <span className={cn("h-2 w-2 rounded-full flex-shrink-0", statusColor, agentStatus !== "offline" ? "animate-pulse" : "")} />
                            )}
                            <span className="text-[11px] font-mono text-muted-foreground truncate">{linkedAgent.name}</span>
                          </>
                        ) : wt.agent_name ? (
                          <span className="text-[11px] font-mono text-muted-foreground/40 truncate">{wt.agent_name}</span>
                        ) : (
                          <span className="text-[11px] text-muted-foreground/20 font-mono">—</span>
                        )}
                      </div>
                      <div>
                        {wt.task_identifier ? (
                          <span className="text-[11px] font-mono text-primary">{wt.task_identifier}</span>
                        ) : (
                          <span className="text-[11px] text-muted-foreground/20 font-mono">—</span>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Activity Feed */}
      <div>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Activity Feed</h2>
        {!projectId ? (
          <div className="text-sm text-muted-foreground/50">Select a project to view activity.</div>
        ) : activityLoading ? (
          <div className="text-sm text-muted-foreground/50">Loading activity...</div>
        ) : activityLogs.length === 0 ? (
          <div className="rounded-xl border border-border/50 bg-card p-6 text-center">
            <Activity className="h-6 w-6 mx-auto text-muted-foreground/20 mb-2" />
            <p className="text-sm text-muted-foreground/40 font-mono">No activity yet. Agent actions will appear here in real-time.</p>
          </div>
        ) : (
          <div className="rounded-xl border border-border/50 bg-card overflow-hidden">
            <div className="grid grid-cols-[3.5rem_4rem_7.5rem_7rem_1fr] gap-2 px-3 py-2 border-b border-border/50 text-[10px] uppercase tracking-wider text-muted-foreground/40 font-semibold">
              <span>Time</span>
              <span>Task</span>
              <span>Action</span>
              <span>Agent</span>
              <span>Details</span>
            </div>
            <div className="max-h-96 overflow-y-auto divide-y divide-border/30">
              {activityLogs.map((log) => {
                const style = ENTRY_TYPE_STYLES[log.entry_type] || "bg-muted text-muted-foreground";
                const identifier = issueIdentifiers[log.issue_id];
                const agent = agents.find(a => a.id === log.agent_id);
                const agentName = agent?.name || log.agent_id.slice(0, 8);

                return (
                  <div
                    key={log.id}
                    className={`grid grid-cols-[3.5rem_4rem_7.5rem_7rem_1fr] gap-2 px-3 py-2 hover:bg-muted/30 items-start transition-colors ${onViewReplay && identifier ? "cursor-pointer" : ""}`}
                    onClick={() => { if (onViewReplay && identifier) onViewReplay(identifier); }}
                  >
                    <span className="text-[10px] font-mono text-muted-foreground/40 tabular-nums">
                      {formatTime(log.timestamp)}
                    </span>
                    <span className="text-[10px] font-mono text-primary truncate">
                      {identifier || "—"}
                    </span>
                    <span className={`text-[10px] font-mono uppercase px-1.5 py-0.5 rounded-md leading-none w-fit ${style}`}>
                      {log.entry_type}
                    </span>
                    <span className="text-[11px] font-mono text-muted-foreground truncate">{agentName}</span>
                    <span className="text-xs text-muted-foreground/60 truncate">{log.message}</span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
      </>}
    </div>
  );
}
