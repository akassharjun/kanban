import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ThemeProvider } from "@/theme/ThemeProvider";

vi.mock("@/data/queries", () => ({
  useSettings: vi.fn(),
  useUpdateSettings: vi.fn(),
}));

import { useSettings } from "@/data/queries";

beforeEach(() => {
  document.documentElement.classList.remove("dark");
  vi.stubGlobal("matchMedia", () => ({
    matches: false,
    addEventListener: () => {},
    removeEventListener: () => {},
    media: "",
    onchange: null,
    dispatchEvent: () => true,
  }));
});

describe("ThemeProvider", () => {
  it("adds dark class when explicit theme is dark", () => {
    (useSettings as unknown as ReturnType<typeof vi.fn>).mockReturnValue({ data: { theme: "dark" } });
    render(<ThemeProvider><span data-testid="child">x</span></ThemeProvider>);
    expect(screen.getByTestId("child")).toBeInTheDocument();
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("does not add dark class when explicit theme is light", () => {
    (useSettings as unknown as ReturnType<typeof vi.fn>).mockReturnValue({ data: { theme: "light" } });
    render(<ThemeProvider><span /></ThemeProvider>);
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});
