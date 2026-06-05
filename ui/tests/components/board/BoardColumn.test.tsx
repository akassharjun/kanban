import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { DndContext } from "@dnd-kit/core";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { IssueDto, StatusDto } from "@/data/bindings";

const mutateMock = vi.hoisted(() => vi.fn());
vi.mock("@/data/mutations", () => ({
  useApply: () => ({ mutate: mutateMock, isPending: false }),
}));

import { BoardColumn } from "@/components/board/BoardColumn";

beforeEach(() => {
  mutateMock.mockReset();
});

const status: StatusDto = {
  id: "s1",
  project_id: "p1",
  name: "To Do",
  category: "unstarted",
  color: "#94a3b8",
  position: 0,
};

const issues: IssueDto[] = [];

function wrapper({ children }: { children: ReactNode }) {
  return (
    <MemoryRouter>
      <DndContext>{children}</DndContext>
    </MemoryRouter>
  );
}

function renderColumn(props: Partial<Parameters<typeof BoardColumn>[0]> = {}) {
  return render(
    <BoardColumn
      status={status}
      issues={issues}
      projectPrefix="AUTH"
      onAddIssue={() => {}}
      index={1}
      count={3}
      {...props}
    />,
    { wrapper },
  );
}

/** Typed access to the most recent `apply.mutate(op, ...)` call. */
function lastOp(): { op: string; args: Record<string, unknown> } {
  const call = mutateMock.mock.calls.at(-1);
  if (!call) throw new Error("mutate was not called");
  return call[0] as { op: string; args: Record<string, unknown> };
}

function openMenu() {
  fireEvent.click(screen.getByLabelText("Column actions for To Do"));
}

describe("BoardColumn column management", () => {
  it("rename dispatches UpdateStatus with a name patch on Enter", () => {
    renderColumn();
    openMenu();
    fireEvent.click(screen.getByText("Rename"));
    const input = screen.getByLabelText("Rename column");
    fireEvent.change(input, { target: { value: "QA" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(lastOp()).toEqual({ op: "UpdateStatus", args: { id: "s1", patch: { name: "QA" } } });
  });

  it("rename does not dispatch when unchanged", () => {
    renderColumn();
    openMenu();
    fireEvent.click(screen.getByText("Rename"));
    const input = screen.getByLabelText("Rename column");
    fireEvent.keyDown(input, { key: "Enter" });
    expect(mutateMock).not.toHaveBeenCalled();
  });

  it("recolor dispatches UpdateStatus with a color patch", () => {
    renderColumn();
    openMenu();
    fireEvent.change(screen.getByLabelText("Column color"), { target: { value: "#ff0000" } });
    expect(lastOp()).toEqual({
      op: "UpdateStatus",
      args: { id: "s1", patch: { color: "#ff0000" } },
    });
  });

  it("delete dispatches DeleteStatus", () => {
    renderColumn();
    openMenu();
    fireEvent.click(screen.getByText("Delete"));
    expect(lastOp()).toEqual({ op: "DeleteStatus", args: { id: "s1" } });
  });

  it("surfaces the core error message when delete is blocked", () => {
    mutateMock.mockImplementation(
      (_op: unknown, opts?: { onError?: (e: unknown) => void }) => {
        opts?.onError?.({ kind: "Conflict", message: "status still has issues" });
      },
    );
    renderColumn();
    openMenu();
    fireEvent.click(screen.getByText("Delete"));
    expect(screen.getByText("status still has issues")).toBeInTheDocument();
  });

  it("move left dispatches ReorderStatus with index-1", () => {
    renderColumn({ index: 1, count: 3 });
    openMenu();
    fireEvent.click(screen.getByLabelText("Move column left"));
    expect(lastOp()).toEqual({ op: "ReorderStatus", args: { id: "s1", new_position: 0 } });
  });

  it("move right dispatches ReorderStatus with index+1", () => {
    renderColumn({ index: 1, count: 3 });
    openMenu();
    fireEvent.click(screen.getByLabelText("Move column right"));
    expect(lastOp()).toEqual({ op: "ReorderStatus", args: { id: "s1", new_position: 2 } });
  });

  it("disables move left at index 0", () => {
    renderColumn({ index: 0, count: 3 });
    openMenu();
    expect(screen.getByLabelText("Move column left")).toBeDisabled();
  });

  it("disables move right at the last index", () => {
    renderColumn({ index: 2, count: 3 });
    openMenu();
    expect(screen.getByLabelText("Move column right")).toBeDisabled();
  });
});
