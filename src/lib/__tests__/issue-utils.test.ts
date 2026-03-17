import { describe, it, expect } from "vitest";
import { getAgentType, safeJsonParse } from "../issue-utils";

describe("getAgentType", () => {
  it("detects [claude] prefix", () => {
    const result = getAgentType("[claude] Agent");
    expect(result).not.toBeNull();
    expect(result!.type).toBe("claude");
    expect(result!.color).toContain("orange");
  });

  it("detects [codex] prefix", () => {
    const result = getAgentType("[codex] Agent");
    expect(result).not.toBeNull();
    expect(result!.type).toBe("codex");
    expect(result!.color).toContain("green");
  });

  it("detects [gemini] prefix", () => {
    const result = getAgentType("[gemini] Agent");
    expect(result).not.toBeNull();
    expect(result!.type).toBe("gemini");
    expect(result!.color).toContain("blue");
  });

  it("returns null for plain names", () => {
    expect(getAgentType("Human User")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(getAgentType("")).toBeNull();
  });
});

describe("safeJsonParse", () => {
  it("parses valid JSON object", () => {
    expect(safeJsonParse('{"key":"value"}', {})).toEqual({ key: "value" });
  });

  it("parses valid JSON array", () => {
    expect(safeJsonParse('["a","b","c"]', [])).toEqual(["a", "b", "c"]);
  });

  it("returns fallback for invalid JSON", () => {
    expect(safeJsonParse("{broken", {})).toEqual({});
  });

  it("returns fallback for null input", () => {
    expect(safeJsonParse(null, [])).toEqual([]);
  });

  it("returns fallback for undefined input", () => {
    expect(safeJsonParse(undefined, [])).toEqual([]);
  });

  it("returns fallback for empty string", () => {
    expect(safeJsonParse("", "default")).toBe("default");
  });

  it("parses valid number", () => {
    expect(safeJsonParse("42", 0)).toBe(42);
  });

  it("parses valid boolean", () => {
    expect(safeJsonParse("true", false)).toBe(true);
  });

  it("returns typed fallback for malformed array", () => {
    expect(safeJsonParse<string[]>("[invalid", [])).toEqual([]);
  });
});
