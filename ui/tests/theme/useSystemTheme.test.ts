import { renderHook, act } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useSystemTheme } from "@/theme/useSystemTheme";

function mockMatchMedia(initial: boolean) {
  const listeners: Array<(e: MediaQueryListEvent) => void> = [];
  const mql = {
    matches: initial,
    addEventListener: (_evt: string, l: (e: MediaQueryListEvent) => void) => listeners.push(l),
    removeEventListener: (_evt: string, l: (e: MediaQueryListEvent) => void) => {
      const i = listeners.indexOf(l);
      if (i >= 0) listeners.splice(i, 1);
    },
    media: "(prefers-color-scheme: dark)",
    onchange: null,
    dispatchEvent: () => true,
  } as unknown as MediaQueryList;
  vi.stubGlobal("matchMedia", () => mql);
  return {
    mql,
    fire: (matches: boolean) => listeners.forEach((l) => l({ matches } as MediaQueryListEvent)),
  };
}

describe("useSystemTheme", () => {
  it("returns 'dark' when matchMedia matches", () => {
    mockMatchMedia(true);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("dark");
  });

  it("returns 'light' when matchMedia does not match", () => {
    mockMatchMedia(false);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");
  });

  it("re-renders when system preference changes", () => {
    const { fire } = mockMatchMedia(false);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");
    act(() => fire(true));
    expect(result.current).toBe("dark");
  });
});
