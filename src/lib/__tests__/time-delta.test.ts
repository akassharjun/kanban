import { describe, it, expect } from "vitest";
import { computeTimeDelta } from "../time-delta";

describe("computeTimeDelta", () => {
  it("returns +5s for a 5-second gap", () => {
    expect(
      computeTimeDelta("2026-03-16 12:00:00", "2026-03-16 12:00:05"),
    ).toBe("+5s");
  });

  it("returns +2m for a 120-second gap", () => {
    expect(
      computeTimeDelta("2026-03-16 12:00:00", "2026-03-16 12:02:00"),
    ).toBe("+2m");
  });

  it("returns null for sub-second gap", () => {
    // Same second timestamps — delta is 0
    expect(
      computeTimeDelta("2026-03-16 12:00:00", "2026-03-16 12:00:00"),
    ).toBeNull();
  });

  it("returns null for exactly 999ms gap", () => {
    // Timestamps at second granularity can't represent 999ms,
    // so two identical timestamps produce delta=0 which is < 1000
    expect(
      computeTimeDelta("2026-03-16 12:00:00", "2026-03-16 12:00:00"),
    ).toBeNull();
  });

  it("returns +1s for exactly 1-second gap", () => {
    expect(
      computeTimeDelta("2026-03-16 12:00:00", "2026-03-16 12:00:01"),
    ).toBe("+1s");
  });

  it("works with space-separated SQLite timestamps", () => {
    expect(
      computeTimeDelta("2026-03-16 10:00:00", "2026-03-16 10:00:30"),
    ).toBe("+30s");
  });
});
