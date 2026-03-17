import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Sidebar } from "../Sidebar";

const defaultProps = {
  projects: [],
  selectedProjectId: null,
  onSelectProject: () => {},
  onCreateProject: () => {},
  onOpenMembers: () => {},
  onOpenSettings: () => {},
  onOpenAgents: () => {},
  collapsed: false,
};

describe("Sidebar", () => {
  it("renders badge when agentCount > 0", () => {
    render(<Sidebar {...defaultProps} agentCount={3} />);
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("renders no badge when agentCount is 0", () => {
    render(<Sidebar {...defaultProps} agentCount={0} />);
    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });

  it("renders no badge when agentCount is undefined", () => {
    render(<Sidebar {...defaultProps} />);
    // Agent Ops text should exist but no count badge
    expect(screen.getByText("Agent Ops")).toBeInTheDocument();
  });
});
