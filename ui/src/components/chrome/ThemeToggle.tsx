import { Moon, Sun, Monitor } from "lucide-react";
import type { ReactNode } from "react";
import { useSettings, useUpdateSettings } from "@/data/queries";
import type { ThemeChoice } from "@/data/bindings";

const ORDER: ThemeChoice[] = ["system", "light", "dark"];
const ICON: Record<ThemeChoice, ReactNode> = {
  system: <Monitor className="h-4 w-4" />,
  light: <Sun className="h-4 w-4" />,
  dark: <Moon className="h-4 w-4" />,
};

export function ThemeToggle() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const current = settings?.theme ?? "system";
  const next = () => {
    const i = ORDER.indexOf(current);
    update.mutate(ORDER[(i + 1) % ORDER.length]);
  };
  return (
    <button
      type="button"
      onClick={next}
      title={`theme: ${current}`}
      className="rounded p-1.5 text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10"
    >
      {ICON[current]}
    </button>
  );
}
