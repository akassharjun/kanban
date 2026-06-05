// Operation + IssueFieldChange builders.
//
// The frontend owns these types because core's `Operation` enum is NOT a specta
// `Type` — `apply()` accepts a `JsonValue`. The shapes below MUST match core's
// serde representation exactly:
//   - `Operation` is `#[serde(tag = "op", content = "args")]`        → { op, args }
//   - `IssueFieldChange` is `#[serde(tag = "field", content = "value")]`
//   - `ReorderIssue.new_sort_key` uses `serde_f64::bits` → a "0x"+16-hex string
//     of the f64 big-endian bit pattern, NOT a plain JSON number.

export type Priority = "none" | "low" | "medium" | "high" | "urgent";

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
} as const;

export const change = {
  title: (v: string): IssueFieldChange => ({ field: "Title", value: v }),
  description: (v: string | null): IssueFieldChange => ({ field: "Description", value: v }),
  status: (v: string): IssueFieldChange => ({ field: "Status", value: v }),
  priority: (v: Priority): IssueFieldChange => ({ field: "Priority", value: v }),
  dueDate: (v: string | null): IssueFieldChange => ({ field: "DueDate", value: v }),
} as const;
