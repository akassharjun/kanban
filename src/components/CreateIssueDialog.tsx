import { useState } from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Status, Member, Label, IssueTemplate } from "@/types";

interface CreateIssueDialogProps {
  projectId: number;
  statuses: Status[];
  members: Member[];
  labels: Label[];
  templates: IssueTemplate[];
  defaultStatusId?: number;
  parentId?: number;
  onClose: () => void;
  onCreate: (input: {
    project_id: number;
    title: string;
    description?: string;
    status_id: number;
    priority?: string;
    assignee_id?: number;
    parent_id?: number;
    label_ids?: number[];
  }) => Promise<unknown>;
}

export function CreateIssueDialog({
  projectId,
  statuses,
  members,
  labels,
  templates,
  defaultStatusId,
  parentId,
  onClose,
  onCreate,
}: CreateIssueDialogProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [statusId, setStatusId] = useState(defaultStatusId || statuses[0]?.id || 0);
  const [priority, setPriority] = useState("none");
  const [assigneeId, setAssigneeId] = useState<number | undefined>();
  const [selectedLabels, setSelectedLabels] = useState<number[]>([]);

  const applyTemplate = (template: IssueTemplate) => {
    if (template.description_template) setDescription(template.description_template);
    if (template.default_status_id) setStatusId(template.default_status_id);
    if (template.default_priority !== "none") setPriority(template.default_priority);
    try {
      const ids = JSON.parse(template.default_label_ids);
      if (Array.isArray(ids)) setSelectedLabels(ids);
    } catch {
      // ignore parse errors
    }
  };

  const handleSubmit = async () => {
    if (!title.trim()) return;
    await onCreate({
      project_id: projectId,
      title: title.trim(),
      description: description || undefined,
      status_id: statusId,
      priority,
      assignee_id: assigneeId,
      parent_id: parentId,
      label_ids: selectedLabels.length > 0 ? selectedLabels : undefined,
    });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div className="w-[520px] rounded-lg border border-border bg-card p-6 shadow-xl max-h-[85vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">{parentId ? "New Sub-issue" : "New Issue"}</h2>
          <button onClick={onClose} className="rounded p-1 hover:bg-accent"><X className="h-4 w-4" /></button>
        </div>

        {templates.length > 0 && (
          <div className="mb-4">
            <label className="block text-sm text-muted-foreground mb-1">Template</label>
            <div className="flex flex-wrap gap-1">
              {templates.map(t => (
                <button key={t.id} onClick={() => applyTemplate(t)} className="rounded-md border border-border px-2 py-1 text-xs hover:bg-accent">
                  {t.name}
                </button>
              ))}
            </div>
          </div>
        )}

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Title</label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter" && e.metaKey) handleSubmit(); }}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="Issue title"
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Description (Markdown)</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono"
              placeholder="Describe the issue..."
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Status</label>
              <select
                value={statusId}
                onChange={(e) => setStatusId(Number(e.target.value))}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Priority</label>
              <select
                value={priority}
                onChange={(e) => setPriority(e.target.value)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                <option value="none">None</option>
                <option value="urgent">Urgent</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
                <option value="low">Low</option>
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Assignee</label>
            <select
              value={assigneeId ?? ""}
              onChange={(e) => setAssigneeId(e.target.value ? Number(e.target.value) : undefined)}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
            >
              <option value="">Unassigned</option>
              {members.map(m => <option key={m.id} value={m.id}>{m.display_name || m.name}</option>)}
            </select>
          </div>

          {labels.length > 0 && (
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Labels</label>
              <div className="flex flex-wrap gap-1">
                {labels.map(l => (
                  <button
                    key={l.id}
                    onClick={() => setSelectedLabels(prev =>
                      prev.includes(l.id) ? prev.filter(id => id !== l.id) : [...prev, l.id]
                    )}
                    className={cn(
                      "rounded-full px-2 py-0.5 text-xs border transition-colors",
                      selectedLabels.includes(l.id)
                        ? "border-transparent"
                        : "border-border opacity-50"
                    )}
                    style={{
                      backgroundColor: selectedLabels.includes(l.id) ? l.color + "30" : "transparent",
                      color: l.color,
                    }}
                  >
                    {l.name}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button onClick={onClose} className="rounded-md px-4 py-2 text-sm hover:bg-accent">Cancel</button>
          <button
            onClick={handleSubmit}
            disabled={!title.trim()}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
