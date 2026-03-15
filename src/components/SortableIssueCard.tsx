import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { IssueCard } from "./IssueCard";
import type { Issue, Member, Label } from "@/types";

interface Props {
  issue: Issue;
  member?: Member;
  labels: Label[];
  issues?: Issue[];
  onClick: () => void;
}

export function SortableIssueCard({ issue, member, labels, issues, onClick }: Props) {
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
