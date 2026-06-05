import type { ApiError } from "./bindings";

/** Type guard: is `value` a tagged `ApiError` from the Rust bridge? */
export function isApiError(value: unknown): value is ApiError {
  return (
    typeof value === "object" &&
    value !== null &&
    "kind" in value &&
    typeof (value as { kind: unknown }).kind === "string" &&
    ["NotFound", "Validation", "Conflict", "Internal"].includes((value as { kind: string }).kind)
  );
}

/** Coerce any thrown/rejected value into an `ApiError`, defaulting to `Internal`. */
export function asApiError(value: unknown): ApiError {
  if (isApiError(value)) return value;
  const message = value instanceof Error ? value.message : String(value);
  return { kind: "Internal", message };
}
