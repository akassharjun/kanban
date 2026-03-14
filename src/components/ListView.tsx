import { useState, useMemo } from "react";
import { cn } from "@/lib/utils";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, ChevronUp, ChevronDown } from "lucide-react";
import type { Issue, Status, Member, Label } from "@/types";

interface ListViewProps {
  issues: Issue[];
  statuses: Status[];
  members: Member[];
  labels: Label[];
  onClickIssue: (issue: Issue) => void;
}

type SortField = "identifier" | "title" | "priority" | "status" | "assignee" | "created_at" | "updated_at" | "due_date";

const priorityOrder: Record<string, number> = { urgent: 0, high: 1, medium: 2, low: 3, none: 4 };
const priorityConfig: Record<string, { icon: React.ElementType; color: string; label: string }> = {
  urgent: { icon: AlertCircle, color: "text-red-500", label: "Urgent" },
  high: { icon: SignalHigh, color: "text-orange-500", label: "High" },
  medium: { icon: SignalMedium, color: "text-yellow-500", label: "Medium" },
  low: { icon: SignalLow, color: "text-blue-500", label: "Low" },
  none: { icon: Minus, color: "text-muted-foreground", label: "None" },
};

export function ListView({ issues, statuses, members, onClickIssue }: ListViewProps) {
  const [sortField, setSortField] = useState<SortField>("created_at");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  const topLevelIssues = issues.filter(i => !i.parent_id);

  const sorted = useMemo(() => {
    return [...topLevelIssues].sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "identifier": cmp = a.identifier.localeCompare(b.identifier); break;
        case "title": cmp = a.title.localeCompare(b.title); break;
        case "priority": cmp = (priorityOrder[a.priority] ?? 4) - (priorityOrder[b.priority] ?? 4); break;
        case "status": cmp = a.status_id - b.status_id; break;
        case "assignee": cmp = (a.assignee_id ?? 999) - (b.assignee_id ?? 999); break;
        case "created_at": cmp = a.created_at.localeCompare(b.created_at); break;
        case "updated_at": cmp = a.updated_at.localeCompare(b.updated_at); break;
        case "due_date": cmp = (a.due_date ?? "9999").localeCompare(b.due_date ?? "9999"); break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [topLevelIssues, sortField, sortDir]);

  const toggleSort = (field: SortField) => {
    if (sortField === field) setSortDir(d => d === "asc" ? "desc" : "asc");
    else { setSortField(field); setSortDir("asc"); }
  };

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null;
    return sortDir === "asc" ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />;
  };

  const getStatus = (id: number) => statuses.find(s => s.id === id);
  const getMember = (id: number | null) => id ? members.find(m => m.id === id) : undefined;

  return (
    <div className="flex-1 overflow-auto p-4">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-border text-left text-xs text-muted-foreground">
            {(
              [
                ["identifier", "ID"],
                ["priority", "Priority"],
                ["title", "Title"],
                ["status", "Status"],
                ["assignee", "Assignee"],
                ["due_date", "Due Date"],
                ["updated_at", "Updated"],
              ] as [SortField, string][]
            ).map(([field, label]) => (
              <th key={field} className="px-3 py-2 font-medium">
                <button onClick={() => toggleSort(field)} className="flex items-center gap-1 hover:text-foreground">
                  {label} <SortIcon field={field} />
                </button>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((issue) => {
            const status = getStatus(issue.status_id);
            const member = getMember(issue.assignee_id);
            const p = priorityConfig[issue.priority] || priorityConfig.none;
            const PIcon = p.icon;
            return (
              <tr
                key={issue.id}
                onClick={() => onClickIssue(issue)}
                className="cursor-pointer border-b border-border/50 hover:bg-accent/30 transition-colors"
              >
                <td className="px-3 py-2 text-muted-foreground">{issue.identifier}</td>
                <td className="px-3 py-2">
                  <div className="flex items-center gap-1">
                    <PIcon className={cn("h-3.5 w-3.5", p.color)} />
                    <span className="text-xs">{p.label}</span>
                  </div>
                </td>
                <td className="px-3 py-2 font-medium">{issue.title}</td>
                <td className="px-3 py-2">
                  {status && (
                    <span className="flex items-center gap-1.5">
                      <span className="h-2 w-2 rounded-full" style={{ backgroundColor: status.color || "#6b7280" }} />
                      {status.name}
                    </span>
                  )}
                </td>
                <td className="px-3 py-2">
                  {member && (
                    <div className="flex items-center gap-1.5">
                      <div
                        className="flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-medium text-white"
                        style={{ backgroundColor: member.avatar_color }}
                      >
                        {(member.display_name || member.name).charAt(0).toUpperCase()}
                      </div>
                      <span>{member.display_name || member.name}</span>
                    </div>
                  )}
                </td>
                <td className="px-3 py-2 text-muted-foreground">{issue.due_date || "-"}</td>
                <td className="px-3 py-2 text-muted-foreground">{issue.updated_at.slice(0, 10)}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {sorted.length === 0 && (
        <div className="py-12 text-center text-muted-foreground">No issues found</div>
      )}
    </div>
  );
}
