import { useState, useEffect } from "react";
import type { AgentMetrics } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats } from "@/tauri/commands";

// Color mappings for execution log entry types (used when activity feed has data)
// CLAIM=blue, THINK=purple, READ=cyan, EDIT=yellow, RUN=orange, ERROR=red, RESULT=green

interface AgentDashboardProps {
  projectId: number | null;
}

function MetricCard({
  label,
  value,
  colorClass,
}: {
  label: string;
  value: number | string;
  colorClass?: string;
}) {
  return (
    <div className="rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-3">
      <div className="text-xs uppercase tracking-wider text-zinc-500">
        {label}
      </div>
      <div
        className={`text-2xl font-mono font-bold mt-1 ${colorClass ?? "text-zinc-100"}`}
      >
        {value}
      </div>
    </div>
  );
}

export function AgentDashboard({ projectId }: AgentDashboardProps) {
  const { agents, loading: agentsLoading } = useAgents();
  const { metrics, loading: metricsLoading } = useProjectMetrics(projectId);
  const [agentStats, setAgentStats] = useState<Record<string, AgentMetrics>>(
    {},
  );

  useEffect(() => {
    if (agents.length === 0) return;
    let cancelled = false;

    async function fetchStats() {
      const results: Record<string, AgentMetrics> = {};
      for (const agent of agents) {
        try {
          const stats = await getAgentStats(agent.id);
          if (!cancelled) results[agent.id] = stats;
        } catch {
          // agent stats unavailable
        }
      }
      if (!cancelled) setAgentStats(results);
    }

    fetchStats();
    return () => {
      cancelled = true;
    };
  }, [agents]);

  return (
    <div className="p-6 space-y-6 h-full overflow-y-auto bg-zinc-950 text-zinc-100">
      {/* Section 1: Metrics Overview */}
      <div>
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
          <MetricCard
            label="Total Tasks"
            value={metricsLoading ? "--" : (metrics?.total_tasks ?? 0)}
          />
          <MetricCard
            label="Completed"
            value={metricsLoading ? "--" : (metrics?.completed ?? 0)}
            colorClass="text-green-500"
          />
          <MetricCard
            label="Queued"
            value={metricsLoading ? "--" : (metrics?.queued ?? 0)}
          />
          <MetricCard
            label="In Progress"
            value={metricsLoading ? "--" : (metrics?.in_progress ?? 0)}
            colorClass="text-amber-500"
          />
          <MetricCard
            label="Blocked"
            value={metricsLoading ? "--" : (metrics?.blocked ?? 0)}
            colorClass="text-red-500"
          />
          <MetricCard
            label="Agents Online"
            value={metricsLoading ? "--" : (metrics?.agents_online ?? 0)}
            colorClass="text-amber-500"
          />
        </div>
      </div>

      {/* Section 2: Agent Status Panel */}
      <div>
        <h2 className="text-xs uppercase tracking-wider text-zinc-500 mb-3">
          Agents
        </h2>
        {agentsLoading ? (
          <div className="text-sm text-zinc-500">Loading agents...</div>
        ) : agents.length === 0 ? (
          <div className="text-sm text-zinc-500">No agents registered.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {agents.map((agent) => {
              const isOnline =
                agent.status === "online" || agent.status === "busy";
              const stats = agentStats[agent.id];
              let skills: string[] = [];
              try {
                skills = JSON.parse(agent.skills);
              } catch {
                // invalid JSON
              }

              return (
                <div
                  key={agent.id}
                  className={`rounded-lg border border-zinc-700 bg-zinc-900 p-4 space-y-2 ${
                    isOnline
                      ? "border-l-2 border-l-amber-500 shadow-lg shadow-amber-500/10"
                      : "opacity-50"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-sm">{agent.name}</span>
                    <span className="flex items-center gap-1.5 text-xs">
                      <span
                        className={`inline-block h-2 w-2 rounded-full ${
                          agent.status === "online" || agent.status === "busy"
                            ? "bg-green-500"
                            : "bg-zinc-600"
                        }`}
                      />
                      <span className="text-zinc-400 font-mono">
                        {agent.status}
                      </span>
                    </span>
                  </div>

                  {skills.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {skills.map((skill) => (
                        <span
                          key={skill}
                          className="bg-zinc-700 text-zinc-300 rounded px-2 py-0.5 text-xs font-mono"
                        >
                          {skill}
                        </span>
                      ))}
                    </div>
                  )}

                  {stats && (
                    <div className="text-xs font-mono text-zinc-400">
                      completed:{" "}
                      <span className="text-green-500">
                        {stats.tasks_completed}
                      </span>{" "}
                      | failed:{" "}
                      <span className="text-red-500">
                        {stats.tasks_failed}
                      </span>{" "}
                      | confidence:{" "}
                      <span className="text-zinc-300">
                        {stats.avg_confidence.toFixed(2)}
                      </span>
                    </div>
                  )}

                  {stats &&
                    stats.current_tasks &&
                    stats.current_tasks.length > 0 && (
                      <div className="space-y-0.5">
                        {stats.current_tasks.map((task) => (
                          <div
                            key={task}
                            className="text-xs font-mono text-amber-500"
                          >
                            {task}
                          </div>
                        ))}
                      </div>
                    )}

                  <div className="text-xs text-zinc-600 font-mono">
                    heartbeat: {agent.last_heartbeat}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Section 3: Activity Feed */}
      <div>
        <h2 className="text-xs uppercase tracking-wider text-zinc-500 mb-3">
          Activity Feed
        </h2>
        {!projectId ? (
          <div className="text-sm text-zinc-500">
            Select a project to view activity.
          </div>
        ) : (
          <div className="rounded-lg border border-zinc-700 bg-zinc-900 p-3 max-h-64 overflow-y-auto">
            <div className="text-sm text-zinc-500 font-mono">
              No recent activity logs available.
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
