import { useState } from "react";
import { useLabels } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";
import { uuidv7 } from "@/lib/uuid";
import type { LabelDto } from "@/data/bindings";

const DEFAULT_COLOR = "#64748b";

export function LabelPicker({
  prefix,
  issueId,
  projectId,
  attached,
}: {
  prefix: string;
  issueId: string;
  projectId: string;
  attached: LabelDto[];
}) {
  const { data: labels = [] } = useLabels(prefix);
  const apply = useApply(prefix);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [color, setColor] = useState(DEFAULT_COLOR);

  const attachedIds = new Set(attached.map((l) => l.id));

  const toggle = (label: LabelDto) => {
    if (attachedIds.has(label.id)) {
      apply.mutate(ops.detachLabel({ issue_id: issueId, label_id: label.id }));
    } else {
      apply.mutate(ops.attachLabel({ issue_id: issueId, label_id: label.id }));
    }
  };

  const add = () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    apply.mutate(ops.createLabel({ id: uuidv7(), project_id: projectId, name: trimmed, color }));
    setName("");
    setColor(DEFAULT_COLOR);
  };

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className="rounded border border-black/15 px-1.5 py-0.5 text-xs text-neutral-500 hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
      >
        + Labels
      </button>

      {open && (
        <div className="absolute left-0 z-50 mt-1 w-56 rounded-md border border-black/10 bg-white p-1.5 shadow-lg dark:border-white/10 dark:bg-neutral-900">
          <ul className="max-h-48 overflow-y-auto">
            {labels.map((l: LabelDto) => {
              const on = attachedIds.has(l.id);
              return (
                <li key={l.id}>
                  <button
                    type="button"
                    onClick={() => toggle(l)}
                    aria-pressed={on}
                    className="flex w-full items-center gap-2 rounded px-1.5 py-1 text-left text-xs hover:bg-black/5 dark:hover:bg-white/10"
                  >
                    <span
                      aria-hidden
                      className="h-2.5 w-2.5 shrink-0 rounded-full border border-black/10"
                      style={{ backgroundColor: l.color }}
                    />
                    <span className="flex-1">{l.name}</span>
                    {on && <span aria-hidden>✓</span>}
                  </button>
                </li>
              );
            })}
          </ul>

          <div className="mt-1 flex items-center gap-1.5 border-t border-black/10 pt-1.5 dark:border-white/10">
            <input
              type="color"
              aria-label="New label color"
              value={color}
              onChange={(e) => setColor(e.target.value)}
              className="h-6 w-6 shrink-0 cursor-pointer rounded border border-black/15 bg-transparent dark:border-white/15"
            />
            <input
              type="text"
              aria-label="New label name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") add();
              }}
              placeholder="New label"
              className="min-w-0 flex-1 rounded border border-black/15 bg-transparent px-1.5 py-0.5 text-xs dark:border-white/15"
            />
            <button
              type="button"
              onClick={add}
              className="rounded border border-black/15 px-1.5 py-0.5 text-xs hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
            >
              Add
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
