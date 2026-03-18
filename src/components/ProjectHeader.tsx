import { LayoutGrid, List, TreePine, Calendar, GanttChart, Search, Plus, Bell, Sun, Moon } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import type { Project } from "@/types";

export type ViewMode = "board" | "list" | "tree" | "calendar" | "gantt";

interface ProjectHeaderProps {
  project: Project;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  onSearch: () => void;
  onCreateIssue: () => void;
  notificationCount: number;
  onOpenNotifications: () => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function ProjectHeader({
  project,
  viewMode,
  onViewModeChange,
  onSearch,
  onCreateIssue,
  notificationCount,
  onOpenNotifications,
  theme,
  onToggleTheme,
}: ProjectHeaderProps) {
  return (
    <div className="flex items-center justify-between px-4 py-3">
      <div className="flex items-center gap-3">
        <span className="text-lg">{project.icon || "📋"}</span>
        <h1 className="text-sm font-semibold tracking-tight">{project.name}</h1>

        <div className="ml-3 flex items-center rounded-lg bg-muted/50 p-0.5">
          {([
            { mode: "board" as const, icon: LayoutGrid, label: "Board" },
            { mode: "list" as const, icon: List, label: "List" },
            { mode: "tree" as const, icon: TreePine, label: "Tree" },
            { mode: "calendar" as const, icon: Calendar, label: "Calendar" },
            { mode: "gantt" as const, icon: GanttChart, label: "Gantt" },
          ]).map(({ mode, icon: Icon, label }) => (
            <button
              key={mode}
              onClick={() => onViewModeChange(mode)}
              className={cn(
                "flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition-all",
                viewMode === mode
                  ? "bg-card text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              <Icon className="h-3.5 w-3.5" /> {label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-1">
        <Tooltip content="Toggle theme">
          <Button variant="ghost" size="icon-sm" onClick={onToggleTheme}>
            {theme === "dark" ? (
              <Sun className="h-4 w-4 text-muted-foreground" />
            ) : (
              <Moon className="h-4 w-4 text-muted-foreground" />
            )}
          </Button>
        </Tooltip>

        <Tooltip content="Notifications">
          <Button variant="ghost" size="icon-sm" className="relative" onClick={onOpenNotifications}>
            <Bell className="h-4 w-4 text-muted-foreground" />
            {notificationCount > 0 && (
              <span className="absolute -right-0.5 -top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-red-500 text-[9px] font-bold text-white">
                {notificationCount > 9 ? "9+" : notificationCount}
              </span>
            )}
          </Button>
        </Tooltip>

        <Tooltip content="Search (Cmd+K)">
          <Button variant="ghost" size="icon-sm" onClick={onSearch}>
            <Search className="h-4 w-4 text-muted-foreground" />
          </Button>
        </Tooltip>

        <Button size="sm" onClick={onCreateIssue} className="ml-1.5 rounded-lg">
          <Plus className="h-3.5 w-3.5 mr-1" /> Issue
        </Button>
      </div>
    </div>
  );
}
