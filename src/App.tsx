import { useState, useEffect, useMemo } from "react";
import { listen } from "./tauri/events";
import { Sidebar } from "./components/Sidebar";
import { ProjectHeader, type ViewMode } from "./components/ProjectHeader";
import { Button } from "./components/ui/button";
import { BoardView } from "./components/BoardView";
import { ListView } from "./components/ListView";
import { TreeView } from "./components/TreeView";
import { IssueDetailPanel } from "./components/IssueDetailPanel";
import { FilterBar, type Filters } from "./components/FilterBar";
import { CreateProjectDialog } from "./components/CreateProjectDialog";
import { CreateIssueDialog } from "./components/CreateIssueDialog";
import { MembersView } from "./components/MembersView";
import { SearchDialog } from "./components/SearchDialog";
import { NotificationsPanel } from "./components/NotificationsPanel";
import { ProjectSettingsView } from "./components/ProjectSettingsView";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AgentDashboard } from "@/components/AgentDashboard";
import { ReplayViewer } from "@/components/ReplayViewer";
import { useProjects } from "./hooks/use-projects";
import { useIssues } from "./hooks/use-issues";
import { useMembers } from "./hooks/use-members";
import { useStatuses } from "./hooks/use-statuses";
import { useLabels } from "./hooks/use-labels";
import { useIssueLabelMap } from "./hooks/use-issue-labels";
import { useAgents } from "./hooks/use-agents";
import * as api from "./tauri/commands";
import type { IssueTemplate } from "./types";

type Page = "project" | "members" | "settings" | "agents";

function App() {
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(null);
  const [page, setPage] = useState<Page>("project");
  const [viewMode, setViewMode] = useState<ViewMode>("board");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [selectedIssueId, setSelectedIssueId] = useState<number | null>(null);
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [showCreateIssue, setShowCreateIssue] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const [notificationCount, setNotificationCount] = useState(0);
  const [templates, setTemplates] = useState<IssueTemplate[]>([]);
  const [createIssueStatusId, setCreateIssueStatusId] = useState<number | undefined>();
  const [filters, setFilters] = useState<Filters>({});
  const [replayIdentifier, setReplayIdentifier] = useState<string | null>(null);

  const [theme, setTheme] = useState<"dark" | "light">(() => {
    return document.documentElement.classList.contains("dark") ? "dark" : "light";
  });

  const toggleTheme = () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    document.documentElement.classList.toggle("dark", next === "dark");
    document.documentElement.classList.toggle("light", next === "light");
  };

  const { projects, refresh: refreshProjects, create: createProject, update: updateProject, remove: _removeProject } = useProjects();
  const { issues, refresh: refreshIssues, create: createIssue, update: updateIssue, remove: deleteIssue, duplicate: duplicateIssue } = useIssues(selectedProjectId);
  const { members, create: createMember, update: updateMember, remove: deleteMember } = useMembers();
  const { statuses, refresh: refreshStatuses } = useStatuses(selectedProjectId);
  const { labels, refresh: refreshLabels } = useLabels(selectedProjectId);
  const { getLabelsForIssue, refresh: refreshIssueLabelMap } = useIssueLabelMap(selectedProjectId, labels);
  const { agents: allAgents } = useAgents();
  const onlineAgentCount = allAgents.filter(a => a.status !== "offline").length;

  // Auto-select first project
  useEffect(() => {
    if (!selectedProjectId && projects.length > 0) {
      setSelectedProjectId(projects[0].id);
    }
  }, [projects, selectedProjectId]);

  // Load templates when project changes
  useEffect(() => {
    if (selectedProjectId) {
      api.listTemplates(selectedProjectId).then(setTemplates).catch(() => {});
    }
  }, [selectedProjectId]);

  // Poll notification count
  useEffect(() => {
    const poll = () => { api.unreadNotificationCount().then(setNotificationCount).catch(() => {}); };
    poll();
    const interval = setInterval(poll, 10000);
    return () => clearInterval(interval);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Ignore when typing in inputs
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") return;

      if (e.key === "c" && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        setShowCreateIssue(true);
      }
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setShowSearch(true);
      }
      if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setSidebarCollapsed(prev => !prev);
      }
      if (e.key === "1" && !e.metaKey) {
        setViewMode("board");
      }
      if (e.key === "2" && !e.metaKey) {
        setViewMode("list");
      }
      if (e.key === "3" && !e.metaKey) {
        setViewMode("tree");
      }
      if (e.key === "4" && !e.metaKey) {
        setPage("agents");
      }
      if (e.key === "Escape") {
        if (selectedIssueId) setSelectedIssueId(null);
        else if (showSearch) setShowSearch(false);
        else if (showCreateIssue) setShowCreateIssue(false);
        else if (showCreateProject) setShowCreateProject(false);
        else if (showNotifications) setShowNotifications(false);
      }
      // Undo/Redo
      if (e.key === "z" && (e.metaKey || e.ctrlKey) && !e.shiftKey) {
        e.preventDefault();
        api.undo().then(async () => { await Promise.all([refreshIssues(), refreshStatuses(), refreshProjects(), refreshLabels()]); });
      }
      if (e.key === "z" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
        e.preventDefault();
        api.redo().then(async () => { await Promise.all([refreshIssues(), refreshStatuses(), refreshProjects(), refreshLabels()]); });
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedIssueId, showSearch, showCreateIssue, showCreateProject, showNotifications, refreshIssues, refreshStatuses, refreshProjects]);

  // Listen for external DB changes (CLI/MCP writes) and refresh all data
  useEffect(() => {
    const unlisten = listen("db-changed", () => {
      refreshProjects();
      refreshIssues();
      refreshStatuses();
      refreshLabels();
      refreshIssueLabelMap();
    });
    return () => { unlisten.then(fn => fn()); };
  }, [refreshProjects, refreshIssues, refreshStatuses, refreshLabels, refreshIssueLabelMap]);

  const handleQuickCreate = async (statusId: number, title: string) => {
    if (!selectedProjectId) return;
    await createIssue({
      project_id: selectedProjectId,
      title,
      status_id: statusId,
    });
  };

  const handleSelectProject = (id: number) => {
    setSelectedProjectId(id);
    setPage("project");
    setSelectedIssueId(null);
    setFilters({});
  };

  const filteredIssues = useMemo(() => {
    return issues.filter(issue => {
      if (filters.status_id && issue.status_id !== filters.status_id) return false;
      if (filters.priority && issue.priority !== filters.priority) return false;
      if (filters.assignee_id && issue.assignee_id !== filters.assignee_id) return false;
      return true;
    });
  }, [issues, filters]);

  const selectedProject = projects.find(p => p.id === selectedProjectId);

  // Compute "stale soon" issues - within 7 days of being auto-closed
  const staleSoonIssueIds = useMemo(() => {
    const ids = new Set<number>();
    if (!selectedProject?.stale_days) return ids;
    const staleDays = selectedProject.stale_days;
    const warningDays = staleDays - 7; // warn 7 days before
    if (warningDays < 0) return ids;
    const unstartedStatuses = new Set(
      statuses.filter(s => s.category === "unstarted").map(s => s.id)
    );
    const now = Date.now();
    for (const issue of issues) {
      if (!unstartedStatuses.has(issue.status_id)) continue;
      const updatedAt = new Date(issue.updated_at).getTime();
      const daysSinceUpdate = (now - updatedAt) / (1000 * 60 * 60 * 24);
      if (daysSinceUpdate >= warningDays) {
        ids.add(issue.id);
      }
    }
    return ids;
  }, [issues, statuses, selectedProject]);

  return (
    <ErrorBoundary>
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        projects={projects}
        selectedProjectId={selectedProjectId}
        activePage={page}
        onSelectProject={handleSelectProject}
        onCreateProject={() => setShowCreateProject(true)}
        onOpenMembers={() => { setPage("members"); setSelectedIssueId(null); }}
        onOpenSettings={() => setPage("settings")}
        onOpenAgents={() => setPage("agents")}
        agentCount={onlineAgentCount}
        collapsed={sidebarCollapsed}
      />

      <div className="flex flex-1 flex-col overflow-hidden">
        {page === "project" && selectedProject && (
          <>
            <ProjectHeader
              project={selectedProject}
              viewMode={viewMode}
              onViewModeChange={setViewMode}
              onSearch={() => setShowSearch(true)}
              onCreateIssue={() => { setCreateIssueStatusId(undefined); setShowCreateIssue(true); }}
              notificationCount={notificationCount}
              onOpenNotifications={() => setShowNotifications(true)}
              theme={theme}
              onToggleTheme={toggleTheme}
            />

            <FilterBar
              statuses={statuses}
              members={members}
              labels={labels}
              filters={filters}
              onFiltersChange={setFilters}
            />

            <div className="flex flex-1 overflow-hidden">
              {viewMode === "board" && (
                <BoardView
                  issues={filteredIssues}
                  allIssues={issues}
                  statuses={statuses}
                  members={members}
                  labels={labels}
                  getLabelsForIssue={getLabelsForIssue}
                  staleSoonIssueIds={staleSoonIssueIds}
                  onUpdateIssue={updateIssue}
                  onClickIssue={(issue) => setSelectedIssueId(issue.id)}
                  onQuickCreate={handleQuickCreate}
                />
              )}
              {viewMode === "list" && (
                <ListView
                  issues={filteredIssues}
                  statuses={statuses}
                  members={members}
                  labels={labels}
                  onClickIssue={(issue) => setSelectedIssueId(issue.id)}
                />
              )}
              {viewMode === "tree" && (
                <TreeView
                  issues={filteredIssues}
                  statuses={statuses}
                  members={members}
                  onClickIssue={(issue) => setSelectedIssueId(issue.id)}
                />
              )}

            </div>
          </>
        )}

        {page === "project" && !selectedProject && (
          <div className="flex flex-1 items-center justify-center text-muted-foreground">
            <div className="text-center">
              <p className="text-lg">No project selected</p>
              <Button className="mt-2" onClick={() => setShowCreateProject(true)}>
                Create your first project
              </Button>
            </div>
          </div>
        )}

        {page === "members" && (
          <MembersView
            members={members}
            onCreate={createMember}
            onUpdate={updateMember}
            onDelete={deleteMember}
          />
        )}

        {page === "settings" && selectedProject && (
          <ProjectSettingsView
            project={selectedProject}
            onUpdateProject={updateProject}
            onRefreshStatuses={refreshStatuses}
            onRefreshLabels={refreshLabels}
            onDeleteProject={async (id) => {
              await _removeProject(id);
              setSelectedProjectId(null);
              setPage("project");
            }}
          />
        )}

        {page === "settings" && !selectedProject && (
          <div className="flex flex-1 items-center justify-center text-muted-foreground">
            Select a project to view settings
          </div>
        )}

        {page === "agents" && (
          <AgentDashboard projectId={selectedProjectId} projectName={selectedProject?.name ?? null} projectPrefix={selectedProject?.prefix ?? null} onViewReplay={(id) => setReplayIdentifier(id)} />
        )}
      </div>

      {selectedIssueId && (
        <IssueDetailPanel
          issueId={selectedIssueId}
          statuses={statuses}
          members={members}
          projectLabels={labels}
          onClose={() => setSelectedIssueId(null)}
          onUpdate={async (id, input) => { await updateIssue(id, input); }}
          onDelete={async (id) => { await deleteIssue(id); setSelectedIssueId(null); }}
          onDuplicate={async (id) => { await duplicateIssue(id); }}
          onClickIssue={(issue) => setSelectedIssueId(issue.id)}
        />
      )}

      {/* Modals */}
      {showCreateProject && (
        <CreateProjectDialog
          onClose={() => setShowCreateProject(false)}
          onCreate={createProject}
        />
      )}

      {showCreateIssue && selectedProjectId && (
        <CreateIssueDialog
          projectId={selectedProjectId}
          statuses={statuses}
          members={members}
          labels={labels}
          templates={templates}
          defaultStatusId={createIssueStatusId || statuses.find(s => s.category === "unstarted")?.id}
          onClose={() => setShowCreateIssue(false)}
          onCreate={createIssue}
        />
      )}

      {showSearch && (
        <SearchDialog
          projects={projects}
          currentProjectId={selectedProjectId}
          onClose={() => setShowSearch(false)}
          onSelectIssue={(issue) => setSelectedIssueId(issue.id)}
          onSelectProject={handleSelectProject}
        />
      )}

      {showNotifications && <NotificationsPanel onClose={() => setShowNotifications(false)} />}

      {replayIdentifier && (
        <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-8">
          <div className="w-full max-w-4xl h-full max-h-[90vh] rounded-xl overflow-hidden">
            <ReplayViewer identifier={replayIdentifier} onClose={() => setReplayIdentifier(null)} />
          </div>
        </div>
      )}
    </div>
    </ErrorBoundary>
  );
}

export default App;
