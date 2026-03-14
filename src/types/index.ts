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
