import { useEffect } from "react";
import { useNavigate, useParams } from "react-router";
import { useIssue, useStatuses } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { EditableField } from "./EditableField";
import { EditableDescription } from "./EditableDescription";

export function IssuePanel() {
  const { prefix = "", key = "" } = useParams();
  const navigate = useNavigate();
  const { data: issue } = useIssue(key);
  const { data: statuses = [] } = useStatuses(prefix);
  const apply = useApply(prefix);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") navigate(`/p/${prefix}`);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navigate, prefix]);

  if (!issue) return null;

  const close = () => navigate(`/p/${prefix}`);

  return (
    <>
      <div onClick={close} className="absolute inset-0 z-40 bg-black/40 backdrop-blur-[1px]" />
      <aside className="absolute right-0 top-0 z-50 flex h-full w-[520px] flex-col border-l border-black/10 bg-white shadow-2xl dark:border-white/10 dark:bg-neutral-950">
        <header className="flex items-center justify-between border-b border-black/10 px-4 py-3 text-sm dark:border-white/10">
          <div className="flex items-center gap-3">
            <span className="font-mono text-xs text-neutral-500">{issue.identifier}</span>
            <select
              value={issue.status_id}
              onChange={(e) =>
                apply.mutate(
                  ops.updateIssueField({ id: issue.id, change: change.status(e.target.value) }),
                )
              }
              className="rounded border border-black/15 bg-transparent px-1.5 py-0.5 text-xs dark:border-white/15"
            >
              {statuses.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </select>
          </div>
          <button
            type="button"
            onClick={close}
            className="rounded p-1 text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10"
          >
            ✕
          </button>
        </header>

        <section className="px-5 pt-4">
          <EditableField
            value={issue.title}
            onCommit={(next) =>
              apply.mutate(ops.updateIssueField({ id: issue.id, change: change.title(next) }))
            }
          />
        </section>

        <section className="flex-1 overflow-y-auto px-5 py-3">
          <EditableDescription
            value={issue.description ?? ""}
            onCommit={(next) =>
              apply.mutate(ops.updateIssueField({ id: issue.id, change: change.description(next) }))
            }
          />
        </section>
      </aside>
    </>
  );
}
