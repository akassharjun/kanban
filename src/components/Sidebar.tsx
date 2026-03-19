import { useState } from "react";
import { Plus, Users, Settings, ChevronDown, FolderKanban, Bot, Star, Clock, Bookmark, MoreHorizontal, Pencil, Trash2, Flame, GitBranch, Terminal } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Project, Issue, SavedView } from "@/types";

interface SidebarProps {
  projects: Project[];
  selectedProjectId: number | null;
  activePage?: string;
  onSelectProject: (id: number) => void;
  onCreateProject: () => void;
  onOpenMembers: () => void;
  onOpenSettings: () => void;
  onOpenAgents?: () => void;
  onOpenCode?: () => void;
  onOpenPipelines?: () => void;
  onToggleTerminal?: () => void;
  terminalOpen?: boolean;
  agentCount?: number;
  collapsed: boolean;
  starredIssues?: Issue[];
  recentIssues?: Issue[];
  savedViews?: SavedView[];
  onClickIssue?: (issue: Issue) => void;
  onSelectSavedView?: (view: SavedView) => void;
  onDeleteSavedView?: (id: number) => void;
  onRenameSavedView?: (id: number, name: string) => void;
}

const navItemBase = "flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] transition-colors";
const navItemActive = "bg-primary/10 text-primary font-medium";
const navItemInactive = "text-muted-foreground hover:bg-muted hover:text-foreground";

export function Sidebar({
  projects,
  selectedProjectId,
  activePage,
  onSelectProject,
  onCreateProject,
  onOpenMembers,
  onOpenSettings,
  onOpenAgents,
  onOpenCode,
  onOpenPipelines,
  onToggleTerminal,
  terminalOpen,
  agentCount,
  collapsed,
  starredIssues = [],
  recentIssues = [],
  savedViews = [],
  onClickIssue,
  onSelectSavedView,
  onDeleteSavedView,
  onRenameSavedView,
}: SidebarProps) {
  const [projectsExpanded, setProjectsExpanded] = useState(true);
  const [starredExpanded, setStarredExpanded] = useState(true);
  const [recentExpanded, setRecentExpanded] = useState(false);
  const [savedViewsExpanded, setSavedViewsExpanded] = useState(true);
  const [editingViewId, setEditingViewId] = useState<number | null>(null);
  const [editingViewName, setEditingViewName] = useState("");
  const [viewMenuId, setViewMenuId] = useState<number | null>(null);

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
                  navItemBase, "font-medium",
                  selectedProjectId === project.id && activePage === "project"
                    ? navItemActive
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                )}
              >
                <span className="text-base leading-none">{project.icon || "📋"}</span>
                <span className="truncate">{project.name}</span>
              </button>
            ))}
            <button
              onClick={onCreateProject}
              className={cn(navItemBase, "text-muted-foreground/60 hover:bg-muted hover:text-foreground")}
            >
              <Plus className="h-4 w-4" />
              <span>New project</span>
            </button>
          </div>
        )}
      </div>

      {/* Saved Views */}
      {savedViews.length > 0 && (
        <div className="px-3 py-1">
          <button
            onClick={() => setSavedViewsExpanded(!savedViewsExpanded)}
            className="flex w-full items-center gap-1.5 px-1 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70 hover:text-muted-foreground"
          >
            <ChevronDown className={cn("h-3 w-3 transition-transform", !savedViewsExpanded && "-rotate-90")} />
            <Bookmark className="h-3 w-3" />
            Saved Views
          </button>
          {savedViewsExpanded && (
            <div className="mt-0.5 space-y-0.5">
              {savedViews.map(view => (
                <div key={view.id} className="group relative">
                  {editingViewId === view.id ? (
                    <input
                      autoFocus
                      value={editingViewName}
                      onChange={(e) => setEditingViewName(e.target.value)}
                      onBlur={() => {
                        if (editingViewName.trim() && onRenameSavedView) {
                          onRenameSavedView(view.id, editingViewName.trim());
                        }
                        setEditingViewId(null);
                      }}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          if (editingViewName.trim() && onRenameSavedView) {
                            onRenameSavedView(view.id, editingViewName.trim());
                          }
                          setEditingViewId(null);
                        }
                        if (e.key === "Escape") setEditingViewId(null);
                      }}
                      className="w-full rounded-lg bg-muted px-2.5 py-2 text-[13px] outline-none"
                    />
                  ) : (
                    <button
                      onClick={() => onSelectSavedView?.(view)}
                      className={cn(navItemBase, "text-muted-foreground hover:bg-muted hover:text-foreground pr-8")}
                    >
                      <Bookmark className="h-3.5 w-3.5 flex-shrink-0" />
                      <span className="truncate">{view.name}</span>
                    </button>
                  )}
                  {editingViewId !== view.id && (
                    <div className="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={(e) => { e.stopPropagation(); setViewMenuId(viewMenuId === view.id ? null : view.id); }}
                        className="rounded p-1 hover:bg-muted"
                      >
                        <MoreHorizontal className="h-3 w-3 text-muted-foreground/50" />
                      </button>
                      {viewMenuId === view.id && (
                        <div className="absolute right-0 top-full mt-0.5 z-50 w-32 rounded-lg border border-border bg-popover p-1 shadow-lg">
                          <button
                            onClick={() => { setEditingViewId(view.id); setEditingViewName(view.name); setViewMenuId(null); }}
                            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-[12px] hover:bg-muted"
                          >
                            <Pencil className="h-3 w-3" /> Rename
                          </button>
                          <button
                            onClick={() => { onDeleteSavedView?.(view.id); setViewMenuId(null); }}
                            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-[12px] text-red-500 hover:bg-red-500/10"
                          >
                            <Trash2 className="h-3 w-3" /> Delete
                          </button>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Starred Issues */}
      {starredIssues.length > 0 && (
        <div className="px-3 py-1">
          <button
            onClick={() => setStarredExpanded(!starredExpanded)}
            className="flex w-full items-center gap-1.5 px-1 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70 hover:text-muted-foreground"
          >
            <ChevronDown className={cn("h-3 w-3 transition-transform", !starredExpanded && "-rotate-90")} />
            <Star className="h-3 w-3" />
            Starred
          </button>
          {starredExpanded && (
            <div className="mt-0.5 space-y-0.5">
              {starredIssues.slice(0, 8).map(issue => (
                <button
                  key={issue.id}
                  onClick={() => onClickIssue?.(issue)}
                  className={cn(navItemBase, "text-muted-foreground hover:bg-muted hover:text-foreground")}
                >
                  <span className="text-[10px] font-mono text-muted-foreground/50 shrink-0">{issue.identifier}</span>
                  <span className="truncate text-[12px]">{issue.title}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Recently Viewed */}
      {recentIssues.length > 0 && (
        <div className="px-3 py-1">
          <button
            onClick={() => setRecentExpanded(!recentExpanded)}
            className="flex w-full items-center gap-1.5 px-1 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70 hover:text-muted-foreground"
          >
            <ChevronDown className={cn("h-3 w-3 transition-transform", !recentExpanded && "-rotate-90")} />
            <Clock className="h-3 w-3" />
            Recent
          </button>
          {recentExpanded && (
            <div className="mt-0.5 space-y-0.5">
              {recentIssues.slice(0, 10).map(issue => (
                <button
                  key={issue.id}
                  onClick={() => onClickIssue?.(issue)}
                  className={cn(navItemBase, "text-muted-foreground hover:bg-muted hover:text-foreground")}
                >
                  <span className="text-[10px] font-mono text-muted-foreground/50 shrink-0">{issue.identifier}</span>
                  <span className="truncate text-[12px]">{issue.title}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Bottom links */}
      <div className="border-t border-border/50 p-3 space-y-0.5">
        <button
          onClick={onOpenMembers}
          className={cn(navItemBase, activePage === "members" ? navItemActive : navItemInactive)}
        >
          <Users className="h-4 w-4" />
          Members
        </button>
        <button
          onClick={onOpenAgents}
          className={cn(navItemBase, activePage === "agents" ? navItemActive : navItemInactive)}
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
          onClick={onOpenCode}
          className={cn(navItemBase, activePage === "code" ? navItemActive : navItemInactive)}
        >
          <Flame className="h-4 w-4" />
          Code Heat Map
        </button>
        <button
          onClick={onOpenPipelines}
          className={cn(navItemBase, activePage === "pipelines" ? navItemActive : navItemInactive)}
        >
          <GitBranch className="h-4 w-4" />
          Pipelines
        </button>
        <button
          onClick={onOpenSettings}
          className={cn(navItemBase, activePage === "settings" ? navItemActive : navItemInactive)}
        >
          <Settings className="h-4 w-4" />
          Settings
        </button>
        {onToggleTerminal && (
          <button
            onClick={onToggleTerminal}
            className={cn(navItemBase, terminalOpen ? navItemActive : navItemInactive)}
          >
            <Terminal className="h-4 w-4" />
            Terminal
          </button>
        )}
      </div>
    </div>
  );
}
