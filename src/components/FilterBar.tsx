import { useState } from "react";
import { Filter, X, Bookmark } from "lucide-react";
import type { Status, Member, Label } from "@/types";

export interface Filters {
  status_id?: number;
  priority?: string;
  assignee_id?: number;
  label_id?: number;
}

interface FilterBarProps {
  statuses: Status[];
  members: Member[];
  labels: Label[];
  filters: Filters;
  onFiltersChange: (filters: Filters) => void;
  onSaveView?: (name: string) => void;
  viewMode?: string;
}

const selectClass = "rounded-lg bg-muted/50 px-2.5 py-1.5 text-xs outline-none hover:bg-muted border-none cursor-pointer transition-colors text-muted-foreground hover:text-foreground";

export function FilterBar({ statuses, members, labels, filters, onFiltersChange, onSaveView }: FilterBarProps) {
  const hasFilters = Object.values(filters).some(v => v !== undefined);
  const [showSaveInput, setShowSaveInput] = useState(false);
  const [viewName, setViewName] = useState("");

  return (
    <div className="flex items-center gap-2 px-4 py-2">
      <Filter className="h-3.5 w-3.5 text-muted-foreground/50" />

      <select
        value={filters.status_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, status_id: e.target.value ? Number(e.target.value) : undefined })}
        className={selectClass}
      >
        <option value="">All statuses</option>
        {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
      </select>

      <select
        value={filters.priority ?? ""}
        onChange={e => onFiltersChange({ ...filters, priority: e.target.value || undefined })}
        className={selectClass}
      >
        <option value="">All priorities</option>
        <option value="urgent">Urgent</option>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
        <option value="none">None</option>
      </select>

      <select
        value={filters.assignee_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, assignee_id: e.target.value ? Number(e.target.value) : undefined })}
        className={selectClass}
      >
        <option value="">All assignees</option>
        {members.map(m => <option key={m.id} value={m.id}>{m.display_name || m.name}</option>)}
      </select>

      <select
        value={filters.label_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, label_id: e.target.value ? Number(e.target.value) : undefined })}
        className={selectClass}
      >
        <option value="">All labels</option>
        {labels.map(l => <option key={l.id} value={l.id}>{l.name}</option>)}
      </select>

      {hasFilters && (
        <button
          onClick={() => onFiltersChange({})}
          className="ml-1 flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <X className="h-3 w-3" />
          Clear
        </button>
      )}

      {hasFilters && onSaveView && !showSaveInput && (
        <button
          onClick={() => setShowSaveInput(true)}
          className="ml-1 flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Bookmark className="h-3 w-3" />
          Save View
        </button>
      )}

      {showSaveInput && (
        <div className="ml-1 flex items-center gap-1.5">
          <input
            autoFocus
            value={viewName}
            onChange={(e) => setViewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && viewName.trim()) {
                onSaveView?.(viewName.trim());
                setViewName("");
                setShowSaveInput(false);
              }
              if (e.key === "Escape") {
                setViewName("");
                setShowSaveInput(false);
              }
            }}
            placeholder="View name..."
            className="rounded-lg bg-muted/50 px-2.5 py-1.5 text-xs outline-none w-32"
          />
          <button
            onClick={() => {
              if (viewName.trim()) {
                onSaveView?.(viewName.trim());
                setViewName("");
                setShowSaveInput(false);
              }
            }}
            className="rounded-lg bg-primary px-2.5 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90"
          >
            Save
          </button>
          <button
            onClick={() => { setViewName(""); setShowSaveInput(false); }}
            className="rounded-lg px-2 py-1.5 text-xs text-muted-foreground hover:bg-muted"
          >
            <X className="h-3 w-3" />
          </button>
        </div>
      )}
    </div>
  );
}
