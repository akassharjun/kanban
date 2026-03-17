import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useAgents, useProjectMetrics } from "../use-agents";

vi.mock("@/tauri/commands", () => ({
  listAgents: vi.fn(),
  projectMetrics: vi.fn(),
  getAgentStats: vi.fn(),
  taskReplay: vi.fn(),
}));

import * as api from "@/tauri/commands";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useAgents", () => {
  it("returns agents from listAgents", async () => {
    const mockAgents = [
      { id: "a1", name: "Agent 1", status: "online" },
    ];
    vi.mocked(api.listAgents).mockResolvedValue(mockAgents as never);

    const { result } = renderHook(() => useAgents());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.agents).toEqual(mockAgents);
  });

  it("sets loading=false after fetch", async () => {
    vi.mocked(api.listAgents).mockResolvedValue([]);

    const { result } = renderHook(() => useAgents());

    await waitFor(() => expect(result.current.loading).toBe(false));
  });
});

describe("useProjectMetrics", () => {
  it("returns null when projectId is null", async () => {
    const { result } = renderHook(() => useProjectMetrics(null));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.metrics).toBeNull();
  });
});
