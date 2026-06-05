import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EditableField } from "@/components/detail/EditableField";

describe("EditableField", () => {
  it("commits new value on blur", async () => {
    const onCommit = vi.fn();
    render(<EditableField value="old" onCommit={onCommit} />);
    await userEvent.click(screen.getByText("old"));
    const input = screen.getByDisplayValue("old");
    await userEvent.clear(input);
    await userEvent.type(input, "new");
    input.blur();
    expect(onCommit).toHaveBeenCalledWith("new");
  });

  it("reverts on Escape", async () => {
    const onCommit = vi.fn();
    render(<EditableField value="old" onCommit={onCommit} />);
    await userEvent.click(screen.getByText("old"));
    const input = screen.getByDisplayValue("old");
    await userEvent.type(input, "{Escape}");
    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByText("old")).toBeInTheDocument();
  });
});
