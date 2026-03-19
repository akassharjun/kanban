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
  AuditLogEntry,
  IssueHistoryEntry,
  MentionEntry,
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
  Epic,
  Milestone,
  MilestoneWithProgress,
  SavedView,
  GitLink,
  AutomationRule,
  AutomationLogEntry,
  GithubConfig,
  GithubEvent,
  CIStatus,
  PRStatus,
  ConnectionTestResult,
  BranchNamePreview,
  IssueFileLink,
  FileHeatEntry,
  DirectoryHeatEntry,
  FileTreeNode,
  ProjectFileContent,
  TaskContext,
  RecurringIssue,
  RecurringPreview,
  DependencyGraph,
  AgentPerformance,
  ProjectAgentSummary,
  AgentRanking,
  AgentRegistryEntry,
  AgentCapability,
  AgentMatch,
  HandoffNote,
  TaskLearning,
  SimilarTaskResult,
  WsjfScore,
  AutoScoreResult,
  Pipeline,
  PipelineRun,
  AgentPermission,
  PermissionPreset,
  PermissionCheckResult,
  TaskCost,
  TaskCostSummary,
  ProjectCostSummary,
  BudgetStatus,
  CostBudget,
  SlaPolicy,
  SlaStatus,
  SlaEvent,
  SlaDashboard,
  GitStatus,
  GitCommit,
  GitBranch,
  GitWorktree,
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
  epic_id?: number;
  milestone_id?: number;
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
  epic_id?: number;
  milestone_id?: number;
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
export const getActivityLog = (issueId: number, limit?: number, offset?: number) => invoke<ActivityLogEntry[]>("get_activity_log", { issueId, limit, offset });
export const getAuditLog = (filter: {
  project_id: number;
  actor_id?: number;
  issue_id?: number;
  field_changed?: string;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}) => invoke<AuditLogEntry[]>("get_audit_log", { filter });
export const getIssueHistory = (issueId: number) => invoke<IssueHistoryEntry[]>("get_issue_history", { issueId });

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

// Mentions
export const listMentions = (memberId: number) => invoke<MentionEntry[]>("list_mentions", { memberId });
export const searchMembersForMention = (query: string) => invoke<Member[]>("search_members_for_mention", { query });

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

// Automations
export const listAutomationRules = (projectId: number) => invoke<AutomationRule[]>("list_automation_rules", { projectId });
export const createAutomationRule = (input: {
  project_id: number;
  name: string;
  trigger_type: string;
  trigger_config?: string;
  conditions?: string;
  actions?: string;
}) => invoke<AutomationRule>("create_automation_rule", { input });
export const updateAutomationRule = (id: number, input: {
  name?: string;
  trigger_type?: string;
  trigger_config?: string;
  conditions?: string;
  actions?: string;
}) => invoke<AutomationRule>("update_automation_rule", { id, input });
export const deleteAutomationRule = (id: number) => invoke<void>("delete_automation_rule", { id });
export const toggleAutomationRule = (id: number, enabled: boolean) => invoke<AutomationRule>("toggle_automation_rule", { id, enabled });
export const listAutomationLog = (projectId: number, limit?: number) => invoke<AutomationLogEntry[]>("list_automation_log", { projectId, limit });

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

// Epics
export const listEpics = (projectId: number) => invoke<Epic[]>("list_epics", { projectId });
export const getEpic = (id: number) => invoke<Epic>("get_epic", { id });
export const createEpic = (input: {
  project_id: number;
  title: string;
  description?: string;
  color?: string;
}) => invoke<Epic>("create_epic", { input });
export const updateEpic = (id: number, input: {
  title?: string;
  description?: string;
  color?: string;
  status?: string;
}) => invoke<Epic>("update_epic", { id, input });
export const deleteEpic = (id: number) => invoke<void>("delete_epic", { id });

// Milestones
export const listMilestones = (projectId: number) => invoke<MilestoneWithProgress[]>("list_milestones", { projectId });
export const getMilestone = (id: number) => invoke<MilestoneWithProgress>("get_milestone", { id });
export const createMilestone = (input: {
  project_id: number;
  title: string;
  description?: string;
  due_date?: string;
}) => invoke<Milestone>("create_milestone", { input });
export const updateMilestone = (id: number, input: {
  title?: string;
  description?: string;
  due_date?: string;
  status?: string;
}) => invoke<Milestone>("update_milestone", { id, input });
export const deleteMilestone = (id: number) => invoke<void>("delete_milestone", { id });

// Saved Views
export const listSavedViews = (projectId: number) => invoke<SavedView[]>("list_saved_views", { projectId });
export const createSavedView = (input: {
  project_id: number;
  name: string;
  filters?: string;
  sort_by?: string;
  sort_direction?: string;
  view_mode?: string;
}) => invoke<SavedView>("create_saved_view", { input });
export const updateSavedView = (id: number, input: {
  name?: string;
  filters?: string;
  sort_by?: string;
  sort_direction?: string;
  view_mode?: string;
}) => invoke<SavedView>("update_saved_view", { id, input });
export const deleteSavedView = (id: number) => invoke<void>("delete_saved_view", { id });

// Starred Issues
export const starIssue = (issueId: number, memberId: number) => invoke<void>("star_issue", { issueId, memberId });
export const unstarIssue = (issueId: number, memberId: number) => invoke<void>("unstar_issue", { issueId, memberId });
export const listStarred = (memberId: number) => invoke<Issue[]>("list_starred", { memberId });
export const isStarred = (issueId: number, memberId: number) => invoke<boolean>("is_starred", { issueId, memberId });

// Recently Viewed
export const recordView = (issueId: number, memberId: number) => invoke<void>("record_view", { issueId, memberId });
export const listRecentlyViewed = (memberId: number, limit?: number) => invoke<Issue[]>("list_recently_viewed", { memberId, limit });

// Advanced Search
export const advancedSearch = (projectId: number, queryString: string, memberId?: number) => invoke<Issue[]>("advanced_search", { projectId, queryString, memberId });
// Git Links
export const createGitLink = (input: {
  issue_id: number;
  link_type: string;
  ref_name: string;
  url?: string;
}) => invoke<GitLink>("create_git_link", { input });
export const listGitLinks = (issueId: number) => invoke<GitLink[]>("list_git_links", { issueId });
export const updateGitLink = (id: number, input: { url?: string; pr_state?: string; pr_merged?: boolean; ci_status?: string; review_status?: string }) => invoke<GitLink>("update_git_link", { id, input });
export const deleteGitLink = (id: number) => invoke<void>("delete_git_link", { id });
export const gitLinkCount = (issueId: number) => invoke<number>("git_link_count", { issueId });

// Stale Issues
export const updateStaleConfig = (projectId: number, input: {
  stale_days: number | null;
  stale_close_status_id: number | null;
}) => invoke<void>("update_stale_config", { projectId, input });
export const checkStaleIssues = (projectId: number) => invoke<Issue[]>("check_stale_issues", { projectId });

// GitHub Integration
export const getGithubConfig = (projectId: number) => invoke<GithubConfig | null>("get_github_config", { projectId });
export const setGithubConfig = (projectId: number, input: {
  repo_owner: string;
  repo_name: string;
  access_token?: string | null;
  branch_pattern?: string;
  auto_link_prs?: boolean;
  auto_transition_on_merge?: boolean;
  merge_target_status_id?: number | null;
}) => invoke<GithubConfig>("set_github_config", { projectId, input });
export const testGithubConnection = (projectId: number) => invoke<ConnectionTestResult>("test_github_connection", { projectId });
export const generateBranchName = (projectId: number, issueIdentifier: string) => invoke<BranchNamePreview>("generate_branch_name", { projectId, issueIdentifier });
export const createBranchForIssue = (projectId: number, issueIdentifier: string) => invoke<GitLink>("create_branch_for_issue", { projectId, issueIdentifier });
export const syncGithubPrs = (projectId: number) => invoke<GitLink[]>("sync_github_prs", { projectId });
export const getPrStatus = (projectId: number, gitLinkId: number) => invoke<PRStatus>("get_pr_status", { projectId, gitLinkId });
export const getCiStatus = (projectId: number, issueIdentifier: string) => invoke<CIStatus>("get_ci_status", { projectId, issueIdentifier });
export const listGithubEvents = (projectId: number) => invoke<GithubEvent[]>("list_github_events", { projectId });

// AI Agent Intelligence
import type { TriageSuggestion, DecomposedTask, ParsedIssue } from "@/types";

export const triageIssue = (projectId: number, title: string, description?: string) =>
  invoke<TriageSuggestion>("triage_issue", { projectId, title, description });
export const autoTriageAndApply = (issueId: number) =>
  invoke<TriageSuggestion>("auto_triage_and_apply", { issueId });
export const decomposeIssue = (issueId: number) =>
  invoke<DecomposedTask[]>("decompose_issue", { issueId });
export const applyDecomposition = (issueId: number) =>
  invoke<Issue[]>("apply_decomposition", { issueId });
export const parseNaturalLanguage = (projectId: number, text: string) =>
  invoke<ParsedIssue>("parse_natural_language", { projectId, text });
export const createFromNaturalLanguage = (projectId: number, text: string, statusId: number) =>
  invoke<Issue>("create_from_natural_language", { projectId, text, statusId });

// Context Assembly
export const getTaskContext = (identifier: string) => invoke<TaskContext>("get_task_context", { identifier });
export const getSimilarIssues = (projectId: number, issueId: number, limit: number) => invoke<Issue[]>("get_similar_issues", { projectId, issueId, limit });

// Code Analysis
export const linkFileToIssue = (input: { issue_id: number; file_path: string; link_type?: string }) => invoke<IssueFileLink>("link_file_to_issue", { input });
export const unlinkFileFromIssue = (issueId: number, filePath: string) => invoke<void>("unlink_file_from_issue", { issueId, filePath });
export const listFileLinks = (issueId: number) => invoke<IssueFileLink[]>("list_file_links", { issueId });
export const getFileHeatMap = (projectId: number, limit: number) => invoke<FileHeatEntry[]>("get_file_heat_map", { projectId, limit });
export const getDirectoryHeatMap = (projectId: number, depth: number) => invoke<DirectoryHeatEntry[]>("get_directory_heat_map", { projectId, depth });
export const getIssuesForFile = (filePath: string, projectId: number) => invoke<Issue[]>("get_issues_for_file", { filePath, projectId });
export const listProjectFiles = (projectId: number) => invoke<FileTreeNode[]>("list_project_files", { projectId });
export const readProjectFile = (projectId: number, filePath: string) => invoke<ProjectFileContent>("read_project_file", { projectId, filePath });

// Diff Issues
export const createIssueFromDiff = (input: {
  project_id: number;
  title: string;
  description?: string;
  file_path: string;
  line_range?: string;
  severity: string;
  status_id?: number;
  assignee_id?: number;
}) => invoke<Issue>("create_issue_from_diff", { input });
// Recurring Issues
export const listRecurring = (projectId: number) => invoke<RecurringIssue[]>("list_recurring", { projectId });
export const createRecurring = (input: {
  project_id: number;
  title_template: string;
  description_template?: string;
  status_id: number;
  priority?: string;
  assignee_id?: number;
  label_ids?: number[];
  recurrence_type: string;
  recurrence_config?: string;
  next_run_at: string;
}) => invoke<RecurringIssue>("create_recurring", { input });
export const updateRecurring = (id: number, input: {
  title_template?: string;
  description_template?: string;
  status_id?: number;
  priority?: string;
  assignee_id?: number;
  label_ids?: number[];
  recurrence_type?: string;
  recurrence_config?: string;
  next_run_at?: string;
  enabled?: boolean;
}) => invoke<RecurringIssue>("update_recurring", { id, input });
export const deleteRecurring = (id: number) => invoke<void>("delete_recurring", { id });
export const toggleRecurring = (id: number, enabled: boolean) => invoke<RecurringIssue>("toggle_recurring", { id, enabled });
export const checkRecurring = (projectId: number) => invoke<Issue[]>("check_recurring", { projectId });
export const previewRecurring = (id: number) => invoke<RecurringPreview>("preview_recurring", { id });

// Dependency Graph
export const dependencyGraph = (projectId: number) => invoke<DependencyGraph>("dependency_graph", { projectId });
// Agent Analytics
export const recordTaskMetric = (input: {
  agent_id: string; task_identifier: string; outcome: string;
  started_at?: string; completed_at?: string; duration_seconds?: number;
  confidence?: number; attempt_number?: number; complexity?: string;
  task_type?: string; files_changed?: number; lines_added?: number; lines_removed?: number;
}) => invoke<void>("record_task_metric", { input });
export const getAgentPerformance = (agentId: string) => invoke<AgentPerformance>("get_agent_performance", { agentId });
export const getProjectAgentSummary = (projectId: number) => invoke<ProjectAgentSummary>("get_project_agent_summary", { projectId });
export const getAgentLeaderboard = (projectId: number) => invoke<AgentRanking[]>("get_agent_leaderboard", { projectId });

// Marketplace
export const marketplaceRegister = (input: {
  agent_id: string; name: string; description?: string; provider?: string;
  version?: string; endpoint?: string; capabilities: string[];
  max_concurrent?: number; max_complexity?: string; hourly_rate?: number;
}) => invoke<AgentRegistryEntry>("marketplace_register", { input });
export const marketplaceUpdate = (agentId: string, input: {
  name?: string; description?: string; version?: string; endpoint?: string;
  capabilities?: string[]; max_concurrent?: number; max_complexity?: string; hourly_rate?: number;
}) => invoke<AgentRegistryEntry>("marketplace_update", { agentId, input });
export const marketplaceDeregister = (agentId: string) => invoke<void>("marketplace_deregister", { agentId });
export const marketplaceList = () => invoke<AgentRegistryEntry[]>("marketplace_list");
export const marketplaceSearch = (skills: string[], maxComplexity?: string) => invoke<AgentRegistryEntry[]>("marketplace_search", { skills, maxComplexity });
export const marketplaceGet = (agentId: string) => invoke<AgentRegistryEntry>("marketplace_get", { agentId });
export const updateAgentProficiency = (agentId: string, capability: string, success: boolean) => invoke<void>("update_agent_proficiency", { agentId, capability, success });
export const getAgentCapabilities = (agentId: string) => invoke<AgentCapability[]>("get_agent_capabilities", { agentId });
export const findBestAgent = (taskSkills: string[], complexity: string) => invoke<AgentMatch[]>("find_best_agent", { taskSkills, complexity });

// Handoff Notes
export const createHandoffNote = (input: {
  task_identifier: string;
  from_agent_id: string;
  to_agent_id?: string;
  note_type: string;
  summary: string;
  details?: string;
  files_changed?: string[];
  risks?: string[];
  test_results?: { passed?: number; failed?: number; skipped?: number };
  metadata?: Record<string, unknown>;
}) => invoke<HandoffNote>("create_handoff_note", { input });
export const listHandoffNotes = (taskIdentifier: string) => invoke<HandoffNote[]>("list_handoff_notes", { taskIdentifier });
export const getHandoffForAgent = (agentId: string, taskIdentifier: string) => invoke<HandoffNote[]>("get_handoff_for_agent", { agentId, taskIdentifier });

// Learnings
export const recordLearning = (input: {
  task_identifier: string;
  agent_id: string;
  outcome: string;
  approach_summary: string;
  key_insight?: string;
  pitfalls?: string[];
  effective_patterns?: string[];
  relevant_files?: string[];
  tags?: string[];
}) => invoke<TaskLearning>("record_learning", { input });
export const findSimilarLearnings = (projectId: number, title: string, description?: string, tags?: string[], limit?: number) => invoke<SimilarTaskResult[]>("find_similar_learnings", { projectId, title, description, tags: tags ?? [], limit });
export const listLearnings = (projectId: number, outcome?: string, limit?: number) => invoke<TaskLearning[]>("list_learnings", { projectId, outcome, limit });
export const getLearningsForTask = (taskIdentifier: string) => invoke<TaskLearning[]>("get_learnings_for_task", { taskIdentifier });
// WSJF Scoring
export const setWsjfScores = (input: {
  issue_id: number;
  business_value: number;
  time_criticality: number;
  risk_reduction: number;
  job_size: number;
}) => invoke<WsjfScore>("set_wsjf_scores", { input });
export const autoScoreIssue = (issueId: number) => invoke<AutoScoreResult>("auto_score_issue", { issueId });
export const getRankedBacklog = (projectId: number) => invoke<WsjfScore[]>("get_ranked_backlog", { projectId });
export const autoScoreProject = (projectId: number) => invoke<AutoScoreResult[]>("auto_score_project", { projectId });
export const recalculateScores = (projectId: number) => invoke<WsjfScore[]>("recalculate_scores", { projectId });
// Pipelines
export const listPipelines = (projectId: number) => invoke<Pipeline[]>("list_pipelines", { projectId });
export const getPipeline = (id: number) => invoke<Pipeline>("get_pipeline", { id });
export const createPipeline = (input: {
  project_id: number;
  name: string;
  description?: string;
  stages: unknown[];
}) => invoke<Pipeline>("create_pipeline", { input });
export const updatePipeline = (id: number, input: {
  name?: string;
  description?: string;
  stages?: unknown[];
  enabled?: boolean;
}) => invoke<Pipeline>("update_pipeline", { id, input });
export const deletePipeline = (id: number) => invoke<void>("delete_pipeline", { id });
export const triggerPipeline = (pipelineId: number, triggerIssueId?: number, context?: string) => invoke<PipelineRun>("trigger_pipeline", { pipelineId, triggerIssueId, context });
export const advancePipeline = (runId: number) => invoke<PipelineRun>("advance_pipeline", { runId });
export const cancelPipeline = (runId: number) => invoke<PipelineRun>("cancel_pipeline", { runId });
export const getPipelineRun = (runId: number) => invoke<PipelineRun>("get_pipeline_run", { runId });
export const listPipelineRuns = (pipelineId: number) => invoke<PipelineRun[]>("list_pipeline_runs", { pipelineId });
// Agent Permissions
export const listAgentPermissions = (agentId: string) => invoke<AgentPermission[]>("list_agent_permissions", { agentId });
export const setAgentPermission = (agentId: string, permissionType: string, scope: string, allowed: boolean) => invoke<AgentPermission>("set_agent_permission", { agentId, permissionType, scope, allowed });
export const removeAgentPermission = (id: number) => invoke<void>("remove_agent_permission", { id });
export const clearAgentPermissions = (agentId: string) => invoke<void>("clear_agent_permissions", { agentId });
export const listPermissionPresets = () => invoke<PermissionPreset[]>("list_permission_presets");
export const createPermissionPreset = (name: string, description: string | null, permissions: string) => invoke<PermissionPreset>("create_permission_preset", { name, description, permissions });
export const applyPresetToAgent = (agentId: string, presetId: number) => invoke<AgentPermission[]>("apply_preset_to_agent", { agentId, presetId });
export const deletePermissionPreset = (id: number) => invoke<void>("delete_permission_preset", { id });
export const checkPermission = (agentId: string, permissionType: string, scope: string) => invoke<PermissionCheckResult>("check_permission", { agentId, permissionType, scope });
export const checkFileAccess = (agentId: string, filePath: string) => invoke<PermissionCheckResult>("check_file_access", { agentId, filePath });
export const checkTaskClaim = (agentId: string, taskIdentifier: string) => invoke<PermissionCheckResult>("check_task_claim", { agentId, taskIdentifier });
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

// Git Intelligence
export const getGitStatus = (projectId: number) => invoke<GitStatus>("get_git_status", { projectId });
export const listGitCommits = (projectId: number, limit?: number) => invoke<GitCommit[]>("list_git_commits", { projectId, limit });
export const listGitBranches = (projectId: number) => invoke<GitBranch[]>("list_git_branches", { projectId });
export const listGitWorktrees = (projectId: number) => invoke<GitWorktree[]>("list_git_worktrees", { projectId });
export const getIssueCommits = (projectId: number, issueIdentifier: string) => invoke<GitCommit[]>("get_issue_commits", { projectId, issueIdentifier });
export const getIssueBranches = (projectId: number, issueIdentifier: string) => invoke<GitBranch[]>("get_issue_branches", { projectId, issueIdentifier });
export const getSlaDashboard = (projectId: number) => invoke<SlaDashboard>("get_sla_dashboard", { projectId });
