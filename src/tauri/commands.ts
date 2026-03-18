import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { isTauri, mockInvoke } from "./mock-backend";
import type {
  Project,
  Status,
  Issue,
  IssueWithLabels,
  Label,
  Member,
  IssueRelation,
  IssueTemplate,
  ActivityLogEntry,
  UndoLogEntry,
  Notification,
  Comment,
  CustomField,
  CustomFieldValue,
  Agent,
  AgentMetrics,
  ProjectMetrics,
  ExecutionLog,
  FullTaskContract,
  TaskGraph,
  ProjectAgentConfig,
  Hook,
  TaskCost,
  TaskCostSummary,
  ProjectCostSummary,
  BudgetStatus,
  CostBudget,
  SlaPolicy,
  SlaStatus,
  SlaEvent,
  SlaDashboard,
} from "@/types";

// Use real Tauri invoke when in Tauri, mock otherwise
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (isTauri) return tauriInvoke<T>(cmd, args);
  return mockInvoke(cmd, args) as Promise<T>;
}

// Health
export const healthCheck = () => invoke<string>("health_check");

// Projects
export const listProjects = () => invoke<Project[]>("list_projects");
export const getProject = (id: number) => invoke<Project>("get_project", { id });
export const createProject = (input: {
  name: string;
  description?: string;
  icon?: string;
  prefix: string;
}) => invoke<Project>("create_project", { input });
export const updateProject = (id: number, input: {
  name?: string;
  description?: string;
  icon?: string;
  status?: string;
}) => invoke<Project>("update_project", { id, input });
export const deleteProject = (id: number) => invoke<void>("delete_project", { id });

// Statuses
export const listStatuses = (projectId: number) => invoke<Status[]>("list_statuses", { projectId });
export const createStatus = (input: {
  project_id: number;
  name: string;
  category: string;
  color?: string;
  icon?: string;
}) => invoke<Status>("create_status", { input });
export const updateStatus = (id: number, input: {
  name?: string;
  category?: string;
  color?: string;
  icon?: string;
  position?: number;
}) => invoke<Status>("update_status", { id, input });
export const deleteStatus = (id: number) => invoke<void>("delete_status", { id });
export const reorderStatuses = (statusIds: number[]) => invoke<void>("reorder_statuses", { statusIds });

// Issues
export const createIssue = (input: {
  project_id: number;
  title: string;
  description?: string;
  status_id: number;
  priority?: string;
  assignee_id?: number;
  parent_id?: number;
  estimate?: number;
  due_date?: string;
  label_ids?: number[];
}) => invoke<Issue>("create_issue", { input });
export const getIssue = (id: number) => invoke<IssueWithLabels>("get_issue", { id });
export const getIssueByIdentifier = (identifier: string) => invoke<IssueWithLabels>("get_issue_by_identifier", { identifier });
export const listIssues = (filter: {
  project_id: number;
  status_id?: number;
  priority?: string;
  assignee_id?: number;
  label_id?: number;
  parent_id?: number;
  search?: string;
}) => invoke<Issue[]>("list_issues", { filter });
export const updateIssue = (id: number, input: {
  title?: string;
  description?: string;
  status_id?: number;
  priority?: string;
  assignee_id?: number;
  parent_id?: number;
  position?: number;
  estimate?: number;
  due_date?: string;
}) => invoke<Issue>("update_issue", { id, input });
export const deleteIssue = (id: number) => invoke<void>("delete_issue", { id });
export const duplicateIssue = (id: number) => invoke<Issue>("duplicate_issue", { id });
export const bulkUpdateIssues = (input: {
  issue_ids: number[];
  status_id?: number;
  priority?: string;
  assignee_id?: number;
}) => invoke<Issue[]>("bulk_update_issues", { input });
export const searchIssues = (projectId: number, query: string) => invoke<Issue[]>("search_issues", { projectId, query });
export const getSubIssues = (parentId: number) => invoke<Issue[]>("get_sub_issues", { parentId });
export const setIssueLabels = (issueId: number, labelIds: number[]) => invoke<void>("set_issue_labels", { issueId, labelIds });
export const listIssueLabelMappings = (projectId: number) => invoke<{ issue_id: number; label_id: number }[]>("list_issue_label_mappings", { projectId });
export const getActivityLog = (issueId: number) => invoke<ActivityLogEntry[]>("get_activity_log", { issueId });

// Members
export const listMembers = () => invoke<Member[]>("list_members");
export const createMember = (input: {
  name: string;
  display_name?: string;
  email?: string;
  avatar_color?: string;
}) => invoke<Member>("create_member", { input });
export const updateMember = (id: number, input: {
  name?: string;
  display_name?: string;
  email?: string;
  avatar_color?: string;
}) => invoke<Member>("update_member", { id, input });
export const deleteMember = (id: number) => invoke<void>("delete_member", { id });

// Labels
export const listLabels = (projectId: number) => invoke<Label[]>("list_labels", { projectId });
export const createLabel = (input: {
  project_id: number;
  name: string;
  color: string;
}) => invoke<Label>("create_label", { input });
export const updateLabel = (id: number, input: {
  name?: string;
  color?: string;
}) => invoke<Label>("update_label", { id, input });
export const deleteLabel = (id: number) => invoke<void>("delete_label", { id });

// Relations
export const listRelations = (issueId: number) => invoke<IssueRelation[]>("list_relations", { issueId });
export const createRelation = (input: {
  source_issue_id: number;
  target_issue_id: number;
  relation_type: string;
}) => invoke<IssueRelation>("create_relation", { input });
export const deleteRelation = (id: number) => invoke<void>("delete_relation", { id });

// Templates
export const listTemplates = (projectId: number) => invoke<IssueTemplate[]>("list_templates", { projectId });
export const createTemplate = (input: {
  project_id: number;
  name: string;
  description_template?: string;
  default_status_id?: number;
  default_priority?: string;
  default_label_ids?: number[];
}) => invoke<IssueTemplate>("create_template", { input });
export const updateTemplate = (id: number, input: {
  name?: string;
  description_template?: string;
  default_status_id?: number;
  default_priority?: string;
  default_label_ids?: number[];
}) => invoke<IssueTemplate>("update_template", { id, input });
export const deleteTemplate = (id: number) => invoke<void>("delete_template", { id });

// Undo/Redo
export const undo = () => invoke<UndoLogEntry | null>("undo");
export const redo = () => invoke<UndoLogEntry | null>("redo");

// Notifications
export const listNotifications = () => invoke<Notification[]>("list_notifications");
export const unreadNotificationCount = () => invoke<number>("unread_notification_count");
export const markNotificationRead = (id: number) => invoke<void>("mark_notification_read", { id });
export const markAllNotificationsRead = () => invoke<void>("mark_all_notifications_read");
export const clearNotifications = () => invoke<void>("clear_notifications");

// Comments
export const listComments = (issueId: number) => invoke<Comment[]>("list_comments", { issueId });
export const createComment = (input: {
  issue_id: number;
  member_id?: number;
  content: string;
}) => invoke<Comment>("create_comment", { input });
export const updateComment = (id: number, input: {
  content: string;
}) => invoke<Comment>("update_comment", { id, input });
export const deleteComment = (id: number) => invoke<void>("delete_comment", { id });
export const commentCount = (issueId: number) => invoke<number>("comment_count", { issueId });

// Custom Fields
export const listCustomFields = (projectId: number) => invoke<CustomField[]>("list_custom_fields", { projectId });
export const getIssueCustomValues = (issueId: number) => invoke<CustomFieldValue[]>("get_issue_custom_values", { issueId });
export const setIssueCustomValue = (issueId: number, fieldId: number, value: string | null) => invoke<void>("set_issue_custom_value", { issueId, fieldId, value });

// Agents
export const listAgents = () => invoke<Agent[]>("list_agents");
export const getAgentStats = (agentId: string) => invoke<AgentMetrics>("agent_metrics_cmd", { agentId });
export const deregisterAgent = (agentId: string) => invoke<void>("deregister_agent", { agentId });

// Metrics & Execution
export const projectMetrics = (projectId: number) => invoke<ProjectMetrics>("project_metrics", { projectId });
export const recentActivity = (projectId: number, limit?: number) => invoke<ExecutionLog[]>("recent_activity", { projectId, limit });
export const taskReplay = (identifier: string) => invoke<ExecutionLog[]>("task_replay", { identifier });
export const getTaskContract = (identifier: string) => invoke<FullTaskContract>("get_task_contract", { identifier });
export const taskGraph = (identifier: string) => invoke<TaskGraph>("task_graph", { identifier });

// Agent Config
export const getProjectAgentConfig = (projectId: number) => invoke<ProjectAgentConfig>("get_project_agent_config", { projectId });
export const updateProjectAgentConfig = (projectId: number, input: Partial<Omit<ProjectAgentConfig, 'project_id'>>) => invoke<ProjectAgentConfig>("update_project_agent_config", { projectId, input });

// Hooks
export const listHooks = (projectId: number) => invoke<Hook[]>("list_hooks", { projectId });
export const createHook = (input: { project_id: number; event_type: string; command: string }) => invoke<Hook>("create_hook", { input });
export const deleteHook = (id: number) => invoke<void>("delete_hook", { id });

// Task Contracts
export const createTaskContract = (input: {
  project_id: number; title: string; objective: string; status_id: number;
  type?: string; description?: string; priority?: string; skills?: string[];
  complexity?: string; constraints?: string[]; success_criteria?: unknown[];
  context_files?: string[]; timeout_minutes?: number; depends_on?: string[];
}) => invoke<FullTaskContract>("create_task_contract", { input });

// Task Lifecycle
export const nextTask = (agentId: string) => invoke<FullTaskContract | null>("next_task", { agentId });
export const startTask = (identifier: string, agentId: string) => invoke<void>("start_task", { identifier, agentId });
export const completeTask = (identifier: string, agentId: string, confidence: number, summary: string, artifacts?: Record<string, unknown>) => invoke<void>("complete_task", { identifier, agentId, confidence, summary, artifacts });
export const failTask = (identifier: string, agentId: string, reason: string) => invoke<void>("fail_task", { identifier, agentId, reason });
export const approveTask = (identifier: string) => invoke<void>("approve_task", { identifier });
export const rejectTask = (identifier: string, reason?: string) => invoke<void>("reject_task", { identifier, reason });
export const unclaimTask = (identifier: string, agentId: string) => invoke<void>("unclaim_task", { identifier, agentId });
export const logTaskActivity = (identifier: string, agentId: string, entryType: string, message: string, metadata?: Record<string, unknown>) => invoke<void>("log_task_activity", { identifier, agentId, entryType, message, metadata });

// Cost Tracking
export const recordCost = (input: {
  task_identifier: string;
  agent_id: string;
  cost_type: string;
  amount: number;
  unit: string;
  description?: string;
}) => invoke<TaskCost>("record_cost", { input });
export const getTaskCostSummary = (taskIdentifier: string) => invoke<TaskCostSummary>("get_task_cost_summary", { taskIdentifier });
export const getProjectCostSummary = (projectId: number) => invoke<ProjectCostSummary>("get_project_cost_summary", { projectId });
export const setBudget = (input: {
  project_id: number;
  budget_type: string;
  amount: number;
  unit?: string;
  alert_threshold?: number;
}) => invoke<CostBudget>("set_budget", { input });
export const listBudgets = (projectId: number) => invoke<BudgetStatus[]>("list_budgets", { projectId });
export const checkBudget = (projectId: number) => invoke<BudgetStatus[]>("check_budget", { projectId });
export const deleteBudget = (id: number) => invoke<void>("delete_budget", { id });

// SLA
export const listSlaPolicies = (projectId: number) => invoke<SlaPolicy[]>("list_sla_policies", { projectId });
export const createSlaPolicy = (input: {
  project_id: number;
  name: string;
  target_type: string;
  priority_filter?: string;
  warning_minutes: number;
  breach_minutes: number;
  escalation_action?: string;
}) => invoke<SlaPolicy>("create_sla_policy", { input });
export const updateSlaPolicy = (id: number, input: {
  name?: string;
  target_type?: string;
  priority_filter?: string;
  warning_minutes?: number;
  breach_minutes?: number;
  escalation_action?: string;
  enabled?: boolean;
}) => invoke<SlaPolicy>("update_sla_policy", { id, input });
export const deleteSlaPolicy = (id: number) => invoke<void>("delete_sla_policy", { id });
export const checkSlaCompliance = (projectId: number) => invoke<SlaStatus[]>("check_sla_compliance", { projectId });
export const enforceSla = (projectId: number) => invoke<SlaEvent[]>("enforce_sla", { projectId });
export const getSlaEvents = (issueId: number) => invoke<SlaEvent[]>("get_sla_events", { issueId });
export const getSlaDashboard = (projectId: number) => invoke<SlaDashboard>("get_sla_dashboard", { projectId });
