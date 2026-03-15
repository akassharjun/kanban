import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AgentMetrics, ExecutionLog } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats, recentActivity, getIssue } from "@/tauri/commands";

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

  // Fetch per-agent stats
  useEffect(() => {
    if (agents.length === 0) return;
    let cancelled = false;
    async function fetchStats() {
      const results: Record<string, AgentMetrics> = {};
      for (const agent of agents) {
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
        <h2 className="text-xs uppercase tracking-wider text-zinc-500 mb-3">Agents</h2>
        {agentsLoading ? (
          <div className="text-sm text-zinc-500">Loading agents...</div>
        ) : agents.length === 0 ? (
          <div className="text-sm text-zinc-500">No agents registered.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {agents.map((agent) => {
              const isOnline = agent.status === "online" || agent.status === "busy" || agent.status === "idle";
              const stats = agentStats[agent.id];
              const agentType = agent.agent_type || "custom";
              const typeStyle = AGENT_TYPE_COLORS[agentType] || AGENT_TYPE_COLORS.custom;

              // Skills are already a parsed JSON value from Postgres JSONB
              const skills: string[] = Array.isArray(agent.skills) ? agent.skills : [];

              return (
                <div
                  key={agent.id}
                  className={`rounded-lg border border-zinc-700 bg-zinc-900 p-4 space-y-2 ${
                    isOnline ? "border-l-2 border-l-amber-500 shadow-lg shadow-amber-500/10" : "opacity-50"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-sm">{agent.name}</span>
                      <span className={`text-[10px] font-mono uppercase px-1.5 py-0.5 rounded border ${typeStyle}`}>
                        {agentType}
                      </span>
                    </div>
                    <span className="flex items-center gap-1.5 text-xs">
                      <span className={`inline-block h-2 w-2 rounded-full ${isOnline ? "bg-green-500 animate-pulse" : "bg-zinc-600"}`} />
                      <span className="text-zinc-400 font-mono">{agent.status}</span>
                    </span>
                  </div>

                  {skills.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {skills.map((skill: string) => (
                        <span key={skill} className="bg-zinc-700 text-zinc-300 rounded px-2 py-0.5 text-xs font-mono">{skill}</span>
                      ))}
                    </div>
                  )}

                  {stats && (
                    <div className="text-xs font-mono text-zinc-400">
                      completed: <span className="text-green-500">{stats.tasks_completed}</span>
                      {" | "}failed: <span className="text-red-500">{stats.tasks_failed}</span>
                      {" | "}confidence: <span className="text-zinc-300">{stats.avg_confidence.toFixed(2)}</span>
                    </div>
                  )}

                  {stats && stats.current_tasks && stats.current_tasks.length > 0 && (
                    <div className="space-y-0.5">
                      {stats.current_tasks.map((task: string) => (
                        <div key={task} className="text-xs font-mono text-amber-500">▶ {task}</div>
                      ))}
                    </div>
                  )}

                  <div className="text-xs text-zinc-600 font-mono">
                    heartbeat: {formatTime(agent.last_heartbeat)}
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
          <div className="rounded-lg border border-zinc-700 bg-zinc-900 divide-y divide-zinc-800 max-h-96 overflow-y-auto">
            {activityLogs.map((log) => {
              const style = ENTRY_TYPE_STYLES[log.entry_type] || "bg-zinc-500/20 text-zinc-400";
              const identifier = issueIdentifiers[log.issue_id];
              return (
                <div
                  key={log.id}
                  className={`flex items-start gap-3 px-3 py-2 hover:bg-zinc-800/50 ${onViewReplay && identifier ? "cursor-pointer" : ""}`}
                  onClick={() => { if (onViewReplay && identifier) onViewReplay(identifier); }}
                >
                  <span className="text-[10px] font-mono text-zinc-600 mt-0.5 shrink-0 w-16">
                    {formatTime(log.timestamp)}
                  </span>
                  {identifier && (
                    <span className="text-[10px] font-mono text-amber-500 mt-0.5 shrink-0">
                      {identifier}
                    </span>
                  )}
                  <span className={`text-[10px] font-mono uppercase px-1.5 py-0.5 rounded shrink-0 ${style}`}>
                    {log.entry_type}
                  </span>
                  <span className="text-xs text-zinc-400 font-mono truncate">{log.agent_id.slice(0, 8)}</span>
                  <span className="text-sm text-zinc-300 truncate flex-1">{log.message}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
