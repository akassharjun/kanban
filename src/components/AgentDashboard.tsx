import { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import { Trash2, Activity, Bot, Cpu, CheckCircle2, Clock, AlertTriangle, Wifi } from "lucide-react";
import type { AgentMetrics, ExecutionLog } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats, recentActivity, getIssue, deregisterAgent } from "@/tauri/commands";

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

export function AgentDashboard({ projectId, onViewReplay }: AgentDashboardProps) {
  const { agents, loading: agentsLoading, refresh: refreshAgents } = useAgents();
  const { metrics, loading: metricsLoading, refresh: refreshMetrics } = useProjectMetrics(projectId);
  const [agentStats, setAgentStats] = useState<Record<string, AgentMetrics>>({});
  const [activityLogs, setActivityLogs] = useState<ExecutionLog[]>([]);
  const [activityLoading, setActivityLoading] = useState(true);
  const [issueIdentifiers, setIssueIdentifiers] = useState<Record<number, string>>({});
  const [showInactive, setShowInactive] = useState(false);

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

                  <div className="text-[10px] text-muted-foreground/30 font-mono">
                    last active: {formatTime(agent.last_activity_at || agent.last_heartbeat)}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

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
    </div>
  );
}
