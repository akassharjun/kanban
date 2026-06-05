import { useState } from "react";
import { Outlet } from "react-router";
import { TitleBar } from "@/components/chrome/TitleBar";
import { ProjectList } from "@/components/sidebar/ProjectList";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";

export default function Layout() {
  const [dialogOpen, setDialogOpen] = useState(false);

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
    </div>
  );
}
