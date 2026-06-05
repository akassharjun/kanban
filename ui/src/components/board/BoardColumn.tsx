import { useState } from "react";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import type { IssueDto, StatusDto } from "@/data/bindings";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";
import { IssueCard } from "./IssueCard";

/** Narrow an unknown error to its `message` string, if present. */
function messageOf(e: unknown): string | undefined {
  if (typeof e === "object" && e !== null && "message" in e) {
    const m = (e as { message: unknown }).message;
    if (typeof m === "string") return m;
  }
  return undefined;
}

export function BoardColumn({
  status,
  issues,
  projectPrefix,
  onAddIssue,
  index,
  count,
}: {
  status: StatusDto;
  issues: IssueDto[];
  projectPrefix: string;
  onAddIssue: (statusId: string) => void;
  index: number;
  count: number;
}) {
  const apply = useApply(projectPrefix);
  const [menuOpen, setMenuOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function commitRename(value: string) {
    setRenaming(false);
    const name = value.trim();
    if (name && name !== status.name) {
      apply.mutate(ops.updateStatus({ id: status.id, name }));
    }
  }

  return (
    <div className="flex w-[260px] shrink-0 flex-col gap-2 rounded-lg bg-black/[0.03] p-2.5 dark:bg-white/[0.04]">
      <div className="flex items-center justify-between px-1">
        {renaming ? (
          <input
            aria-label="Rename column"
            defaultValue={status.name}
            autoFocus
            className="w-full rounded border border-black/15 bg-transparent px-1 text-sm font-semibold dark:border-white/15"
            onKeyDown={(e) => {
              if (e.key === "Enter") commitRename(e.currentTarget.value);
              else if (e.key === "Escape") setRenaming(false);
            }}
            onBlur={(e) => commitRename(e.currentTarget.value)}
          />
        ) : (
          <span className="text-sm font-semibold">{status.name}</span>
        )}
        <div className="flex items-center gap-1">
          <span className="text-xs text-neutral-500">{issues.length}</span>
          <div className="relative">
            <button
              type="button"
              aria-label={`Column actions for ${status.name}`}
              aria-expanded={menuOpen}
              onClick={() => setMenuOpen((o) => !o)}
              className="rounded px-1 text-sm text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10"
            >
              ⋯
            </button>
            {menuOpen && (
              <div className="absolute right-0 z-10 mt-1 flex w-44 flex-col gap-1 rounded-md border border-black/10 bg-white p-1 shadow-lg dark:border-white/10 dark:bg-neutral-800">
                <button
                  type="button"
                  className="rounded px-2 py-1 text-left text-xs hover:bg-black/5 dark:hover:bg-white/10"
                  onClick={() => {
                    setRenaming(true);
                    setMenuOpen(false);
                  }}
                >
                  Rename
                </button>
                <label className="flex cursor-pointer items-center justify-between rounded px-2 py-1 text-xs hover:bg-black/5 dark:hover:bg-white/10">
                  Color
                  <input
                    type="color"
                    aria-label="Column color"
                    value={status.color}
                    onChange={(e) =>
                      apply.mutate(ops.updateStatus({ id: status.id, color: e.target.value }))
                    }
                    className="h-4 w-6 cursor-pointer border-0 bg-transparent p-0"
                  />
                </label>
                <button
                  type="button"
                  aria-label="Move column left"
                  disabled={index === 0}
                  className="rounded px-2 py-1 text-left text-xs hover:bg-black/5 disabled:opacity-40 dark:hover:bg-white/10"
                  onClick={() =>
                    apply.mutate(ops.reorderStatus({ id: status.id, new_position: index - 1 }))
                  }
                >
                  Move left
                </button>
                <button
                  type="button"
                  aria-label="Move column right"
                  disabled={index === count - 1}
                  className="rounded px-2 py-1 text-left text-xs hover:bg-black/5 disabled:opacity-40 dark:hover:bg-white/10"
                  onClick={() =>
                    apply.mutate(ops.reorderStatus({ id: status.id, new_position: index + 1 }))
                  }
                >
                  Move right
                </button>
                <button
                  type="button"
                  className="rounded px-2 py-1 text-left text-xs text-red-600 hover:bg-black/5 dark:text-red-400 dark:hover:bg-white/10"
                  onClick={() => {
                    setError(null);
                    apply.mutate(ops.deleteStatus({ id: status.id }), {
                      onError: (e) => setError(messageOf(e) ?? "Can't delete this column"),
                      onSuccess: () => setMenuOpen(false),
                    });
                  }}
                >
                  Delete
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
      {error && <p className="px-1 text-xs text-red-500">{error}</p>}
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
