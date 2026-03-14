import { useState, useRef } from "react";
import { useDroppable } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import { SortableIssueCard } from "./SortableIssueCard";
import type { Issue, Status, Member, Label } from "@/types";

interface BoardColumnProps {
  status: Status;
  issues: Issue[];
  members: Member[];
  labels: Label[];
  onClickIssue: (issue: Issue) => void;
  onQuickCreate: (title: string) => Promise<unknown>;
}

export function BoardColumn({ status, issues, members, labels, onClickIssue, onQuickCreate }: BoardColumnProps) {
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
    <div className={cn("flex w-72 flex-shrink-0 flex-col rounded-lg", collapsed && "w-10")}>
      {/* Column header */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex items-center gap-2 px-3 py-2 text-sm font-medium"
      >
        <span
          className="h-2.5 w-2.5 rounded-full"
          style={{ backgroundColor: status.color || "#6b7280" }}
        />
        {!collapsed && (
          <>
            <span className="truncate">{status.name}</span>
            <span className="text-xs text-muted-foreground">{issues.length}</span>
          </>
        )}
      </button>

      {!collapsed && (
        <div
          ref={setNodeRef}
          className={cn(
            "flex flex-1 flex-col gap-1.5 overflow-y-auto rounded-lg p-1.5 transition-colors",
            isOver && "bg-accent/30"
          )}
        >
          <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
            {issues.map((issue) => (
              <SortableIssueCard
                key={issue.id}
                issue={issue}
                member={getMember(issue.assignee_id)}
                labels={labels.filter((l) => l.project_id === issue.project_id)}
                onClick={() => onClickIssue(issue)}
              />
            ))}
          </SortableContext>

          {isAdding ? (
            <div className="rounded-md border border-border bg-card p-2">
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
                className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
              />
            </div>
          ) : (
            <button
              onClick={() => setIsAdding(true)}
              className="flex items-center gap-1 rounded-md px-2 py-1.5 text-xs text-muted-foreground hover:bg-accent/50 hover:text-foreground"
            >
              <Plus className="h-3 w-3" />
              New issue
            </button>
          )}
        </div>
      )}
    </div>
  );
}
