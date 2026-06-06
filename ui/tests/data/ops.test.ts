import { describe, expect, it } from "vitest";
import { ops, change, sortKeyBits, STATUS_CATEGORIES } from "@/data/ops";

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

  it("createLabel wraps args", () => {
    expect(ops.createLabel({ id: "l1", project_id: "p1", name: "bug", color: "#f00" })).toEqual({
      op: "CreateLabel",
      args: { id: "l1", project_id: "p1", name: "bug", color: "#f00" },
    });
  });

  it("attachLabel uses issue_id + label_id", () => {
    expect(ops.attachLabel({ issue_id: "u1", label_id: "l1" })).toEqual({
      op: "AttachLabel",
      args: { issue_id: "u1", label_id: "l1" },
    });
  });

  it("detachLabel uses issue_id + label_id", () => {
    expect(ops.detachLabel({ issue_id: "u1", label_id: "l1" })).toEqual({
      op: "DetachLabel",
      args: { issue_id: "u1", label_id: "l1" },
    });
  });

  it("updateProject wraps a name patch", () => {
    expect(ops.updateProject({ id: "p1", name: "Renamed" })).toEqual({
      op: "UpdateProject",
      args: { id: "p1", patch: { name: "Renamed" } },
    });
  });

  it("updateLabel wraps a patch", () => {
    expect(ops.updateLabel({ id: "l1", color: "#0f0" })).toEqual({
      op: "UpdateLabel",
      args: { id: "l1", patch: { color: "#0f0" } },
    });
  });

  it("createStatus wraps args under {op,args}", () => {
    expect(
      ops.createStatus({
        id: "s1",
        project_id: "p1",
        name: "QA",
        category: "started",
        color: "#94a3b8",
        position: 2,
      }),
    ).toEqual({
      op: "CreateStatus",
      args: {
        id: "s1",
        project_id: "p1",
        name: "QA",
        category: "started",
        color: "#94a3b8",
        position: 2,
      },
    });
  });

  it("updateStatus wraps a name patch", () => {
    expect(ops.updateStatus({ id: "s1", name: "QA" })).toEqual({
      op: "UpdateStatus",
      args: { id: "s1", patch: { name: "QA" } },
    });
  });

  it("updateStatus wraps a color patch", () => {
    expect(ops.updateStatus({ id: "s1", color: "#0f0" })).toEqual({
      op: "UpdateStatus",
      args: { id: "s1", patch: { color: "#0f0" } },
    });
  });

  it("updateStatus omits undefined fields from the patch", () => {
    expect(ops.updateStatus({ id: "s1", category: "completed" })).toEqual({
      op: "UpdateStatus",
      args: { id: "s1", patch: { category: "completed" } },
    });
  });

  it("deleteStatus takes just id", () => {
    expect(ops.deleteStatus({ id: "s1" })).toEqual({ op: "DeleteStatus", args: { id: "s1" } });
  });

  it("reorderStatus carries id and new_position", () => {
    expect(ops.reorderStatus({ id: "s1", new_position: 0 })).toEqual({
      op: "ReorderStatus",
      args: { id: "s1", new_position: 0 },
    });
  });

  it("exposes the five status categories", () => {
    expect(STATUS_CATEGORIES).toEqual([
      "unstarted",
      "started",
      "blocked",
      "completed",
      "discarded",
    ]);
  });

  it("archiveProject / deleteProject / deleteLabel take just id", () => {
    expect(ops.archiveProject({ id: "p1" })).toEqual({ op: "ArchiveProject", args: { id: "p1" } });
    expect(ops.deleteProject({ id: "p1" })).toEqual({ op: "DeleteProject", args: { id: "p1" } });
    expect(ops.deleteLabel({ id: "l1" })).toEqual({ op: "DeleteLabel", args: { id: "l1" } });
  });

  it("createMember wraps args under {op,args}", () => {
    expect(ops.createMember({ id: "m1", project_id: "p1", name: "Ada" })).toEqual({
      op: "CreateMember",
      args: { id: "m1", project_id: "p1", name: "Ada" },
    });
  });

  it("updateMember wraps a sparse name patch", () => {
    expect(ops.updateMember({ id: "m1", name: "Grace" })).toEqual({
      op: "UpdateMember",
      args: { id: "m1", patch: { name: "Grace" } },
    });
  });

  it("updateMember omits undefined name from the patch", () => {
    expect(ops.updateMember({ id: "m1" })).toEqual({
      op: "UpdateMember",
      args: { id: "m1", patch: {} },
    });
  });

  it("deleteMember takes just id", () => {
    expect(ops.deleteMember({ id: "m1" })).toEqual({ op: "DeleteMember", args: { id: "m1" } });
  });

  it("change.assignee assigns a member id", () => {
    expect(change.assignee("m1")).toEqual({ field: "Assignee", value: "m1" });
  });

  it("change.assignee unassigns with null", () => {
    expect(change.assignee(null)).toEqual({ field: "Assignee", value: null });
  });
});
