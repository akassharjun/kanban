import { useMutation, useQueryClient, type QueryClient } from "@tanstack/react-query";
import { commands, type JsonValue } from "./bindings";
import { asApiError } from "./client";
import type { Operation } from "./ops";
import { qk } from "./queries";

type Ctx = { rollback?: () => void };
type WithId = { id: string };

/**
 * Optimistically patch the query cache for an operation, returning a rollback
 * closure (used by `onError`). Operations without a cheap local patch
 * (`UpdateIssueField`, `ReorderIssue`, label ops) fall through to a refetch on
 * settle — granular per-field patches are Spec #3 polish.
 */
function optimisticPatch(qc: QueryClient, op: Operation, prefix?: string): Ctx {
  const { args } = op;
  switch (op.op) {
    case "CreateProject": {
      const prev = qc.getQueryData<unknown[]>(qk.projects()) ?? [];
      qc.setQueryData(qk.projects(), [...prev, args]);
      return { rollback: () => qc.setQueryData(qk.projects(), prev) };
    }
    case "DeleteProject": {
      const prev = qc.getQueryData<WithId[]>(qk.projects()) ?? [];
      const id = (args as WithId).id;
      qc.setQueryData(
        qk.projects(),
        prev.filter((p) => p.id !== id),
      );
      return { rollback: () => qc.setQueryData(qk.projects(), prev) };
    }
    case "CreateIssue": {
      if (!prefix) return {};
      const prev = qc.getQueryData<unknown[]>(qk.issues(prefix)) ?? [];
      qc.setQueryData(qk.issues(prefix), [...prev, args]);
      return { rollback: () => qc.setQueryData(qk.issues(prefix), prev) };
    }
    case "DeleteIssue": {
      if (!prefix) return {};
      const prev = qc.getQueryData<WithId[]>(qk.issues(prefix)) ?? [];
      const id = (args as WithId).id;
      qc.setQueryData(
        qk.issues(prefix),
        prev.filter((i) => i.id !== id),
      );
      return { rollback: () => qc.setQueryData(qk.issues(prefix), prev) };
    }
    default:
      return {};
  }
}

/** Invalidate the query keys affected by an operation, so the next read refetches. */
function invalidateFor(qc: QueryClient, op: Operation, prefix?: string): void {
  const tag = op.op;
  if (["CreateProject", "UpdateProject", "ArchiveProject", "DeleteProject"].includes(tag)) {
    void qc.invalidateQueries({ queryKey: qk.projects() });
  }
  if (["CreateIssue", "UpdateIssueField", "ReorderIssue", "DeleteIssue"].includes(tag)) {
    if (prefix) void qc.invalidateQueries({ queryKey: qk.issues(prefix) });
    if (tag === "UpdateIssueField") {
      const id = (op.args as Partial<WithId>).id;
      if (id) void qc.invalidateQueries({ queryKey: qk.issue(id) });
    }
  }
  if (["AttachLabel", "DetachLabel", "CreateLabel", "UpdateLabel", "DeleteLabel"].includes(tag)) {
    if (prefix) {
      void qc.invalidateQueries({ queryKey: qk.labels(prefix) });
      void qc.invalidateQueries({ queryKey: qk.issues(prefix) });
    }
  }
  if (tag === "ImportSnapshot") void qc.invalidateQueries();
}

/**
 * The single domain mutator hook: dispatches an `Operation` via `commands.apply`,
 * optimistically patches the cache, rolls back on error, and invalidates on settle.
 *
 * @param prefixForCache the active project prefix, used to scope issue-cache patches.
 */
export function useApply(prefixForCache?: string) {
  const qc = useQueryClient();
  return useMutation<unknown, unknown, Operation, Ctx>({
    mutationFn: async (op: Operation) => {
      const r = await commands.apply(op as unknown as JsonValue);
      if (r.status !== "ok") throw asApiError(r.error);
      return r.data;
    },
    onMutate: (op) => optimisticPatch(qc, op, prefixForCache),
    onError: (_err, _op, ctx) => ctx?.rollback?.(),
    onSettled: (_data, _err, op) => invalidateFor(qc, op, prefixForCache),
  });
}

/**
 * Undo the last applied operation via `commands.undo`. Because undo can touch
 * any entity, it invalidates every query on settle rather than scoping.
 */
export function useUndo() {
  const qc = useQueryClient();
  return useMutation<unknown, unknown, void>({
    mutationFn: async () => {
      const r = await commands.undo();
      if (r.status !== "ok") throw asApiError(r.error);
      return r.data;
    },
    onSettled: () => {
      void qc.invalidateQueries();
    },
  });
}

/**
 * Redo the last undone operation via `commands.redo`. Invalidates every query
 * on settle (see `useUndo`).
 */
export function useRedo() {
  const qc = useQueryClient();
  return useMutation<unknown, unknown, void>({
    mutationFn: async () => {
      const r = await commands.redo();
      if (r.status !== "ok") throw asApiError(r.error);
      return r.data;
    },
    onSettled: () => {
      void qc.invalidateQueries();
    },
  });
}
