import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { describe, expect, it, vi } from "vitest";
import { ProjectList } from "@/components/sidebar/ProjectList";

vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate: vi.fn(), isPending: false }) }));
vi.mock("@/data/queries", () => ({
  useProjects: () => ({
    data: [
      { id: "p1", prefix: "AUTH", name: "Auth", description: null, icon: null, status: "active", created_at: "", updated_at: "" },
      { id: "p2", prefix: "PAY", name: "Payments", description: null, icon: null, status: "active", created_at: "", updated_at: "" },
    ],
    isLoading: false,
    isError: false,
  }),
}));

describe("ProjectList", () => {
  it("renders each project's prefix and name as a link", () => {
    render(<MemoryRouter><ProjectList /></MemoryRouter>);
    expect(screen.getByText("AUTH")).toBeInTheDocument();
    expect(screen.getByText("Auth")).toBeInTheDocument();
    expect(screen.getByText("PAY")).toBeInTheDocument();
    // links point at /p/:prefix
    expect(screen.getByRole("link", { name: /AUTH/ })).toHaveAttribute("href", "/p/AUTH");
  });
});
