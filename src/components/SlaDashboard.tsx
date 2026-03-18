import { useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Shield, AlertTriangle, XCircle, CheckCircle, Plus, Trash2, Clock } from "lucide-react";
import * as api from "@/tauri/commands";
import type { SlaPolicy, SlaStatus, SlaEvent, SlaDashboard as SlaDashboardType } from "@/types";

interface SlaDashboardProps {
  projectId: number;
}

export function SlaDashboard({ projectId }: SlaDashboardProps) {
  const [dashboard, setDashboard] = useState<SlaDashboardType | null>(null);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);

  const refresh = useCallback(() => {
    setLoading(true);
    api.getSlaDashboard(projectId)
      .then(setDashboard)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  if (loading && !dashboard) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        Loading SLA data...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Summary Metrics */}
      {dashboard && (
        <div className="grid grid-cols-4 gap-4">
          <SlaMetricCard icon={Shield} label="Tracked" value={dashboard.total_tracked} color="text-blue-500" />
          <SlaMetricCard icon={CheckCircle} label="On Track" value={dashboard.total_ok} color="text-green-500" />
          <SlaMetricCard icon={AlertTriangle} label="Warning" value={dashboard.total_warning} color="text-yellow-500" />
          <SlaMetricCard icon={XCircle} label="Breached" value={dashboard.total_breached} color="text-red-500" />
        </div>
      )}

      {/* SLA Policies */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-semibold text-foreground">SLA Policies</h3>
          <Button size="sm" variant="outline" onClick={() => setShowCreate(!showCreate)}>
            <Plus className="h-3.5 w-3.5 mr-1" />
            Add Policy
          </Button>
        </div>

        {showCreate && (
          <CreatePolicyForm
            projectId={projectId}
            onCreated={() => { setShowCreate(false); refresh(); }}
            onCancel={() => setShowCreate(false)}
          />
        )}

        <div className="space-y-2">
          {(dashboard?.policies ?? []).map((policy) => (
            <PolicyRow key={policy.id} policy={policy} onDelete={() => {
              api.deleteSlaPolicy(policy.id).then(refresh);
            }} />
          ))}
          {(dashboard?.policies ?? []).length === 0 && (
            <div className="text-sm text-muted-foreground text-center py-4">
              No SLA policies configured. Add one to start tracking.
            </div>
          )}
        </div>
      </div>

      {/* Live Compliance */}
      {dashboard && dashboard.statuses.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-3">Live Compliance</h3>
          <div className="space-y-2">
            {dashboard.statuses.map((s, i) => (
              <ComplianceRow key={i} status={s} />
            ))}
          </div>
        </div>
      )}

      {/* Recent Events */}
      {dashboard && dashboard.recent_events.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-3">Recent Events</h3>
          <div className="rounded-lg border border-border bg-card overflow-hidden">
            {dashboard.recent_events.map((event) => (
              <EventRow key={event.id} event={event} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function SlaMetricCard({ icon: Icon, label, value, color }: { icon: React.ElementType; label: string; value: number; color: string }) {
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <div className="flex items-center gap-2 mb-1">
        <Icon className={cn("h-4 w-4", color)} />
        <span className="text-xs text-muted-foreground">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
    </div>
  );
}

function PolicyRow({ policy, onDelete }: { policy: SlaPolicy; onDelete: () => void }) {
  const isEnabled = policy.enabled === 1;

  return (
    <div className={cn(
      "flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3",
      !isEnabled && "opacity-50"
    )}>
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm">{policy.name}</span>
          <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground capitalize">
            {policy.target_type.replace("_", " ")}
          </span>
          {policy.priority_filter && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-500 capitalize">
              {policy.priority_filter}
            </span>
          )}
        </div>
        <div className="text-xs text-muted-foreground mt-0.5 flex items-center gap-3">
          <span className="flex items-center gap-1">
            <Clock className="h-3 w-3" />
            Warn at {policy.warning_minutes}m
          </span>
          <span>Breach at {policy.breach_minutes}m</span>
        </div>
      </div>
      <Button size="sm" variant="ghost" onClick={onDelete} className="text-muted-foreground hover:text-red-500">
        <Trash2 className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}

function ComplianceRow({ status }: { status: SlaStatus }) {
  const statusConfig = {
    ok: { color: "text-green-500", bg: "bg-green-500/10", icon: CheckCircle },
    warning: { color: "text-yellow-500", bg: "bg-yellow-500/10", icon: AlertTriangle },
    breached: { color: "text-red-500", bg: "bg-red-500/10", icon: XCircle },
  };
  const config = statusConfig[status.status] ?? statusConfig.ok;
  const StatusIcon = config.icon;

  return (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <StatusIcon className={cn("h-4 w-4 shrink-0", config.color)} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-mono text-xs text-muted-foreground">{status.issue_identifier}</span>
          <span className="text-sm font-medium truncate">{status.issue_title}</span>
        </div>
        <div className="text-xs text-muted-foreground">
          {status.policy_name} - {status.elapsed_minutes.toFixed(0)}m elapsed
          {status.remaining_minutes > 0 && `, ${status.remaining_minutes.toFixed(0)}m remaining`}
        </div>
      </div>
      <span className={cn("text-xs px-2 py-0.5 rounded-full font-medium", config.color, config.bg)}>
        {status.status}
      </span>
    </div>
  );
}

function EventRow({ event }: { event: SlaEvent }) {
  const typeConfig: Record<string, string> = {
    warning: "text-yellow-500",
    breach: "text-red-500",
    escalated: "text-orange-500",
    resolved: "text-green-500",
  };

  return (
    <div className="flex items-center gap-3 px-4 py-2 border-b border-border/50 last:border-0 text-sm">
      <span className={cn("text-xs font-medium uppercase w-16 shrink-0", typeConfig[event.event_type] ?? "text-muted-foreground")}>
        {event.event_type}
      </span>
      <span className="flex-1 truncate text-muted-foreground">{event.message}</span>
      <span className="text-xs text-muted-foreground/70 shrink-0">
        {new Date(event.created_at).toLocaleTimeString()}
      </span>
    </div>
  );
}

function CreatePolicyForm({ projectId, onCreated, onCancel }: { projectId: number; onCreated: () => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [targetType, setTargetType] = useState("resolution_time");
  const [priorityFilter, setPriorityFilter] = useState("");
  const [warningMinutes, setWarningMinutes] = useState("30");
  const [breachMinutes, setBreachMinutes] = useState("60");
  const [escalationType, setEscalationType] = useState("notify");
  const [saving, setSaving] = useState(false);

  const handleSubmit = async () => {
    if (!name.trim()) return;
    setSaving(true);
    try {
      await api.createSlaPolicy({
        project_id: projectId,
        name,
        target_type: targetType,
        priority_filter: priorityFilter || undefined,
        warning_minutes: parseInt(warningMinutes) || 30,
        breach_minutes: parseInt(breachMinutes) || 60,
        escalation_action: JSON.stringify({ type: escalationType }),
      });
      onCreated();
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-lg border border-border bg-card p-4 mb-4 space-y-3">
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Policy Name</label>
          <Input value={name} onChange={e => setName(e.target.value)} placeholder="e.g. Urgent SLA" />
        </div>
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Target Type</label>
          <select
            value={targetType}
            onChange={e => setTargetType(e.target.value)}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="response_time">Response Time</option>
            <option value="resolution_time">Resolution Time</option>
            <option value="task_timeout">Task Timeout</option>
          </select>
        </div>
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Priority Filter</label>
          <select
            value={priorityFilter}
            onChange={e => setPriorityFilter(e.target.value)}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="">All Priorities</option>
            <option value="urgent">Urgent</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Escalation Action</label>
          <select
            value={escalationType}
            onChange={e => setEscalationType(e.target.value)}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="notify">Notify</option>
            <option value="change_priority">Escalate Priority</option>
            <option value="reassign">Reassign Task</option>
          </select>
        </div>
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Warning (minutes)</label>
          <Input type="number" value={warningMinutes} onChange={e => setWarningMinutes(e.target.value)} />
        </div>
        <div>
          <label className="text-xs text-muted-foreground block mb-1">Breach (minutes)</label>
          <Input type="number" value={breachMinutes} onChange={e => setBreachMinutes(e.target.value)} />
        </div>
      </div>
      <div className="flex justify-end gap-2">
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
        <Button size="sm" onClick={handleSubmit} disabled={saving || !name.trim()}>
          {saving ? "Creating..." : "Create Policy"}
        </Button>
      </div>
    </div>
  );
}
