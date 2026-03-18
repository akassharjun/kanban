import { listen as tauriListen } from "@tauri-apps/api/event";
import { isTauri } from "./mock-backend";

/**
 * Safe event listener that works both in Tauri and browser-only mode.
 * In browser mode, returns a no-op unlisten function.
 */
export function listen(event: string, handler: (event: any) => void): Promise<() => void> {
  if (isTauri) {
    return tauriListen(event, handler);
  }
  // In browser mode, no events to listen to
  return Promise.resolve(() => {});
}
