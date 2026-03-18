import { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import { Trash2, Activity, Bot, Cpu, CheckCircle2, Clock, AlertTriangle, Wifi, BarChart3, Store, ChevronLeft, Search, Star, Zap, TrendingUp, Trophy, ArrowUpDown } from "lucide-react";
import type { AgentMetrics, ExecutionLog, AgentPerformance, ProjectAgentSummary, AgentRegistryEntry, AgentCapability, AgentMatch } from "@/types";
import { useAgents, useProjectMetrics } from "@/hooks/use-agents";
import { getAgentStats, recentActivity, getIssue, deregisterAgent, getAgentPerformance, getProjectAgentSummary, marketplaceList, getAgentCapabilities, findBestAgent } from "@/tauri/commands";

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
  gpt: "bg-emerald-500/15 text-emerald-400",
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

type DashboardTab = "overview" | "analytics" | "marketplace";

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

// Pure SVG bar chart for completion trend
function CompletionTrendChart({ data }: { data: { date: string; completed: number; failed: number }[] }) {
  if (data.length === 0) return <div className="text-sm text-muted-foreground/40 text-center py-8">No data yet</div>;

  const maxVal = Math.max(...data.map(d => d.completed + d.failed), 1);
  const barWidth = Math.max(8, Math.min(24, Math.floor(600 / data.length) - 4));
  const chartWidth = data.length * (barWidth + 4);
  const chartHeight = 120;

  return (
    <div className="overflow-x-auto">
      <svg width={Math.max(chartWidth, 200)} height={chartHeight + 24} className="block">
        {data.map((d, i) => {
          const completedH = (d.completed / maxVal) * chartHeight;
          const failedH = (d.failed / maxVal) * chartHeight;
          const x = i * (barWidth + 4) + 2;
          return (
            <g key={d.date}>
              <title>{`${d.date}: ${d.completed} completed, ${d.failed} failed`}</title>
              <rect x={x} y={chartHeight - completedH - failedH} width={barWidth} height={failedH} rx={2} className="fill-red-500/60" />
              <rect x={x} y={chartHeight - completedH} width={barWidth} height={completedH} rx={2} className="fill-green-500/60" />
              {i % Math.max(1, Math.floor(data.length / 7)) === 0 && (
                <text x={x + barWidth / 2} y={chartHeight + 14} textAnchor="middle" className="fill-muted-foreground/40 text-[9px]">
                  {d.date.slice(5)}
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
}

// Horizontal stacked bar for task type distribution
function TaskTypeBar({ distribution }: { distribution: Record<string, number> }) {
  const total = Object.values(distribution).reduce((a, b) => a + b, 0);
  if (total === 0) return <div className="text-sm text-muted-foreground/40">No data</div>;

  const colors: Record<string, string> = {
    implementation: "#f97316",
    review: "#a855f7",
    testing: "#22c55e",
    research: "#06b6d4",
  };

  return (
    <div className="space-y-2">
      <div className="flex h-6 rounded-lg overflow-hidden">
        {Object.entries(distribution).map(([type, count]) => (
          <div
            key={type}
            className="h-full transition-all"
            style={{
              width: `${(count / total) * 100}%`,
              backgroundColor: colors[type] || "#6b7280",
              minWidth: count > 0 ? "4px" : 0,
            }}
            title={`${type}: ${count} (${((count / total) * 100).toFixed(0)}%)`}
          />
        ))}
      </div>
      <div className="flex flex-wrap gap-3">
        {Object.entries(distribution).map(([type, count]) => (
          <div key={type} className="flex items-center gap-1.5 text-[11px]">
            <span className="w-2 h-2 rounded-full" style={{ backgroundColor: colors[type] || "#6b7280" }} />
            <span className="text-muted-foreground">{type}</span>
            <span className="font-mono text-foreground">{count}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function ProficiencyBar({ value }: { value: number }) {
  const color = value >= 0.8 ? "bg-green-500" : value >= 0.5 ? "bg-amber-500" : "bg-red-500";
  return (
    <div className="flex items-center gap-2">
      <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
        <div className={`h-full rounded-full ${color}`} style={{ width: `${value * 100}%` }} />
      </div>
      <span className="text-[10px] font-mono text-muted-foreground">{(value * 100).toFixed(0)}%</span>
    </div>
  );
}

// Agent detail panel
function AgentDetailPanel({ agentId, onBack }: { agentId: string; onBack: () => void }) {
  const [perf, setPerf] = useState<AgentPerformance | null>(null);
  const [caps, setCaps] = useState<AgentCapability[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [p, c] = await Promise.all([
          getAgentPerformance(agentId),
          getAgentCapabilities(agentId),
        ]);
        if (!cancelled) { setPerf(p); setCaps(c); }
      } catch (e) { console.error(e); }
      finally { if (!cancelled) setLoading(false); }
    }
    load();
    return () => { cancelled = true; };
  }, [agentId]);

  if (loading) return <div className="text-sm text-muted-foreground/50 p-6">Loading agent details...</div>;
  if (!perf) return <div className="text-sm text-muted-foreground/50 p-6">Agent not found</div>;

  return (
    <div className="space-y-6">
      <button onClick={onBack} className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
        <ChevronLeft className="h-4 w-4" /> Back to overview
      </button>

      <div>
        <h2 className="text-lg font-semibold">{perf.agent_name}</h2>
        <p className="text-sm text-muted-foreground/50 font-mono">{perf.agent_id}</p>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <MetricCard icon={Cpu} label="Total Tasks" value={perf.total_tasks} />
        <MetricCard icon={CheckCircle2} label="Completed" value={perf.completed} colorClass="text-green-500" />
        <MetricCard icon={TrendingUp} label="Success Rate" value={`${(perf.success_rate * 100).toFixed(1)}%`} colorClass={perf.success_rate >= 0.8 ? "text-green-500" : "text-amber-500"} />
        <MetricCard icon={Clock} label="Avg Time" value={`${perf.avg_duration_minutes.toFixed(1)}m`} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Daily Completions (Last 30 days)</h3>
          <CompletionTrendChart data={perf.daily_completions} />
        </div>
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Task Type Breakdown</h3>
          <TaskTypeBar distribution={perf.tasks_by_type} />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Complexity Breakdown</h3>
          <TaskTypeBar distribution={perf.tasks_by_complexity} />
        </div>
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Outcome Breakdown</h3>
          <div className="space-y-2 text-sm font-mono">
            <div className="flex justify-between"><span className="text-green-500">Completed</span><span>{perf.completed}</span></div>
            <div className="flex justify-between"><span className="text-red-500">Failed</span><span>{perf.failed}</span></div>
            <div className="flex justify-between"><span className="text-amber-500">Rejected</span><span>{perf.rejected}</span></div>
            <div className="flex justify-between"><span className="text-muted-foreground">Timeout</span><span>{perf.timeout}</span></div>
            <div className="flex justify-between border-t border-border/50 pt-2"><span>Lines Changed</span><span>{perf.total_lines_changed.toLocaleString()}</span></div>
            <div className="flex justify-between"><span>Avg Confidence</span><span>{perf.avg_confidence.toFixed(2)}</span></div>
          </div>
        </div>
      </div>

      {caps.length > 0 && (
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Capabilities</h3>
          <div className="space-y-2">
            {caps.map(cap => (
              <div key={cap.capability} className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="bg-muted text-muted-foreground rounded-md px-2 py-0.5 text-[11px] font-mono">{cap.capability}</span>
                  <span className="text-[10px] text-muted-foreground/50">{cap.tasks_completed}c / {cap.tasks_failed}f</span>
                </div>
                <ProficiencyBar value={cap.proficiency} />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// Analytics Tab
function AnalyticsTab({ projectId }: { projectId: number | null }) {
  const [summary, setSummary] = useState<ProjectAgentSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);

  useEffect(() => {
    if (!projectId) { setLoading(false); return; }
    let cancelled = false;
    async function load() {
      try {
        const s = await getProjectAgentSummary(projectId!);
        if (!cancelled) setSummary(s);
      } catch (e) { console.error(e); }
      finally { if (!cancelled) setLoading(false); }
    }
    load();
    return () => { cancelled = true; };
  }, [projectId]);

  if (selectedAgent) {
    return <AgentDetailPanel agentId={selectedAgent} onBack={() => setSelectedAgent(null)} />;
  }

  if (!projectId) return <div className="text-sm text-muted-foreground/50 p-6">Select a project to view analytics.</div>;
  if (loading) return <div className="text-sm text-muted-foreground/50 p-6">Loading analytics...</div>;
  if (!summary) return <div className="text-sm text-muted-foreground/50 p-6">No analytics data available.</div>;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
        <MetricCard icon={Cpu} label="Total Tasks" value={summary.total_agent_tasks} />
        <MetricCard icon={CheckCircle2} label="Completed" value={summary.total_completed} colorClass="text-green-500" />
        <MetricCard icon={AlertTriangle} label="Failed" value={summary.total_failed} colorClass="text-red-500" />
        <MetricCard icon={Clock} label="Avg Time" value={`${summary.avg_completion_time_minutes.toFixed(1)}m`} />
        <MetricCard icon={Wifi} label="Active Agents" value={summary.agents_active} colorClass="text-primary" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Completion Trend (30 days)</h3>
          <CompletionTrendChart data={summary.completion_trend} />
          <div className="flex gap-4 mt-2">
            <div className="flex items-center gap-1.5 text-[10px]"><span className="w-2 h-2 rounded-full bg-green-500/60" /> Completed</div>
            <div className="flex items-center gap-1.5 text-[10px]"><span className="w-2 h-2 rounded-full bg-red-500/60" /> Failed</div>
          </div>
        </div>
        <div className="rounded-xl border border-border/50 bg-card p-4">
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Task Type Distribution</h3>
          <TaskTypeBar distribution={summary.task_type_distribution} />
        </div>
      </div>

      {/* Leaderboard */}
      <div className="rounded-xl border border-border/50 bg-card overflow-hidden">
        <div className="px-4 py-3 border-b border-border/50 flex items-center gap-2">
          <Trophy className="h-3.5 w-3.5 text-amber-500" />
          <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Agent Leaderboard</h3>
        </div>
        <div className="grid grid-cols-[2rem_1fr_4rem_4rem_4rem_4rem] gap-2 px-4 py-2 border-b border-border/30 text-[10px] uppercase tracking-wider text-muted-foreground/40 font-semibold">
          <span>#</span><span>Agent</span><span>Score</span><span>Success</span><span>Confidence</span><span>Tasks</span>
        </div>
        {summary.top_performers.map((agent, i) => (
          <div
            key={agent.agent_id}
            className="grid grid-cols-[2rem_1fr_4rem_4rem_4rem_4rem] gap-2 px-4 py-2.5 hover:bg-muted/30 transition-colors cursor-pointer items-center border-b border-border/20 last:border-0"
            onClick={() => setSelectedAgent(agent.agent_id)}
          >
            <span className={`text-sm font-bold ${i === 0 ? "text-amber-500" : i === 1 ? "text-muted-foreground" : "text-muted-foreground/50"}`}>{i + 1}</span>
            <span className="text-sm font-medium truncate">{agent.agent_name}</span>
            <span className="text-sm font-mono font-bold text-primary">{agent.score.toFixed(2)}</span>
            <span className="text-sm font-mono text-green-500">{(agent.success_rate * 100).toFixed(0)}%</span>
            <span className="text-sm font-mono text-foreground">{agent.avg_confidence.toFixed(2)}</span>
            <span className="text-sm font-mono text-muted-foreground">{agent.tasks_completed}</span>
          </div>
        ))}
        {summary.top_performers.length === 0 && (
          <div className="px-4 py-6 text-center text-sm text-muted-foreground/40">No agent data yet</div>
        )}
      </div>
    </div>
  );
}

// Marketplace Tab
function MarketplaceTab() {
  const [entries, setEntries] = useState<AgentRegistryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [matchQuery, setMatchQuery] = useState("");
  const [matchResults, setMatchResults] = useState<AgentMatch[] | null>(null);
  const [selectedCaps, setSelectedCaps] = useState<Record<string, AgentCapability[]>>({});

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const data = await marketplaceList();
        if (!cancelled) setEntries(data);
      } catch (e) { console.error(e); }
      finally { if (!cancelled) setLoading(false); }
    }
    load();
    return () => { cancelled = true; };
  }, []);

  const loadCaps = useCallback(async (agentId: string) => {
    if (selectedCaps[agentId]) return;
    try {
      const caps = await getAgentCapabilities(agentId);
      setSelectedCaps(prev => ({ ...prev, [agentId]: caps }));
    } catch (e) { console.error(e); }
  }, [selectedCaps]);

  const handleFindBest = useCallback(async () => {
    if (!matchQuery.trim()) return;
    const skills = matchQuery.split(",").map(s => s.trim()).filter(Boolean);
    try {
      const results = await findBestAgent(skills, "large");
      setMatchResults(results);
    } catch (e) { console.error(e); }
  }, [matchQuery]);

  if (loading) return <div className="text-sm text-muted-foreground/50 p-6">Loading marketplace...</div>;

  const parseCaps = (capsStr: string): string[] => {
    try { return JSON.parse(capsStr); } catch { return []; }
  };

  return (
    <div className="space-y-6">
      {/* Best Match Finder */}
      <div className="rounded-xl border border-border/50 bg-card p-4">
        <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3 flex items-center gap-2">
          <Zap className="h-3 w-3 text-amber-500" /> Best Match Finder
        </h3>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground/40" />
            <input
              type="text"
              value={matchQuery}
              onChange={e => setMatchQuery(e.target.value)}
              onKeyDown={e => e.key === "Enter" && handleFindBest()}
              placeholder="Enter required skills (comma-separated): rust, react, testing"
              className="w-full pl-9 pr-3 py-2 text-sm bg-muted/50 rounded-lg border border-border/50 focus:outline-none focus:ring-1 focus:ring-primary/50"
            />
          </div>
          <button onClick={handleFindBest} className="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">
            Find
          </button>
        </div>
        {matchResults && (
          <div className="mt-3 space-y-2">
            {matchResults.length === 0 ? (
              <div className="text-sm text-muted-foreground/40">No matching agents found</div>
            ) : matchResults.map(m => (
              <div key={m.agent_id} className="flex items-center justify-between py-2 px-3 rounded-lg bg-muted/30">
                <div className="flex items-center gap-3">
                  <span className={`inline-block h-2 w-2 rounded-full ${STATUS_COLORS[m.status] || STATUS_COLORS.offline}`} />
                  <span className="text-sm font-medium">{m.name}</span>
                  <div className="flex gap-1">
                    {m.matched_skills.map(s => (
                      <span key={s} className="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[10px] font-mono">{s}</span>
                    ))}
                  </div>
                </div>
                <div className="flex items-center gap-4 text-[11px] font-mono">
                  <span className="text-muted-foreground">prof: <span className="text-foreground">{(m.avg_proficiency * 100).toFixed(0)}%</span></span>
                  {m.rating !== null && <span className="text-muted-foreground">rating: <span className="text-amber-500">{m.rating.toFixed(2)}</span></span>}
                  <span className="font-bold text-primary">{m.score.toFixed(2)}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Registry */}
      <div>
        <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-3">Agent Registry ({entries.length})</h3>
        {entries.length === 0 ? (
          <div className="rounded-xl border border-border/50 bg-card p-8 text-center">
            <Store className="h-8 w-8 mx-auto text-muted-foreground/30 mb-2" />
            <p className="text-sm text-muted-foreground/50">No agents registered in marketplace</p>
            <p className="text-xs text-muted-foreground/30 mt-1">Agents can register via MCP or CLI</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {entries.map(entry => {
              const caps = parseCaps(entry.capabilities);
              const agentCaps = selectedCaps[entry.agent_id];
              const providerStyle = AGENT_TYPE_COLORS[entry.provider || "custom"] || AGENT_TYPE_COLORS.custom;

              return (
                <div
                  key={entry.agent_id}
                  className="rounded-xl border border-border/50 bg-card p-4 space-y-3 hover:border-border transition-colors"
                  onClick={() => loadCaps(entry.agent_id)}
                >
                  <div className="flex items-start justify-between">
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-sm truncate">{entry.name}</span>
                        {entry.provider && (
                          <span className={`text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded-md ${providerStyle}`}>
                            {entry.provider}
                          </span>
                        )}
                      </div>
                      {entry.description && (
                        <p className="text-xs text-muted-foreground/60 mt-0.5 line-clamp-2">{entry.description}</p>
                      )}
                    </div>
                    {entry.rating !== null && (
                      <div className="flex items-center gap-1 shrink-0 ml-2">
                        <Star className="h-3 w-3 text-amber-500 fill-amber-500" />
                        <span className="text-sm font-mono font-bold">{entry.rating.toFixed(2)}</span>
                      </div>
                    )}
                  </div>

                  <div className="text-[10px] font-mono text-muted-foreground/50">
                    {entry.version && <>v{entry.version} · </>}
                    max: <span className="text-muted-foreground">{entry.max_complexity}</span>
                    {" · "}concurrency: <span className="text-muted-foreground">{entry.max_concurrent}</span>
                    {entry.hourly_rate !== null && <> · <span className="text-amber-500">${entry.hourly_rate}/hr</span></>}
                  </div>

                  <div className="flex flex-wrap gap-1">
                    {caps.map(cap => (
                      <span key={cap} className="bg-muted text-muted-foreground rounded-md px-2 py-0.5 text-[11px] font-mono">{cap}</span>
                    ))}
                  </div>

                  <div className="text-[11px] font-mono text-muted-foreground">
                    tasks: <span className="text-foreground">{entry.total_tasks}</span>
                  </div>

                  {agentCaps && agentCaps.length > 0 && (
                    <div className="border-t border-border/30 pt-2 space-y-1.5">
                      <div className="text-[10px] uppercase text-muted-foreground/40 font-semibold">Proficiency</div>
                      {agentCaps.map(ac => (
                        <div key={ac.capability} className="flex items-center justify-between">
                          <span className="text-[11px] font-mono text-muted-foreground">{ac.capability}</span>
                          <ProficiencyBar value={ac.proficiency} />
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Capability Matrix */}
      {entries.length > 0 && (
        <div className="rounded-xl border border-border/50 bg-card overflow-hidden">
          <div className="px-4 py-3 border-b border-border/50 flex items-center gap-2">
            <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground/50" />
            <h3 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">Capability Matrix</h3>
          </div>
          <div className="overflow-x-auto">
            <CapabilityMatrix entries={entries} />
          </div>
        </div>
      )}
    </div>
  );
}

function CapabilityMatrix({ entries }: { entries: AgentRegistryEntry[] }) {
  // Collect all unique capabilities
  const allCaps = new Set<string>();
  const entryCaps: Record<string, string[]> = {};
  for (const e of entries) {
    const caps: string[] = (() => { try { return JSON.parse(e.capabilities); } catch { return []; } })();
    entryCaps[e.agent_id] = caps;
    caps.forEach(c => allCaps.add(c));
  }
  const capList = Array.from(allCaps).sort();

  if (capList.length === 0) return <div className="p-4 text-sm text-muted-foreground/40">No capabilities data</div>;

  return (
    <table className="w-full text-[11px]">
      <thead>
        <tr className="border-b border-border/30">
          <th className="text-left px-3 py-2 font-semibold text-muted-foreground/50">Agent</th>
          {capList.map(cap => (
            <th key={cap} className="px-2 py-2 font-mono text-muted-foreground/40 text-center whitespace-nowrap">{cap}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {entries.map(entry => (
          <tr key={entry.agent_id} className="border-b border-border/20 last:border-0">
            <td className="px-3 py-2 font-medium">{entry.name}</td>
            {capList.map(cap => {
              const has = (entryCaps[entry.agent_id] || []).includes(cap);
              return (
                <td key={cap} className="px-2 py-2 text-center">
                  {has ? (
                    <span className="inline-block w-3 h-3 rounded-full bg-primary/30 border border-primary/50" />
                  ) : (
                    <span className="inline-block w-3 h-3 rounded-full bg-muted/30" />
                  )}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function AgentDashboard({ projectId, onViewReplay }: AgentDashboardProps) {
  const [tab, setTab] = useState<DashboardTab>("overview");
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

  const tabs: { id: DashboardTab; label: string; icon: React.ElementType }[] = [
    { id: "overview", label: "Overview", icon: Activity },
    { id: "analytics", label: "Analytics", icon: BarChart3 },
    { id: "marketplace", label: "Marketplace", icon: Store },
  ];

  return (
    <div className="p-6 space-y-6 h-full overflow-y-auto">
      {/* Tab Navigation */}
      <div className="flex items-center gap-1 border-b border-border/50 pb-0">
        {tabs.map(t => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={`flex items-center gap-1.5 px-3 py-2 text-sm font-medium border-b-2 transition-colors -mb-[1px] ${
              tab === t.id
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground/50 hover:text-muted-foreground"
            }`}
          >
            <t.icon className="h-3.5 w-3.5" />
            {t.label}
          </button>
        ))}
      </div>

      {tab === "analytics" && <AnalyticsTab projectId={projectId} />}
      {tab === "marketplace" && <MarketplaceTab />}

      {tab === "overview" && (
        <>
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
                            <div key={task} className="text-xs font-mono text-amber-500">&#9654; {task}</div>
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
                          {identifier || "\u2014"}
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
        </>
      )}
    </div>
  );
}
