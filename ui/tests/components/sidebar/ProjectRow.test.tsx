import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { describe, expect, it, vi } from "vitest";
import type { ProjectDto } from "@/data/bindings";

const mutate = vi.hoisted(() => vi.fn());
const navigate = vi.hoisted(() => vi.fn());

vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));
vi.mock("react-router", async () => {
  const actual = await vi.importActual<typeof import("react-router")>("react-router");
  return { ...actual, useNavigate: () => navigate };
});

import { ProjectRow } from "@/components/sidebar/ProjectRow";

const project: ProjectDto = {
  id: "p1",
  name: "Auth",
  prefix: "AUTH",
  description: null,
  icon: null,
  status: "active",
  created_at: "",
  updated_at: "",
};

function renderRow(at = "/p/PAY") {
  return render(
    <MemoryRouter initialEntries={[at]}>
      <Routes>
        <Route path="/p/:prefix" element={<ProjectRow project={project} />} />
        <Route path="/" element={<ProjectRow project={project} />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ProjectRow", () => {
  it("renders the project's prefix and name as a link", () => {
    renderRow();
    expect(screen.getByText("AUTH")).toBeInTheDocument();
    expect(screen.getByText("Auth")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /AUTH/ })).toHaveAttribute("href", "/p/AUTH");
  });

  it("rename dispatches UpdateProject with the new name", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^rename$/i }));
    const input = screen.getByRole("textbox", { name: /rename project/i });
    await userEvent.clear(input);
    await userEvent.type(input, "Authentication{enter}");
    expect(mutate).toHaveBeenCalledTimes(1);
    const op = mutate.mock.calls.at(-1)?.[0] as {
      op: string;
      args: { id: string; patch: { name: string } };
    };
    expect(op.op).toBe("UpdateProject");
    expect(op.args).toEqual({ id: "p1", patch: { name: "Authentication" } });
  });

  it("rename with an unchanged name dispatches nothing", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^rename$/i }));
    const input = screen.getByRole("textbox", { name: /rename project/i });
    await userEvent.type(input, "{enter}");
    expect(mutate).not.toHaveBeenCalled();
  });

  it("rename Escape cancels without dispatching", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^rename$/i }));
    const input = screen.getByRole("textbox", { name: /rename project/i });
    await userEvent.clear(input);
    await userEvent.type(input, "Nope{escape}");
    expect(mutate).not.toHaveBeenCalled();
    expect(screen.queryByRole("textbox", { name: /rename project/i })).not.toBeInTheDocument();
  });

  it("archive dispatches ArchiveProject", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^archive$/i }));
    expect(mutate).toHaveBeenCalledTimes(1);
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { id: string } };
    expect(op.op).toBe("ArchiveProject");
    expect(op.args).toEqual({ id: "p1" });
  });

  it("delete dispatches DeleteProject only after confirming", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    expect(mutate).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: /confirm delete/i }));
    expect(mutate).toHaveBeenCalledTimes(1);
    const op = mutate.mock.calls.at(-1)?.[0] as { op: string; args: { id: string } };
    expect(op.op).toBe("DeleteProject");
    expect(op.args).toEqual({ id: "p1" });
  });

  it("toggling the menu resets a pending delete confirmation", async () => {
    mutate.mockReset();
    renderRow();
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeInTheDocument();
    // Close then reopen the menu — the confirm state must NOT persist.
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    expect(screen.queryByRole("button", { name: /confirm delete/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^delete$/i })).toBeInTheDocument();
    expect(mutate).not.toHaveBeenCalled();
  });

  it("archiving the active project navigates home", async () => {
    mutate.mockReset();
    navigate.mockReset();
    renderRow("/p/AUTH");
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^archive$/i }));
    expect(navigate).toHaveBeenCalledWith("/");
  });

  it("deleting the active project navigates home", async () => {
    mutate.mockReset();
    navigate.mockReset();
    renderRow("/p/AUTH");
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    await userEvent.click(screen.getByRole("button", { name: /confirm delete/i }));
    expect(navigate).toHaveBeenCalledWith("/");
  });

  it("archiving a non-active project does not navigate", async () => {
    mutate.mockReset();
    navigate.mockReset();
    renderRow("/p/PAY");
    await userEvent.click(screen.getByRole("button", { name: /actions for AUTH/i }));
    await userEvent.click(screen.getByRole("button", { name: /^archive$/i }));
    expect(navigate).not.toHaveBeenCalled();
  });
});
