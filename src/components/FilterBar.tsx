import { Filter, X } from "lucide-react";
import type { Status, Member, Label, Epic, MilestoneWithProgress } from "@/types";

export interface Filters {
  status_id?: number;
  priority?: string;
  assignee_id?: number;
  label_id?: number;
  epic_id?: number;
  milestone_id?: number;
}

interface FilterBarProps {
  statuses: Status[];
  members: Member[];
  labels: Label[];
  epics?: Epic[];
  milestones?: MilestoneWithProgress[];
  filters: Filters;
  onFiltersChange: (filters: Filters) => void;
}

const selectClass = "rounded-lg bg-muted/50 px-2.5 py-1.5 text-xs outline-none hover:bg-muted border-none cursor-pointer transition-colors text-muted-foreground hover:text-foreground";

export function FilterBar({ statuses, members, labels, epics, milestones, filters, onFiltersChange }: FilterBarProps) {
  const hasFilters = Object.values(filters).some(v => v !== undefined);

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

      {epics && epics.length > 0 && (
        <select
          value={filters.epic_id ?? ""}
          onChange={e => onFiltersChange({ ...filters, epic_id: e.target.value ? Number(e.target.value) : undefined })}
          className={selectClass}
        >
          <option value="">All epics</option>
          {epics.map(e => <option key={e.id} value={e.id}>{e.title}</option>)}
        </select>
      )}

      {milestones && milestones.length > 0 && (
        <select
          value={filters.milestone_id ?? ""}
          onChange={e => onFiltersChange({ ...filters, milestone_id: e.target.value ? Number(e.target.value) : undefined })}
          className={selectClass}
        >
          <option value="">All milestones</option>
          {milestones.map(m => <option key={m.id} value={m.id}>{m.title}</option>)}
        </select>
      )}

      {hasFilters && (
        <button
          onClick={() => onFiltersChange({})}
          className="ml-1 flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <X className="h-3 w-3" />
          Clear
        </button>
      )}
    </div>
  );
}
