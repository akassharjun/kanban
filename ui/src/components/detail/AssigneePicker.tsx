import { useState } from "react";
import { useMembers } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { uuidv7 } from "@/lib/uuid";
import type { MemberDto } from "@/data/bindings";

export function AssigneePicker({
  prefix,
  issueId,
  projectId,
  assigneeId,
}: {
  prefix: string;
  issueId: string;
  projectId: string;
  assigneeId: string | null;
}) {
  const { data: members = [] } = useMembers(prefix);
  const apply = useApply(prefix);
  const [name, setName] = useState("");

  const assign = (value: string) => {
    apply.mutate(
      ops.updateIssueField({ id: issueId, change: change.assignee(value || null) }),
    );
  };

  const add = () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    apply.mutate(ops.createMember({ id: uuidv7(), project_id: projectId, name: trimmed }));
    setName("");
  };

  const remove = (member: MemberDto) => {
    apply.mutate(ops.deleteMember({ id: member.id }));
  };

  return (
    <div className="flex flex-col gap-1.5">
      <label className="flex items-center gap-2 text-neutral-500">
        <span>Assignee</span>
        <select
          aria-label="Assignee"
          value={assigneeId ?? ""}
          onChange={(e) => assign(e.target.value)}
          className="rounded border border-black/15 bg-transparent px-1.5 py-0.5 text-xs dark:border-white/15"
        >
          <option value="">Unassigned</option>
          {members.map((m: MemberDto) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>
      </label>

      <div className="flex items-center gap-1.5">
        <input
          type="text"
          aria-label="New member name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") add();
          }}
          placeholder="Add member"
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

      {members.length > 0 && (
        <ul className="flex flex-wrap gap-1">
          {members.map((m: MemberDto) => (
            <li
              key={m.id}
              className="inline-flex items-center gap-1 rounded-full border border-black/10 px-2 py-0.5 dark:border-white/10"
            >
              <span>{m.name}</span>
              <button
                type="button"
                aria-label={`Remove member ${m.name}`}
                onClick={() => remove(m)}
                className="text-neutral-500 hover:text-neutral-900 dark:hover:text-white"
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
