import { useMemo, useState } from "react";
import {
  DndContext,
  DragOverlay,
  closestCorners,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
} from "@dnd-kit/core";
import { BoardColumn } from "./BoardColumn";
import { IssueCard } from "./IssueCard";
import type { Issue, Status, Member, Label } from "@/types";

interface BoardViewProps {
  issues: Issue[];
  statuses: Status[];
  members: Member[];
  labels: Label[];
  onUpdateIssue: (id: number, input: { status_id?: number; position?: number }) => Promise<unknown>;
  onClickIssue: (issue: Issue) => void;
  onQuickCreate: (statusId: number, title: string) => Promise<unknown>;
}

export function BoardView({
  issues,
  statuses,
  members,
  labels,
  onUpdateIssue,
  onClickIssue,
  onQuickCreate,
}: BoardViewProps) {
  const [activeIssue, setActiveIssue] = useState<Issue | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const issuesByStatus = useMemo(() => {
    const map = new Map<number, Issue[]>();
    for (const status of statuses) {
      map.set(status.id, []);
    }
    for (const issue of issues.filter(i => !i.parent_id)) {
      const list = map.get(issue.status_id);
      if (list) list.push(issue);
    }
    // Sort each column by position
    for (const [, list] of map) {
      list.sort((a, b) => a.position - b.position);
    }
    return map;
  }, [issues, statuses]);

  const handleDragStart = (event: DragStartEvent) => {
    const found = issues.find((i) => i.id === event.active.id);
    if (found) setActiveIssue(found);
  };

  const handleDragEnd = async (event: DragEndEvent) => {
    setActiveIssue(null);
    const { active, over } = event;
    if (!over) return;

    const draggedIssue = issues.find((i) => i.id === active.id);
    if (!draggedIssue) return;

    // Determine target status - over could be a column or another issue
    let targetStatusId: number;
    let targetPosition: number;

    const overIssue = issues.find((i) => i.id === over.id);
    if (overIssue) {
      targetStatusId = overIssue.status_id;
      const columnIssues = (issuesByStatus.get(overIssue.status_id) || [])
        .filter(i => i.id !== draggedIssue.id);
      const overIndex = columnIssues.findIndex(i => i.id === overIssue.id);
      if (overIndex <= 0) {
        targetPosition = overIssue.position - 0.5;
      } else {
        targetPosition = (columnIssues[overIndex - 1].position + overIssue.position) / 2;
      }
    } else {
      // Dropped on a column
      targetStatusId = over.id as number;
      const columnIssues = issuesByStatus.get(targetStatusId) || [];
      targetPosition = columnIssues.length > 0
        ? columnIssues[columnIssues.length - 1].position + 1
        : 0;
    }

    if (draggedIssue.status_id !== targetStatusId || draggedIssue.position !== targetPosition) {
      await onUpdateIssue(draggedIssue.id, {
        status_id: targetStatusId,
        position: targetPosition,
      });
    }
  };

  const getMember = (id: number | null) => members.find((m) => m.id === id);

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div className="flex h-full gap-4 overflow-x-auto p-4">
        {statuses.map((status) => {
          const columnIssues = issuesByStatus.get(status.id) || [];
          return (
            <BoardColumn
              key={status.id}
              status={status}
              issues={columnIssues}
              members={members}
              labels={labels}
              onClickIssue={onClickIssue}
              onQuickCreate={(title) => onQuickCreate(status.id, title)}
            />
          );
        })}
      </div>

      <DragOverlay>
        {activeIssue ? (
          <IssueCard
            issue={activeIssue}
            member={getMember(activeIssue.assignee_id)}
            labels={[]}
            onClick={() => {}}
            isDragging
          />
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
