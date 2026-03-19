import { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { DollarSign, TrendingUp, BarChart3, Users, AlertTriangle } from "lucide-react";
import * as api from "@/tauri/commands";
import type { ProjectCostSummary, BudgetStatus } from "@/types";

interface CostDashboardProps {
  projectId: number;
}

export function CostDashboard({ projectId }: CostDashboardProps) {
  const [summary, setSummary] = useState<ProjectCostSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    api.getProjectCostSummary(projectId)
      .then(setSummary)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [projectId]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        Loading cost data...
      </div>
    );
  }

  if (!summary) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        No cost data available
      </div>
    );
  }

  const totalTasks = summary.cost_by_agent.reduce((sum, a) => sum + a.task_count, 0);
  const avgCostPerTask = totalTasks > 0 ? summary.total_cost / totalTasks : 0;
  const maxDailyCost = Math.max(...summary.daily_costs.map(d => d.cost), 1);

  return (
    <div className="space-y-6">
      {/* Overview Cards */}
      <div className="grid grid-cols-4 gap-4">
        <MetricCard
          icon={DollarSign}
          label="Total Spend"
          value={`$${summary.total_cost.toFixed(2)}`}
          color="text-green-500"
        />
        <MetricCard
          icon={BarChart3}
          label="Tasks Tracked"
          value={String(totalTasks)}
          color="text-blue-500"
        />
        <MetricCard
          icon={TrendingUp}
          label="Avg Cost / Task"
          value={`$${avgCostPerTask.toFixed(2)}`}
          color="text-purple-500"
        />
        <MetricCard
          icon={Users}
          label="Active Agents"
          value={String(summary.cost_by_agent.length)}
          color="text-orange-500"
        />
      </div>

      {/* Budget Gauges */}
      {summary.budget_status.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-3">Budget Utilization</h3>
          <div className="grid grid-cols-2 gap-4">
            {summary.budget_status.map((b) => (
              <BudgetGauge key={b.budget_id} budget={b} />
            ))}
          </div>
        </div>
      )}

      {/* Daily Cost Trend */}
      {summary.daily_costs.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-3">Daily Cost Trend</h3>
          <div className="rounded-lg border border-border bg-card p-4">
            <div className="flex items-end gap-1 h-32">
              {summary.daily_costs.map((d) => {
                const height = (d.cost / maxDailyCost) * 100;
                return (
                  <div key={d.date} className="flex-1 flex flex-col items-center gap-1">
                    <span className="text-[10px] text-muted-foreground">${d.cost.toFixed(0)}</span>
                    <div
                      className="w-full rounded-t bg-primary/80 hover:bg-primary transition-colors min-h-[2px]"
                      style={{ height: `${Math.max(height, 2)}%` }}
                      title={`${d.date}: $${d.cost.toFixed(2)} (${d.task_count} tasks)`}
                    />
                    <span className="text-[9px] text-muted-foreground/70">
                      {d.date.slice(5)}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}

      {/* Cost by Agent */}
      {summary.cost_by_agent.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-3">Cost by Agent</h3>
          <div className="rounded-lg border border-border bg-card overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border bg-muted/50">
                  <th className="text-left px-4 py-2 font-medium text-muted-foreground">Agent</th>
                  <th className="text-right px-4 py-2 font-medium text-muted-foreground">Tasks</th>
                  <th className="text-right px-4 py-2 font-medium text-muted-foreground">Total Cost</th>
                  <th className="text-right px-4 py-2 font-medium text-muted-foreground">Avg / Task</th>
                </tr>
              </thead>
              <tbody>
                {summary.cost_by_agent.map((a) => (
                  <tr key={a.agent_id} className="border-b border-border/50 last:border-0">
                    <td className="px-4 py-2 font-medium">{a.agent_name}</td>
                    <td className="px-4 py-2 text-right text-muted-foreground">{a.task_count}</td>
                    <td className="px-4 py-2 text-right">${a.total_cost.toFixed(2)}</td>
                    <td className="px-4 py-2 text-right text-muted-foreground">${a.avg_cost_per_task.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}

function MetricCard({
  icon: Icon,
  label,
  value,
  color,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
  color: string;
}) {
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <div className="flex items-center gap-2 mb-2">
        <Icon className={cn("h-4 w-4", color)} />
        <span className="text-xs text-muted-foreground">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
    </div>
  );
}

function BudgetGauge({ budget }: { budget: BudgetStatus }) {
  const pct = Math.min(budget.percentage * 100, 100);
  const isAlert = budget.alert;
  const isOver = budget.percentage >= 1;

  return (
    <div className={cn(
      "rounded-lg border p-4",
      isOver ? "border-red-500/50 bg-red-500/5" : isAlert ? "border-yellow-500/50 bg-yellow-500/5" : "border-border bg-card"
    )}>
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium capitalize">{budget.budget_type} Budget</span>
        {isAlert && <AlertTriangle className={cn("h-4 w-4", isOver ? "text-red-500" : "text-yellow-500")} />}
      </div>
      <div className="flex items-baseline gap-1 mb-2">
        <span className="text-lg font-bold">${budget.spent.toFixed(2)}</span>
        <span className="text-sm text-muted-foreground">/ ${budget.amount.toFixed(2)}</span>
      </div>
      <div className="h-2 rounded-full bg-muted overflow-hidden">
        <div
          className={cn(
            "h-full rounded-full transition-all",
            isOver ? "bg-red-500" : isAlert ? "bg-yellow-500" : "bg-primary"
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
      <div className="text-xs text-muted-foreground mt-1">{pct.toFixed(0)}% utilized</div>
    </div>
  );
}
