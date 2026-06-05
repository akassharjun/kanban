import { describe, expect, it } from "vitest";
import { classifyUndoRedo } from "@/lib/undoRedoKey";

const base = { metaKey: false, ctrlKey: false, shiftKey: false, key: "z" };

describe("classifyUndoRedo", () => {
  it("classifies ⌘Z as undo", () => {
    expect(classifyUndoRedo({ ...base, metaKey: true, key: "z" })).toBe("undo");
  });
  it("classifies ⌘⇧Z as redo", () => {
    expect(classifyUndoRedo({ ...base, metaKey: true, shiftKey: true, key: "z" })).toBe("redo");
  });
  it("classifies Ctrl+Z as undo", () => {
    expect(classifyUndoRedo({ ...base, ctrlKey: true, key: "z" })).toBe("undo");
  });
  it("classifies Ctrl+Y as redo", () => {
    expect(classifyUndoRedo({ ...base, ctrlKey: true, key: "y" })).toBe("redo");
  });
  it("classifies ⌘Y as redo", () => {
    expect(classifyUndoRedo({ ...base, metaKey: true, key: "y" })).toBe("redo");
  });
  it("handles uppercase keys", () => {
    expect(classifyUndoRedo({ ...base, metaKey: true, key: "Z" })).toBe("undo");
    expect(classifyUndoRedo({ ...base, metaKey: true, shiftKey: true, key: "Z" })).toBe("redo");
  });
  it("returns null for a plain z (no modifier)", () => {
    expect(classifyUndoRedo({ ...base, key: "z" })).toBeNull();
  });
  it("returns null for ⌘A (modifier but not z/y)", () => {
    expect(classifyUndoRedo({ ...base, metaKey: true, key: "a" })).toBeNull();
  });
});
