import { Outlet } from "react-router";

export default function Layout() {
  return (
    <div className="grid h-screen grid-cols-[200px_1fr] bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <aside className="border-r border-black/10 p-3 text-sm dark:border-white/10">
        {/* Sidebar lands in Task 14 */}
      </aside>
      <main className="overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
