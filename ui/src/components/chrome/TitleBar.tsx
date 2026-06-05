import { ThemeToggle } from "./ThemeToggle";

export function TitleBar() {
  return (
    <header
      data-tauri-drag-region
      className="flex h-11 shrink-0 items-center justify-between border-b border-black/10 pl-[76px] pr-3 dark:border-white/10"
    >
      <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">kanban</span>
      <ThemeToggle />
    </header>
  );
}
