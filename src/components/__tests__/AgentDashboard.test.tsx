import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { AgentDashboard } from "../AgentDashboard";

vi.mock("@/hooks/use-agents", () => ({
  useAgents: vi.fn(() => ({
    agents: [],
    loading: false,
    refresh: vi.fn(),
  })),
  useProjectMetrics: vi.fn(() => ({
    metrics: null,
    loading: false,
    refresh: vi.fn(),
  })),
}));

vi.mock("@/tauri/commands", () => ({
  getAgentStats: vi.fn(),
  recentActivity: vi.fn(() => Promise.resolve([])),
  getIssue: vi.fn(),
  deregisterAgent: vi.fn(),
}));

describe("AgentDashboard", () => {
  it("renders metrics overview cards", () => {
    render(<AgentDashboard projectId={1} />);
    expect(screen.getByText("Total Tasks")).toBeInTheDocument();
    expect(screen.getByText("Completed")).toBeInTheDocument();
    expect(screen.getByText("Agents Online")).toBeInTheDocument();
  });

  it("renders 'No agents registered' when empty", () => {
    render(<AgentDashboard projectId={1} />);
    expect(screen.getByText("No agents registered")).toBeInTheDocument();
  });

  it("renders agents section header", () => {
    render(<AgentDashboard projectId={1} />);
    expect(screen.getByText("Agents")).toBeInTheDocument();
  });
});
