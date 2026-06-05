import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

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
      { id: "s1", name: "To Do", position: 1 },
      { id: "s2", name: "In Progress", position: 2 },
    ],
    isLoading: false,
    isError: false,
  }),
}));
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate: vi.fn(), isPending: false }) }));

import { Board } from "@/components/board/Board";

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
});
