import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { MemberDto } from "@/data/bindings";

const mutate = vi.hoisted(() => vi.fn());
const members = vi.hoisted<() => MemberDto[]>(() => () => [
  { id: "m1", project_id: "p1", name: "Ada" },
  { id: "m2", project_id: "p1", name: "Grace" },
]);

vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));
vi.mock("@/data/queries", () => ({ useMembers: () => ({ data: members() }) }));

import { AssigneePicker } from "@/components/detail/AssigneePicker";

type Op = { op: string; args: Record<string, unknown> };
const lastOp = () => mutate.mock.calls.at(-1)?.[0] as Op;

function renderPicker(assigneeId: string | null = null) {
  return render(
    <AssigneePicker prefix="AUTH" issueId="u1" projectId="p1" assigneeId={assigneeId} />,
  );
}

describe("AssigneePicker", () => {
  it("lists every member as an option", () => {
    mutate.mockReset();
    renderPicker();
    const select = screen.getByLabelText(/assignee/i);
    expect(select).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Ada" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Grace" })).toBeInTheDocument();
  });

  it("selecting a member dispatches an Assignee field change", async () => {
    mutate.mockReset();
    renderPicker();
    await userEvent.selectOptions(screen.getByLabelText(/assignee/i), "m2");
    const op = lastOp();
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args).toMatchObject({ id: "u1", change: { field: "Assignee", value: "m2" } });
  });

  it("selecting Unassigned dispatches a null Assignee value", async () => {
    mutate.mockReset();
    renderPicker("m1");
    await userEvent.selectOptions(screen.getByLabelText(/assignee/i), "");
    const op = lastOp();
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args).toMatchObject({ id: "u1", change: { field: "Assignee", value: null } });
  });

  it("adding a member dispatches CreateMember with a generated id", async () => {
    mutate.mockReset();
    renderPicker();
    await userEvent.type(screen.getByLabelText(/new member name/i), "Linus");
    await userEvent.click(screen.getByRole("button", { name: /^add$/i }));
    const op = lastOp();
    expect(op.op).toBe("CreateMember");
    expect(op.args).toMatchObject({ project_id: "p1", name: "Linus" });
    expect(typeof op.args.id).toBe("string");
    expect((op.args.id as string).length).toBeGreaterThan(0);
  });
});
