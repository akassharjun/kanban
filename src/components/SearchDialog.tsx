import { useState, useEffect, useRef } from "react";
import { Search } from "lucide-react";
import type { Issue, Project } from "@/types";
import * as api from "@/tauri/commands";

interface SearchDialogProps {
  projects: Project[];
  currentProjectId: number | null;
  onClose: () => void;
  onSelectIssue: (issue: Issue) => void;
  onSelectProject: (id: number) => void;
}

export function SearchDialog({ projects, currentProjectId, onClose, onSelectIssue, onSelectProject }: SearchDialogProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Issue[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    if (!query.trim()) { setResults([]); return; }
    const timeout = setTimeout(async () => {
      if (currentProjectId) {
        const data = await api.searchIssues(currentProjectId, query);
        setResults(data);
      }
    }, 200);
    return () => clearTimeout(timeout);
  }, [query, currentProjectId]);

  const filteredProjects = projects.filter(p =>
    p.name.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh] bg-black/50" onClick={onClose}>
      <div className="w-[560px] rounded-lg border border-border bg-card shadow-xl" onClick={e => e.stopPropagation()}>
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <Search className="h-4 w-4 text-muted-foreground" />
          <input
            ref={inputRef}
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder="Search issues and projects..."
            className="flex-1 bg-transparent text-sm outline-none"
            onKeyDown={e => { if (e.key === "Escape") onClose(); }}
          />
        </div>

        <div className="max-h-[400px] overflow-y-auto p-2">
          {!query.trim() && (
            <div className="px-3 py-6 text-center text-sm text-muted-foreground">
              Type to search issues and projects
            </div>
          )}

          {query.trim() && filteredProjects.length > 0 && (
            <>
              <div className="px-3 py-1 text-xs font-medium text-muted-foreground">Projects</div>
              {filteredProjects.map(p => (
                <button
                  key={p.id}
                  onClick={() => { onSelectProject(p.id); onClose(); }}
                  className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-accent"
                >
                  <span>{p.icon || "📋"}</span>
                  <span>{p.name}</span>
                </button>
              ))}
            </>
          )}

          {results.length > 0 && (
            <>
              <div className="px-3 py-1 text-xs font-medium text-muted-foreground mt-2">Issues</div>
              {results.map(issue => (
                <button
                  key={issue.id}
                  onClick={() => { onSelectIssue(issue); onClose(); }}
                  className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-accent"
                >
                  <span className="text-muted-foreground">{issue.identifier}</span>
                  <span className="truncate">{issue.title}</span>
                </button>
              ))}
            </>
          )}

          {query.trim() && results.length === 0 && filteredProjects.length === 0 && (
            <div className="px-3 py-6 text-center text-sm text-muted-foreground">No results found</div>
          )}
        </div>
      </div>
    </div>
  );
}
