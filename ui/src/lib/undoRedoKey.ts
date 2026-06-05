/** Classify a keyboard event into an undo/redo intent (⌘/Ctrl+Z, ⌘/Ctrl+Shift+Z, ⌘/Ctrl+Y). */
export function classifyUndoRedo(e: {
  metaKey: boolean;
  ctrlKey: boolean;
  shiftKey: boolean;
  key: string;
}): "undo" | "redo" | null {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return null;
  const k = e.key.toLowerCase();
  if (k === "z") return e.shiftKey ? "redo" : "undo";
  if (k === "y") return "redo";
  return null;
}
