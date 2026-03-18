import { useState, useRef } from "react";
import { useDroppable } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import { SortableIssueCard } from "./SortableIssueCard";
import type { Issue, Status, Member, Label, Epic } from "@/types";

interface BoardColumnProps {
  status: Status;
  issues: Issue[];
  allIssues?: Issue[];
  members: Member[];
  epics?: Epic[];
  getLabelsForIssue: (issueId: number) => Label[];
  onClickIssue: (issue: Issue) => void;
  onQuickCreate: (title: string) => Promise<unknown>;
}

export function BoardColumn({ status, issues, allIssues, members, epics, getLabelsForIssue, onClickIssue, onQuickCreate }: BoardColumnProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [collapsed, setCollapsed] = useState(false);
  const escapePressedRef = useRef(false);

  const { setNodeRef, isOver } = useDroppable({ id: status.id });

  const handleSubmit = async () => {
    if (escapePressedRef.current) {
      escapePressedRef.current = false;
      setIsAdding(false);
      setNewTitle("");
      return;
    }
    if (!newTitle.trim()) { setIsAdding(false); return; }
    await onQuickCreate(newTitle.trim());
    setNewTitle("");
    setIsAdding(false);
  };

  const getMember = (id: number | null) => members.find((m) => m.id === id);

  return (
    <div className={cn("flex w-72 flex-shrink-0 flex-col", collapsed && "w-10")}>
      {/* Column header */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex items-center gap-2 px-2 py-2 text-sm font-medium mb-1"
      >
        <span
          className="h-2.5 w-2.5 rounded-full ring-2 ring-offset-1 ring-offset-background"
          style={{
            backgroundColor: status.color || "#6b7280",
            boxShadow: `0 0 0 2px ${status.color || "#6b7280"}33`,
          }}
        />
        {!collapsed && (
          <>
            <span className="truncate text-[13px]">{status.name}</span>
            <span className="text-[11px] font-normal text-muted-foreground/50">{issues.length}</span>
          </>
        )}
      </button>

      {!collapsed && (
        <div
          ref={setNodeRef}
          className={cn(
            "flex flex-1 flex-col gap-2 overflow-y-auto rounded-xl p-1.5 transition-colors",
            isOver && "bg-primary/5 ring-1 ring-primary/20"
          )}
        >
          <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
            {issues.map((issue) => (
              <SortableIssueCard
                key={issue.id}
                issue={issue}
                member={getMember(issue.assignee_id)}
                labels={getLabelsForIssue(issue.id)}
                epic={issue.epic_id ? epics?.find(e => e.id === issue.epic_id) : undefined}
                issues={allIssues}
                onClick={() => onClickIssue(issue)}
              />
            ))}
          </SortableContext>

          {isAdding ? (
            <div className="rounded-lg border border-primary/30 bg-card p-2.5 shadow-sm">
              <input
                autoFocus
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSubmit();
                  if (e.key === "Escape") {
                    escapePressedRef.current = true;
                    setIsAdding(false);
                    setNewTitle("");
                  }
                }}
                onBlur={handleSubmit}
                placeholder="Issue title..."
                className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground/50"
              />
            </div>
          ) : (
            <button
              onClick={() => setIsAdding(true)}
              className="flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-xs text-muted-foreground/50 hover:bg-muted hover:text-muted-foreground transition-colors"
            >
              <Plus className="h-3.5 w-3.5" />
              New issue
            </button>
          )}
        </div>
      )}
    </div>
  );
}
