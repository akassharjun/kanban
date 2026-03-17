import { useState } from "react";
import { Plus, Users, Settings, ChevronDown, FolderKanban, Bot } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Project } from "@/types";

interface SidebarProps {
  projects: Project[];
  selectedProjectId: number | null;
  onSelectProject: (id: number) => void;
  onCreateProject: () => void;
  onOpenMembers: () => void;
  onOpenSettings: () => void;
  onOpenAgents?: () => void;
  agentCount?: number;
  collapsed: boolean;
}

export function Sidebar({
  projects,
  selectedProjectId,
  onSelectProject,
  onCreateProject,
  onOpenMembers,
  onOpenSettings,
  onOpenAgents,
  agentCount,
  collapsed,
}: SidebarProps) {
  const [projectsExpanded, setProjectsExpanded] = useState(true);

  if (collapsed) return null;

  return (
    <div className="flex h-full w-60 flex-col border-r border-border bg-card">
      {/* App header */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-border">
        <FolderKanban className="h-5 w-5 text-primary" />
        <span className="font-semibold text-sm">Kanban</span>
      </div>

      {/* Projects section */}
      <div className="flex-1 overflow-y-auto py-2">
        <button
          onClick={() => setProjectsExpanded(!projectsExpanded)}
          className="flex w-full items-center gap-1 px-4 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground"
        >
          <ChevronDown className={cn("h-3 w-3 transition-transform", !projectsExpanded && "-rotate-90")} />
          Projects
        </button>

        {projectsExpanded && (
          <div className="space-y-0.5 px-2">
            {projects.map((project) => (
              <button
                key={project.id}
                onClick={() => onSelectProject(project.id)}
                className={cn(
                  "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
                  selectedProjectId === project.id
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                )}
              >
                <span className="text-base">{project.icon || "📋"}</span>
                <span className="truncate">{project.name}</span>
              </button>
            ))}
            <button
              onClick={onCreateProject}
              className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground"
            >
              <Plus className="h-4 w-4" />
              <span>New project</span>
            </button>
          </div>
        )}
      </div>

      {/* Bottom links */}
      <div className="border-t border-border p-2 space-y-0.5">
        <button
          onClick={onOpenMembers}
          className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground"
        >
          <Users className="h-4 w-4" />
          Members
        </button>
        <button
          onClick={onOpenAgents}
          className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground"
        >
          <Bot className="h-4 w-4" />
          Agent Ops
          {agentCount != null && agentCount > 0 && (
            <span className="ml-auto rounded-full bg-amber-500/20 text-amber-500 text-[10px] font-mono px-1.5 py-0.5 leading-none">
              {agentCount}
            </span>
          )}
        </button>
        <button
          onClick={onOpenSettings}
          className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground"
        >
          <Settings className="h-4 w-4" />
          Settings
        </button>
      </div>
    </div>
  );
}
