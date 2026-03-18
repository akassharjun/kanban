import { useState, useEffect } from "react";
import { Plus, Pencil, Trash2, X, Zap, Play, Pause, ChevronDown, ChevronUp, Clock, CheckCircle2, XCircle, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Project, Status, Label, IssueTemplate, Hook, ProjectAgentConfig, AutomationRule, AutomationLogEntry, AutomationTriggerType, AutomationActionType } from "@/types";
import * as api from "@/tauri/commands";

export interface ProjectSettingsViewProps {
  project: Project;
  onUpdateProject: (id: number, input: { name?: string; description?: string; icon?: string; status?: string; path?: string }) => Promise<unknown>;
  onRefreshStatuses: () => void;
  onRefreshLabels: () => void;
  onDeleteProject?: (id: number) => Promise<unknown>;
}

type Tab = "general" | "statuses" | "labels" | "templates" | "hooks" | "agents" | "automations";

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

  // Automations
  const [automationRules, setAutomationRules] = useState<AutomationRule[]>([]);
  const [automationLog, setAutomationLog] = useState<AutomationLogEntry[]>([]);
  const [showAddAutomation, setShowAddAutomation] = useState(false);
  const [editingAutomationId, setEditingAutomationId] = useState<number | null>(null);
  const [automationName, setAutomationName] = useState("");
  const [automationTrigger, setAutomationTrigger] = useState<AutomationTriggerType>("status_change");
  const [automationTriggerConfig, setAutomationTriggerConfig] = useState("{}");
  const [automationConditions, setAutomationConditions] = useState<Array<{ field: string; operator: string; value: string }>>([]);
  const [automationActions, setAutomationActions] = useState<Array<{ type: AutomationActionType; config: Record<string, unknown> }>>([]);
  const [showAutomationLog, setShowAutomationLog] = useState(false);

  useEffect(() => {
    loadData();
  }, [project.id]);

  const loadData = async () => {
    const [s, l, t, h, ac, ar, al] = await Promise.all([
      api.listStatuses(project.id),
      api.listLabels(project.id),
      api.listTemplates(project.id),
      api.listHooks(project.id),
      api.getProjectAgentConfig(project.id),
      api.listAutomationRules(project.id),
      api.listAutomationLog(project.id, 50),
    ]);
    setStatuses(s);
    setLabels(l);
    setTemplates(t);
    setHooks(h);
    setAgentConfig(ac);
    setAutomationRules(ar);
    setAutomationLog(al);
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

  // Automation handlers
  const handleAddAutomation = async () => {
    if (!automationName.trim()) return;
    const conditionsJson = JSON.stringify(automationConditions);
    const actionsJson = JSON.stringify(automationActions);
    if (editingAutomationId) {
      await api.updateAutomationRule(editingAutomationId, {
        name: automationName,
        trigger_type: automationTrigger,
        trigger_config: automationTriggerConfig,
        conditions: conditionsJson,
        actions: actionsJson,
      });
    } else {
      await api.createAutomationRule({
        project_id: project.id,
        name: automationName,
        trigger_type: automationTrigger,
        trigger_config: automationTriggerConfig,
        conditions: conditionsJson,
        actions: actionsJson,
      });
    }
    resetAutomationForm();
    await loadData();
  };

  const resetAutomationForm = () => {
    setShowAddAutomation(false);
    setEditingAutomationId(null);
    setAutomationName("");
    setAutomationTrigger("status_change");
    setAutomationTriggerConfig("{}");
    setAutomationConditions([]);
    setAutomationActions([]);
  };

  const startEditAutomation = (r: AutomationRule) => {
    setEditingAutomationId(r.id);
    setAutomationName(r.name);
    setAutomationTrigger(r.trigger_type);
    setAutomationTriggerConfig(r.trigger_config);
    try { setAutomationConditions(JSON.parse(r.conditions)); } catch { setAutomationConditions([]); }
    try { setAutomationActions(JSON.parse(r.actions)); } catch { setAutomationActions([]); }
    setShowAddAutomation(true);
  };

  const handleDeleteAutomation = async (ruleId: number) => {
    await api.deleteAutomationRule(ruleId);
    await loadData();
  };

  const handleToggleAutomation = async (ruleId: number, enabled: boolean) => {
    await api.toggleAutomationRule(ruleId, enabled);
    await loadData();
  };

  const addCondition = () => {
    setAutomationConditions(prev => [...prev, { field: "priority", operator: "equals", value: "" }]);
  };

  const removeCondition = (index: number) => {
    setAutomationConditions(prev => prev.filter((_, i) => i !== index));
  };

  const updateCondition = (index: number, field: string, value: string) => {
    setAutomationConditions(prev => prev.map((c, i) => i === index ? { ...c, [field]: value } : c));
  };

  const addAction = () => {
    setAutomationActions(prev => [...prev, { type: "change_status" as AutomationActionType, config: {} }]);
  };

  const removeAction = (index: number) => {
    setAutomationActions(prev => prev.filter((_, i) => i !== index));
  };

  const updateAction = (index: number, field: string, value: unknown) => {
    setAutomationActions(prev => prev.map((a, i) => {
      if (i !== index) return a;
      if (field === "type") return { type: value as AutomationActionType, config: {} };
      return { ...a, config: { ...a.config, [field]: value } };
    }));
  };

  const moveAction = (index: number, direction: "up" | "down") => {
    setAutomationActions(prev => {
      const next = [...prev];
      const swap = direction === "up" ? index - 1 : index + 1;
      if (swap < 0 || swap >= next.length) return next;
      [next[index], next[swap]] = [next[swap], next[index]];
      return next;
    });
  };

  const tabs: { value: Tab; label: string }[] = [
    { value: "general", label: "General" },
    { value: "statuses", label: "Statuses" },
    { value: "labels", label: "Labels" },
    { value: "templates", label: "Templates" },
    { value: "hooks", label: "Hooks" },
    { value: "automations", label: "Automations" },
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

        {/* Automations Tab */}
        {tab === "automations" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <div>
                <p className="text-sm text-muted-foreground">Define automation rules: "When X happens, if Y is true, do Z"</p>
                <p className="text-xs text-muted-foreground/70 mt-1">
                  Rules are evaluated automatically when triggers fire
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowAutomationLog(prev => !prev)}
                  className="flex items-center gap-1 rounded-md border border-border px-3 py-1.5 text-sm hover:bg-accent"
                >
                  <Clock className="h-4 w-4" /> {showAutomationLog ? "Hide Log" : "View Log"}
                </button>
                <button
                  onClick={() => { resetAutomationForm(); setShowAddAutomation(true); }}
                  className="flex items-center gap-1 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
                >
                  <Plus className="h-4 w-4" /> Add Rule
                </button>
              </div>
            </div>

            {/* Create/Edit Form */}
            {showAddAutomation && (
              <div className="mb-4 rounded-lg border border-border bg-card p-4 space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium flex items-center gap-2">
                    <Zap className="h-4 w-4 text-yellow-500" />
                    {editingAutomationId ? "Edit Rule" : "New Automation Rule"}
                  </h3>
                  <button onClick={resetAutomationForm}><X className="h-4 w-4" /></button>
                </div>

                {/* Name */}
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">Rule Name</label>
                  <input
                    value={automationName}
                    onChange={e => setAutomationName(e.target.value)}
                    placeholder="e.g., Auto-assign urgent bugs to Claude"
                    className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
                  />
                </div>

                {/* Trigger */}
                <div>
                  <label className="block text-xs text-muted-foreground mb-1">When (Trigger)</label>
                  <select
                    value={automationTrigger}
                    onChange={e => setAutomationTrigger(e.target.value as AutomationTriggerType)}
                    className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none"
                  >
                    <option value="status_change">Status Changed</option>
                    <option value="issue_created">Issue Created</option>
                    <option value="issue_updated">Issue Updated</option>
                    <option value="priority_changed">Priority Changed</option>
                    <option value="comment_added">Comment Added</option>
                    <option value="label_added">Label Added</option>
                    <option value="agent_assigned">Agent Assigned</option>
                    <option value="task_completed">Task Completed</option>
                    <option value="task_failed">Task Failed</option>
                    <option value="pr_merged">PR Merged</option>
                    <option value="pr_opened">PR Opened</option>
                    <option value="schedule">Schedule (Cron)</option>
                  </select>
                </div>

                {/* Trigger Config */}
                {(automationTrigger === "status_change" || automationTrigger === "schedule") && (
                  <div>
                    <label className="block text-xs text-muted-foreground mb-1">
                      Trigger Config (JSON)
                    </label>
                    <input
                      value={automationTriggerConfig}
                      onChange={e => setAutomationTriggerConfig(e.target.value)}
                      placeholder={automationTrigger === "schedule" ? '{"cron": "0 9 * * 1"}' : '{"from_status_id": 2, "to_status_id": 3}'}
                      className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary font-mono"
                    />
                  </div>
                )}

                {/* Conditions */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-xs text-muted-foreground">If (Conditions) - all must match</label>
                    <button onClick={addCondition} className="text-xs text-primary hover:underline">+ Add condition</button>
                  </div>
                  {automationConditions.length === 0 && (
                    <p className="text-xs text-muted-foreground/60 italic">No conditions - rule will fire on every trigger</p>
                  )}
                  {automationConditions.map((cond, i) => (
                    <div key={i} className="flex gap-2 mb-2 items-center">
                      <select
                        value={cond.field}
                        onChange={e => updateCondition(i, "field", e.target.value)}
                        className="flex-1 rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                      >
                        <option value="priority">Priority</option>
                        <option value="status_id">Status ID</option>
                        <option value="assignee_id">Assignee ID</option>
                        <option value="title">Title</option>
                        <option value="old_value">Old Value</option>
                        <option value="new_value">New Value</option>
                      </select>
                      <select
                        value={cond.operator}
                        onChange={e => updateCondition(i, "operator", e.target.value)}
                        className="rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                      >
                        <option value="equals">equals</option>
                        <option value="not_equals">not equals</option>
                        <option value="contains">contains</option>
                        <option value="not_contains">not contains</option>
                        <option value="greater_than">greater than</option>
                        <option value="less_than">less than</option>
                        <option value="is_empty">is empty</option>
                        <option value="is_not_empty">is not empty</option>
                      </select>
                      <input
                        value={cond.value}
                        onChange={e => updateCondition(i, "value", e.target.value)}
                        placeholder="value"
                        className="flex-1 rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                      />
                      <button onClick={() => removeCondition(i)} className="rounded p-1 hover:bg-destructive/20">
                        <Trash2 className="h-3 w-3 text-muted-foreground" />
                      </button>
                    </div>
                  ))}
                </div>

                {/* Actions */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-xs text-muted-foreground">Then (Actions) - executed in order</label>
                    <button onClick={addAction} className="text-xs text-primary hover:underline">+ Add action</button>
                  </div>
                  {automationActions.length === 0 && (
                    <p className="text-xs text-muted-foreground/60 italic">No actions configured</p>
                  )}
                  {automationActions.map((action, i) => (
                    <div key={i} className="mb-2 rounded-md border border-border bg-background/50 p-3">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-[10px] text-muted-foreground font-mono">#{i + 1}</span>
                        <select
                          value={action.type}
                          onChange={e => updateAction(i, "type", e.target.value)}
                          className="flex-1 rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                        >
                          <option value="change_status">Change Status</option>
                          <option value="set_priority">Set Priority</option>
                          <option value="assign_to">Assign To</option>
                          <option value="add_label">Add Label</option>
                          <option value="create_issue">Create Issue</option>
                          <option value="create_task_contract">Create Task Contract</option>
                          <option value="add_comment">Add Comment</option>
                          <option value="send_notification">Send Notification</option>
                          <option value="trigger_webhook">Trigger Webhook</option>
                        </select>
                        <div className="flex gap-0.5">
                          <button onClick={() => moveAction(i, "up")} disabled={i === 0} className="rounded p-1 hover:bg-accent disabled:opacity-30">
                            <ChevronUp className="h-3 w-3" />
                          </button>
                          <button onClick={() => moveAction(i, "down")} disabled={i === automationActions.length - 1} className="rounded p-1 hover:bg-accent disabled:opacity-30">
                            <ChevronDown className="h-3 w-3" />
                          </button>
                        </div>
                        <button onClick={() => removeAction(i)} className="rounded p-1 hover:bg-destructive/20">
                          <Trash2 className="h-3 w-3 text-muted-foreground" />
                        </button>
                      </div>
                      {/* Action-specific config */}
                      {action.type === "change_status" && (
                        <input
                          value={String(action.config.status_id ?? "")}
                          onChange={e => updateAction(i, "status_id", parseInt(e.target.value) || 0)}
                          placeholder="Status ID"
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                        />
                      )}
                      {action.type === "set_priority" && (
                        <select
                          value={String(action.config.priority ?? "none")}
                          onChange={e => updateAction(i, "priority", e.target.value)}
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                        >
                          <option value="none">None</option>
                          <option value="urgent">Urgent</option>
                          <option value="high">High</option>
                          <option value="medium">Medium</option>
                          <option value="low">Low</option>
                        </select>
                      )}
                      {action.type === "assign_to" && (
                        <input
                          value={String(action.config.member_id ?? "")}
                          onChange={e => updateAction(i, "member_id", parseInt(e.target.value) || 0)}
                          placeholder="Member ID (or agent_type for first_available)"
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                        />
                      )}
                      {action.type === "add_label" && (
                        <input
                          value={String(action.config.label_id ?? "")}
                          onChange={e => updateAction(i, "label_id", parseInt(e.target.value) || 0)}
                          placeholder="Label ID"
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                        />
                      )}
                      {action.type === "create_issue" && (
                        <div className="space-y-2">
                          <input
                            value={String(action.config.title_template ?? "")}
                            onChange={e => updateAction(i, "title_template", e.target.value)}
                            placeholder="Title template (supports {{issue.title}})"
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                          />
                          <select
                            value={String(action.config.priority ?? "none")}
                            onChange={e => updateAction(i, "priority", e.target.value)}
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                          >
                            <option value="none">None</option>
                            <option value="urgent">Urgent</option>
                            <option value="high">High</option>
                            <option value="medium">Medium</option>
                            <option value="low">Low</option>
                          </select>
                        </div>
                      )}
                      {action.type === "add_comment" && (
                        <textarea
                          value={String(action.config.content_template ?? "")}
                          onChange={e => updateAction(i, "content_template", e.target.value)}
                          placeholder="Comment template (supports {{issue.identifier}}, {{actor.name}}, etc.)"
                          rows={2}
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary font-mono"
                        />
                      )}
                      {action.type === "send_notification" && (
                        <input
                          value={String(action.config.message_template ?? "")}
                          onChange={e => updateAction(i, "message_template", e.target.value)}
                          placeholder="Notification message (supports {{issue.identifier}})"
                          className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                        />
                      )}
                      {action.type === "create_task_contract" && (
                        <div className="space-y-2">
                          <select
                            value={String(action.config.type ?? "implementation")}
                            onChange={e => updateAction(i, "type", e.target.value)}
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                          >
                            <option value="implementation">Implementation</option>
                            <option value="review">Review</option>
                            <option value="testing">Testing</option>
                            <option value="research">Research</option>
                          </select>
                          <select
                            value={String(action.config.complexity ?? "medium")}
                            onChange={e => updateAction(i, "complexity", e.target.value)}
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                          >
                            <option value="low">Low</option>
                            <option value="medium">Medium</option>
                            <option value="high">High</option>
                          </select>
                          <input
                            value={String(action.config.skills ?? "")}
                            onChange={e => updateAction(i, "skills", e.target.value.split(",").map(s => s.trim()).filter(Boolean))}
                            placeholder="Skills (comma-separated): rust, typescript, react"
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary"
                          />
                        </div>
                      )}
                      {action.type === "trigger_webhook" && (
                        <div className="space-y-2">
                          <input
                            value={String(action.config.url ?? "")}
                            onChange={e => updateAction(i, "url", e.target.value)}
                            placeholder="URL (e.g., https://hooks.example.com/notify)"
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none focus:border-primary font-mono"
                          />
                          <select
                            value={String(action.config.method ?? "POST")}
                            onChange={e => updateAction(i, "method", e.target.value)}
                            className="w-full rounded-md border border-border bg-background px-2 py-1.5 text-xs outline-none"
                          >
                            <option value="POST">POST</option>
                            <option value="GET">GET</option>
                            <option value="PUT">PUT</option>
                          </select>
                        </div>
                      )}
                    </div>
                  ))}
                </div>

                <div className="flex items-center gap-2 pt-2 text-[10px] text-muted-foreground/60">
                  <AlertTriangle className="h-3 w-3" />
                  Template variables: {"{{issue.title}}, {{issue.identifier}}, {{issue.priority}}, {{actor.name}}, {{old_value}}, {{new_value}}, {{agent.name}}, {{task.confidence}}"}
                </div>

                <div className="flex justify-end gap-2">
                  <button onClick={resetAutomationForm} className="rounded-md px-3 py-1.5 text-sm hover:bg-accent">Cancel</button>
                  <button onClick={handleAddAutomation} className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90">
                    {editingAutomationId ? "Save" : "Add Rule"}
                  </button>
                </div>
              </div>
            )}

            {/* Rules List */}
            <div className="space-y-1">
              {automationRules.map(r => {
                let parsedActions: Array<{ type: string }> = [];
                try { parsedActions = JSON.parse(r.actions); } catch { /* empty */ }
                let parsedConditions: Array<{ field: string; operator: string; value: string }> = [];
                try { parsedConditions = JSON.parse(r.conditions); } catch { /* empty */ }

                return (
                  <div key={r.id} className={cn(
                    "rounded-lg border bg-card px-4 py-3",
                    r.enabled ? "border-border" : "border-border/50 opacity-60"
                  )}>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3 min-w-0">
                        <button
                          onClick={() => handleToggleAutomation(r.id, !r.enabled)}
                          className={cn("rounded-full p-1", r.enabled ? "text-green-500 hover:bg-green-500/10" : "text-muted-foreground hover:bg-accent")}
                          title={r.enabled ? "Disable" : "Enable"}
                        >
                          {r.enabled ? <Play className="h-3.5 w-3.5" /> : <Pause className="h-3.5 w-3.5" />}
                        </button>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="text-sm font-medium truncate">{r.name}</span>
                            <span className="rounded-full bg-accent px-2 py-0.5 text-[10px] text-muted-foreground whitespace-nowrap">
                              {r.trigger_type.replace(/_/g, " ")}
                            </span>
                          </div>
                          <div className="flex items-center gap-3 mt-0.5 text-[10px] text-muted-foreground">
                            {parsedConditions.length > 0 && (
                              <span>{parsedConditions.length} condition{parsedConditions.length !== 1 ? "s" : ""}</span>
                            )}
                            <span>{parsedActions.length} action{parsedActions.length !== 1 ? "s" : ""}: {parsedActions.map(a => a.type?.replace(/_/g, " ")).join(", ")}</span>
                            {r.execution_count > 0 && (
                              <span>Ran {r.execution_count}x</span>
                            )}
                            {r.last_executed_at && (
                              <span>Last: {new Date(r.last_executed_at).toLocaleDateString()}</span>
                            )}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-1 ml-2">
                        <button onClick={() => startEditAutomation(r)} className="rounded p-1.5 hover:bg-accent">
                          <Pencil className="h-3.5 w-3.5 text-muted-foreground" />
                        </button>
                        <button onClick={() => handleDeleteAutomation(r.id)} className="rounded p-1.5 hover:bg-destructive/20">
                          <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
              {automationRules.length === 0 && !showAddAutomation && (
                <div className="py-8 text-center text-sm text-muted-foreground">
                  No automation rules yet. Create one to automate your workflow.
                </div>
              )}
            </div>

            {/* Automation Log */}
            {showAutomationLog && (
              <div className="mt-6">
                <h3 className="text-sm font-medium mb-3 flex items-center gap-2">
                  <Clock className="h-4 w-4" /> Recent Executions
                </h3>
                <div className="space-y-1">
                  {automationLog.map(entry => {
                    const rule = automationRules.find(r => r.id === entry.rule_id);
                    return (
                      <div key={entry.id} className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-2.5 text-xs">
                        {entry.success ? (
                          <CheckCircle2 className="h-3.5 w-3.5 text-green-500 shrink-0" />
                        ) : (
                          <XCircle className="h-3.5 w-3.5 text-red-500 shrink-0" />
                        )}
                        <div className="min-w-0 flex-1">
                          <span className="font-medium">{rule?.name ?? `Rule #${entry.rule_id}`}</span>
                          <span className="text-muted-foreground ml-2">{entry.trigger_type.replace(/_/g, " ")}</span>
                          {entry.issue_id && <span className="text-muted-foreground ml-1">(issue #{entry.issue_id})</span>}
                          {entry.error_message && <span className="text-red-400 ml-2">{entry.error_message}</span>}
                        </div>
                        <span className="text-muted-foreground whitespace-nowrap shrink-0">
                          {new Date(entry.executed_at).toLocaleString()}
                        </span>
                      </div>
                    );
                  })}
                  {automationLog.length === 0 && (
                    <div className="py-4 text-center text-xs text-muted-foreground">No executions yet</div>
                  )}
                </div>
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
