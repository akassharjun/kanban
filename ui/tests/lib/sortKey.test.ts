import { describe, expect, it } from "vitest";
import { midpoint } from "@/lib/sortKey";

describe("midpoint", () => {
  it("returns 1024 for an empty column", () => {
    expect(midpoint(undefined, undefined)).toBe(1024);
  });
  it("subtracts 1024 when dropped at top", () => {
    expect(midpoint(undefined, 5000)).toBe(5000 - 1024);
  });
  it("adds 1024 when dropped at bottom", () => {
    expect(midpoint(5000, undefined)).toBe(5000 + 1024);
  });
  it("returns the average between two neighbors", () => {
    expect(midpoint(1, 2)).toBe(1.5);
    expect(midpoint(0, 10)).toBe(5);
  });
});
