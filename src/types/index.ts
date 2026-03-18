export type Priority = "none" | "urgent" | "high" | "medium" | "low";

export type StatusCategory =
  | "unstarted"
  | "started"
  | "blocked"
  | "completed"
  | "discarded";

export type ProjectStatus = "active" | "paused" | "completed" | "archived";

export type RelationType = "related" | "blocks" | "blocked_by" | "duplicate";

export interface Project {
  id: number;
  name: string;
  description: string | null;
  icon: string | null;
  status: ProjectStatus;
  prefix: string;
  issue_counter: number;
  path: string | null;
  created_at: string;
  updated_at: string;
  stale_days: number | null;
  stale_close_status_id: number | null;
}

export interface Status {
  id: number;
  project_id: number;
  name: string;
  category: StatusCategory;
  color: string | null;
  icon: string | null;
  position: number;
}

export interface Issue {
  id: number;
  project_id: number;
  identifier: string;
  title: string;
  description: string | null;
  status_id: number;
  priority: Priority;
  assignee_id: number | null;
  parent_id: number | null;
  position: number;
  estimate: number | null;
  due_date: string | null;
  epic_id: number | null;
  milestone_id: number | null;
  business_value: number | null;
  time_criticality: number | null;
  risk_reduction: number | null;
  job_size: number | null;
  wsjf_score: number | null;
  created_at: string;
  updated_at: string;
}

export type EpicStatus = "active" | "closed";

export interface Epic {
  id: number;
  project_id: number;
  title: string;
  description: string | null;
  color: string;
  status: EpicStatus;
  created_at: string;
  updated_at: string;
}

export type MilestoneStatus = "open" | "closed";

export interface Milestone {
  id: number;
  project_id: number;
  title: string;
  description: string | null;
  due_date: string | null;
  status: MilestoneStatus;
  created_at: string;
  updated_at: string;
}

export interface MilestoneWithProgress extends Milestone {
  total_issues: number;
  completed_issues: number;
}

export interface Label {
  id: number;
  project_id: number;
  name: string;
  color: string;
}

export interface Member {
  id: number;
  name: string;
  display_name: string | null;
  email: string | null;
  avatar_color: string;
  created_at: string;
}

export interface IssueRelation {
  id: number;
  source_issue_id: number;
  target_issue_id: number;
  relation_type: RelationType;
}

export interface IssueTemplate {
  id: number;
  project_id: number;
  name: string;
  description_template: string | null;
  default_status_id: number | null;
  default_priority: Priority;
  default_label_ids: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityLogEntry {
  id: number;
  issue_id: number;
  field_changed: string;
  old_value: string | null;
  new_value: string | null;
  actor_id: number | null;
  actor_type: string | null;
  timestamp: string;
}

export interface AuditLogEntry {
  id: number;
  issue_id: number;
  issue_identifier: string;
  issue_title: string;
  field_changed: string;
  old_value: string | null;
  new_value: string | null;
  actor_id: number | null;
  actor_type: string | null;
  actor_name: string | null;
  actor_avatar_color: string | null;
  timestamp: string;
}

export interface IssueHistoryEntry {
  id: number;
  issue_id: number;
  field_changed: string;
  old_value: string | null;
  new_value: string | null;
  actor_id: number | null;
  actor_type: string | null;
  actor_name: string | null;
  actor_avatar_color: string | null;
  timestamp: string;
}

export interface MentionEntry {
  id: number;
  issue_id: number;
  issue_identifier: string;
  issue_title: string;
  comment_id: number | null;
  member_id: number;
  source: "description" | "comment";
  created_at: string;
}

export interface IssueWithLabels extends Issue {
  labels: Label[];
}

export interface UndoLogEntry {
  id: number;
  operation_type: string;
  entity_type: string;
  entity_id: number;
  snapshot_before: string | null;
  snapshot_after: string | null;
  undone: boolean;
  timestamp: string;
}

export interface Comment {
  id: number;
  issue_id: number;
  member_id: number | null;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface Notification {
  id: number;
  type: string;
  issue_id: number | null;
  message: string;
  read: boolean;
  created_at: string;
}

export interface FullTaskContract {
  id: number;
  project_id: number;
  identifier: string;
  title: string;
  objective: string;
  status_id: number;
  type: string | null;
  description: string | null;
  priority: string | null;
  skills: string[] | null;
  complexity: string | null;
  constraints: string[] | null;
  success_criteria: unknown[] | null;
  context_files: string[] | null;
  timeout_minutes: number | null;
  depends_on: string[] | null;
  assigned_agent: string | null;
  confidence: number | null;
  summary: string | null;
  artifacts: Record<string, unknown> | null;
  task_state: string;
  claimed_by: string | null;
  claimed_at: string | null;
  attempt_count: number;
  created_at: string;
  updated_at: string;
}

export interface CustomField {
  id: number;
  project_id: number;
  name: string;
  field_type: "text" | "number" | "date" | "select";
  options: string | null;
  position: number;
}

export interface CustomFieldValue {
  id: number;
  issue_id: number;
  field_id: number;
  value: string | null;
}

// Agent Orchestration Types

export type TaskState = "queued" | "claimed" | "executing" | "validating" | "completed" | "blocked" | "cancelled";
export type TaskType = "implementation" | "research" | "testing" | "review" | "decomposition";
export type AgentStatus = "idle" | "busy" | "offline" | "online";

export interface Agent {
  id: string;
  name: string;
  agent_type: string | null;
  skills: string[];
  task_types: string[];
  max_concurrent: number;
  max_complexity: string;
  member_id: number | null;
  worktree_path: string | null;
  status: AgentStatus;
  registered_at: string;
  last_heartbeat: string;
  last_activity_at: string | null;
}

export interface AgentMetrics {
  agent_id: string;
  name: string;
  status: string;
  tasks_completed: number;
  tasks_failed: number;
  success_rate: number;
  avg_confidence: number;
  avg_completion_time_minutes: number;
  current_tasks: string[];
  skills_success_rate: Record<string, unknown>;
}

export interface ProjectMetrics {
  total_tasks: number;
  completed: number;
  queued: number;
  in_progress: number;
  blocked: number;
  validating: number;
  failed_attempts: number;
  agents_online: number;
  avg_confidence: number | null;
  avg_completion_time_minutes: number | null;
  tasks_completed_24h: number;
  task_type_breakdown: Record<string, { count: number }>;
}

export interface ExecutionLog {
  id: number;
  issue_id: number;
  agent_id: string;
  attempt_number: number;
  entry_type: string;
  message: string;
  metadata: string | null;
  timestamp: string;
}

export interface GraphNode {
  id: number;
  identifier: string;
  title: string;
  state: string;
  type?: string;
}

export interface GraphEdge {
  from: number;
  to: number;
  type: string;
}

export interface TaskGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface Hook {
  id: number;
  project_id: number;
  event_type: string;
  command: string;
}

export interface SavedView {
  id: number;
  project_id: number;
  name: string;
  filters: string;
  sort_by: string | null;
  sort_direction: string | null;
  view_mode: string | null;
  created_at: string;
  updated_at: string;
}

export type AutomationTriggerType =
  | "status_change"
  | "issue_created"
  | "issue_updated"
  | "pr_merged"
  | "pr_opened"
  | "task_completed"
  | "task_failed"
  | "agent_assigned"
  | "label_added"
  | "priority_changed"
  | "comment_added"
  | "schedule";

export type AutomationActionType =
  | "change_status"
  | "set_priority"
  | "assign_to"
  | "add_label"
  | "create_issue"
  | "create_task_contract"
  | "add_comment"
  | "send_notification"
  | "trigger_webhook";

export interface AutomationCondition {
  field: string;
  operator: string;
  value: string;
}

export interface AutomationAction {
  type: AutomationActionType;
  config: Record<string, unknown>;
}

export interface AutomationRule {
  id: number;
  project_id: number;
  name: string;
  enabled: boolean;
  trigger_type: AutomationTriggerType;
  trigger_config: string;
  conditions: string;
  actions: string;
  execution_count: number;
  last_executed_at: string | null;
  created_at: string;
  updated_at: string;
}

export type RecurrenceType = "daily" | "weekly" | "biweekly" | "monthly" | "custom";

export interface RecurringIssue {
  id: number;
  project_id: number;
  title_template: string;
  description_template: string | null;
  status_id: number;
  priority: Priority;
  assignee_id: number | null;
  label_ids: string;
  recurrence_type: RecurrenceType;
  recurrence_config: string;
  next_run_at: string;
  last_run_at: string | null;
  enabled: boolean;
  total_created: number;
  created_at: string;
  updated_at: string;
}

export type GitLinkType = "branch" | "pr" | "commit";

export interface GitLink {
  id: number;
  issue_id: number;
  link_type: GitLinkType;
  url: string | null;
  ref_name: string;
  pr_number: number | null;
  pr_state: string | null;
  pr_merged: boolean;
  ci_status: string | null;
  review_status: string | null;
  created_at: string;
  updated_at: string;
}

export interface StarredIssue {
  id: number;
  issue_id: number;
  member_id: number;
  created_at: string;
}

export interface RecentlyViewedEntry {
  id: number;
  issue_id: number;
  member_id: number;
  viewed_at: string;
}

export interface AutomationLogEntry {
  id: number;
  rule_id: number;
  issue_id: number | null;
  trigger_type: string;
  actions_executed: string;
  success: boolean;
  error_message: string | null;
  executed_at: string;
}

// AI Agent Intelligence Types

export interface TriageSuggestion {
  suggested_priority: string | null;
  suggested_label_ids: number[];
  suggested_assignee_id: number | null;
  suggested_epic_id: number | null;
  confidence: number;
  reasoning: string;
}

export interface DecomposedTask {
  title: string;
  description: string | null;
  suggested_priority: string | null;
  suggested_labels: string[];
}

export interface ParsedIssue {
  title: string;
  description: string;
  suggested_priority: string | null;
  suggested_label_ids: number[];
  suggested_assignee_id: number | null;
}

export interface RecurringPreview {
  title: string;
  description: string | null;
  next_dates: string[];
}

export interface DependencyNode {
  id: number;
  identifier: string;
  title: string;
  status_category: string;
  priority: string;
  assignee_name: string | null;
}

export interface DependencyEdge {
  source_id: number;
  target_id: number;
  relation_type: string;
}

export interface DependencyGraph {
  nodes: DependencyNode[];
  edges: DependencyEdge[];
}

export interface ProjectAgentConfig {
  project_id: number;
  auto_accept_threshold: number;
  human_review_threshold: number;
  max_attempts: number;
  heartbeat_interval_seconds: number;
  missed_heartbeats_before_offline: number;
  use_wsjf_scoring: boolean;
}

export interface WsjfScore {
  issue_id: number;
  identifier: string;
  title: string;
  business_value: number;
  time_criticality: number;
  risk_reduction: number;
  job_size: number;
  wsjf_score: number;
  priority: string;
}

export interface AutoScoreResult {
  issue_id: number;
  business_value: number;
  time_criticality: number;
  risk_reduction: number;
  job_size: number;
  wsjf_score: number;
  reasoning: string;
}

// GitHub Integration Types

export interface GithubConfig {
  id: number;
  project_id: number;
  repo_owner: string;
  repo_name: string;
  access_token: string | null;
  branch_pattern: string;
  auto_link_prs: boolean;
  auto_transition_on_merge: boolean;
  merge_target_status_id: number | null;
  created_at: string;
  updated_at: string;
}

export interface GithubEvent {
  id: number;
  project_id: number;
  event_type: "pr_opened" | "pr_merged" | "pr_closed" | "pr_review" | "check_run" | "push";
  issue_id: number | null;
  payload: string;
  processed: boolean;
  created_at: string;
}

export interface CIStatus {
  status: "pending" | "success" | "failure" | "neutral" | "unknown";
  checks: CICheck[];
}

export interface CICheck {
  name: string;
  status: string;
  conclusion: string | null;
  url: string | null;
}

export interface PRStatus {
  number: number;
  title: string;
  state: string;
  merged: boolean;
  review_status: "approved" | "changes_requested" | "pending" | "none";
  ci_status: "pending" | "success" | "failure";
  url: string;
  author: string;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  rate_limit_remaining: number | null;
}

export interface BranchNamePreview {
  branch_name: string;
  pattern: string;
}

// Code Analysis Types

export interface IssueFileLink {
  id: number;
  issue_id: number;
  file_path: string;
  link_type: "related" | "cause" | "fix";
  created_at: string;
}

export interface FileHeatEntry {
  file_path: string;
  issue_count: number;
  bug_count: number;
  last_issue_at: string;
}

export interface DirectoryHeatEntry {
  directory: string;
  issue_count: number;
  file_count: number;
}

// Context Assembly Types

export interface PriorAttempt {
  agent_name: string;
  attempt_number: number;
  result: string;
  reason: string | null;
}

export interface TaskContext {
  issue: Issue;
  labels: Label[];
  parent_issue: Issue | null;
  sub_issues: Issue[];
  related_issues: IssueRelation[];
  blocking_issues: Issue[];
  blocked_issues: Issue[];
  comments: Comment[];
  activity_log: ActivityLogEntry[];
  prior_attempts: PriorAttempt[];
  similar_completed_issues: Issue[];
  project_path: string | null;
  context_files: string[];
}

// Agent Analytics Types

export interface DailyMetric {
  date: string;
  completed: number;
  failed: number;
}

export interface AgentPerformance {
  agent_id: string;
  agent_name: string;
  total_tasks: number;
  completed: number;
  failed: number;
  rejected: number;
  timeout: number;
  success_rate: number;
  avg_confidence: number;
  avg_duration_minutes: number;
  total_lines_changed: number;
  tasks_by_type: Record<string, number>;
  tasks_by_complexity: Record<string, number>;
  daily_completions: DailyMetric[];
}

export interface AgentRanking {
  agent_id: string;
  agent_name: string;
  score: number;
  success_rate: number;
  avg_confidence: number;
  tasks_completed: number;
}

export interface ProjectAgentSummary {
  total_agent_tasks: number;
  total_completed: number;
  total_failed: number;
  avg_completion_time_minutes: number;
  agents_active: number;
  top_performers: AgentRanking[];
  task_type_distribution: Record<string, number>;
  completion_trend: DailyMetric[];
}

// Marketplace Types

export interface AgentRegistryEntry {
  id: number;
  agent_id: string;
  name: string;
  description: string | null;
  provider: string | null;
  version: string | null;
  endpoint: string | null;
  capabilities: string; // JSON string of string[]
  max_concurrent: number;
  max_complexity: string;
  hourly_rate: number | null;
  rating: number | null;
  total_tasks: number;
  registered_at: string;
  last_seen_at: string | null;
}

export interface AgentCapability {
  id: number;
  agent_id: string;
  capability: string;
  proficiency: number;
  tasks_completed: number;
  tasks_failed: number;
  avg_confidence: number | null;
  updated_at: string;
}

export interface AgentMatch {
  agent_id: string;
  name: string;
  score: number;
  matched_skills: string[];
  avg_proficiency: number;
  rating: number | null;
  status: string;
}

// Handoff Notes Types

export type HandoffNoteType = "completion" | "review_request" | "escalation" | "context" | "warning" | "suggestion";

export interface HandoffNote {
  id: number;
  task_identifier: string;
  from_agent_id: string;
  to_agent_id: string | null;
  note_type: HandoffNoteType;
  summary: string;
  details: string | null;
  files_changed: string[];
  risks: string[];
  test_results: { passed?: number; failed?: number; skipped?: number } | null;
  metadata: Record<string, unknown>;
  created_at: string;
}

export type LearningOutcome = "success" | "failure" | "partial";

export interface TaskLearning {
  id: number;
  task_identifier: string;
  agent_id: string;
  outcome: LearningOutcome;
  approach_summary: string;
  key_insight: string | null;
  pitfalls: string[];
  effective_patterns: string[];
  relevant_files: string[];
  tags: string[];
  created_at: string;
}

export interface SimilarTaskResult {
  learning: TaskLearning;
  similarity_score: number;
  issue_title: string;
  issue_identifier: string;
}

// Pipeline Types

export interface PipelineStage {
  name: string;
  task_type: string;
  required_skills: string[];
  max_complexity: string;
  timeout_minutes: number;
  title_template: string;
  objective_template: string;
  success_criteria: string[];
  auto_advance: boolean;
}

export interface Pipeline {
  id: number;
  project_id: number;
  name: string;
  description: string | null;
  stages: string; // JSON string of PipelineStage[]
  enabled: boolean;
  total_runs: number;
  created_at: string;
  updated_at: string;
}

export interface PipelineRun {
  id: number;
  pipeline_id: number;
  trigger_issue_id: number | null;
  status: "running" | "completed" | "failed" | "cancelled";
  current_stage: number;
  stage_tasks: string; // JSON string of stage task info
  context: string; // JSON string
  started_at: string;
  completed_at: string | null;
  error_message: string | null;
}

// Agent Permissions

export type PermissionType = "project_access" | "file_access" | "action" | "task_type" | "max_cost";

export interface AgentPermission {
  id: number;
  agent_id: string;
  permission_type: PermissionType;
  scope: string;
  allowed: boolean;
  created_at: string;
}

export interface PermissionPreset {
  id: number;
  name: string;
  description: string | null;
  permissions: string; // JSON array of {permission_type, scope, allowed}
  created_at: string;
}

export interface PermissionCheckResult {
  allowed: boolean;
  reason: string | null;
  matched_rule: AgentPermission | null;
}

// Cost Tracking Types

export interface TaskCost {
  id: number;
  task_identifier: string;
  agent_id: string;
  cost_type: "compute_time" | "api_tokens" | "custom";
  amount: number;
  unit: string;
  description: string | null;
  recorded_at: string;
}

export interface TaskCostSummary {
  task_identifier: string;
  total_compute_minutes: number;
  total_tokens: number;
  total_cost_dollars: number;
  cost_breakdown: TaskCost[];
}

export interface AgentCostEntry {
  agent_id: string;
  agent_name: string;
  total_cost: number;
  task_count: number;
  avg_cost_per_task: number;
}

export interface DailyCostEntry {
  date: string;
  cost: number;
  task_count: number;
}

export interface BudgetStatus {
  budget_id: number;
  budget_type: "daily" | "weekly" | "monthly" | "per_task" | "total";
  amount: number;
  unit: string;
  spent: number;
  percentage: number;
  alert: boolean;
  alert_threshold: number;
}

export interface ProjectCostSummary {
  project_id: number;
  total_cost: number;
  cost_by_agent: AgentCostEntry[];
  daily_costs: DailyCostEntry[];
  budget_status: BudgetStatus[];
}

export interface CostBudget {
  id: number;
  project_id: number;
  budget_type: string;
  amount: number;
  unit: string;
  spent: number;
  period_start: string | null;
  period_end: string | null;
  alert_threshold: number | null;
  created_at: string;
}

// SLA Types

export interface SlaPolicy {
  id: number;
  project_id: number;
  name: string;
  target_type: "response_time" | "resolution_time" | "task_timeout";
  priority_filter: string | null;
  warning_minutes: number;
  breach_minutes: number;
  escalation_action: string;
  enabled: number;
  created_at: string;
}

export interface SlaStatus {
  issue_id: number;
  issue_identifier: string;
  issue_title: string;
  policy_id: number;
  policy_name: string;
  status: "ok" | "warning" | "breached";
  elapsed_minutes: number;
  remaining_minutes: number;
  breach_at: string;
}

export interface SlaEvent {
  id: number;
  sla_policy_id: number;
  issue_id: number;
  event_type: "warning" | "breach" | "escalated" | "resolved";
  message: string;
  metadata: string | null;
  created_at: string;
}

export interface SlaDashboard {
  total_tracked: number;
  total_ok: number;
  total_warning: number;
  total_breached: number;
  policies: SlaPolicy[];
  statuses: SlaStatus[];
  recent_events: SlaEvent[];
}

// Agent presence types (for card UI)
export type ExecutionEntryType =
  | "reasoning"
  | "file_read"
  | "file_edit"
  | "command"
  | "discovery"
  | "error"
  | "checkpoint"
  | "claim"
  | "start"
  | "complete"
  | "fail"
  | "timeout";

export interface AgentPresenceData {
  agentId: string;
  agentName: string;
  agentType: "claude" | "codex" | "gemini" | "custom";
  status: "active" | "idle" | "error" | "offline";
  lastAction?: string;
  lastActionType?: ExecutionEntryType;
}

export interface TickerEntry {
  id: number;
  agentName: string;
  agentId: string;
  action: string;
  entryType: string;
  issueIdentifier: string | null;
  issueId: number;
  timestamp: string;
}
