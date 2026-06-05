import { describe, expect, it } from "vitest";
import { ops, change, sortKeyBits } from "@/data/ops";

describe("ops builders", () => {
  it("createProject wraps args under {op,args}", () => {
    const o = ops.createProject({ id: "p1", name: "Auth", prefix: "AUTH" });
    expect(o).toEqual({ op: "CreateProject", args: { id: "p1", name: "Auth", prefix: "AUTH" } });
  });

  it("createIssue defaults label_ids to an empty array", () => {
    const o = ops.createIssue({
      id: "i1",
      project_id: "p1",
      title: "Add login",
      status_id: "s1",
      priority: "high",
    });
    expect(o.op).toBe("CreateIssue");
    expect(o.args.label_ids).toEqual([]);
  });

  it("updateIssueField uses id + change shape", () => {
    const o = ops.updateIssueField({ id: "u1", change: change.title("hi") });
    expect(o.op).toBe("UpdateIssueField");
    expect(o.args.id).toBe("u1");
    expect(o.args.change).toEqual({ field: "Title", value: "hi" });
  });

  it("change.priority emits the lowercase Priority value", () => {
    expect(change.priority("high")).toEqual({ field: "Priority", value: "high" });
  });

  it("sortKeyBits encodes f64 as core's serde_f64::bits hex form", () => {
    expect(sortKeyBits(1.0)).toBe("0x3ff0000000000000");
    expect(sortKeyBits(1.5)).toBe("0x3ff8000000000000");
    expect(sortKeyBits(0)).toBe("0x0000000000000000");
  });

  it("reorderIssue carries id and bit-encoded new_sort_key", () => {
    const o = ops.reorderIssue({ id: "u2", new_sort_key: 1.5 });
    expect(o).toEqual({
      op: "ReorderIssue",
      args: { id: "u2", new_sort_key: "0x3ff8000000000000" },
    });
  });
});
