import { render, screen } from "@testing-library/react";
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
      identifier: "AUTH-12",
      title: "Hello",
      description: "# body",
      status_id: "s1",
      priority: "high",
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
    await userEvent.selectOptions(screen.getByRole("combobox"), "s2");
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { change: unknown } };
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "Status", value: "s2" });
  });
});
