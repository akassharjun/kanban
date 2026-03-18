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
    <div className="flex h-full w-56 flex-col border-r border-border/50 bg-sidebar">
      {/* App header */}
      <div className="flex items-center gap-2.5 px-4 py-4">
        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <FolderKanban className="h-4 w-4" />
        </div>
        <span className="font-semibold text-sm tracking-tight">Kanban</span>
      </div>

      {/* Projects section */}
      <div className="flex-1 overflow-y-auto px-3 py-1">
        <button
          onClick={() => setProjectsExpanded(!projectsExpanded)}
          className="flex w-full items-center gap-1.5 px-1 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70 hover:text-muted-foreground"
        >
          <ChevronDown className={cn("h-3 w-3 transition-transform", !projectsExpanded && "-rotate-90")} />
          Projects
        </button>

        {projectsExpanded && (
          <div className="mt-0.5 space-y-0.5">
            {projects.map((project) => (
              <button
                key={project.id}
                onClick={() => onSelectProject(project.id)}
                className={cn(
                  "flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] font-medium transition-all",
                  selectedProjectId === project.id
                    ? "bg-primary/10 text-primary shadow-sm"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                )}
              >
                <span className="text-base leading-none">{project.icon || "📋"}</span>
                <span className="truncate">{project.name}</span>
              </button>
            ))}
            <button
              onClick={onCreateProject}
              className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] text-muted-foreground/60 hover:bg-muted hover:text-foreground transition-colors"
            >
              <Plus className="h-4 w-4" />
              <span>New project</span>
            </button>
          </div>
        )}
      </div>

      {/* Bottom links */}
      <div className="border-t border-border/50 p-3 space-y-0.5">
        <button
          onClick={onOpenMembers}
          className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Users className="h-4 w-4" />
          Members
        </button>
        <button
          onClick={onOpenAgents}
          className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Bot className="h-4 w-4" />
          Agent Ops
          {agentCount != null && agentCount > 0 && (
            <span className="ml-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-primary/15 text-primary text-[10px] font-semibold px-1.5">
              {agentCount}
            </span>
          )}
        </button>
        <button
          onClick={onOpenSettings}
          className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Settings className="h-4 w-4" />
          Settings
        </button>
      </div>
    </div>
  );
}
