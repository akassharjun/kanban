// Operation + IssueFieldChange builders.
//
// The frontend owns these types because core's `Operation` enum is NOT a specta
// `Type` — `apply()` accepts a `JsonValue`. The shapes below MUST match core's
// serde representation exactly:
//   - `Operation` is `#[serde(tag = "op", content = "args")]`        → { op, args }
//   - `IssueFieldChange` is `#[serde(tag = "field", content = "value")]`
//   - `ReorderIssue.new_sort_key` uses `serde_f64::bits` → a "0x"+16-hex string
//     of the f64 big-endian bit pattern, NOT a plain JSON number.

export const PRIORITIES = ["none", "low", "medium", "high", "urgent"] as const;
export type Priority = (typeof PRIORITIES)[number];

/** Status categories, matching core's `StatusCategory` (lowercase serde). */
export const STATUS_CATEGORIES = [
  "unstarted",
  "started",
  "blocked",
  "completed",
  "discarded",
] as const;
export type StatusCategory = (typeof STATUS_CATEGORIES)[number];

export type IssueFieldChange =
  | { field: "Title"; value: string }
  | { field: "Description"; value: string | null }
  | { field: "Status"; value: string } // status_id (uuid)
  | { field: "Priority"; value: Priority }
  | { field: "DueDate"; value: string | null }; // "YYYY-MM-DD" or null

export interface Operation {
  op: string;
  args: Record<string, unknown>;
}

/**
 * Encode an `f64` as core's `serde_f64::bits` wire form: `"0x"` followed by 16
 * lowercase hex digits of the big-endian `u64` bit pattern.
 * e.g. `1.0` → `"0x3ff0000000000000"`, `1.5` → `"0x3ff8000000000000"`.
 *
 * Core's `ReorderIssue.new_sort_key` and `Issue.sort_key` use this encoding to
 * round-trip doubles bit-exactly; a plain JSON number would fail to deserialize.
 */
export function sortKeyBits(value: number): string {
  const view = new DataView(new ArrayBuffer(8));
  view.setFloat64(0, value, false); // big-endian
  const hi = BigInt(view.getUint32(0, false));
  const lo = BigInt(view.getUint32(4, false));
  return "0x" + ((hi << 32n) | lo).toString(16).padStart(16, "0");
}

export const ops = {
  createProject: (args: {
    id: string;
    name: string;
    prefix: string;
    description?: string | null;
    icon?: string | null;
  }): Operation => ({ op: "CreateProject", args }),

  createIssue: (args: {
    id: string;
    project_id: string;
    title: string;
    status_id: string;
    priority: Priority;
    description?: string | null;
    due_date?: string | null;
    label_ids?: string[];
  }): Operation => ({ op: "CreateIssue", args: { label_ids: [], ...args } }),

  updateIssueField: (args: { id: string; change: IssueFieldChange }): Operation => ({
    op: "UpdateIssueField",
    args,
  }),

  reorderIssue: (args: { id: string; new_sort_key: number }): Operation => ({
    op: "ReorderIssue",
    args: { id: args.id, new_sort_key: sortKeyBits(args.new_sort_key) },
  }),

  deleteIssue: (args: { id: string }): Operation => ({ op: "DeleteIssue", args }),

  createLabel: (args: {
    id: string;
    project_id: string;
    name: string;
    color: string;
  }): Operation => ({ op: "CreateLabel", args }),

  updateLabel: (args: { id: string; name?: string; color?: string }): Operation => {
    const patch: Record<string, unknown> = {};
    if (args.name !== undefined) patch.name = args.name;
    if (args.color !== undefined) patch.color = args.color;
    return { op: "UpdateLabel", args: { id: args.id, patch } };
  },

  deleteLabel: (args: { id: string }): Operation => ({ op: "DeleteLabel", args }),

  attachLabel: (args: { issue_id: string; label_id: string }): Operation => ({
    op: "AttachLabel",
    args,
  }),

  detachLabel: (args: { issue_id: string; label_id: string }): Operation => ({
    op: "DetachLabel",
    args,
  }),

  updateProject: (args: {
    id: string;
    name?: string;
    description?: string | null;
  }): Operation => {
    const patch: Record<string, unknown> = {};
    if (args.name !== undefined) patch.name = args.name;
    if (args.description !== undefined) patch.description = args.description;
    return { op: "UpdateProject", args: { id: args.id, patch } };
  },

  createStatus: (args: {
    id: string;
    project_id: string;
    name: string;
    category: string;
    color: string;
    position: number;
  }): Operation => ({ op: "CreateStatus", args }),

  updateStatus: (args: {
    id: string;
    name?: string;
    category?: string;
    color?: string;
  }): Operation => {
    const patch: Record<string, unknown> = {};
    if (args.name !== undefined) patch.name = args.name;
    if (args.category !== undefined) patch.category = args.category;
    if (args.color !== undefined) patch.color = args.color;
    return { op: "UpdateStatus", args: { id: args.id, patch } };
  },

  deleteStatus: (args: { id: string }): Operation => ({ op: "DeleteStatus", args }),

  reorderStatus: (args: { id: string; new_position: number }): Operation => ({
    op: "ReorderStatus",
    args,
  }),

  archiveProject: (args: { id: string }): Operation => ({ op: "ArchiveProject", args }),

  deleteProject: (args: { id: string }): Operation => ({ op: "DeleteProject", args }),
} as const;

export const change = {
  title: (v: string): IssueFieldChange => ({ field: "Title", value: v }),
  description: (v: string | null): IssueFieldChange => ({ field: "Description", value: v }),
  status: (v: string): IssueFieldChange => ({ field: "Status", value: v }),
  priority: (v: Priority): IssueFieldChange => ({ field: "Priority", value: v }),
  dueDate: (v: string | null): IssueFieldChange => ({ field: "DueDate", value: v }),
} as const;
