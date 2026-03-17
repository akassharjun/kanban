import { vi } from "vitest";

export async function mockInvoke(handler: (cmd: string, args?: Record<string, unknown>) => unknown) {
  const { invoke } = vi.mocked(await import("@tauri-apps/api/core"));
  invoke.mockImplementation(((cmd: string, args?: Record<string, unknown>) =>
    Promise.resolve(handler(cmd, args))
  ) as typeof invoke);
  return invoke;
}

export async function mockInvokeOnce(result: unknown) {
  const { invoke } = vi.mocked(await import("@tauri-apps/api/core"));
  invoke.mockResolvedValueOnce(result);
  return invoke;
}
