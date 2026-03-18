import { useState, useEffect } from "react";
import { Plus, Pencil, Trash2, X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Project, Status, Label, IssueTemplate, Hook, ProjectAgentConfig } from "@/types";
import * as api from "@/tauri/commands";

export interface ProjectSettingsViewProps {
  project: Project;
  onUpdateProject: (id: number, input: { name?: string; description?: string; icon?: string; status?: string; path?: string }) => Promise<unknown>;
  onRefreshStatuses: () => void;
  onRefreshLabels: () => void;
  onDeleteProject?: (id: number) => Promise<unknown>;
}

type Tab = "general" | "statuses" | "labels" | "templates" | "hooks" | "agents" | "stale";

const statusCategories = [
  { value: "unstarted", label: "Unstarted" },
  { value: "started", label: "Started" },
  { value: "blocked", label: "Blocked" },
  { value: "completed", label: "Completed" },
  { value: "discarded", label: "Discarded" },
];

const labelColors = [
  "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#22c55e",
  "#14b8a6", "#06b6d4", "#3b82f6", "#6366f1", "#8b5cf6",
  "#a855f7", "#d946ef", "#ec4899", "#f43f5e",
];

export function ProjectSettingsView({ project, onUpdateProject, onRefreshStatuses, onRefreshLabels, onDeleteProject: _onDeleteProject }: ProjectSettingsViewProps) {
  const [tab, setTab] = useState<Tab>("general");
  const [statuses, setStatuses] = useState<Status[]>([]);
  const [labels, setLabels] = useState<Label[]>([]);
  const [templates, setTemplates] = useState<IssueTemplate[]>([]);
  const [hooks, setHooks] = useState<Hook[]>([]);
  const [agentConfig, setAgentConfig] = useState<ProjectAgentConfig | null>(null);
  const [agentConfigSaving, setAgentConfigSaving] = useState(false);
  const [path, setPath] = useState(project.path || "");

  // General form
  const [name, setName] = useState(project.name);
  const [description, setDescription] = useState(project.description || "");
  const [icon, setIcon] = useState(project.icon || "");
  const [projectStatus, setProjectStatus] = useState(project.status);

  // Status form
  const [showAddStatus, setShowAddStatus] = useState(false);
  const [statusName, setStatusName] = useState("");
  const [statusCategory, setStatusCategory] = useState("unstarted");
  const [statusColor, setStatusColor] = useState("#6b7280");
  const [editingStatusId, setEditingStatusId] = useState<number | null>(null);

  // Label form
  const [showAddLabel, setShowAddLabel] = useState(false);
  const [labelName, setLabelName] = useState("");
  const [labelColor, setLabelColor] = useState(labelColors[0]);
  const [editingLabelId, setEditingLabelId] = useState<number | null>(null);

  // Template form
  const [showAddTemplate, setShowAddTemplate] = useState(false);
  const [templateName, setTemplateName] = useState("");
  const [templateDesc, setTemplateDesc] = useState("");
  const [templatePriority, setTemplatePriority] = useState("none");
  const [editingTemplateId, setEditingTemplateId] = useState<number | null>(null);

  // Hook form
  const [showAddHook, setShowAddHook] = useState(false);
  const [hookEventType, setHookEventType] = useState("task_completed");
  const [hookCommand, setHookCommand] = useState("");

  // Stale issues config
  const [staleEnabled, setStaleEnabled] = useState(project.stale_days !== null);
  const [staleDays, setStaleDays] = useState(project.stale_days ?? 30);
  const [staleCloseStatusId, setStaleCloseStatusId] = useState<number | null>(project.stale_close_status_id);

  useEffect(() => {
    loadData();
  }, [project.id]);

  const loadData = async () => {
    const [s, l, t, h, ac] = await Promise.all([
      api.listStatuses(project.id),
      api.listLabels(project.id),
      api.listTemplates(project.id),
      api.listHooks(project.id),
      api.getProjectAgentConfig(project.id),
    ]);
    setStatuses(s);
    setLabels(l);
    setTemplates(t);
    setHooks(h);
    setAgentConfig(ac);
  };

  const handleSaveGeneral = async () => {
    await onUpdateProject(project.id, {
      name,
      description: description || undefined,
      icon,
      status: projectStatus,
      path: path || undefined,
    });
  };

  // Status handlers
  const handleAddStatus = async () => {
    if (!statusName.trim()) return;
    if (editingStatusId) {
      await api.updateStatus(editingStatusId, { name: statusName, category: statusCategory, color: statusColor });
    } else {
      await api.createStatus({ project_id: project.id, name: statusName, category: statusCategory, color: statusColor });
    }
    setShowAddStatus(false); setEditingStatusId(null); setStatusName(""); setStatusCategory("unstarted"); setStatusColor("#6b7280");
    await loadData();
    onRefreshStatuses();
  };

  const handleDeleteStatus = async (id: number) => {
    await api.deleteStatus(id);
    await loadData();
    onRefreshStatuses();
  };

  const startEditStatus = (s: Status) => {
    setEditingStatusId(s.id);
    setStatusName(s.name);
    setStatusCategory(s.category);
    setStatusColor(s.color || "#6b7280");
    setShowAddStatus(true);
  };

  // Label handlers
  const handleAddLabel = async () => {
    if (!labelName.trim()) return;
    if (editingLabelId) {
      await api.updateLabel(editingLabelId, { name: labelName, color: labelColor });
    } else {
      await api.createLabel({ project_id: project.id, name: labelName, color: labelColor });
    }
    setShowAddLabel(false); setEditingLabelId(null); setLabelName(""); setLabelColor(labelColors[0]);
    await loadData();
    onRefreshLabels();
  };

  const handleDeleteLabel = async (id: number) => {
    await api.deleteLabel(id);
    await loadData();
    onRefreshLabels();
  };

  const startEditLabel = (l: Label) => {
    setEditingLabelId(l.id);
    setLabelName(l.name);
    setLabelColor(l.color);
    setShowAddLabel(true);
  };

  // Template handlers
  const handleAddTemplate = async () => {
    if (!templateName.trim()) return;
    if (editingTemplateId) {
      await api.updateTemplate(editingTemplateId, { name: templateName, description_template: templateDesc || undefined, default_priority: templatePriority });
    } else {
      await api.createTemplate({ project_id: project.id, name: templateName, description_template: templateDesc || undefined, default_priority: templatePriority });
    }
    setShowAddTemplate(false); setEditingTemplateId(null); setTemplateName(""); setTemplateDesc(""); setTemplatePriority("none");
    await loadData();
  };

  const handleDeleteTemplate = async (id: number) => {
    await api.deleteTemplate(id);
    await loadData();
  };

  const startEditTemplate = (t: IssueTemplate) => {
    setEditingTemplateId(t.id);
    setTemplateName(t.name);
    setTemplateDesc(t.description_template || "");
    setTemplatePriority(t.default_priority);
    setShowAddTemplate(true);
  };

  // Hook handlers
  const handleAddHook = async () => {
    if (!hookCommand.trim()) return;
    await api.createHook({ project_id: project.id, event_type: hookEventType, command: hookCommand });
    setShowAddHook(false); setHookEventType("task_completed"); setHookCommand("");
    await loadData();
  };

  const handleDeleteHook = async (id: number) => {
    await api.deleteHook(id);
    await loadData();
  };

  const tabs: { value: Tab; label: string }[] = [
    { value: "general", label: "General" },
    { value: "statuses", label: "Statuses" },
    { value: "labels", label: "Labels" },
    { value: "templates", label: "Templates" },
    { value: "hooks", label: "Hooks" },
    { value: "stale", label: "Stale Issues" },
    { value: "agents", label: "Agent Config" },
  ];

  const handleSaveAgentConfig = async () => {
    if (!agentConfig) return;
    setAgentConfigSaving(true);
    try {
      const updated = await api.updateProjectAgentConfig(project.id, {
        auto_accept_threshold: agentConfig.auto_accept_threshold,
        human_review_threshold: agentConfig.human_review_threshold,
        max_attempts: agentConfig.max_attempts,
        heartbeat_interval_seconds: agentConfig.heartbeat_interval_seconds,
        missed_heartbeats_before_offline: agentConfig.missed_heartbeats_before_offline,
      });
      setAgentConfig(updated);
    } finally {
      setAgentConfigSaving(false);
    }
  };

  return (
    <div className="flex-1 overflow-auto p-6">
      <div className="max-w-2xl">
        <h1 className="text-xl font-semibold mb-6">Project Settings</h1>

        {/* Tabs */}
        <div className="flex gap-1 mb-6 border-b border-border">
          {tabs.map(t => (
            <button
              key={t.value}
              onClick={() => setTab(t.value)}
              className={cn(
                "px-4 py-2 text-sm border-b-2 transition-colors -mb-[1px]",
                tab === t.value ? "border-primary text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"
              )}
            >
              {t.label}
            </button>
          ))}
        </div>

        {/* General Tab */}
        {tab === "general" && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Project Name</label>
              <input value={name} onChange={e => setName(e.target.value)} className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Description</label>
              <textarea value={description} onChange={e => setDescription(e.target.value)} rows={3} className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div className="flex gap-4">
              <div>
                <label className="block text-sm text-muted-foreground mb-1">Icon</label>
                <input value={icon} onChange={e => setIcon(e.target.value)} className="w-20 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none text-center" />
              </div>
              <div>
                <label className="block text-sm text-muted-foreground mb-1">Status</label>
                <select value={projectStatus} onChange={e => setProjectStatus(e.target.value as Project["status"])} className="rounded-md border border-border bg-background px-3 py-2 text-sm outline-none">
                  <option value="active">Active</option>
                  <option value="paused">Paused</option>
                  <option value="completed">Completed</option>
                  <option value="archived">Archived</option>
                </select>
              </div>
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Prefix</label>
              <input value={project.prefix} disabled className="w-32 rounded-md border border-border bg-background/50 px-3 py-2 text-sm text-muted-foreground" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Project Path</label>
              <p className="text-xs text-muted-foreground/70 mb-1">Local directory for this project. Used by agents to access the codebase.</p>
              <input value={path} onChange={e => setPath(e.target.value)} placeholder="/path/to/project" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono" />
            </div>
            <button onClick={handleSaveGeneral} className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90">
              Save Changes
            </button>
          </div>
        )}

        {/* Statuses Tab */}
        {tab === "statuses" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <p className="text-sm text-muted-foreground">Configure workflow statuses for this project</p>
              <button onClick={() => { setShowAddStatus(true); setEditingStatusId(null); setStatusName(""); setStatusCategory("unstarted"); setStatusColor("#6b7280"); }} className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                <Plus className="h-4 w-4" /> Add Status
              </button>
            </div>

            {showAddStatus && (
              <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium">{editingStatusId ? "Edit Status" : "New Status"}</h3>
                  <button onClick={() => { setShowAddStatus(false); setEditingStatusId(null); }}><X className="h-4 w-4" /></button>
                </div>
                <input value={statusName} onChange={e => setStatusName(e.target.value)} placeholder="Status name" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
                <div className="flex gap-4">
                  <div className="flex-1">
                    <label className="block text-xs text-muted-foreground mb-1">Category</label>
                    <select value={statusCategory} onChange={e => setStatusCategory(e.target.value)} className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none">
                      {statusCategories.map(c => <option key={c.value} value={c.value}>{c.label}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs text-muted-foreground mb-1">Color</label>
                    <input type="color" value={statusColor} onChange={e => setStatusColor(e.target.value)} className="h-9 w-12 rounded border border-border bg-background cursor-pointer" />
                  </div>
                </div>
                <div className="flex justify-end gap-2">
                  <button onClick={() => { setShowAddStatus(false); setEditingStatusId(null); }} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
                  <button onClick={handleAddStatus} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">{editingStatusId ? "Save" : "Add"}</button>
                </div>
              </div>
            )}

            <div className="space-y-1">
              {statuses.map(s => (
                <div key={s.id} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                  <div className="flex items-center gap-3">
                    <span className="h-3 w-3 rounded-full" style={{ backgroundColor: s.color || "#6b7280" }} />
                    <span className="text-sm font-medium">{s.name}</span>
                    <span className="rounded-full bg-accent px-2 py-0.5 text-[10px] text-muted-foreground">{s.category}</span>
                  </div>
                  <div className="flex items-center gap-1">
                    <button onClick={() => startEditStatus(s)} className="rounded p-1.5 hover:bg-accent"><Pencil className="h-3.5 w-3.5 text-muted-foreground" /></button>
                    <button onClick={() => handleDeleteStatus(s.id)} className="rounded p-1.5 hover:bg-destructive/20"><Trash2 className="h-3.5 w-3.5 text-muted-foreground" /></button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Labels Tab */}
        {tab === "labels" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <p className="text-sm text-muted-foreground">Manage labels for this project</p>
              <button onClick={() => { setShowAddLabel(true); setEditingLabelId(null); setLabelName(""); setLabelColor(labelColors[0]); }} className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                <Plus className="h-4 w-4" /> Add Label
              </button>
            </div>

            {showAddLabel && (
              <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium">{editingLabelId ? "Edit Label" : "New Label"}</h3>
                  <button onClick={() => { setShowAddLabel(false); setEditingLabelId(null); }}><X className="h-4 w-4" /></button>
                </div>
                <input value={labelName} onChange={e => setLabelName(e.target.value)} placeholder="Label name" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">Color</label>
                  <div className="flex flex-wrap gap-1">
                    {labelColors.map(c => (
                      <button key={c} onClick={() => setLabelColor(c)} className={`h-6 w-6 rounded-full border-2 ${labelColor === c ? "border-white" : "border-transparent"}`} style={{ backgroundColor: c }} />
                    ))}
                  </div>
                </div>
                <div className="flex justify-end gap-2">
                  <button onClick={() => { setShowAddLabel(false); setEditingLabelId(null); }} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
                  <button onClick={handleAddLabel} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">{editingLabelId ? "Save" : "Add"}</button>
                </div>
              </div>
            )}

            <div className="space-y-1">
              {labels.map(l => (
                <div key={l.id} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                  <div className="flex items-center gap-2">
                    <span className="rounded-full px-2.5 py-0.5 text-xs font-medium" style={{ backgroundColor: l.color + "20", color: l.color }}>{l.name}</span>
                  </div>
                  <div className="flex items-center gap-1">
                    <button onClick={() => startEditLabel(l)} className="rounded p-1.5 hover:bg-accent"><Pencil className="h-3.5 w-3.5 text-muted-foreground" /></button>
                    <button onClick={() => handleDeleteLabel(l.id)} className="rounded p-1.5 hover:bg-destructive/20"><Trash2 className="h-3.5 w-3.5 text-muted-foreground" /></button>
                  </div>
                </div>
              ))}
              {labels.length === 0 && <div className="py-8 text-center text-sm text-muted-foreground">No labels yet</div>}
            </div>
          </div>
        )}

        {/* Templates Tab */}
        {tab === "templates" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <p className="text-sm text-muted-foreground">Issue templates for quick creation</p>
              <button onClick={() => { setShowAddTemplate(true); setEditingTemplateId(null); setTemplateName(""); setTemplateDesc(""); setTemplatePriority("none"); }} className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                <Plus className="h-4 w-4" /> Add Template
              </button>
            </div>

            {showAddTemplate && (
              <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium">{editingTemplateId ? "Edit Template" : "New Template"}</h3>
                  <button onClick={() => { setShowAddTemplate(false); setEditingTemplateId(null); }}><X className="h-4 w-4" /></button>
                </div>
                <input value={templateName} onChange={e => setTemplateName(e.target.value)} placeholder="Template name (e.g. Bug Report)" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
                <textarea value={templateDesc} onChange={e => setTemplateDesc(e.target.value)} placeholder="Description template (Markdown)" rows={4} className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono" />
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">Default Priority</label>
                  <select value={templatePriority} onChange={e => setTemplatePriority(e.target.value)} className="rounded-md border border-border bg-background px-3 py-2 text-sm outline-none">
                    <option value="none">None</option>
                    <option value="urgent">Urgent</option>
                    <option value="high">High</option>
                    <option value="medium">Medium</option>
                    <option value="low">Low</option>
                  </select>
                </div>
                <div className="flex justify-end gap-2">
                  <button onClick={() => { setShowAddTemplate(false); setEditingTemplateId(null); }} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
                  <button onClick={handleAddTemplate} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">{editingTemplateId ? "Save" : "Add"}</button>
                </div>
              </div>
            )}

            <div className="space-y-1">
              {templates.map(t => (
                <div key={t.id} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                  <div>
                    <div className="text-sm font-medium">{t.name}</div>
                    {t.description_template && <div className="text-xs text-muted-foreground mt-0.5 truncate max-w-md">{t.description_template.slice(0, 80)}...</div>}
                  </div>
                  <div className="flex items-center gap-1">
                    <button onClick={() => startEditTemplate(t)} className="rounded p-1.5 hover:bg-accent"><Pencil className="h-3.5 w-3.5 text-muted-foreground" /></button>
                    <button onClick={() => handleDeleteTemplate(t.id)} className="rounded p-1.5 hover:bg-destructive/20"><Trash2 className="h-3.5 w-3.5 text-muted-foreground" /></button>
                  </div>
                </div>
              ))}
              {templates.length === 0 && <div className="py-8 text-center text-sm text-muted-foreground">No templates yet</div>}
            </div>
          </div>
        )}

        {/* Hooks Tab */}
        {tab === "hooks" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <div>
                <p className="text-sm text-muted-foreground">Configure hooks that trigger on task state changes</p>
                <p className="text-xs text-muted-foreground mt-1">Hooks are executed by the MCP server or CLI, not the desktop app</p>
              </div>
              <button onClick={() => { setShowAddHook(true); setHookEventType("task_completed"); setHookCommand(""); }} className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                <Plus className="h-4 w-4" /> Add Hook
              </button>
            </div>

            {showAddHook && (
              <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium">New Hook</h3>
                  <button onClick={() => setShowAddHook(false)}><X className="h-4 w-4" /></button>
                </div>
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">Event Type</label>
                  <select value={hookEventType} onChange={e => setHookEventType(e.target.value)} className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none">
                    <option value="task_claimed">Task Claimed</option>
                    <option value="task_started">Task Started</option>
                    <option value="task_completed">Task Completed</option>
                    <option value="task_failed">Task Failed</option>
                    <option value="task_approved">Task Approved</option>
                    <option value="task_rejected">Task Rejected</option>
                  </select>
                </div>
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">Command</label>
                  <input value={hookCommand} onChange={e => setHookCommand(e.target.value)} placeholder="e.g. notify-send 'Task completed'" className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono" />
                </div>
                <div className="flex justify-end gap-2">
                  <button onClick={() => setShowAddHook(false)} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
                  <button onClick={handleAddHook} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">Add</button>
                </div>
              </div>
            )}

            <div className="space-y-1">
              {hooks.map(h => (
                <div key={h.id} className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="rounded-full bg-accent px-2 py-0.5 text-[10px] text-muted-foreground whitespace-nowrap">{h.event_type}</span>
                    </div>
                    <div className="text-xs text-muted-foreground mt-1 font-mono truncate">{h.command}</div>
                  </div>
                  <div className="flex items-center gap-1 ml-2">
                    <button onClick={() => handleDeleteHook(h.id)} className="rounded p-1.5 hover:bg-destructive/20"><Trash2 className="h-3.5 w-3.5 text-muted-foreground" /></button>
                  </div>
                </div>
              ))}
              {hooks.length === 0 && <div className="py-8 text-center text-sm text-muted-foreground">No hooks configured yet</div>}
            </div>
          </div>
        )}

        {/* Stale Issues Tab */}
        {tab === "stale" && (
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground mb-4">
              Automatically close issues that have been inactive for a specified number of days. Only affects issues in &quot;unstarted&quot; category statuses (e.g. Backlog, Todo).
            </p>

            <div className="flex items-center gap-3">
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={staleEnabled}
                  onChange={(e) => setStaleEnabled(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-muted rounded-full peer peer-checked:bg-primary transition-colors after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full" />
              </label>
              <span className="text-sm font-medium">Enable auto-close for stale issues</span>
            </div>

            {staleEnabled && (
              <>
                <div>
                  <label className="block text-sm text-muted-foreground mb-1">Days before marking stale</label>
                  <p className="text-xs text-muted-foreground/70 mb-1">Issues with no activity for this many days will be auto-closed</p>
                  <input
                    type="number"
                    min={1}
                    max={365}
                    value={staleDays}
                    onChange={(e) => setStaleDays(parseInt(e.target.value) || 30)}
                    className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
                  />
                </div>

                <div>
                  <label className="block text-sm text-muted-foreground mb-1">Move stale issues to</label>
                  <p className="text-xs text-muted-foreground/70 mb-1">Status to assign when an issue is auto-closed</p>
                  <select
                    value={staleCloseStatusId ?? ""}
                    onChange={(e) => setStaleCloseStatusId(e.target.value ? parseInt(e.target.value) : null)}
                    className="w-64 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
                  >
                    <option value="">Select a status...</option>
                    {statuses
                      .filter(s => s.category === "completed" || s.category === "discarded")
                      .map(s => (
                        <option key={s.id} value={s.id}>{s.name} ({s.category})</option>
                      ))
                    }
                  </select>
                </div>
              </>
            )}

            <button
              onClick={async () => {
                await api.updateStaleConfig(project.id, {
                  stale_days: staleEnabled ? staleDays : null,
                  stale_close_status_id: staleEnabled ? staleCloseStatusId : null,
                });
              }}
              className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              Save Changes
            </button>

            {staleEnabled && (
              <div className="mt-6 border-t border-border pt-4">
                <button
                  onClick={async () => {
                    const closed = await api.checkStaleIssues(project.id);
                    if (closed.length === 0) {
                      alert("No stale issues found.");
                    } else {
                      alert(`Auto-closed ${closed.length} stale issue(s).`);
                    }
                  }}
                  className="rounded-md border border-border px-4 py-2 text-sm font-medium hover:bg-muted transition-colors"
                >
                  Run Stale Check Now
                </button>
                <p className="text-xs text-muted-foreground mt-1">Manually trigger the stale issue check for this project</p>
              </div>
            )}
          </div>
        )}

        {/* Agent Config Tab */}
        {tab === "agents" && agentConfig && (
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground mb-4">Configure agent behavior thresholds and monitoring</p>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Auto-accept threshold</label>
              <p className="text-xs text-muted-foreground/70 mb-1">Tasks above this confidence are auto-approved</p>
              <input type="number" min={0} max={1} step={0.05} value={agentConfig.auto_accept_threshold}
                onChange={e => setAgentConfig({ ...agentConfig, auto_accept_threshold: parseFloat(e.target.value) || 0 })}
                className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Human review threshold</label>
              <p className="text-xs text-muted-foreground/70 mb-1">Tasks below this confidence require human review</p>
              <input type="number" min={0} max={1} step={0.05} value={agentConfig.human_review_threshold}
                onChange={e => setAgentConfig({ ...agentConfig, human_review_threshold: parseFloat(e.target.value) || 0 })}
                className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Max attempts</label>
              <p className="text-xs text-muted-foreground/70 mb-1">Maximum retry attempts before task is blocked</p>
              <input type="number" min={1} max={10} step={1} value={agentConfig.max_attempts}
                onChange={e => setAgentConfig({ ...agentConfig, max_attempts: parseInt(e.target.value) || 1 })}
                className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Heartbeat interval (seconds)</label>
              <input type="number" min={10} max={600} step={10} value={agentConfig.heartbeat_interval_seconds}
                onChange={e => setAgentConfig({ ...agentConfig, heartbeat_interval_seconds: parseInt(e.target.value) || 60 })}
                className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">Missed heartbeats before offline</label>
              <input type="number" min={1} max={10} step={1} value={agentConfig.missed_heartbeats_before_offline}
                onChange={e => setAgentConfig({ ...agentConfig, missed_heartbeats_before_offline: parseInt(e.target.value) || 3 })}
                className="w-32 rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary" />
            </div>
            <button onClick={handleSaveAgentConfig} disabled={agentConfigSaving}
              className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50">
              {agentConfigSaving ? "Saving..." : "Save Changes"}
            </button>
          </div>
        )}
        {tab === "agents" && !agentConfig && (
          <div className="py-8 text-center text-sm text-muted-foreground">Loading agent configuration...</div>
        )}
      </div>
    </div>
  );
}
