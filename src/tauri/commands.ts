import { invoke } from "@tauri-apps/api/core";
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
} from "@/types";

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
export const listCustomFields = (projectId: number) =>
  invoke<CustomField[]>("list_custom_fields", { projectId });
export const createCustomField = (input: {
  project_id: number;
  name: string;
  field_type?: string;
  options?: string;
  position?: number;
}) => invoke<CustomField>("create_custom_field", { input });
export const updateCustomField = (id: number, input: {
  name?: string;
  field_type?: string;
  options?: string;
  position?: number;
}) => invoke<CustomField>("update_custom_field", { id, input });
export const deleteCustomField = (id: number) =>
  invoke<void>("delete_custom_field", { id });
export const getIssueCustomValues = (issueId: number) =>
  invoke<CustomFieldValue[]>("get_issue_custom_values", { issueId });
export const setIssueCustomValue = (issueId: number, fieldId: number, value: string | null) =>
  invoke<void>("set_issue_custom_value", { issueId, fieldId, value });

// Agent Orchestration
export const listAgents = () => invoke<Agent[]>("list_agents");
export const getAgentStats = (agentId: string) => invoke<AgentMetrics>("agent_metrics_cmd", { agentId });
export const projectMetrics = (projectId: number) => invoke<ProjectMetrics>("project_metrics", { projectId });
export const taskReplay = (identifier: string) => invoke<ExecutionLog[]>("task_replay", { identifier });
export const getTaskContract = (identifier: string) => invoke<FullTaskContract>("get_task_contract", { identifier });
export const taskGraph = (identifier: string) => invoke<TaskGraph>("task_graph", { identifier });
export const recentActivity = (projectId: number, limit?: number) => invoke<ExecutionLog[]>("recent_activity", { projectId, limit });
export const deregisterAgent = (agentId: string) => invoke<void>("deregister_agent", { agentId });
