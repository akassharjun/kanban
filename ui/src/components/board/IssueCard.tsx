import { useNavigate } from "react-router";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { IssueDto } from "@/data/bindings";

export function IssueCard({ projectPrefix, issue }: { projectPrefix: string; issue: IssueDto }) {
  const navigate = useNavigate();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: issue.id,
  });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
  };
  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      onClick={() => navigate(`/p/${projectPrefix}/i/${issue.identifier}`)}
      className="cursor-grab rounded-md border border-black/10 bg-white p-2.5 shadow-sm hover:border-black/20 dark:border-white/10 dark:bg-neutral-900"
    >
      <div className="font-mono text-[11px] text-neutral-500">{issue.identifier}</div>
      <div className="mt-1 text-[13px] font-medium leading-snug">{issue.title}</div>
      {issue.priority && issue.priority !== "none" && (
        <span className="mt-1 inline-block rounded bg-red-100 px-1.5 py-0.5 text-[10px] text-red-700 dark:bg-red-900/40 dark:text-red-300">
          {issue.priority}
        </span>
      )}
    </div>
  );
}
