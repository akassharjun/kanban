import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { formatTimestamp, formatTime, normalizeAgentType } from "../format-utils";

describe("formatTimestamp", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-16T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns time only for today", () => {
    expect(formatTimestamp("2026-03-16 14:30:22")).toBe("14:30:22");
  });

  it("returns date+time for other day", () => {
    expect(formatTimestamp("2026-03-14 14:30:22")).toBe("03-14 14:30:22");
  });

  it("passes through short strings", () => {
    expect(formatTimestamp("abc")).toBe("abc");
  });

  it("returns empty time for date-only input", () => {
    expect(formatTimestamp("2026-03-16")).toBe("");
  });
});

describe("formatTime", () => {
  it("extracts HH:MM:SS from full timestamp", () => {
    expect(formatTime("2026-03-16 14:30:22")).toBe("14:30:22");
  });

  it("passes through short strings", () => {
    expect(formatTime("12:00")).toBe("12:00");
  });
});

describe("normalizeAgentType", () => {
  it("maps 'claude-code' to 'claude'", () => {
    expect(normalizeAgentType("claude-code")).toBe("claude");
  });

  it("keeps 'claude' as is", () => {
    expect(normalizeAgentType("claude")).toBe("claude");
  });

  it("keeps 'codex' as is", () => {
    expect(normalizeAgentType("codex")).toBe("codex");
  });

  it("keeps 'gemini' as is", () => {
    expect(normalizeAgentType("gemini")).toBe("gemini");
  });

  it("maps null to 'custom'", () => {
    expect(normalizeAgentType(null)).toBe("custom");
  });

  it("maps empty string to 'custom'", () => {
    expect(normalizeAgentType("")).toBe("custom");
  });

  it("passes through unknown types", () => {
    expect(normalizeAgentType("openai")).toBe("openai");
  });
});
