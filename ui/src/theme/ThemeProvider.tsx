import { useEffect, type ReactNode } from "react";
import { useSettings } from "@/data/queries";
import { useSystemTheme } from "./useSystemTheme";

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { data: settings } = useSettings();
  const systemTheme = useSystemTheme();

  useEffect(() => {
    const choice = settings?.theme ?? "system";
    const effective = choice === "system" ? systemTheme : choice;
    document.documentElement.classList.toggle("dark", effective === "dark");
  }, [settings?.theme, systemTheme]);

  return <>{children}</>;
}
