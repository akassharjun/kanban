import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { EpicBadge } from "@/components/EpicBadge";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, Calendar } from "lucide-react";
import type { Issue, Member, Label, Epic } from "@/types";

interface IssueCardProps {
  issue: Issue;
  member?: Member;
  labels: Label[];
  epic?: Epic;
  issues?: Issue[];
  onClick: () => void;
  isDragging?: boolean;
}

const priorityConfig: Record<string, { icon: React.ElementType; color: string; bg: string }> = {
  urgent: { icon: AlertCircle, color: "text-red-500", bg: "bg-red-500/10" },
  high: { icon: SignalHigh, color: "text-orange-500", bg: "bg-orange-500/10" },
  medium: { icon: SignalMedium, color: "text-yellow-500", bg: "bg-yellow-500/10" },
  low: { icon: SignalLow, color: "text-blue-400", bg: "bg-blue-400/10" },
  none: { icon: Minus, color: "text-muted-foreground/50", bg: "" },
};

function formatDueDate(dateStr: string): { text: string; urgent: boolean } {
  const due = new Date(dateStr);
  const now = new Date();
  const diffDays = Math.ceil((due.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
  if (diffDays < 0) return { text: `${Math.abs(diffDays)}d overdue`, urgent: true };
  if (diffDays === 0) return { text: "Due today", urgent: true };
  if (diffDays === 1) return { text: "Due tomorrow", urgent: false };
  if (diffDays <= 7) return { text: `Due in ${diffDays}d`, urgent: false };
  return { text: dateStr, urgent: false };
}

export function IssueCard({ issue, member, labels, epic, issues, onClick, isDragging }: IssueCardProps) {
  const priority = priorityConfig[issue.priority] || priorityConfig.none;
  const PriorityIcon = priority.icon;
  const parent = issue.parent_id ? issues?.find(i => i.id === issue.parent_id) : undefined;

  return (
    <div
      onClick={onClick}
      className={cn(
        "group cursor-pointer rounded-lg border border-border/60 bg-card p-3 transition-all",
        "hover:border-border hover:shadow-sm",
        isDragging && "rotate-2 shadow-xl opacity-95 ring-2 ring-primary/20"
      )}
    >
      {parent && (
        <div className="flex items-center gap-1 mb-1.5 overflow-hidden">
          <span className="text-[10px] font-mono text-muted-foreground/70 bg-muted px-1.5 py-0.5 rounded shrink-0 whitespace-nowrap">
            ↑ {parent.identifier}
          </span>
          <span className="text-[10px] text-muted-foreground/50 truncate">{parent.title}</span>
        </div>
      )}

      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 mb-1">
            <PriorityIcon className={cn("h-3.5 w-3.5 flex-shrink-0", priority.color)} />
            <span className="text-[11px] font-mono text-muted-foreground/60">{issue.identifier}</span>
          </div>
          <p className="text-[13px] font-medium leading-snug text-card-foreground/90">{issue.title}</p>
        </div>

        {member && (
          <div
            className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-semibold text-white ring-2 ring-card"
            style={{ backgroundColor: member.avatar_color }}
            title={member.display_name || member.name}
          >
            {(member.display_name || member.name).charAt(0).toUpperCase()}
          </div>
        )}
      </div>

      {(labels.length > 0 || issue.due_date || epic) && (
        <div className="mt-2 flex items-center gap-1.5 flex-wrap">
          {epic && <EpicBadge epic={epic} />}
          {labels.map((label) => (
            <Badge
              key={label.id}
              variant="outline"
              className="border-transparent py-0 px-1.5 text-[10px] font-medium rounded-md"
              style={{ backgroundColor: label.color + "18", color: label.color }}
            >
              {label.name}
            </Badge>
          ))}
          {issue.due_date && (() => {
            const { text, urgent } = formatDueDate(issue.due_date);
            return (
              <span className={cn(
                "ml-auto flex items-center gap-1 text-[10px]",
                urgent ? "text-red-500" : "text-muted-foreground/60"
              )}>
                <Calendar className="h-3 w-3" />
                {text}
              </span>
            );
          })()}
        </div>
      )}
    </div>
  );
}
