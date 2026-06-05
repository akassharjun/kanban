import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import type { IssueDto, StatusDto } from "@/data/bindings";
import { IssueCard } from "./IssueCard";

export function BoardColumn({
  status,
  issues,
  projectPrefix,
  onAddIssue,
}: {
  status: StatusDto;
  issues: IssueDto[];
  projectPrefix: string;
  onAddIssue: (statusId: string) => void;
}) {
  return (
    <div className="flex w-[260px] shrink-0 flex-col gap-2 rounded-lg bg-black/[0.03] p-2.5 dark:bg-white/[0.04]">
      <div className="flex items-center justify-between px-1">
        <span className="text-sm font-semibold">{status.name}</span>
        <span className="text-xs text-neutral-500">{issues.length}</span>
      </div>
      <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
        {issues.map((issue) => (
          <IssueCard key={issue.id} projectPrefix={projectPrefix} issue={issue} />
        ))}
      </SortableContext>
      <button
        type="button"
        onClick={() => onAddIssue(status.id)}
        className="mt-auto rounded-md border border-dashed border-black/15 px-2 py-1 text-xs text-neutral-600 hover:bg-black/5 dark:border-white/15 dark:text-neutral-300 dark:hover:bg-white/10"
      >
        + Add issue
      </button>
    </div>
  );
}
