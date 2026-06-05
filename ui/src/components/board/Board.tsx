import { useMemo, useState } from "react";
import { DndContext, type DragEndEvent, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import { useIssues, useStatuses } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { midpoint } from "@/lib/sortKey";
import { BoardColumn } from "./BoardColumn";
import { NewIssueInline } from "./NewIssueInline";

export function Board({ projectPrefix, projectId }: { projectPrefix: string; projectId: string }) {
  const { data: issues = [] } = useIssues(projectPrefix);
  const { data: statuses = [] } = useStatuses(projectPrefix);
  const apply = useApply(projectPrefix);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));
  const [addingStatus, setAddingStatus] = useState<string | null>(null);

  const columns = useMemo(
    () =>
      statuses
        .slice()
        .sort((a, b) => a.position - b.position)
        .map((s) => ({
          status: s,
          issues: issues.filter((i) => i.status_id === s.id).sort((a, b) => a.sort_key - b.sort_key),
        })),
    [statuses, issues],
  );

  function handleDragEnd(event: DragEndEvent) {
    const draggedId = String(event.active.id);
    const overId = event.over ? String(event.over.id) : undefined;
    if (!overId || draggedId === overId) return;

    const dragged = issues.find((i) => i.id === draggedId);
    if (!dragged) return;

    const target = issues.find((i) => i.id === overId);
    const targetStatusId = target?.status_id ?? statuses.find((s) => s.id === overId)?.id;
    if (!targetStatusId) return;

    const colIssues = issues
      .filter((i) => i.status_id === targetStatusId && i.id !== draggedId)
      .sort((a, b) => a.sort_key - b.sort_key);
    const idx = target ? colIssues.findIndex((i) => i.id === target.id) : colIssues.length;
    const before = idx > 0 ? colIssues[idx - 1]?.sort_key : undefined;
    const after = colIssues[idx]?.sort_key;

    if (dragged.status_id !== targetStatusId) {
      apply.mutate(ops.updateIssueField({ id: dragged.id, change: change.status(targetStatusId) }));
    }
    apply.mutate(ops.reorderIssue({ id: dragged.id, new_sort_key: midpoint(before, after) }));
  }

  return (
    <div className="flex h-full flex-col">
      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <div className="flex h-full gap-3 overflow-x-auto p-4">
          {columns.map(({ status, issues: colIssues }) => (
            <BoardColumn
              key={status.id}
              status={status}
              issues={colIssues}
              projectPrefix={projectPrefix}
              onAddIssue={() => setAddingStatus(status.id)}
            />
          ))}
        </div>
      </DndContext>
      {addingStatus && (
        <NewIssueInline
          projectId={projectId}
          projectPrefix={projectPrefix}
          statusId={addingStatus}
          onDone={() => setAddingStatus(null)}
        />
      )}
    </div>
  );
}
