import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AgentMetrics, ExecutionLog } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats, recentActivity, getIssue, deregisterAgent } from "@/tauri/commands";

interface AgentDashboardProps {
  projectId: number | null;
  onViewReplay?: (identifier: string) => void;
}

const ENTRY_TYPE_STYLES: Record<string, string> = {
  claim: "bg-blue-500/20 text-blue-400",
  start: "bg-blue-500/20 text-blue-400",
  reasoning: "bg-purple-500/20 text-purple-400",
  file_read: "bg-cyan-500/20 text-cyan-400",
  file_edit: "bg-yellow-500/20 text-yellow-400",
  command: "bg-orange-500/20 text-orange-400",
  discovery: "bg-emerald-500/20 text-emerald-400",
  error: "bg-red-500/20 text-red-400",
  result: "bg-green-500/20 text-green-400",
  complete: "bg-green-500/20 text-green-400",
  fail: "bg-red-500/20 text-red-400",
  checkpoint: "bg-zinc-500/20 text-zinc-400",
  timeout: "bg-red-500/20 text-red-400",
  unblocked: "bg-emerald-500/20 text-emerald-400",
  approve: "bg-green-500/20 text-green-400",
  reject: "bg-red-500/20 text-red-400",
  reclaim: "bg-yellow-500/20 text-yellow-400",
  unclaim: "bg-zinc-500/20 text-zinc-400",
};

const AGENT_TYPE_COLORS: Record<string, string> = {
  claude: "bg-orange-500/20 text-orange-400 border-orange-500/30",
  "claude-code": "bg-orange-500/20 text-orange-400 border-orange-500/30",
  codex: "bg-green-500/20 text-green-400 border-green-500/30",
  gemini: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  custom: "bg-purple-500/20 text-purple-400 border-purple-500/30",
};

function MetricCard({ label, value, colorClass }: { label: string; value: number | string; colorClass?: string }) {
  return (
    <div className="rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-3">
      <div className="text-xs uppercase tracking-wider text-zinc-500">{label}</div>
      <div className={`text-2xl font-mono font-bold mt-1 ${colorClass ?? "text-zinc-100"}`}>{value}</div>
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

  // Fetch per-agent stats
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

  // Fetch activity logs
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

  // Fetch issue identifiers for activity log entries
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
        } catch { /* issue may have been deleted */ }
      }
      if (!cancelled) setIssueIdentifiers(prev => ({ ...prev, ...results }));
    }
    fetchIdentifiers();
    return () => { cancelled = true; };
  }, [activityLogs]);

  // Live refresh on db-changed
  useEffect(() => {
    const unlisten = listen("db-changed", () => {
      refreshAgents();
      refreshMetrics();
      refreshActivity();
    });
    return () => { unlisten.then(fn => fn()); };
  }, [refreshAgents, refreshMetrics, refreshActivity]);

  return (
    <div className="p-6 space-y-6 h-full overflow-y-auto bg-zinc-950 text-zinc-100">
      {/* Metrics Overview */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <MetricCard label="Total Tasks" value={metricsLoading ? "--" : (metrics?.total_tasks ?? 0)} />
        <MetricCard label="Completed" value={metricsLoading ? "--" : (metrics?.completed ?? 0)} colorClass="text-green-500" />
        <MetricCard label="Queued" value={metricsLoading ? "--" : (metrics?.queued ?? 0)} />
        <MetricCard label="In Progress" value={metricsLoading ? "--" : (metrics?.in_progress ?? 0)} colorClass="text-amber-500" />
        <MetricCard label="Blocked" value={metricsLoading ? "--" : (metrics?.blocked ?? 0)} colorClass="text-red-500" />
        <MetricCard label="Agents Online" value={metricsLoading ? "--" : (metrics?.agents_online ?? 0)} colorClass="text-amber-500" />
      </div>

      {/* Agents */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xs uppercase tracking-wider text-zinc-500">Agents</h2>
          <button
            onClick={() => setShowInactive(!showInactive)}
            className="text-[10px] font-mono text-zinc-500 hover:text-zinc-300 transition-colors px-2 py-1 rounded border border-zinc-700 hover:border-zinc-500"
          >
            {showInactive ? "Hide inactive" : "Show inactive"}
            <span className="ml-1 text-zinc-600">
              ({agents.filter(a => a.status === "offline").length})
            </span>
          </button>
        </div>
        {agentsLoading ? (
          <div className="text-sm text-zinc-500">Loading agents...</div>
        ) : agents.length === 0 ? (
          <div className="text-sm text-zinc-500">No agents registered.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {agents
              .filter(agent => {
                if (showInactive) return true;
                return agent.status !== "offline";
              })
              .map((agent) => {
              const isOnline = agent.status === "online" || agent.status === "busy" || agent.status === "idle";
              const stats = agentStats[agent.id];
              const agentType = agent.agent_type || "custom";
              const typeStyle = AGENT_TYPE_COLORS[agentType] || AGENT_TYPE_COLORS.custom;
              const skills: string[] = Array.isArray(agent.skills) ? agent.skills : [];

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
                  className={`rounded-lg border border-zinc-700 bg-zinc-900 p-4 space-y-2 ${
                    isOnline ? "border-l-2 border-l-amber-500 shadow-lg shadow-amber-500/10" : "opacity-50"
                  }`}
                >
                  {/* Header: name + type + status */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-semibold text-sm truncate">{agent.name}</span>
                      <span className={`text-[10px] font-mono uppercase px-1.5 py-0.5 rounded border shrink-0 ${typeStyle}`}>
                        {agentType}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="flex items-center gap-1.5 text-xs">
                        <span className={`inline-block h-2 w-2 rounded-full ${isOnline ? "bg-green-500 animate-pulse" : "bg-zinc-600"}`} />
                        <span className="text-zinc-400 font-mono">{agent.status}</span>
                      </span>
                      <button
                        onClick={handleDelete}
                        className="text-zinc-600 hover:text-red-400 transition-colors p-0.5"
                        title="Remove agent"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                      </button>
                    </div>
                  </div>

                  {/* Model info */}
                  <div className="text-[10px] font-mono text-zinc-500">
                    model: <span className="text-zinc-400">{agentType}</span>
                    {" · "}concurrency: <span className="text-zinc-400">{agent.max_concurrent}</span>
                    {" · "}max: <span className="text-zinc-400">{agent.max_complexity}</span>
                  </div>
                  {agent.worktree_path && (
                    <div className="text-[10px] font-mono text-zinc-500 truncate">
                      worktree: <span className="text-zinc-400">{agent.worktree_path}</span>
                    </div>
                  )}

                  {/* Skills */}
                  {skills.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {skills.map((skill: string) => (
                        <span key={skill} className="bg-zinc-700 text-zinc-300 rounded px-2 py-0.5 text-xs font-mono">{skill}</span>
                      ))}
                    </div>
                  )}

                  {/* Stats */}
                  {stats && (
                    <div className="text-xs font-mono text-zinc-400">
                      completed: <span className="text-green-500">{stats.tasks_completed}</span>
                      {" | "}failed: <span className="text-red-500">{stats.tasks_failed}</span>
                      {" | "}confidence: <span className="text-zinc-300">{stats.avg_confidence.toFixed(2)}</span>
                    </div>
                  )}

                  {/* Active tasks */}
                  {stats && stats.current_tasks && stats.current_tasks.length > 0 && (
                    <div className="space-y-0.5">
                      {stats.current_tasks.map((task: string) => (
                        <div key={task} className="text-xs font-mono text-amber-500">▶ {task}</div>
                      ))}
                    </div>
                  )}

                  {/* Last activity */}
                  <div className="text-[10px] text-zinc-600 font-mono">
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
        <h2 className="text-xs uppercase tracking-wider text-zinc-500 mb-3">Activity Feed</h2>
        {!projectId ? (
          <div className="text-sm text-zinc-500">Select a project to view activity.</div>
        ) : activityLoading ? (
          <div className="text-sm text-zinc-500">Loading activity...</div>
        ) : activityLogs.length === 0 ? (
          <div className="rounded-lg border border-zinc-700 bg-zinc-900 p-4">
            <div className="text-sm text-zinc-500 font-mono">No activity yet. Agent actions will appear here in real-time.</div>
          </div>
        ) : (
          <div className="rounded-lg border border-zinc-700 bg-zinc-900 overflow-hidden">
            {/* Header */}
            <div className="grid grid-cols-[3.5rem_4rem_7.5rem_7rem_1fr] gap-2 px-3 py-1.5 border-b border-zinc-700 bg-zinc-800/50 text-[10px] uppercase tracking-wider text-zinc-500 font-medium">
              <span>Time</span>
              <span>Task</span>
              <span>Action</span>
              <span>Agent</span>
              <span>Details</span>
            </div>
            {/* Rows */}
            <div className="max-h-96 overflow-y-auto divide-y divide-zinc-800/50">
              {activityLogs.map((log) => {
                const style = ENTRY_TYPE_STYLES[log.entry_type] || "bg-zinc-500/20 text-zinc-400";
                const identifier = issueIdentifiers[log.issue_id];
                const agent = agents.find(a => a.id === log.agent_id);
                const agentName = agent?.name || log.agent_id.slice(0, 8);
                const agentType = agent?.agent_type || "system";
                const icons: Record<string, string> = {
                  claim: "🤖", start: "▶️", reasoning: "💭", file_read: "📖",
                  file_edit: "✏️", command: "⚡", discovery: "🔍", error: "❌",
                  result: "✅", complete: "✅", fail: "❌", checkpoint: "📌",
                  timeout: "⏰", unblocked: "🔓", approve: "✅", reject: "❌",
                  unclaim: "↩️", reclaim: "🔄",
                };
                const icon = icons[log.entry_type] || "•";

                return (
                  <div
                    key={log.id}
                    className={`grid grid-cols-[3.5rem_4rem_7.5rem_7rem_1fr] gap-2 px-3 py-2 hover:bg-zinc-800/50 items-start ${onViewReplay && identifier ? "cursor-pointer" : ""}`}
                    onClick={() => { if (onViewReplay && identifier) onViewReplay(identifier); }}
                  >
                    <span className="text-[10px] font-mono text-zinc-600 tabular-nums">
                      {formatTime(log.timestamp)}
                    </span>
                    <span className="text-[10px] font-mono text-amber-500 truncate">
                      {identifier || "—"}
                    </span>
                    <span className="flex items-center gap-1">
                      <span className="text-xs">{icon}</span>
                      <span className={`text-[10px] font-mono uppercase px-1 py-0.5 rounded leading-none ${style}`}>
                        {log.entry_type}
                      </span>
                    </span>
                    <span className="flex items-center gap-1 min-w-0">
                      <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${agentType === 'claude' ? 'bg-orange-500' : agentType === 'codex' ? 'bg-green-500' : agentType === 'gemini' ? 'bg-blue-500' : 'bg-purple-500'}`} />
                      <span className="text-[11px] font-mono text-zinc-300 truncate">{agentName}</span>
                    </span>
                    <span className="text-xs text-zinc-400 truncate">{log.message}</span>
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
