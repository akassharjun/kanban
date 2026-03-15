import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus } from "lucide-react";
import type { Issue, Member, Label } from "@/types";

interface IssueCardProps {
  issue: Issue;
  member?: Member;
  labels: Label[];
  issues?: Issue[];
  onClick: () => void;
  isDragging?: boolean;
}

const priorityConfig: Record<string, { icon: React.ElementType; color: string }> = {
  urgent: { icon: AlertCircle, color: "text-red-500" },
  high: { icon: SignalHigh, color: "text-orange-500" },
  medium: { icon: SignalMedium, color: "text-yellow-500" },
  low: { icon: SignalLow, color: "text-blue-500" },
  none: { icon: Minus, color: "text-muted-foreground" },
};

export function IssueCard({ issue, member, labels, issues, onClick, isDragging }: IssueCardProps) {
  const priority = priorityConfig[issue.priority] || priorityConfig.none;
  const PriorityIcon = priority.icon;
  const parent = issue.parent_id ? issues?.find(i => i.id === issue.parent_id) : undefined;

  return (
    <div
      onClick={onClick}
      className={cn(
        "cursor-pointer rounded-md border border-border bg-card p-3 transition-all hover:border-foreground/20",
        isDragging && "rotate-2 shadow-lg opacity-90"
      )}
    >
      {parent && (
        <div className="flex items-center gap-1 mb-1 overflow-hidden">
          <span className="text-[10px] font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded shrink-0 whitespace-nowrap">
            ↑ {parent.identifier}
          </span>
          <span className="text-[10px] text-muted-foreground truncate">{parent.title}</span>
        </div>
      )}
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 mb-1">
            <PriorityIcon className={cn("h-3.5 w-3.5 flex-shrink-0", priority.color)} />
            <span className="text-xs text-muted-foreground">{issue.identifier}</span>
          </div>
          <p className="text-sm font-medium leading-tight">{issue.title}</p>
        </div>

        {member && (
          <div
            className="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-medium text-white"
            style={{ backgroundColor: member.avatar_color }}
            title={member.display_name || member.name}
          >
            {(member.display_name || member.name).charAt(0).toUpperCase()}
          </div>
        )}
      </div>

      {labels.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {labels.map((label) => (
            <Badge
              key={label.id}
              variant="outline"
              className="border-transparent py-0 text-[10px] font-medium"
              style={{ backgroundColor: label.color + "20", color: label.color }}
            >
              {label.name}
            </Badge>
          ))}
        </div>
      )}

      {issue.due_date && (
        <div className="mt-2 text-[10px] text-muted-foreground">
          Due {issue.due_date}
        </div>
      )}
    </div>
  );
}
