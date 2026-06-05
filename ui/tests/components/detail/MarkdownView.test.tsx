import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const openMock = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/plugin-shell", () => ({ open: openMock }));

import { MarkdownView } from "@/components/detail/MarkdownView";

describe("MarkdownView", () => {
  it("renders headings, lists, and code", () => {
    render(<MarkdownView source={"# Hello\n\n- one\n- two\n\n`code`"} />);
    expect(screen.getByRole("heading", { name: "Hello" })).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("code")).toBeInTheDocument();
  });

  it("opens links via the tauri shell instead of in-WebView nav", () => {
    openMock.mockReset();
    render(<MarkdownView source={"[link](https://example.com)"} />);
    const a = screen.getByRole("link", { name: "link" });
    a.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    expect(openMock).toHaveBeenCalledWith("https://example.com");
  });
});
