import { LayoutGrid, List, TreePine, Search, Plus, Bell, Sun, Moon } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import type { Project } from "@/types";

export type ViewMode = "board" | "list" | "tree";

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
    <div className="flex items-center justify-between border-b border-border px-4 py-2">
      <div className="flex items-center gap-3">
        <span className="text-base">{project.icon || "📋"}</span>
        <h1 className="text-sm font-semibold">{project.name}</h1>

        <div className="ml-4 flex items-center rounded-md border border-border">
          <button
            onClick={() => onViewModeChange("board")}
            className={cn(
              "flex items-center gap-1 px-2.5 py-1 text-xs transition-colors rounded-l-md",
              viewMode === "board" ? "bg-accent text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <LayoutGrid className="h-3.5 w-3.5" /> Board
          </button>
          <button
            onClick={() => onViewModeChange("list")}
            className={cn(
              "flex items-center gap-1 px-2.5 py-1 text-xs transition-colors",
              viewMode === "list" ? "bg-accent text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <List className="h-3.5 w-3.5" /> List
          </button>
          <button
            onClick={() => onViewModeChange("tree")}
            className={cn(
              "flex items-center gap-1 px-2.5 py-1 text-xs transition-colors rounded-r-md",
              viewMode === "tree" ? "bg-accent text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <TreePine className="h-3.5 w-3.5" /> Tree
          </button>
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

        <Button size="sm" onClick={onCreateIssue} className="ml-1">
          <Plus className="h-3.5 w-3.5 mr-1" /> Issue
        </Button>
      </div>
    </div>
  );
}
