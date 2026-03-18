import { useState, useEffect, useCallback } from "react";
import { listen } from "@/tauri/events";
import {
  Plus, Trash2, Play, X, ChevronRight, GitBranch,
  CheckCircle2, Clock, AlertTriangle, XCircle, Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { Pipeline, PipelineRun, PipelineStage, Issue } from "@/types";
import * as api from "@/tauri/commands";

interface PipelinesViewProps {
  projectId: number | null;
  projectName?: string | null;
  issues?: Issue[];
}

const RUN_STATUS_STYLES: Record<string, { icon: typeof CheckCircle2; color: string; label: string }> = {
  running: { icon: Clock, color: "text-blue-400", label: "Running" },
  completed: { icon: CheckCircle2, color: "text-green-400", label: "Completed" },
  failed: { icon: XCircle, color: "text-red-400", label: "Failed" },
  cancelled: { icon: X, color: "text-muted-foreground", label: "Cancelled" },
};

const STAGE_STATUS_COLORS: Record<string, string> = {
  completed: "bg-green-500 border-green-500",
  queued: "bg-blue-500/30 border-blue-500",
  running: "bg-blue-500 border-blue-500 animate-pulse",
  failed: "bg-red-500 border-red-500",
};

function parseStages(stagesJson: string): PipelineStage[] {
  try {
    return JSON.parse(stagesJson);
  } catch {
    return [];
  }
}

function parseStageTasks(json: string): Array<{ stage_index: number; task_identifier: string; status: string }> {
  try {
    return JSON.parse(json);
  } catch {
    return [];
  }
}

export function PipelinesView({ projectId, projectName: _projectName, issues }: PipelinesViewProps) {
  const [pipelines, setPipelines] = useState<Pipeline[]>([]);
  const [selectedPipeline, setSelectedPipeline] = useState<Pipeline | null>(null);
  const [runs, setRuns] = useState<PipelineRun[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [showTrigger, setShowTrigger] = useState(false);
  const [triggerIssueId, setTriggerIssueId] = useState<number | undefined>();

  const refresh = useCallback(async () => {
    if (!projectId) return;
    try {
      const p = await api.listPipelines(projectId);
      setPipelines(p);
      if (selectedPipeline) {
        const r = await api.listPipelineRuns(selectedPipeline.id);
        setRuns(r);
      }
    } catch {
      // ignore
    }
  }, [projectId, selectedPipeline]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("db-changed", refresh);
    return () => { unlisten.then(fn => fn()); };
  }, [refresh]);

  useEffect(() => {
    if (selectedPipeline) {
      api.listPipelineRuns(selectedPipeline.id).then(setRuns).catch(() => {});
    } else {
      setRuns([]);
    }
  }, [selectedPipeline]);

  const handleDelete = async (id: number) => {
    await api.deletePipeline(id);
    if (selectedPipeline?.id === id) setSelectedPipeline(null);
    refresh();
  };

  const handleTrigger = async () => {
    if (!selectedPipeline) return;
    await api.triggerPipeline(selectedPipeline.id, triggerIssueId);
    setShowTrigger(false);
    setTriggerIssueId(undefined);
    refresh();
  };

  const handleCancel = async (runId: number) => {
    await api.cancelPipeline(runId);
    refresh();
  };

  if (!projectId) {
    return (
      <div className="flex flex-1 items-center justify-center text-muted-foreground">
        Select a project to manage pipelines
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* Pipeline list sidebar */}
      <div className="w-72 border-r border-border/50 flex flex-col">
        <div className="flex items-center justify-between p-4 border-b border-border/50">
          <div className="flex items-center gap-2">
            <GitBranch className="h-4 w-4 text-primary" />
            <h2 className="font-semibold text-sm">Pipelines</h2>
          </div>
          <Button size="sm" variant="ghost" onClick={() => setShowCreate(true)} className="h-7 w-7 p-0">
            <Plus className="h-4 w-4" />
          </Button>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {pipelines.length === 0 && (
            <p className="text-xs text-muted-foreground px-2 py-4 text-center">
              No pipelines yet. Create one to define multi-agent stage chains.
            </p>
          )}
          {pipelines.map(p => {
            const stages = parseStages(p.stages);
            const isSelected = selectedPipeline?.id === p.id;
            return (
              <button
                key={p.id}
                onClick={() => setSelectedPipeline(p)}
                className={cn(
                  "w-full text-left rounded-lg px-3 py-2.5 transition-colors",
                  isSelected ? "bg-primary/10 text-primary" : "text-foreground hover:bg-muted"
                )}
              >
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium truncate">{p.name}</span>
                  {!p.enabled && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground">off</span>
                  )}
                </div>
                <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
                  <span>{stages.length} stages</span>
                  <span>-</span>
                  <span>{p.total_runs} runs</span>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Pipeline detail */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {!selectedPipeline ? (
          <div className="flex flex-1 items-center justify-center text-muted-foreground text-sm">
            Select a pipeline to view details
          </div>
        ) : (
          <>
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-border/50">
              <div>
                <h2 className="text-lg font-semibold">{selectedPipeline.name}</h2>
                {selectedPipeline.description && (
                  <p className="text-sm text-muted-foreground mt-0.5">{selectedPipeline.description}</p>
                )}
              </div>
              <div className="flex items-center gap-2">
                <Button size="sm" variant="outline" onClick={() => setShowTrigger(true)}>
                  <Play className="h-3.5 w-3.5 mr-1.5" />
                  Trigger
                </Button>
                <Button size="sm" variant="ghost" className="text-destructive" onClick={() => handleDelete(selectedPipeline.id)}>
                  <Trash2 className="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>

            {/* Stages visualization */}
            <div className="p-4 border-b border-border/50">
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">Stages</h3>
              <div className="flex items-center gap-1 overflow-x-auto pb-2">
                {parseStages(selectedPipeline.stages).map((stage, i, arr) => (
                  <div key={i} className="flex items-center">
                    <div className="flex flex-col items-center">
                      <div className="flex items-center justify-center w-10 h-10 rounded-full border-2 border-primary/30 bg-primary/10">
                        <span className="text-xs font-bold text-primary">{i + 1}</span>
                      </div>
                      <span className="text-[11px] mt-1.5 font-medium text-center max-w-[80px] truncate">
                        {stage.name}
                      </span>
                      <span className="text-[10px] text-muted-foreground">{stage.task_type}</span>
                    </div>
                    {i < arr.length - 1 && (
                      <ChevronRight className="h-4 w-4 text-muted-foreground/50 mx-1 flex-shrink-0" />
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* Runs */}
            <div className="flex-1 overflow-y-auto p-4">
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">
                Run History ({runs.length})
              </h3>
              {runs.length === 0 && (
                <p className="text-sm text-muted-foreground text-center py-8">
                  No runs yet. Click "Trigger" to start a pipeline run.
                </p>
              )}
              <div className="space-y-3">
                {runs.map(run => {
                  const stages = parseStages(selectedPipeline.stages);
                  const stageTasks = parseStageTasks(run.stage_tasks);
                  const statusInfo = RUN_STATUS_STYLES[run.status] ?? RUN_STATUS_STYLES.running;
                  const StatusIcon = statusInfo.icon;

                  return (
                    <div key={run.id} className="rounded-lg border border-border/50 p-4">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <StatusIcon className={cn("h-4 w-4", statusInfo.color)} />
                          <span className="text-sm font-medium">Run #{run.id}</span>
                          <span className={cn("text-xs px-1.5 py-0.5 rounded", statusInfo.color, "bg-current/10")}>
                            {statusInfo.label}
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-muted-foreground">
                            {new Date(run.started_at).toLocaleString()}
                          </span>
                          {run.status === "running" && (
                            <Button size="sm" variant="ghost" className="h-6 text-xs text-destructive" onClick={() => handleCancel(run.id)}>
                              Cancel
                            </Button>
                          )}
                        </div>
                      </div>

                      {/* Stage progress */}
                      <div className="flex items-center gap-1">
                        {stages.map((stage, i) => {
                          const stageTask = stageTasks.find(st => st.stage_index === i);
                          const stageStatus = stageTask?.status ?? (i <= run.current_stage ? "pending" : "pending");
                          const colorClass = STAGE_STATUS_COLORS[stageStatus] ?? "bg-muted border-border";

                          return (
                            <div key={i} className="flex items-center">
                              <div className="flex flex-col items-center">
                                <div className={cn(
                                  "w-8 h-8 rounded-full border-2 flex items-center justify-center",
                                  colorClass
                                )}>
                                  {stageStatus === "completed" ? (
                                    <CheckCircle2 className="h-4 w-4 text-white" />
                                  ) : stageStatus === "failed" ? (
                                    <XCircle className="h-4 w-4 text-white" />
                                  ) : (
                                    <span className="text-[10px] font-bold text-foreground">{i + 1}</span>
                                  )}
                                </div>
                                <span className="text-[10px] mt-1 text-muted-foreground truncate max-w-[60px]">
                                  {stage.name}
                                </span>
                                {stageTask && (
                                  <span className="text-[9px] text-muted-foreground/70 font-mono">
                                    {stageTask.task_identifier}
                                  </span>
                                )}
                              </div>
                              {i < stages.length - 1 && (
                                <div className={cn(
                                  "w-6 h-0.5 mx-0.5",
                                  stageTask?.status === "completed" ? "bg-green-500" : "bg-border"
                                )} />
                              )}
                            </div>
                          );
                        })}
                      </div>

                      {run.error_message && (
                        <div className="mt-2 flex items-center gap-1.5 text-xs text-red-400">
                          <AlertTriangle className="h-3 w-3" />
                          {run.error_message}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </>
        )}
      </div>

      {/* Create pipeline dialog */}
      {showCreate && (
        <CreatePipelineDialog
          projectId={projectId}
          onClose={() => setShowCreate(false)}
          onCreated={() => { setShowCreate(false); refresh(); }}
        />
      )}

      {/* Trigger dialog */}
      {showTrigger && selectedPipeline && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setShowTrigger(false)}>
          <div className="bg-popover rounded-xl border border-border shadow-xl w-[400px] p-6" onClick={e => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4">Trigger Pipeline</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Start a new run of <span className="font-medium text-foreground">{selectedPipeline.name}</span>
            </p>
            <div className="mb-4">
              <label className="text-xs font-medium text-muted-foreground mb-1 block">Trigger Issue (optional)</label>
              <select
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={triggerIssueId ?? ""}
                onChange={e => setTriggerIssueId(e.target.value ? Number(e.target.value) : undefined)}
              >
                <option value="">None</option>
                {(issues ?? []).map(i => (
                  <option key={i.id} value={i.id}>{i.identifier} - {i.title}</option>
                ))}
              </select>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={() => setShowTrigger(false)}>Cancel</Button>
              <Button onClick={handleTrigger}>
                <Zap className="h-3.5 w-3.5 mr-1.5" />
                Start Run
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// --- Create Pipeline Dialog ---

function CreatePipelineDialog({ projectId, onClose, onCreated }: { projectId: number; onClose: () => void; onCreated: () => void }) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [stages, setStages] = useState<PipelineStage[]>([
    { name: "", task_type: "implementation", required_skills: [], max_complexity: "medium", timeout_minutes: 30, title_template: "{{pipeline.name}}: {{stage.name}} - {{trigger.title}}", objective_template: "", success_criteria: [], auto_advance: true },
  ]);

  const addStage = () => {
    setStages([...stages, {
      name: "", task_type: "implementation", required_skills: [], max_complexity: "medium",
      timeout_minutes: 30, title_template: "{{pipeline.name}}: {{stage.name}} - {{trigger.title}}",
      objective_template: "", success_criteria: [], auto_advance: true,
    }]);
  };

  const removeStage = (i: number) => {
    setStages(stages.filter((_, idx) => idx !== i));
  };

  const updateStage = (i: number, field: keyof PipelineStage, value: unknown) => {
    const updated = [...stages];
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (updated[i] as any)[field] = value;
    setStages(updated);
  };

  const handleSubmit = async () => {
    if (!name.trim() || stages.length === 0) return;
    await api.createPipeline({
      project_id: projectId,
      name: name.trim(),
      description: description.trim() || undefined,
      stages,
    });
    onCreated();
  };

  const loadTemplate = (template: "sdlc" | "bugfix" | "research") => {
    const templates: Record<string, { name: string; desc: string; stages: PipelineStage[] }> = {
      sdlc: {
        name: "SDLC Pipeline", desc: "Standard software development lifecycle",
        stages: [
          { name: "Implement", task_type: "implementation", required_skills: ["rust", "react"], max_complexity: "large", timeout_minutes: 60, title_template: "{{pipeline.name}}: {{stage.name}} - {{trigger.title}}", objective_template: "Implement the changes described in: {{trigger.description}}", success_criteria: ["code compiles", "tests pass"], auto_advance: true },
          { name: "Test", task_type: "testing", required_skills: ["testing"], max_complexity: "medium", timeout_minutes: 30, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Write and run tests for the implementation", success_criteria: ["all tests pass"], auto_advance: true },
          { name: "Review", task_type: "review", required_skills: ["code-review"], max_complexity: "medium", timeout_minutes: 30, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Review the implementation and tests", success_criteria: ["code quality approved"], auto_advance: true },
          { name: "Deploy", task_type: "implementation", required_skills: ["devops"], max_complexity: "small", timeout_minutes: 15, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Deploy the changes", success_criteria: ["deployment successful"], auto_advance: false },
        ],
      },
      bugfix: {
        name: "Bug Fix Pipeline", desc: "Reproduce, fix, test, and review a bug",
        stages: [
          { name: "Reproduce", task_type: "research", required_skills: ["debugging"], max_complexity: "small", timeout_minutes: 15, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Reproduce the bug described in: {{trigger.description}}", success_criteria: ["bug reproduced"], auto_advance: true },
          { name: "Fix", task_type: "implementation", required_skills: ["rust", "react"], max_complexity: "medium", timeout_minutes: 45, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Fix the bug", success_criteria: ["bug fixed", "no regressions"], auto_advance: true },
          { name: "Test", task_type: "testing", required_skills: ["testing"], max_complexity: "small", timeout_minutes: 20, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Verify the fix", success_criteria: ["regression test passes"], auto_advance: true },
          { name: "Review", task_type: "review", required_skills: ["code-review"], max_complexity: "small", timeout_minutes: 15, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Review the fix", success_criteria: ["approved"], auto_advance: false },
        ],
      },
      research: {
        name: "Research Pipeline", desc: "Research, document, and review",
        stages: [
          { name: "Research", task_type: "research", required_skills: ["analysis"], max_complexity: "medium", timeout_minutes: 60, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Research the topic: {{trigger.title}}", success_criteria: ["findings documented"], auto_advance: true },
          { name: "Document", task_type: "implementation", required_skills: ["documentation"], max_complexity: "small", timeout_minutes: 30, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Write documentation based on research findings", success_criteria: ["docs complete"], auto_advance: true },
          { name: "Review", task_type: "review", required_skills: ["code-review"], max_complexity: "small", timeout_minutes: 15, title_template: "{{pipeline.name}}: {{stage.name}}", objective_template: "Review documentation quality", success_criteria: ["approved"], auto_advance: false },
        ],
      },
    };
    const t = templates[template];
    setName(t.name);
    setDescription(t.desc);
    setStages(t.stages);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div className="bg-popover rounded-xl border border-border shadow-xl w-[640px] max-h-[85vh] flex flex-col" onClick={e => e.stopPropagation()}>
        <div className="p-6 border-b border-border/50">
          <h3 className="text-lg font-semibold">Create Pipeline</h3>
          <div className="flex gap-2 mt-2">
            <button onClick={() => loadTemplate("sdlc")} className="text-xs px-2 py-1 rounded bg-primary/10 text-primary hover:bg-primary/20 transition-colors">
              SDLC Template
            </button>
            <button onClick={() => loadTemplate("bugfix")} className="text-xs px-2 py-1 rounded bg-orange-500/10 text-orange-400 hover:bg-orange-500/20 transition-colors">
              Bug Fix Template
            </button>
            <button onClick={() => loadTemplate("research")} className="text-xs px-2 py-1 rounded bg-purple-500/10 text-purple-400 hover:bg-purple-500/20 transition-colors">
              Research Template
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">Name</label>
            <input
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              value={name} onChange={e => setName(e.target.value)} placeholder="Pipeline name"
            />
          </div>
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">Description</label>
            <input
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              value={description} onChange={e => setDescription(e.target.value)} placeholder="Optional description"
            />
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-xs font-medium text-muted-foreground">Stages</label>
              <Button size="sm" variant="ghost" onClick={addStage} className="h-6 text-xs">
                <Plus className="h-3 w-3 mr-1" /> Add Stage
              </Button>
            </div>
            <div className="space-y-3">
              {stages.map((stage, i) => (
                <div key={i} className="rounded-lg border border-border/50 p-3 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-muted-foreground">Stage {i + 1}</span>
                    {stages.length > 1 && (
                      <Button size="sm" variant="ghost" className="h-5 w-5 p-0 text-destructive" onClick={() => removeStage(i)}>
                        <X className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <label className="text-[10px] text-muted-foreground">Name</label>
                      <input className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.name} onChange={e => updateStage(i, "name", e.target.value)} placeholder="Stage name" />
                    </div>
                    <div>
                      <label className="text-[10px] text-muted-foreground">Task Type</label>
                      <select className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.task_type} onChange={e => updateStage(i, "task_type", e.target.value)}>
                        <option value="implementation">Implementation</option>
                        <option value="testing">Testing</option>
                        <option value="review">Review</option>
                        <option value="research">Research</option>
                        <option value="decomposition">Decomposition</option>
                      </select>
                    </div>
                    <div>
                      <label className="text-[10px] text-muted-foreground">Complexity</label>
                      <select className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.max_complexity} onChange={e => updateStage(i, "max_complexity", e.target.value)}>
                        <option value="small">Small</option>
                        <option value="medium">Medium</option>
                        <option value="large">Large</option>
                      </select>
                    </div>
                    <div>
                      <label className="text-[10px] text-muted-foreground">Timeout (min)</label>
                      <input type="number" className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.timeout_minutes} onChange={e => updateStage(i, "timeout_minutes", parseInt(e.target.value) || 30)} />
                    </div>
                    <div className="col-span-2">
                      <label className="text-[10px] text-muted-foreground">Skills (comma-separated)</label>
                      <input className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.required_skills.join(", ")}
                        onChange={e => updateStage(i, "required_skills", e.target.value.split(",").map(s => s.trim()).filter(Boolean))}
                        placeholder="rust, react, testing" />
                    </div>
                    <div className="col-span-2">
                      <label className="text-[10px] text-muted-foreground">Objective Template</label>
                      <input className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.objective_template} onChange={e => updateStage(i, "objective_template", e.target.value)}
                        placeholder="Use {{trigger.title}}, {{trigger.description}}, {{pipeline.name}}, {{stage.name}}" />
                    </div>
                    <div className="col-span-2">
                      <label className="text-[10px] text-muted-foreground">Success Criteria (comma-separated)</label>
                      <input className="w-full rounded border border-input bg-background px-2 py-1 text-xs"
                        value={stage.success_criteria.join(", ")}
                        onChange={e => updateStage(i, "success_criteria", e.target.value.split(",").map(s => s.trim()).filter(Boolean))}
                        placeholder="code compiles, tests pass" />
                    </div>
                    <div className="col-span-2 flex items-center gap-2">
                      <input type="checkbox" id={`auto-${i}`} checked={stage.auto_advance}
                        onChange={e => updateStage(i, "auto_advance", e.target.checked)} />
                      <label htmlFor={`auto-${i}`} className="text-xs text-muted-foreground">Auto-advance on completion</label>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 p-4 border-t border-border/50">
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSubmit} disabled={!name.trim() || stages.length === 0 || stages.some(s => !s.name.trim())}>
            Create Pipeline
          </Button>
        </div>
      </div>
    </div>
  );
}
