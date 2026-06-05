import { useState } from "react";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";
import { uuidv7 } from "@/lib/uuid";

export function NewIssueInline({
  projectId,
  projectPrefix,
  statusId,
  onDone,
}: {
  projectId: string;
  projectPrefix: string;
  statusId: string;
  onDone: () => void;
}) {
  const [title, setTitle] = useState("");
  const apply = useApply(projectPrefix);
  return (
    <div className="border-t border-black/10 p-3 dark:border-white/10">
      <input
        autoFocus
        placeholder="New issue title…"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") onDone();
          if (e.key === "Enter" && title.trim()) {
            apply.mutate(
              ops.createIssue({
                id: uuidv7(),
                project_id: projectId,
                title: title.trim(),
                description: null,
                status_id: statusId,
                priority: "medium",
                due_date: null,
                label_ids: [],
              }),
            );
            setTitle("");
            onDone();
          }
        }}
        className="w-full rounded-md border border-black/15 bg-white px-2 py-1.5 text-sm dark:border-white/15 dark:bg-neutral-900"
      />
    </div>
  );
}
