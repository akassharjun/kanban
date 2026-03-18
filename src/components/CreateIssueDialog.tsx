import { useState, useEffect, useRef, useCallback } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select } from "@/components/ui/select";
import { DialogOverlay, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Check, Sparkles } from "lucide-react";
import type { Status, Member, Label, IssueTemplate, TriageSuggestion } from "@/types";
import * as api from "@/tauri/commands";

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
  const [smartTriage, setSmartTriage] = useState(false);
  const [triageSuggestion, setTriageSuggestion] = useState<TriageSuggestion | null>(null);
  const triageTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounced triage on title change when smart triage is enabled
  const runTriage = useCallback(async (titleText: string) => {
    if (!smartTriage || !titleText.trim() || titleText.trim().length < 5) {
      setTriageSuggestion(null);
      return;
    }
    try {
      const suggestion = await api.triageIssue(projectId, titleText, description || undefined);
      setTriageSuggestion(suggestion);
      // Auto-fill only if user hasn't manually set values
      if (suggestion.suggested_priority && priority === "none") {
        setPriority(suggestion.suggested_priority);
      }
      if (suggestion.suggested_label_ids.length > 0 && selectedLabels.length === 0) {
        setSelectedLabels(suggestion.suggested_label_ids);
      }
      if (suggestion.suggested_assignee_id && !assigneeId) {
        setAssigneeId(suggestion.suggested_assignee_id);
      }
    } catch {
      // triage is best-effort
    }
  }, [smartTriage, projectId, description, priority, selectedLabels.length, assigneeId]);

  useEffect(() => {
    if (!smartTriage) return;
    if (triageTimerRef.current) clearTimeout(triageTimerRef.current);
    triageTimerRef.current = setTimeout(() => runTriage(title), 500);
    return () => { if (triageTimerRef.current) clearTimeout(triageTimerRef.current); };
  }, [title, smartTriage, runTriage]);

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
            <label className="block text-[13px] text-muted-foreground mb-1.5">Template</label>
            <div className="flex flex-wrap gap-1.5">
              {templates.map(t => (
                <Button key={t.id} variant="outline" size="sm" onClick={() => applyTemplate(t)}>
                  {t.name}
                </Button>
              ))}
            </div>
          </div>
        )}

        <div className="space-y-4">
          {/* Smart Triage Toggle */}
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setSmartTriage(!smartTriage)}
              className={cn(
                "inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-all",
                smartTriage
                  ? "bg-primary/10 text-primary ring-1 ring-primary/30"
                  : "text-muted-foreground hover:bg-muted"
              )}
            >
              <Sparkles className="h-3 w-3" />
              Smart Triage
            </button>
            {triageSuggestion && smartTriage && (
              <span className="text-[10px] text-muted-foreground/60">
                {Math.round(triageSuggestion.confidence * 100)}% confidence
              </span>
            )}
          </div>

          <div>
            <label className="block text-[13px] text-muted-foreground mb-1.5">Title</label>
            <Input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter" && e.metaKey) handleSubmit(); }}
              placeholder="Issue title"
            />
          </div>

          <div>
            <label className="block text-[13px] text-muted-foreground mb-1.5">Description (Markdown)</label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              placeholder="Describe the issue..."
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-[13px] text-muted-foreground mb-1.5">Status</label>
              <Select
                value={statusId}
                onChange={(e) => setStatusId(Number(e.target.value))}
              >
                {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
              </Select>
            </div>
            <div>
              <label className="block text-[13px] text-muted-foreground mb-1.5">Priority</label>
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
            <label className="block text-[13px] text-muted-foreground mb-1.5">Assignee</label>
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
              <label className="block text-[13px] text-muted-foreground mb-1.5">Labels</label>
              <div className="flex flex-wrap gap-1.5">
                {labels.map(l => {
                  const selected = selectedLabels.includes(l.id);
                  return (
                    <button
                      key={l.id}
                      type="button"
                      onClick={() => setSelectedLabels(prev =>
                        prev.includes(l.id) ? prev.filter(id => id !== l.id) : [...prev, l.id]
                      )}
                      className={cn(
                        "inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-all",
                        selected
                          ? "ring-1 ring-offset-1 ring-offset-card"
                          : "opacity-50 hover:opacity-80"
                      )}
                      style={{
                        backgroundColor: l.color + (selected ? "25" : "12"),
                        color: l.color,
                        ...(selected ? { ringColor: l.color + "50" } : {}),
                      }}
                    >
                      {selected && <Check className="h-3 w-3" />}
                      {l.name}
                    </button>
                  );
                })}
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
