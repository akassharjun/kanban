import { useState, useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, Calendar, GitBranch } from "lucide-react";
import type { Issue, Member, Label, Priority } from "@/types";

interface IssueCardProps {
  issue: Issue;
  member?: Member;
  labels: Label[];
  issues?: Issue[];
  members?: Member[];
  hasGitLinks?: boolean;
  isStaleSoon?: boolean;
  onClick: () => void;
  onUpdateIssue?: (id: number, input: { title?: string; priority?: string; assignee_id?: number }) => Promise<unknown>;
  isDragging?: boolean;
}

const priorityConfig: Record<string, { icon: React.ElementType; color: string; bg: string; label: string }> = {
  urgent: { icon: AlertCircle, color: "text-red-500", bg: "bg-red-500/10", label: "Urgent" },
  high: { icon: SignalHigh, color: "text-orange-500", bg: "bg-orange-500/10", label: "High" },
  medium: { icon: SignalMedium, color: "text-yellow-500", bg: "bg-yellow-500/10", label: "Medium" },
  low: { icon: SignalLow, color: "text-blue-400", bg: "bg-blue-400/10", label: "Low" },
  none: { icon: Minus, color: "text-muted-foreground/50", bg: "", label: "None" },
};

const priorityList: Priority[] = ["urgent", "high", "medium", "low", "none"];

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

/** Simple inline dropdown that closes on outside click */
function InlineDropdown({ open, onClose, children, className }: {
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
  className?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div ref={ref} className={cn("absolute z-50 min-w-[140px] rounded-lg border border-border bg-popover p-1 shadow-lg animate-in fade-in-0 zoom-in-95 duration-100", className)}>
      {children}
    </div>
  );
}

export function IssueCard({ issue, member, labels, issues, members, hasGitLinks, isStaleSoon, onClick, onUpdateIssue, isDragging }: IssueCardProps) {
  const priority = priorityConfig[issue.priority] || priorityConfig.none;
  const PriorityIcon = priority.icon;
  const parent = issue.parent_id ? issues?.find(i => i.id === issue.parent_id) : undefined;

  // Inline editing state
  const [editingField, setEditingField] = useState<"title" | "priority" | "assignee" | null>(null);
  const [editTitle, setEditTitle] = useState(issue.title);
  const titleInputRef = useRef<HTMLInputElement>(null);

  // Reset edit title when issue changes
  useEffect(() => { setEditTitle(issue.title); }, [issue.title]);

  const handleTitleDoubleClick = (e: React.MouseEvent) => {
    if (!onUpdateIssue) return;
    e.stopPropagation();
    e.preventDefault();
    setEditingField("title");
    setEditTitle(issue.title);
    setTimeout(() => titleInputRef.current?.focus(), 0);
  };

  const handleTitleSave = async () => {
    if (editTitle.trim() && editTitle !== issue.title && onUpdateIssue) {
      await onUpdateIssue(issue.id, { title: editTitle.trim() });
    }
    setEditingField(null);
  };

  const handlePriorityClick = (e: React.MouseEvent) => {
    if (!onUpdateIssue) return;
    e.stopPropagation();
    e.preventDefault();
    setEditingField(editingField === "priority" ? null : "priority");
  };

  const handleAssigneeClick = (e: React.MouseEvent) => {
    if (!onUpdateIssue || !members) return;
    e.stopPropagation();
    e.preventDefault();
    setEditingField(editingField === "assignee" ? null : "assignee");
  };

  const handleCardClick = () => {
    if (editingField) return;
    onClick();
  };

  return (
    <div
      onClick={handleCardClick}
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
            <div className="relative">
              <PriorityIcon
                className={cn("h-3.5 w-3.5 flex-shrink-0 cursor-pointer transition-transform hover:scale-125", priority.color)}
                onClick={handlePriorityClick}
              />
              <InlineDropdown
                open={editingField === "priority"}
                onClose={() => setEditingField(null)}
                className="top-full left-0 mt-1"
              >
                {priorityList.map((p) => {
                  const cfg = priorityConfig[p];
                  const Icon = cfg.icon;
                  return (
                    <button
                      key={p}
                      onClick={async (e) => {
                        e.stopPropagation();
                        if (onUpdateIssue && p !== issue.priority) {
                          await onUpdateIssue(issue.id, { priority: p });
                        }
                        setEditingField(null);
                      }}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12px] hover:bg-muted transition-colors",
                        p === issue.priority && "bg-muted font-medium"
                      )}
                    >
                      <Icon className={cn("h-3.5 w-3.5", cfg.color)} />
                      {cfg.label}
                    </button>
                  );
                })}
              </InlineDropdown>
            </div>
            <span className="text-[11px] font-mono text-muted-foreground/60">{issue.identifier}</span>
            {hasGitLinks && (
              <GitBranch className="h-3 w-3 text-muted-foreground/40 flex-shrink-0" />
            )}
          </div>
          {editingField === "title" ? (
            <input
              ref={titleInputRef}
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              onBlur={handleTitleSave}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleTitleSave();
                if (e.key === "Escape") { setEditTitle(issue.title); setEditingField(null); }
              }}
              onClick={(e) => e.stopPropagation()}
              className="w-full bg-transparent text-[13px] font-medium leading-snug text-card-foreground/90 outline-none border-b border-primary/50"
            />
          ) : (
            <p
              className="text-[13px] font-medium leading-snug text-card-foreground/90"
              onDoubleClick={handleTitleDoubleClick}
            >
              {issue.title}
            </p>
          )}
        </div>

        <div className="relative">
          {member ? (
            <div
              className={cn(
                "flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-semibold text-white ring-2 ring-card",
                onUpdateIssue && members && "cursor-pointer hover:ring-primary/50 transition-all"
              )}
              style={{ backgroundColor: member.avatar_color }}
              title={member.display_name || member.name}
              onClick={handleAssigneeClick}
            >
              {(member.display_name || member.name).charAt(0).toUpperCase()}
            </div>
          ) : onUpdateIssue && members ? (
            <div
              className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full border border-dashed border-muted-foreground/30 cursor-pointer hover:border-primary/50 transition-all opacity-0 group-hover:opacity-100"
              title="Assign"
              onClick={handleAssigneeClick}
            />
          ) : null}
          <InlineDropdown
            open={editingField === "assignee"}
            onClose={() => setEditingField(null)}
            className="top-full right-0 mt-1"
          >
            <button
              onClick={async (e) => {
                e.stopPropagation();
                if (onUpdateIssue) await onUpdateIssue(issue.id, { assignee_id: -1 });
                setEditingField(null);
              }}
              className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12px] hover:bg-muted transition-colors"
            >
              <div className="h-5 w-5 rounded-full border border-dashed border-muted-foreground/30" />
              Unassigned
            </button>
            {(members || []).map(m => (
              <button
                key={m.id}
                onClick={async (e) => {
                  e.stopPropagation();
                  if (onUpdateIssue) await onUpdateIssue(issue.id, { assignee_id: m.id });
                  setEditingField(null);
                }}
                className={cn(
                  "flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12px] hover:bg-muted transition-colors",
                  m.id === issue.assignee_id && "bg-muted font-medium"
                )}
              >
                <div
                  className="flex h-5 w-5 items-center justify-center rounded-full text-[9px] font-semibold text-white"
                  style={{ backgroundColor: m.avatar_color }}
                >
                  {(m.display_name || m.name).charAt(0).toUpperCase()}
                </div>
                {m.display_name || m.name}
              </button>
            ))}
          </InlineDropdown>
        </div>
      </div>

      {isStaleSoon && (
        <div className="mt-1.5">
          <span className="inline-flex items-center gap-1 rounded-full bg-amber-500/10 px-2 py-0.5 text-[10px] font-medium text-amber-600 dark:text-amber-400">
            Stale soon
          </span>
        </div>
      )}

      {(labels.length > 0 || issue.due_date) && (
        <div className="mt-2 flex items-center gap-1.5 flex-wrap">
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
