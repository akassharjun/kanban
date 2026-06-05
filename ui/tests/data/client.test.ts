import { describe, expect, it } from "vitest";
import { isApiError, asApiError } from "@/data/client";

describe("client error helpers", () => {
  it("identifies a tagged ApiError", () => {
    const e = { kind: "NotFound", resource: "issue", key: "AUTH-12" };
    expect(isApiError(e)).toBe(true);
  });

  it("wraps an unknown error as Internal", () => {
    const e = new Error("boom");
    const api = asApiError(e);
    expect(api.kind).toBe("Internal");
    if (api.kind === "Internal") expect(api.message).toContain("boom");
  });

  it("passes through an existing ApiError", () => {
    const original = { kind: "Validation", field: "prefix", message: "bad" } as const;
    const api = asApiError(original);
    expect(api).toBe(original);
  });
});
