import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

const mutate = vi.hoisted(() => vi.fn());
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));
vi.mock("@/data/queries", () => ({
  useIssue: () => ({
    data: {
      id: "u1",
      project_id: "p1",
      identifier: "AUTH-12",
      title: "Hello",
      description: "# body",
      status_id: "s1",
      priority: "medium",
      due_date: "2026-01-01",
      labels: [{ id: "l1", project_id: "p1", name: "bug", color: "#f00" }],
    },
    isLoading: false,
    isError: false,
  }),
  useStatuses: () => ({
    data: [
      { id: "s1", name: "To Do", position: 1 },
      { id: "s2", name: "Done", position: 2 },
    ],
  }),
  useLabels: () => ({ data: [{ id: "l1", project_id: "p1", name: "bug", color: "#f00" }] }),
}));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

import { IssuePanel } from "@/components/detail/IssuePanel";

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={new QueryClient()}>
      <MemoryRouter initialEntries={["/p/AUTH/i/AUTH-12"]}>
        <Routes>
          <Route path="/p/:prefix/i/:key" element={children} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("IssuePanel", () => {
  it("renders issue title and identifier", () => {
    render(<IssuePanel />, { wrapper });
    expect(screen.getByText("AUTH-12")).toBeInTheDocument();
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });

  it("dispatches updateIssueField on status change", async () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    await userEvent.selectOptions(screen.getByLabelText(/status/i), "s2");
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { change: unknown } };
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "Status", value: "s2" });
  });

  it("dispatches updateIssueField on priority change", async () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    await userEvent.selectOptions(screen.getByLabelText(/priority/i), "high");
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { change: unknown } };
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "Priority", value: "high" });
  });

  it("dispatches updateIssueField on due-date change", () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    fireEvent.change(screen.getByLabelText(/due date/i), {
      target: { value: "2026-12-31" },
    });
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { change: unknown } };
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "DueDate", value: "2026-12-31" });
  });

  it("clearing the due date dispatches a null value", () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    fireEvent.change(screen.getByLabelText(/due date/i), {
      target: { value: "" },
    });
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { change: unknown } };
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "DueDate", value: null });
  });

  it("renders a chip for an attached label and detaches on remove", async () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    expect(screen.getByText("bug")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /remove bug/i }));
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: Record<string, unknown> };
    expect(op.op).toBe("DetachLabel");
    expect(op.args).toEqual({ issue_id: "u1", label_id: "l1" });
  });

  it("dispatches deleteIssue after confirming delete", async () => {
    mutate.mockReset();
    render(<IssuePanel />, { wrapper });
    await userEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    await userEvent.click(screen.getByRole("button", { name: /confirm/i }));
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { id: string } };
    expect(op.op).toBe("DeleteIssue");
    expect(op.args.id).toBe("u1");
  });
});
