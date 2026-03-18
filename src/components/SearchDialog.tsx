import { useState, useEffect, useRef } from "react";
import { Search, ArrowRight } from "lucide-react";
import { DialogOverlay } from "@/components/ui/dialog";
import type { Issue, Project } from "@/types";
import * as api from "@/tauri/commands";

interface SearchDialogProps {
  projects: Project[];
  currentProjectId: number | null;
  onClose: () => void;
  onSelectIssue: (issue: Issue) => void;
  onSelectProject: (id: number) => void;
}

export function SearchDialog({ projects, currentProjectId: _currentProjectId, onClose, onSelectIssue, onSelectProject }: SearchDialogProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Issue[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    if (!query.trim()) { setResults([]); return; }
    const timeout = setTimeout(async () => {
      try {
        const settled = await Promise.allSettled(
          projects.map(p => api.searchIssues(p.id, query))
        );
        const successfulResults = settled
          .filter((r): r is PromiseFulfilledResult<Issue[]> => r.status === "fulfilled")
          .flatMap(r => r.value);
        setResults(successfulResults);
        setSelectedIndex(0);
      } catch (e) {
        console.error("Search failed", e);
      }
    }, 200);
    return () => clearTimeout(timeout);
  }, [query, projects]);

  const filteredProjects = projects.filter(p =>
    p.name.toLowerCase().includes(query.toLowerCase())
  );

  // Build flat list of selectable items for keyboard nav
  const allItems: { type: "project" | "issue"; id: number; project?: Project; issue?: Issue }[] = [];
  if (query.trim()) {
    filteredProjects.forEach(p => allItems.push({ type: "project", id: p.id, project: p }));
    results.forEach(i => allItems.push({ type: "issue", id: i.id, issue: i }));
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIndex(i => Math.min(i + 1, allItems.length - 1)); }
    if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIndex(i => Math.max(i - 1, 0)); }
    if (e.key === "Enter" && allItems[selectedIndex]) {
      const item = allItems[selectedIndex];
      if (item.type === "project" && item.project) { onSelectProject(item.project.id); onClose(); }
      if (item.type === "issue" && item.issue) { onSelectIssue(item.issue); onClose(); }
    }
  };

  let itemIndex = -1;

  return (
    <DialogOverlay className="items-start pt-[18vh]" onClose={onClose}>
      <div className="w-[560px] rounded-xl border border-border/50 bg-card shadow-2xl" onClick={e => e.stopPropagation()}>
        <div className="flex items-center gap-3 px-4 py-3.5">
          <Search className="h-4 w-4 text-muted-foreground/50" />
          <input
            ref={inputRef}
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder="Search issues and projects..."
            className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/40"
            onKeyDown={handleKeyDown}
          />
          <kbd className="hidden sm:inline-flex h-5 items-center rounded border border-border/50 bg-muted px-1.5 text-[10px] font-mono text-muted-foreground/50">
            ESC
          </kbd>
        </div>

        <div className="border-t border-border/50" />

        <div className="max-h-[400px] overflow-y-auto p-1.5">
          {!query.trim() && (
            <div className="px-3 py-8 text-center text-sm text-muted-foreground/40">
              Type to search issues and projects
            </div>
          )}

          {query.trim() && filteredProjects.length > 0 && (
            <>
              <div className="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/40">Projects</div>
              {filteredProjects.map(p => {
                itemIndex++;
                const idx = itemIndex;
                return (
                  <button
                    key={p.id}
                    onClick={() => { onSelectProject(p.id); onClose(); }}
                    className={`flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-colors ${idx === selectedIndex ? "bg-primary/10 text-primary" : "hover:bg-muted"}`}
                  >
                    <span className="text-base">{p.icon || "📋"}</span>
                    <span className="font-medium">{p.name}</span>
                    <ArrowRight className="ml-auto h-3.5 w-3.5 text-muted-foreground/30" />
                  </button>
                );
              })}
            </>
          )}

          {results.length > 0 && (
            <>
              <div className="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/40 mt-1">Issues</div>
              {results.map(issue => {
                itemIndex++;
                const idx = itemIndex;
                return (
                  <button
                    key={issue.id}
                    onClick={() => { onSelectIssue(issue); onClose(); }}
                    className={`flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-colors ${idx === selectedIndex ? "bg-primary/10 text-primary" : "hover:bg-muted"}`}
                  >
                    <span className="text-xs font-mono text-muted-foreground/50">{issue.identifier}</span>
                    <span className="truncate">{issue.title}</span>
                  </button>
                );
              })}
            </>
          )}

          {query.trim() && results.length === 0 && filteredProjects.length === 0 && (
            <div className="px-3 py-8 text-center text-sm text-muted-foreground/40">No results found</div>
          )}
        </div>

        {query.trim() && allItems.length > 0 && (
          <div className="border-t border-border/50 px-4 py-2 flex items-center gap-3 text-[11px] text-muted-foreground/40">
            <span><kbd className="font-mono">↑↓</kbd> navigate</span>
            <span><kbd className="font-mono">↵</kbd> select</span>
            <span><kbd className="font-mono">esc</kbd> close</span>
          </div>
        )}
      </div>
    </DialogOverlay>
  );
}
