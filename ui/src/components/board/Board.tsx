import { useMemo, useState } from "react";
import { DndContext, type DragEndEvent, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import { useIssues, useStatuses } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change, STATUS_CATEGORIES } from "@/data/ops";
import { midpoint } from "@/lib/sortKey";
import { uuidv7 } from "@/lib/uuid";
import { BoardColumn } from "./BoardColumn";
import { NewIssueInline } from "./NewIssueInline";

const DEFAULT_COLUMN_COLOR = "#94a3b8";

export function Board({ projectPrefix, projectId }: { projectPrefix: string; projectId: string }) {
  const { data: issues = [] } = useIssues(projectPrefix);
  const { data: statuses = [] } = useStatuses(projectPrefix);
  const apply = useApply(projectPrefix);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));
  const [addingStatus, setAddingStatus] = useState<string | null>(null);
  const [addingColumn, setAddingColumn] = useState(false);
  const [newColumnName, setNewColumnName] = useState("");
  const [newColumnCategory, setNewColumnCategory] = useState<string>(STATUS_CATEGORIES[0]);
  const [newColumnColor, setNewColumnColor] = useState(DEFAULT_COLUMN_COLOR);

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

  function addColumn() {
    const name = newColumnName.trim();
    if (!name) return;
    apply.mutate(
      ops.createStatus({
        id: uuidv7(),
        project_id: projectId,
        name,
        category: newColumnCategory,
        color: newColumnColor,
        position: columns.length,
      }),
    );
    setNewColumnName("");
    setNewColumnCategory(STATUS_CATEGORIES[0]);
    setNewColumnColor(DEFAULT_COLUMN_COLOR);
    setAddingColumn(false);
  }

  return (
    <div className="flex h-full flex-col">
      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <div className="flex h-full gap-3 overflow-x-auto p-4">
          {columns.map(({ status, issues: colIssues }, i) => (
            <BoardColumn
              key={status.id}
              status={status}
              issues={colIssues}
              projectPrefix={projectPrefix}
              onAddIssue={() => setAddingStatus(status.id)}
              index={i}
              count={columns.length}
            />
          ))}
          <div className="w-[260px] shrink-0">
            {addingColumn ? (
              <form
                className="flex flex-col gap-2 rounded-lg border border-dashed border-black/15 p-2.5 dark:border-white/15"
                onSubmit={(e) => {
                  e.preventDefault();
                  addColumn();
                }}
              >
                <input
                  aria-label="New column name"
                  autoFocus
                  value={newColumnName}
                  onChange={(e) => setNewColumnName(e.target.value)}
                  placeholder="Column name"
                  className="rounded border border-black/15 bg-transparent px-2 py-1 text-sm dark:border-white/15"
                />
                <div className="flex items-center gap-2">
                  <select
                    aria-label="New column category"
                    value={newColumnCategory}
                    onChange={(e) => setNewColumnCategory(e.target.value)}
                    className="flex-1 rounded border border-black/15 bg-transparent px-1 py-1 text-xs dark:border-white/15"
                  >
                    {STATUS_CATEGORIES.map((c) => (
                      <option key={c} value={c}>
                        {c}
                      </option>
                    ))}
                  </select>
                  <input
                    type="color"
                    aria-label="New column color"
                    value={newColumnColor}
                    onChange={(e) => setNewColumnColor(e.target.value)}
                    className="h-7 w-9 cursor-pointer border-0 bg-transparent p-0"
                  />
                </div>
                <div className="flex gap-2">
                  <button
                    type="submit"
                    className="rounded-md bg-black/80 px-2 py-1 text-xs text-white hover:bg-black dark:bg-white/80 dark:text-black dark:hover:bg-white"
                  >
                    Add
                  </button>
                  <button
                    type="button"
                    onClick={() => setAddingColumn(false)}
                    className="rounded-md px-2 py-1 text-xs text-neutral-600 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            ) : (
              <button
                type="button"
                onClick={() => setAddingColumn(true)}
                className="w-full rounded-lg border border-dashed border-black/15 px-2 py-2 text-xs text-neutral-600 hover:bg-black/5 dark:border-white/15 dark:text-neutral-300 dark:hover:bg-white/10"
              >
                + Add column
              </button>
            )}
          </div>
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
