import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

// `vi.hoisted` so the mock factory (hoisted above module init) can reference it.
const applyMock = vi.hoisted(() => vi.fn());
vi.mock("@/data/bindings", () => ({
  commands: { apply: applyMock, undo: vi.fn(), redo: vi.fn(), updateSettings: vi.fn() },
}));

beforeEach(() => {
  applyMock.mockReset();
  applyMock.mockResolvedValue({ status: "ok", data: { op_id: 42 } });
});

import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";
import { qk } from "@/data/queries";

function makeWrapper(client: QueryClient) {
  return function W({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

describe("useApply optimistic dispatcher", () => {
  it("optimistically removes a deleted issue from the cache and rolls back on error", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(qk.issues("AUTH"), [{ id: "u1" }, { id: "u2" }]);

    applyMock.mockRejectedValueOnce({ kind: "Conflict", message: "no" });

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(ops.deleteIssue({ id: "u1" }));
      await waitFor(() => expect(result.current.isError).toBe(true));
    });

    const cache = client.getQueryData<Array<{ id: string }>>(qk.issues("AUTH"));
    expect(cache?.map((i) => i.id)).toEqual(["u1", "u2"]); // rolled back
  });

  it("optimistically adds a created project to the cache on success", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: 7 } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(qk.projects(), [{ id: "p1", prefix: "AUTH", name: "Auth" }]);

    const { result } = renderHook(() => useApply(), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(ops.createProject({ id: "p2", prefix: "PAY", name: "Pay" }));
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    const cache = client.getQueryData<Array<{ prefix: string }>>(qk.projects());
    expect(cache?.map((p) => p.prefix)).toContain("PAY");
  });
});
