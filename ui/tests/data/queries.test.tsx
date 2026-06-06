import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@/data/bindings", () => ({
  commands: {
    listProjects: vi
      .fn()
      .mockResolvedValue({ status: "ok", data: [{ id: "p1", prefix: "AUTH", name: "Auth" }] }),
    listIssues: vi.fn().mockResolvedValue({ status: "ok", data: [] }),
    getIssue: vi.fn(),
    listStatuses: vi.fn(),
    listLabels: vi.fn(),
    listMembers: vi
      .fn()
      .mockResolvedValue({ status: "ok", data: [{ id: "m1", project_id: "p1", name: "Ada" }] }),
    getSettings: vi.fn(),
    updateSettings: vi.fn(),
  },
}));

import { useMembers, useProjects } from "@/data/queries";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useProjects", () => {
  it("returns the project list when commands.listProjects resolves", async () => {
    const { result } = renderHook(() => useProjects(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0].prefix).toBe("AUTH");
  });

  it("surfaces an ApiError when the command returns status error", async () => {
    const { commands } = await import("@/data/bindings");
    (commands.listProjects as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      status: "error",
      error: { kind: "Internal", message: "boom" },
    });
    const { result } = renderHook(() => useProjects(), { wrapper });
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect((result.current.error as { kind?: string })?.kind).toBe("Internal");
  });
});

describe("useMembers", () => {
  it("returns the member roster for a project prefix", async () => {
    const { result } = renderHook(() => useMembers("AUTH"), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0].name).toBe("Ada");
  });

  it("is disabled when prefix is empty", () => {
    const { result } = renderHook(() => useMembers(""), { wrapper });
    expect(result.current.fetchStatus).toBe("idle");
  });
});
