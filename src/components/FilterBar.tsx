import { Filter, X } from "lucide-react";
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
}

export function FilterBar({ statuses, members, labels, filters, onFiltersChange }: FilterBarProps) {
  const hasFilters = Object.values(filters).some(v => v !== undefined);

  return (
    <div className="flex items-center gap-2 border-b border-border px-4 py-2">
      <Filter className="h-3.5 w-3.5 text-muted-foreground" />

      {/* Status filter */}
      <select
        value={filters.status_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, status_id: e.target.value ? Number(e.target.value) : undefined })}
        className="rounded bg-transparent px-2 py-1 text-xs outline-none hover:bg-accent border border-border"
      >
        <option value="">All statuses</option>
        {statuses.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
      </select>

      {/* Priority filter */}
      <select
        value={filters.priority ?? ""}
        onChange={e => onFiltersChange({ ...filters, priority: e.target.value || undefined })}
        className="rounded bg-transparent px-2 py-1 text-xs outline-none hover:bg-accent border border-border"
      >
        <option value="">All priorities</option>
        <option value="urgent">Urgent</option>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
        <option value="none">None</option>
      </select>

      {/* Assignee filter */}
      <select
        value={filters.assignee_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, assignee_id: e.target.value ? Number(e.target.value) : undefined })}
        className="rounded bg-transparent px-2 py-1 text-xs outline-none hover:bg-accent border border-border"
      >
        <option value="">All assignees</option>
        {members.map(m => <option key={m.id} value={m.id}>{m.display_name || m.name}</option>)}
      </select>

      {/* Label filter */}
      <select
        value={filters.label_id ?? ""}
        onChange={e => onFiltersChange({ ...filters, label_id: e.target.value ? Number(e.target.value) : undefined })}
        className="rounded bg-transparent px-2 py-1 text-xs outline-none hover:bg-accent border border-border"
      >
        <option value="">All labels</option>
        {labels.map(l => <option key={l.id} value={l.id}>{l.name}</option>)}
      </select>

      {hasFilters && (
        <button
          onClick={() => onFiltersChange({})}
          className="ml-1 flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
        >
          <X className="h-3 w-3" />
          Clear
        </button>
      )}
    </div>
  );
}
