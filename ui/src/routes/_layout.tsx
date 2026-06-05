import { useEffect, useRef, useState } from "react";
import { Outlet } from "react-router";
import { TitleBar } from "@/components/chrome/TitleBar";
import { ProjectList } from "@/components/sidebar/ProjectList";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";
import { useUndo, useRedo } from "@/data/mutations";
import { classifyUndoRedo } from "@/lib/undoRedoKey";

/** Safely pull a human message off an unknown thrown value (the `ApiError`). */
function messageOf(e: unknown): string | undefined {
  if (e && typeof e === "object" && "message" in e && typeof (e as { message: unknown }).message === "string") {
    return (e as { message: string }).message;
  }
  return undefined;
}

export default function Layout() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const undo = useUndo();
  const redo = useRedo();
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    function showToast(msg: string) {
      setToast(msg);
      if (toastTimer.current) clearTimeout(toastTimer.current);
      toastTimer.current = setTimeout(() => setToast(null), 1500);
    }

    function onKeyDown(e: KeyboardEvent) {
      // Let native text-undo win inside editable fields.
      const t = e.target as HTMLElement | null;
      if (t && (t.isContentEditable || t.tagName === "INPUT" || t.tagName === "TEXTAREA")) return;

      const intent = classifyUndoRedo(e);
      if (!intent) return;
      e.preventDefault();

      if (intent === "undo") {
        undo.mutate(undefined, {
          onSuccess: () => showToast("Undone"),
          onError: (err) => showToast(messageOf(err) ?? "Nothing to undo"),
        });
      } else {
        redo.mutate(undefined, {
          onSuccess: () => showToast("Redone"),
          onError: (err) => showToast(messageOf(err) ?? "Nothing to redo"),
        });
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      if (toastTimer.current) clearTimeout(toastTimer.current);
    };
  }, [undo, redo]);

  return (
    <div className="flex h-screen flex-col bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <TitleBar />
      <div className="grid flex-1 grid-cols-[200px_1fr] overflow-hidden">
        <aside className="flex flex-col gap-2 border-r border-black/10 p-3 text-sm dark:border-white/10">
          <div className="flex items-center justify-between px-2">
            <h2 className="text-xs font-semibold uppercase tracking-wide text-neutral-500">
              Projects
            </h2>
            <button
              type="button"
              onClick={() => setDialogOpen(true)}
              className="rounded-md px-1.5 py-0.5 text-xs text-neutral-600 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
            >
              + New project
            </button>
          </div>
          <ProjectList />
        </aside>
        <main className="overflow-hidden">
          <Outlet />
        </main>
      </div>
      <NewProjectDialog open={dialogOpen} onOpenChange={setDialogOpen} />
      {toast && (
        <div
          role="status"
          className="pointer-events-none fixed bottom-4 left-1/2 z-[60] -translate-x-1/2 rounded-md bg-neutral-900 px-3 py-1.5 text-xs text-white shadow-lg dark:bg-white dark:text-neutral-900"
        >
          {toast}
        </div>
      )}
    </div>
  );
}
