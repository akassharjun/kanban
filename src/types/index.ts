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
  type: string | null;
  skills: string; // JSON array
  task_types: string; // JSON array
  max_concurrent: number;
  max_complexity: string;
  status: AgentStatus;
  registered_at: string;
  last_heartbeat: string;
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

export interface FullTaskContract {
  identifier: string;
  title: string;
  description: string | null;
  priority: string;
  parent_id: number | null;
  issue_id: number;
  type: TaskType;
  task_state: TaskState;
  objective: string;
  context: Record<string, unknown>;
  constraints: unknown[];
  success_criteria: unknown[];
  required_skills: string[];
  estimated_complexity: string | null;
  timeout_minutes: number;
  attempt_count: number;
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
