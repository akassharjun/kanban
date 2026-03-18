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
  created_at: string;
  updated_at: string;
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
  timestamp: string;
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

export interface ProjectAgentConfig {
  project_id: number;
  auto_accept_threshold: number;
  human_review_threshold: number;
  max_attempts: number;
  heartbeat_interval_seconds: number;
  missed_heartbeats_before_offline: number;
}
