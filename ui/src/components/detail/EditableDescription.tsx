import { useState } from "react";
import { MarkdownView } from "./MarkdownView";

export function EditableDescription({
  value,
  onCommit,
}: {
  value: string;
  onCommit: (next: string | null) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  if (!editing) {
    return (
      <div>
        <div className="mb-2 flex items-center justify-between">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-500">
            Description
          </span>
          <button
            type="button"
            onClick={() => {
              setDraft(value);
              setEditing(true);
            }}
            className="rounded-md border border-black/15 px-2 py-0.5 text-[11px] hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
          >
            Edit
          </button>
        </div>
        <MarkdownView source={value || "_no description_"} />
      </div>
    );
  }

  return (
    <div>
      <textarea
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        rows={10}
        className="w-full rounded-md border border-black/15 bg-transparent p-2 font-mono text-sm dark:border-white/15"
      />
      <div className="mt-2 flex justify-end gap-2 text-sm">
        <button type="button" onClick={() => setEditing(false)} className="px-3 py-1">
          Cancel
        </button>
        <button
          type="button"
          onClick={() => {
            onCommit(draft.length ? draft : null);
            setEditing(false);
          }}
          className="rounded-md bg-blue-600 px-3 py-1 text-white hover:bg-blue-700"
        >
          Save
        </button>
      </div>
    </div>
  );
}
