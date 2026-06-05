import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { LabelDto } from "@/data/bindings";

const mutate = vi.hoisted(() => vi.fn());
const labels = vi.hoisted<() => LabelDto[]>(() => () => [
  { id: "l1", project_id: "p1", name: "bug", color: "#f00" },
  { id: "l2", project_id: "p1", name: "feature", color: "#0f0" },
]);

vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));
vi.mock("@/data/queries", () => ({ useLabels: () => ({ data: labels() }) }));

import { LabelPicker } from "@/components/detail/LabelPicker";

type Op = { op: string; args: Record<string, unknown> };
const lastOp = () => mutate.mock.calls.at(-1)?.[0] as Op;

function renderPicker(attached: LabelDto[] = []) {
  return render(
    <LabelPicker prefix="AUTH" issueId="u1" projectId="p1" attached={attached} />,
  );
}

describe("LabelPicker", () => {
  it("opening the picker lists every project label", async () => {
    mutate.mockReset();
    renderPicker();
    await userEvent.click(screen.getByRole("button", { name: /labels/i }));
    expect(screen.getByText("bug")).toBeInTheDocument();
    expect(screen.getByText("feature")).toBeInTheDocument();
  });

  it("clicking an unattached label dispatches AttachLabel", async () => {
    mutate.mockReset();
    renderPicker([]);
    await userEvent.click(screen.getByRole("button", { name: /labels/i }));
    await userEvent.click(screen.getByRole("button", { name: /^bug$/i }));
    const op = lastOp();
    expect(op.op).toBe("AttachLabel");
    expect(op.args).toEqual({ issue_id: "u1", label_id: "l1" });
  });

  it("clicking an attached label dispatches DetachLabel", async () => {
    mutate.mockReset();
    renderPicker([{ id: "l1", project_id: "p1", name: "bug", color: "#f00" }]);
    await userEvent.click(screen.getByRole("button", { name: /labels/i }));
    await userEvent.click(screen.getByRole("button", { name: /^bug$/i }));
    const op = lastOp();
    expect(op.op).toBe("DetachLabel");
    expect(op.args).toEqual({ issue_id: "u1", label_id: "l1" });
  });

  it("typing a name and clicking Add dispatches CreateLabel", async () => {
    mutate.mockReset();
    renderPicker();
    await userEvent.click(screen.getByRole("button", { name: /labels/i }));
    await userEvent.type(screen.getByLabelText(/new label name/i), "urgent");
    await userEvent.click(screen.getByRole("button", { name: /^add$/i }));
    const op = lastOp();
    expect(op.op).toBe("CreateLabel");
    expect(op.args).toMatchObject({ project_id: "p1", name: "urgent" });
    expect(typeof op.args.id).toBe("string");
    expect((op.args.id as string).length).toBeGreaterThan(0);
    expect(typeof op.args.color).toBe("string");
  });
});
