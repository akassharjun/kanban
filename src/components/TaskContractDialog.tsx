import { useState } from "react";
import { X } from "lucide-react";
import * as api from "@/tauri/commands";

interface TaskContractDialogProps {
  projectId: number;
  statusId: number;
  defaultTitle?: string;
  onClose: () => void;
  onCreated: () => void;
}

const taskTypes = ["implementation", "research", "testing", "review"];
const complexities = ["small", "medium", "large"];

export function TaskContractDialog({
  projectId,
  statusId,
  defaultTitle = "",
  onClose,
  onCreated,
}: TaskContractDialogProps) {
  const [title, setTitle] = useState(defaultTitle);
  const [objective, setObjective] = useState("");
  const [type, setType] = useState("implementation");
  const [skills, setSkills] = useState("");
  const [complexity, setComplexity] = useState("medium");
  const [timeoutMinutes, setTimeoutMinutes] = useState<number | undefined>();
  const [successCriteria, setSuccessCriteria] = useState("");
  const [contextFiles, setContextFiles] = useState("");
  const [dependsOn, setDependsOn] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async () => {
    if (!title.trim() || !objective.trim()) return;
    setSaving(true);
    setError(null);

    try {
      const skillsArray = skills.trim()
        ? skills.split(",").map(s => s.trim()).filter(Boolean)
        : undefined;
      const criteriaArray = successCriteria.trim()
        ? successCriteria.split("\n").map(s => s.trim()).filter(Boolean)
        : undefined;
      const filesArray = contextFiles.trim()
        ? contextFiles.split("\n").map(s => s.trim()).filter(Boolean)
        : undefined;
      const dependsArray = dependsOn.trim()
        ? dependsOn.split(",").map(s => s.trim()).filter(Boolean)
        : undefined;

      await api.createTaskContract({
        project_id: projectId,
        title: title.trim(),
        objective: objective.trim(),
        status_id: statusId,
        type,
        skills: skillsArray,
        complexity,
        timeout_minutes: timeoutMinutes,
        success_criteria: criteriaArray,
        context_files: filesArray,
        depends_on: dependsArray,
      });

      onCreated();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="w-[560px] rounded-lg border border-border bg-card p-6 shadow-xl max-h-[85vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Create Task Contract</h2>
          <button onClick={onClose} className="rounded p-1 hover:bg-accent">
            <X className="h-4 w-4" />
          </button>
        </div>

        {error && (
          <div className="mb-4 rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-400">
            {error}
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
              placeholder="Task title"
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Objective</label>
            <textarea
              value={objective}
              onChange={(e) => setObjective(e.target.value)}
              rows={3}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="What should this task accomplish?"
            />
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Type</label>
              <select
                value={type}
                onChange={(e) => setType(e.target.value)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {taskTypes.map(t => (
                  <option key={t} value={t}>{t.charAt(0).toUpperCase() + t.slice(1)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Complexity</label>
              <select
                value={complexity}
                onChange={(e) => setComplexity(e.target.value)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
              >
                {complexities.map(c => (
                  <option key={c} value={c}>{c.charAt(0).toUpperCase() + c.slice(1)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Timeout (min)</label>
              <input
                type="number"
                min="1"
                value={timeoutMinutes ?? ""}
                onChange={(e) => setTimeoutMinutes(e.target.value ? Number(e.target.value) : undefined)}
                className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
                placeholder="Optional"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Skills (comma-separated)</label>
            <input
              value={skills}
              onChange={(e) => setSkills(e.target.value)}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="rust, typescript, sql"
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Success Criteria (one per line)</label>
            <textarea
              value={successCriteria}
              onChange={(e) => setSuccessCriteria(e.target.value)}
              rows={3}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono"
              placeholder={"All tests pass\nNo regressions\nDocumented"}
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Context Files (one per line)</label>
            <textarea
              value={contextFiles}
              onChange={(e) => setContextFiles(e.target.value)}
              rows={2}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono"
              placeholder={"src/components/App.tsx\nsrc/types/index.ts"}
            />
          </div>

          <div>
            <label className="block text-sm text-muted-foreground mb-1">Depends On (comma-separated identifiers)</label>
            <input
              value={dependsOn}
              onChange={(e) => setDependsOn(e.target.value)}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              placeholder="KAN-12, KAN-15"
            />
          </div>
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button onClick={onClose} className="rounded-md px-4 py-2 text-sm hover:bg-accent">
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!title.trim() || !objective.trim() || saving}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {saving ? "Creating..." : "Create Contract"}
          </button>
        </div>
      </div>
    </div>
  );
}
