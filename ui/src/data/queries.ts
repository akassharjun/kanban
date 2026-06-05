// STUB for Task 11 — real TanStack Query hooks land in Task 16. Do not expand here.
import type { Settings, ThemeChoice } from "./bindings";

export function useSettings(): { data: Settings | undefined } {
  return { data: undefined };
}

export function useUpdateSettings(): { mutate: (_theme: ThemeChoice) => void } {
  return { mutate: (_theme: ThemeChoice) => {} };
}
