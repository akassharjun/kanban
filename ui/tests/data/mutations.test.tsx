import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

// `vi.hoisted` so the mock factory (hoisted above module init) can reference it.
const applyMock = vi.hoisted(() => vi.fn());
const undoMock = vi.hoisted(() => vi.fn());
const redoMock = vi.hoisted(() => vi.fn());
vi.mock("@/data/bindings", () => ({
  commands: { apply: applyMock, undo: undoMock, redo: redoMock, updateSettings: vi.fn() },
}));

beforeEach(() => {
  applyMock.mockReset();
  applyMock.mockResolvedValue({ status: "ok", data: { op_id: 42 } });
  undoMock.mockReset();
  undoMock.mockResolvedValue({ status: "ok", data: null });
  redoMock.mockReset();
  redoMock.mockResolvedValue({ status: "ok", data: null });
});

import { useApply, useUndo, useRedo } from "@/data/mutations";
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

  it("invalidates the issues key-space on AttachLabel so the open panel refetches", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: 8 } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(ops.attachLabel({ issue_id: "u1", label_id: "l1" }));
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["issues"] });
  });

  it("invalidates statuses + issues on CreateStatus", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: 9 } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(
        ops.createStatus({
          id: "s9",
          project_id: "p1",
          name: "QA",
          category: "started",
          color: "#94a3b8",
          position: 3,
        }),
      );
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: qk.statuses("AUTH") });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: qk.issues("AUTH") });
  });

  it("invalidates members + issues on CreateMember", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: 10 } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(ops.createMember({ id: "m1", project_id: "p1", name: "Ada" }));
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: qk.members("AUTH") });
  });

  it("invalidates members + issues on DeleteMember (assignees cleared)", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: 11 } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate(ops.deleteMember({ id: "m1" }));
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: qk.members("AUTH") });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["issues"] });
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

describe("useUndo / useRedo", () => {
  it("useUndo calls commands.undo and invalidates queries", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useUndo(), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate();
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(undoMock).toHaveBeenCalledTimes(1);
    expect(invalidateSpy).toHaveBeenCalled();
  });

  it("useRedo calls commands.redo and invalidates queries", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");

    const { result } = renderHook(() => useRedo(), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate();
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });

    expect(redoMock).toHaveBeenCalledTimes(1);
    expect(invalidateSpy).toHaveBeenCalled();
  });

  it("useUndo surfaces an error when the command fails", async () => {
    undoMock.mockResolvedValueOnce({ status: "error", error: { kind: "Validation", message: "nothing to undo" } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    const { result } = renderHook(() => useUndo(), { wrapper: makeWrapper(client) });
    await act(async () => {
      result.current.mutate();
      await waitFor(() => expect(result.current.isError).toBe(true));
    });

    expect(result.current.error).toMatchObject({ kind: "Validation", message: "nothing to undo" });
  });
});
