import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

// Hoisted command mocks so the `vi.mock` factory can reference them.
const undoMock = vi.hoisted(() => vi.fn());
const redoMock = vi.hoisted(() => vi.fn());
const okEmpty = vi.hoisted(() => () => Promise.resolve({ status: "ok", data: [] }));

vi.mock("@/data/bindings", () => ({
  commands: {
    apply: vi.fn(),
    undo: undoMock,
    redo: redoMock,
    listProjects: okEmpty,
    listIssues: okEmpty,
    listStatuses: okEmpty,
    listLabels: okEmpty,
    getIssue: () => Promise.resolve({ status: "ok", data: null }),
    getSettings: () => Promise.resolve({ status: "ok", data: { theme: "system" } }),
    updateSettings: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  },
}));

beforeEach(() => {
  undoMock.mockReset();
  undoMock.mockResolvedValue({ status: "ok", data: null });
  redoMock.mockReset();
  redoMock.mockResolvedValue({ status: "ok", data: null });
});

import Layout from "@/routes/_layout";

function renderLayout() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const router = createMemoryRouter([{ path: "/", element: <Layout /> }], { initialEntries: ["/"] });
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

function press(key: string, init: Partial<KeyboardEventInit> = {}) {
  window.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init }));
}

describe("Layout undo/redo keyboard handler", () => {
  it("fires undo on ⌘Z and shows a toast", async () => {
    renderLayout();
    press("z", { metaKey: true });
    await waitFor(() => expect(undoMock).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("status")).toHaveTextContent("Undone");
  });

  it("fires redo on ⌘⇧Z", async () => {
    renderLayout();
    press("z", { metaKey: true, shiftKey: true });
    await waitFor(() => expect(redoMock).toHaveBeenCalledTimes(1));
    expect(undoMock).not.toHaveBeenCalled();
  });

  it("ignores ⌘Z when an input is focused", async () => {
    renderLayout();
    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();
    // dispatch with the input as the target
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "z", metaKey: true, bubbles: true, cancelable: true }),
    );
    // give any microtasks a chance, then assert undo never ran
    await Promise.resolve();
    expect(undoMock).not.toHaveBeenCalled();
    input.remove();
  });
});
