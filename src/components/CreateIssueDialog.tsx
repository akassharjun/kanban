import { useState } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { DialogOverlay, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
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
    <DialogOverlay onClose={onClose}>
      <DialogContent>
        <DialogHeader onClose={onClose}>
          <DialogTitle>{parentId ? "New Sub-issue" : "New Issue"}</DialogTitle>
        </DialogHeader>

        {templates.length > 0 && (
          <div className="mb-4">
            <label className="block text-sm text-muted-foreground mb-1">Template</label>
            <div className="flex flex-wrap gap-1">
              {templates.map(t => (
                <Button key={t.id} variant="outline" size="sm" onClick={() => applyTemplate(t)}>
                  {t.name}
                </Button>
              ))}
            </div>
          </div>
        )}

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-muted-foreground mb-1">Title</label>
            <Input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter" && e.metaKey) handleSubmit(); }}
              placeholder="Issue title"
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Description (Markdown)</label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              className="font-mono"
              placeholder="Describe the issue..."
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Status</label>
              <Select
                value={statusId}
                onChange={(e) => setStatusId(Number(e.target.value))}
              >
                {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
              </Select>
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Priority</label>
              <Select
                value={priority}
                onChange={(e) => setPriority(e.target.value)}
              >
                <option value="none">None</option>
                <option value="urgent">Urgent</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
                <option value="low">Low</option>
              </Select>
            </div>
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Assignee</label>
            <Select
              value={assigneeId ?? ""}
              onChange={(e) => setAssigneeId(e.target.value ? Number(e.target.value) : undefined)}
            >
              <option value="">Unassigned</option>
              {members.map(m => <option key={m.id} value={m.id}>{m.display_name || m.name}</option>)}
            </Select>
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
                  >
                    <Badge
                      variant="outline"
                      className={cn(
                        "cursor-pointer transition-colors",
                        selectedLabels.includes(l.id)
                          ? "border-transparent"
                          : "opacity-50"
                      )}
                      style={{
                        backgroundColor: selectedLabels.includes(l.id) ? l.color + "30" : "transparent",
                        color: l.color,
                      }}
                    >
                      {l.name}
                    </Badge>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSubmit} disabled={!title.trim()}>
            Create
          </Button>
        </div>
      </DialogContent>
    </DialogOverlay>
  );
}
