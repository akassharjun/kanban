import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

const mutate = vi.hoisted(() => vi.fn());
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));

import { NewIssueInline } from "@/components/board/NewIssueInline";

describe("NewIssueInline", () => {
  it("submits a CreateIssue Operation on Enter", async () => {
    mutate.mockReset();
    render(<NewIssueInline projectId="p1" projectPrefix="AUTH" statusId="s1" onDone={() => {}} />);
    await userEvent.type(screen.getByPlaceholderText(/new issue/i), "Add login{enter}");
    expect(mutate).toHaveBeenCalledTimes(1);
    const op = mutate.mock.calls[0][0] as { op: string; args: { title: string; project_id: string; status_id: string } };
    expect(op.op).toBe("CreateIssue");
    expect(op.args.title).toBe("Add login");
    expect(op.args.project_id).toBe("p1");
    expect(op.args.status_id).toBe("s1");
  });
});
