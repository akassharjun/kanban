import { useState, useEffect, useRef } from "react";
import { Search, ArrowRight, HelpCircle } from "lucide-react";
import { DialogOverlay } from "@/components/ui/dialog";
import type { Issue, Project, Status, Member, Label } from "@/types";
import * as api from "@/tauri/commands";

interface SearchDialogProps {
  projects: Project[];
  currentProjectId: number | null;
  onClose: () => void;
  onSelectIssue: (issue: Issue) => void;
  onSelectProject: (id: number) => void;
  statuses?: Status[];
  members?: Member[];
  labels?: Label[];
  memberId?: number;
}

const SEARCH_OPERATORS = [
  { prefix: "status:", description: "Filter by status name", examples: ["status:todo", 'status:"In Progress"'] },
  { prefix: "priority:", description: "Filter by priority", examples: ["priority:high", "priority:urgent"] },
  { prefix: "assignee:", description: "Filter by assignee", examples: ["assignee:arjun", "assignee:me"] },
  { prefix: "label:", description: "Filter by label", examples: ["label:bug"] },
  { prefix: "is:", description: "Filter by state", examples: ["is:open", "is:closed", "is:blocked", "is:starred"] },
  { prefix: "due:", description: "Filter by due date", examples: ["due:today", "due:overdue", "due:this-week"] },
  { prefix: "has:", description: "Filter by field presence", examples: ["has:description", "has:assignee", "has:due-date"] },
  { prefix: "created:", description: "Filter by creation date", examples: ["created:>2024-01-01"] },
  { prefix: "updated:", description: "Filter by update date", examples: ["updated:<2024-06-01"] },
];

function hasAdvancedSyntax(query: string): boolean {
  return /\b(status|priority|assignee|label|is|due|has|created|updated):/.test(query) ||
    /\bNOT\b/i.test(query) || /\bOR\b/i.test(query) || query.startsWith("-");
}

export function SearchDialog({ projects, currentProjectId: _currentProjectId, onClose, onSelectIssue, onSelectProject, statuses = [], members = [], labels = [], memberId }: SearchDialogProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Issue[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showHelp, setShowHelp] = useState(false);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [searching, setSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Compute autocomplete suggestions based on current input
  useEffect(() => {
    const trimmed = query.trimEnd();
    const lastToken = trimmed.split(/\s+/).pop() || "";

    if (lastToken.endsWith(":")) {
      const key = lastToken.slice(0, -1).toLowerCase();
      let options: string[] = [];
      switch (key) {
        case "status": options = statuses.map(s => s.name.includes(" ") ? `"${s.name}"` : s.name); break;
        case "priority": options = ["urgent", "high", "medium", "low", "none"]; break;
        case "assignee": options = ["me", ...members.map(m => m.name)]; break;
        case "label": options = labels.map(l => l.name.includes(" ") ? `"${l.name}"` : l.name); break;
        case "is": options = ["open", "closed", "blocked", "starred"]; break;
        case "due": options = ["today", "overdue", "this-week"]; break;
        case "has": options = ["description", "assignee", "due-date"]; break;
        default: options = []; break;
      }
      setSuggestions(options.map(o => `${lastToken}${o}`));
    } else {
      // Show operator suggestions if partially typed
      const partialOperators = SEARCH_OPERATORS
        .filter(op => op.prefix.startsWith(lastToken.toLowerCase()) && lastToken.length > 0 && !lastToken.includes(":"))
        .map(op => op.prefix);
      setSuggestions(partialOperators);
    }
  }, [query, statuses, members, labels]);

  useEffect(() => {
    if (!query.trim()) { setResults([]); setSearching(false); return; }
    const timeout = setTimeout(async () => {
      setSearching(true);
      try {
        if (hasAdvancedSyntax(query)) {
          const allResults = await Promise.all(
            projects.map(p => api.advancedSearch(p.id, query, memberId))
          );
          setResults(allResults.flat());
        } else {
          const allResults = await Promise.all(
            projects.map(p => api.searchIssues(p.id, query))
          );
          setResults(allResults.flat());
        }
        setSelectedIndex(0);
      } catch (e) {
        console.error("Search failed", e);
      } finally {
        setSearching(false);
      }
    }, 200);
    return () => clearTimeout(timeout);
  }, [query, projects, memberId]);

  const filteredProjects = projects.filter(p =>
    p.name.toLowerCase().includes(query.toLowerCase())
  );

  // Build flat list of selectable items for keyboard nav
  const allItems: { type: "project" | "issue"; id: number; project?: Project; issue?: Issue }[] = [];
  if (query.trim()) {
    filteredProjects.forEach(p => allItems.push({ type: "project", id: p.id, project: p }));
    results.forEach(i => allItems.push({ type: "issue", id: i.id, issue: i }));
  }

  const applySuggestion = (suggestion: string) => {
    const parts = query.trimEnd().split(/\s+/);
    parts[parts.length - 1] = suggestion;
    setQuery(parts.join(" ") + " ");
    inputRef.current?.focus();
    setSuggestions([]);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "Tab" && suggestions.length > 0) {
      e.preventDefault();
      applySuggestion(suggestions[0]);
      return;
    }
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
            placeholder="Search issues... (try status:todo, priority:high)"
            className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/40"
            onKeyDown={handleKeyDown}
          />
          {searching && (
            <span className="text-xs text-muted-foreground/60 animate-pulse">Searching...</span>
          )}
          <button
            onClick={() => setShowHelp(!showHelp)}
            className="rounded-md p-1 hover:bg-muted transition-colors"
            title="Search syntax help"
          >
            <HelpCircle className="h-4 w-4 text-muted-foreground/50" />
          </button>
          <kbd className="hidden sm:inline-flex h-5 items-center rounded border border-border/50 bg-muted px-1.5 text-[10px] font-mono text-muted-foreground/50">
            ESC
          </kbd>
        </div>

        {/* Autocomplete suggestions */}
        {suggestions.length > 0 && query.trim() && (
          <div className="border-t border-border/50 px-4 py-2 flex flex-wrap gap-1.5">
            {suggestions.slice(0, 8).map(s => (
              <button
                key={s}
                onClick={() => applySuggestion(s)}
                className="rounded-md bg-muted px-2 py-1 text-[11px] font-mono text-muted-foreground hover:bg-primary/10 hover:text-primary transition-colors"
              >
                {s}
              </button>
            ))}
            <span className="flex items-center text-[10px] text-muted-foreground/30 ml-1">Tab to complete</span>
          </div>
        )}

        <div className="border-t border-border/50" />

        {/* Help panel */}
        {showHelp && (
          <div className="border-b border-border/50 px-4 py-3 max-h-[250px] overflow-y-auto">
            <h4 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Search Syntax</h4>
            <div className="space-y-2">
              {SEARCH_OPERATORS.map(op => (
                <div key={op.prefix} className="flex items-start gap-2 text-[12px]">
                  <span className="font-mono text-blue-400 font-medium w-20 shrink-0">{op.prefix}</span>
                  <span className="text-muted-foreground/60">{op.description}</span>
                  <span className="ml-auto text-muted-foreground/40 font-mono text-[11px]">
                    {op.examples[0]}
                  </span>
                </div>
              ))}
              <div className="mt-2 pt-2 border-t border-border/50 text-[11px] text-muted-foreground/40">
                <p>Prefix with <span className="text-red-400 font-mono">-</span> or <span className="text-red-400 font-mono">NOT</span> to negate. Use plain text for fuzzy title/description match.</p>
              </div>
            </div>
          </div>
        )}

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
                    <span className="ml-auto text-[10px] text-muted-foreground/30 capitalize">{issue.priority !== "none" ? issue.priority : ""}</span>
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
            <span><kbd className="font-mono">Tab</kbd> complete</span>
            <span><kbd className="font-mono">esc</kbd> close</span>
          </div>
        )}
      </div>
    </DialogOverlay>
  );
}
