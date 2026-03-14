import { useState, useEffect } from "react";
import { Sidebar } from "./components/Sidebar";
import { ProjectHeader, type ViewMode } from "./components/ProjectHeader";
import { BoardView } from "./components/BoardView";
import { ListView } from "./components/ListView";
import { TreeView } from "./components/TreeView";
import { IssueDetailPanel } from "./components/IssueDetailPanel";
import { CreateProjectDialog } from "./components/CreateProjectDialog";
import { CreateIssueDialog } from "./components/CreateIssueDialog";
import { MembersView } from "./components/MembersView";
import { SearchDialog } from "./components/SearchDialog";
import { NotificationsPanel } from "./components/NotificationsPanel";
import { ProjectSettingsView } from "./components/ProjectSettingsView";
import { useProjects } from "./hooks/use-projects";
import { useIssues } from "./hooks/use-issues";
import { useMembers } from "./hooks/use-members";
import { useStatuses } from "./hooks/use-statuses";
import { useLabels } from "./hooks/use-labels";
import * as api from "./tauri/commands";
import type { IssueTemplate } from "./types";

type Page = "project" | "members" | "settings";

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
        api.undo().then(() => { refreshIssues(); refreshStatuses(); refreshProjects(); });
      }
      if (e.key === "z" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
        e.preventDefault();
        api.redo().then(() => { refreshIssues(); refreshStatuses(); refreshProjects(); });
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedIssueId, showSearch, showCreateIssue, showCreateProject, showNotifications, refreshIssues, refreshStatuses, refreshProjects]);

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
  };

  const selectedProject = projects.find(p => p.id === selectedProjectId);

  return (
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        projects={projects}
        selectedProjectId={selectedProjectId}
        onSelectProject={handleSelectProject}
        onCreateProject={() => setShowCreateProject(true)}
        onOpenMembers={() => { setPage("members"); setSelectedIssueId(null); }}
        onOpenSettings={() => setPage("settings")}
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

            <div className="flex flex-1 overflow-hidden">
              {viewMode === "board" && (
                <BoardView
                  issues={issues}
                  statuses={statuses}
                  members={members}
                  labels={labels}
                  onUpdateIssue={updateIssue}
                  onClickIssue={(issue) => setSelectedIssueId(issue.id)}
                  onQuickCreate={handleQuickCreate}
                />
              )}
              {viewMode === "list" && (
                <ListView
                  issues={issues}
                  statuses={statuses}
                  members={members}
                  labels={labels}
                  onClickIssue={(issue) => setSelectedIssueId(issue.id)}
                />
              )}
              {viewMode === "tree" && (
                <TreeView
                  issues={issues}
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
              <button
                onClick={() => setShowCreateProject(true)}
                className="mt-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Create your first project
              </button>
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
          />
        )}

        {page === "settings" && !selectedProject && (
          <div className="flex flex-1 items-center justify-center text-muted-foreground">
            Select a project to view settings
          </div>
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
    </div>
  );
}

export default App;
