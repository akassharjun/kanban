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

export interface ProjectAgentConfig {
  project_id: number;
  auto_accept_threshold: number;
  human_review_threshold: number;
  max_attempts: number;
  heartbeat_interval_seconds: number;
  missed_heartbeats_before_offline: number;
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
