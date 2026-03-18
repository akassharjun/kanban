import { useState, useEffect } from "react";
import { Plus, Trash2, Pencil, X, Play, Pause, Eye } from "lucide-react";
import { cn } from "@/lib/utils";
import type { RecurringIssue, RecurringPreview, Status, Member, RecurrenceType } from "@/types";
import * as api from "@/tauri/commands";

interface RecurringIssuesTabProps {
  projectId: number;
  statuses: Status[];
  members: Member[];
}

const recurrenceTypes: { value: RecurrenceType; label: string }[] = [
  { value: "daily", label: "Daily" },
  { value: "weekly", label: "Weekly" },
  { value: "biweekly", label: "Biweekly" },
  { value: "monthly", label: "Monthly" },
  { value: "custom", label: "Custom" },
];

const priorities = [
  { value: "urgent", label: "Urgent" },
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric", hour: "2-digit", minute: "2-digit" });
  } catch {
    return dateStr.slice(0, 16);
  }
}

export function RecurringIssuesTab({ projectId, statuses, members }: RecurringIssuesTabProps) {
  const [items, setItems] = useState<RecurringIssue[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [preview, setPreview] = useState<RecurringPreview | null>(null);
  const [previewId, setPreviewId] = useState<number | null>(null);

  // Form state
  const [titleTemplate, setTitleTemplate] = useState("");
  const [descTemplate, setDescTemplate] = useState("");
  const [recType, setRecType] = useState<RecurrenceType>("daily");
  const [intervalDays, setIntervalDays] = useState(7);
  const [statusId, setStatusId] = useState<number>(statuses[0]?.id ?? 0);
  const [priority, setPriority] = useState("medium");
  const [assigneeId, setAssigneeId] = useState<number | null>(null);
  const [nextRunAt, setNextRunAt] = useState("");

  useEffect(() => {
    loadData();
  }, [projectId]);

  const loadData = async () => {
    try {
      const data = await api.listRecurring(projectId);
      setItems(data);
    } catch (e) {
      console.error("Failed to load recurring issues", e);
    }
  };

  const resetForm = () => {
    setTitleTemplate("");
    setDescTemplate("");
    setRecType("daily");
    setIntervalDays(7);
    setStatusId(statuses[0]?.id ?? 0);
    setPriority("medium");
    setAssigneeId(null);
    setNextRunAt(new Date(Date.now() + 86400000).toISOString().slice(0, 16));
    setEditingId(null);
    setShowForm(false);
  };

  const handleSave = async () => {
    if (!titleTemplate.trim()) return;
    const config = recType === "custom" ? JSON.stringify({ interval_days: intervalDays }) : "{}";
    const nextRun = nextRunAt ? new Date(nextRunAt).toISOString().replace("T", " ").slice(0, 19) + "Z" : new Date(Date.now() + 86400000).toISOString().replace("T", " ").slice(0, 19) + "Z";

    try {
      if (editingId) {
        await api.updateRecurring(editingId, {
          title_template: titleTemplate,
          description_template: descTemplate || undefined,
          recurrence_type: recType,
          recurrence_config: config,
          status_id: statusId,
          priority,
          assignee_id: assigneeId ?? undefined,
          next_run_at: nextRun,
        });
      } else {
        await api.createRecurring({
          project_id: projectId,
          title_template: titleTemplate,
          description_template: descTemplate || undefined,
          recurrence_type: recType,
          recurrence_config: config,
          status_id: statusId,
          priority,
          assignee_id: assigneeId ?? undefined,
          next_run_at: nextRun,
        });
      }
      resetForm();
      await loadData();
    } catch (e) {
      console.error("Failed to save recurring issue", e);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await api.deleteRecurring(id);
      await loadData();
    } catch (e) {
      console.error("Failed to delete recurring issue", e);
    }
  };

  const handleToggle = async (id: number, enabled: boolean) => {
    try {
      await api.toggleRecurring(id, enabled);
      await loadData();
    } catch (e) {
      console.error("Failed to toggle recurring issue", e);
    }
  };

  const handlePreview = async (id: number) => {
    if (previewId === id) {
      setPreviewId(null);
      setPreview(null);
      return;
    }
    try {
      const p = await api.previewRecurring(id);
      setPreview(p);
      setPreviewId(id);
    } catch (e) {
      console.error("Failed to preview", e);
    }
  };

  const startEdit = (r: RecurringIssue) => {
    setEditingId(r.id);
    setTitleTemplate(r.title_template);
    setDescTemplate(r.description_template || "");
    setRecType(r.recurrence_type as RecurrenceType);
    setStatusId(r.status_id);
    setPriority(r.priority);
    setAssigneeId(r.assignee_id);
    setNextRunAt(r.next_run_at.replace(" ", "T").slice(0, 16));
    const config = JSON.parse(r.recurrence_config || "{}");
    if (config.interval_days) setIntervalDays(config.interval_days);
    setShowForm(true);
  };

  const handleCheckDue = async () => {
    try {
      const created = await api.checkRecurring(projectId);
      if (created.length > 0) {
        await loadData();
      }
    } catch (e) {
      console.error("Failed to check recurring", e);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <p className="text-sm text-muted-foreground">Auto-create issues on a schedule</p>
          <p className="text-xs text-muted-foreground/70 mt-0.5">
            Template variables: {"{{date}}"}, {"{{count}}"}, {"{{day}}"}
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleCheckDue}
            className="flex items-center gap-1 rounded-md border border-border px-3 py-1.5 text-sm hover:bg-accent"
          >
            <Play className="h-3.5 w-3.5" /> Check Due
          </button>
          <button
            onClick={() => {
              resetForm();
              setNextRunAt(new Date(Date.now() + 86400000).toISOString().slice(0, 16));
              setShowForm(true);
            }}
            className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="h-4 w-4" /> Add Recurring
          </button>
        </div>
      </div>

      {/* Form */}
      {showForm && (
        <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">{editingId ? "Edit Recurring Issue" : "New Recurring Issue"}</h3>
            <button onClick={resetForm}><X className="h-4 w-4" /></button>
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">Title Template</label>
            <input
              value={titleTemplate}
              onChange={e => setTitleTemplate(e.target.value)}
              placeholder='e.g. "Daily standup review - {{date}}"'
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
            />
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">Description Template</label>
            <textarea
              value={descTemplate}
              onChange={e => setDescTemplate(e.target.value)}
              rows={3}
              placeholder="Optional description (Markdown, supports {{date}}, {{count}}, {{day}})"
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono"
            />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs text-muted-foreground mb-1">Recurrence</label>
              <select
                value={recType}
                onChange={e => setRecType(e.target.value as RecurrenceType)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {recurrenceTypes.map(t => <option key={t.value} value={t.value}>{t.label}</option>)}
              </select>
            </div>
            {recType === "custom" && (
              <div>
                <label className="block text-xs text-muted-foreground mb-1">Interval (days)</label>
                <input
                  type="number"
                  min={1}
                  value={intervalDays}
                  onChange={e => setIntervalDays(parseInt(e.target.value) || 1)}
                  className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
                />
              </div>
            )}
            <div>
              <label className="block text-xs text-muted-foreground mb-1">Status</label>
              <select
                value={statusId}
                onChange={e => setStatusId(parseInt(e.target.value))}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-muted-foreground mb-1">Priority</label>
              <select
                value={priority}
                onChange={e => setPriority(e.target.value)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {priorities.map(p => <option key={p.value} value={p.value}>{p.label}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-muted-foreground mb-1">Assignee</label>
              <select
                value={assigneeId ?? ""}
                onChange={e => setAssigneeId(e.target.value ? parseInt(e.target.value) : null)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                <option value="">Unassigned</option>
                {members.map(m => <option key={m.id} value={m.id}>{m.display_name || m.name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs text-muted-foreground mb-1">First Run</label>
              <input
                type="datetime-local"
                value={nextRunAt}
                onChange={e => setNextRunAt(e.target.value)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button onClick={resetForm} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
            <button
              onClick={handleSave}
              disabled={!titleTemplate.trim()}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {editingId ? "Save" : "Add"}
            </button>
          </div>
        </div>
      )}

      {/* List */}
      <div className="space-y-1">
        {items.map(r => {
          const status = statuses.find(s => s.id === r.status_id);
          const member = members.find(m => m.id === r.assignee_id);
          return (
            <div key={r.id}>
              <div className={cn(
                "flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3",
                !r.enabled && "opacity-50"
              )}>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium truncate">{r.title_template}</span>
                    <span className={cn(
                      "rounded-full px-2 py-0.5 text-[10px] font-medium",
                      r.enabled ? "bg-green-500/10 text-green-500" : "bg-muted text-muted-foreground"
                    )}>
                      {r.enabled ? "Active" : "Paused"}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                    <span className="capitalize">{r.recurrence_type}</span>
                    {status && (
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full" style={{ backgroundColor: status.color || "#6b7280" }} />
                        {status.name}
                      </span>
                    )}
                    <span>{r.priority}</span>
                    {member && <span>{member.display_name || member.name}</span>}
                    <span>Created: {r.total_created}</span>
                    {r.last_run_at && <span>Last: {formatDate(r.last_run_at)}</span>}
                    <span>Next: {formatDate(r.next_run_at)}</span>
                  </div>
                </div>
                <div className="flex items-center gap-1 ml-2">
                  <button
                    onClick={() => handlePreview(r.id)}
                    className="rounded p-1.5 hover:bg-accent"
                    title="Preview"
                  >
                    <Eye className="h-3.5 w-3.5 text-muted-foreground" />
                  </button>
                  <button
                    onClick={() => handleToggle(r.id, !r.enabled)}
                    className="rounded p-1.5 hover:bg-accent"
                    title={r.enabled ? "Pause" : "Resume"}
                  >
                    {r.enabled ? <Pause className="h-3.5 w-3.5 text-muted-foreground" /> : <Play className="h-3.5 w-3.5 text-muted-foreground" />}
                  </button>
                  <button
                    onClick={() => startEdit(r)}
                    className="rounded p-1.5 hover:bg-accent"
                  >
                    <Pencil className="h-3.5 w-3.5 text-muted-foreground" />
                  </button>
                  <button
                    onClick={() => handleDelete(r.id)}
                    className="rounded p-1.5 hover:bg-destructive/20"
                  >
                    <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
                  </button>
                </div>
              </div>

              {/* Preview */}
              {previewId === r.id && preview && (
                <div className="ml-4 mt-1 mb-2 rounded-lg border border-border/50 bg-muted/30 p-3 text-sm">
                  <div className="font-medium text-xs text-muted-foreground mb-1">Preview - Next Instance</div>
                  <div className="font-medium">{preview.title}</div>
                  {preview.description && (
                    <div className="text-xs text-muted-foreground mt-1">{preview.description}</div>
                  )}
                  <div className="mt-2 text-xs text-muted-foreground">
                    <span className="font-medium">Next 3 dates: </span>
                    {preview.next_dates.map((d, i) => (
                      <span key={i}>
                        {i > 0 && ", "}
                        {formatDate(d)}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          );
        })}
        {items.length === 0 && (
          <div className="py-8 text-center text-sm text-muted-foreground">
            No recurring issues configured yet
          </div>
        )}
      </div>
    </div>
  );
}
