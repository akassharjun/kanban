import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/data/queries", () => ({
  useIssues: () => ({
    data: [
      { id: "u1", identifier: "AUTH-1", title: "First", status_id: "s1", sort_key: 1, priority: "none" },
      { id: "u2", identifier: "AUTH-2", title: "Second", status_id: "s2", sort_key: 1, priority: "none" },
    ],
    isLoading: false,
    isError: false,
  }),
  useStatuses: () => ({
    data: [
      { id: "s1", name: "To Do", category: "unstarted", color: "#94a3b8", position: 1 },
      { id: "s2", name: "In Progress", category: "started", color: "#3b82f6", position: 2 },
    ],
    isLoading: false,
    isError: false,
  }),
}));
const mutateMock = vi.hoisted(() => vi.fn());
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate: mutateMock, isPending: false }) }));

import { Board } from "@/components/board/Board";

beforeEach(() => {
  mutateMock.mockReset();
});

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={new QueryClient()}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

describe("Board", () => {
  it("renders one column per status with the right issues", () => {
    render(<Board projectPrefix="AUTH" projectId="p1" />, { wrapper });
    expect(screen.getByText("To Do")).toBeInTheDocument();
    expect(screen.getByText("In Progress")).toBeInTheDocument();
    expect(screen.getByText("First")).toBeInTheDocument();
    expect(screen.getByText("Second")).toBeInTheDocument();
  });

  it("add-column form dispatches CreateStatus with the entered fields", () => {
    render(<Board projectPrefix="AUTH" projectId="p1" />, { wrapper });
    fireEvent.click(screen.getByText("+ Add column"));
    fireEvent.change(screen.getByLabelText("New column name"), { target: { value: "QA" } });
    fireEvent.change(screen.getByLabelText("New column category"), { target: { value: "completed" } });
    fireEvent.change(screen.getByLabelText("New column color"), { target: { value: "#112233" } });
    fireEvent.click(screen.getByText("Add"));

    const call = mutateMock.mock.calls.at(-1);
    expect(call).toBeTruthy();
    const op = call?.[0] as { op: string; args: Record<string, unknown> };
    expect(op.op).toBe("CreateStatus");
    expect(op.args).toMatchObject({
      project_id: "p1",
      name: "QA",
      category: "completed",
      color: "#112233",
      position: 2,
    });
    expect(typeof op.args.id).toBe("string");
  });
});
