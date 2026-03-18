import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { IssueCard } from "./IssueCard";
import type { Issue, Member, Label } from "@/types";

interface Props {
  issue: Issue;
  member?: Member;
  labels: Label[];
  issues?: Issue[];
  members?: Member[];
  hasGitLinks?: boolean;
  isStaleSoon?: boolean;
  onClick: () => void;
  onUpdateIssue?: (id: number, input: { title?: string; priority?: string; assignee_id?: number }) => Promise<unknown>;
}

export function SortableIssueCard({ issue, member, labels, issues, members: _members, hasGitLinks: _hasGitLinks, isStaleSoon: _isStaleSoon, onClick, onUpdateIssue: _onUpdateIssue }: Props) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: issue.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <IssueCard
        issue={issue}
        member={member}
        labels={labels}
        issues={issues}
        onClick={onClick}
      />
    </div>
  );
}
