import { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import type { IssueHistoryEntry, Status, Member } from "@/types";
import * as api from "@/tauri/commands";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus, ArrowRight } from "lucide-react";

interface IssueHistoryPanelProps {
  issueId: number;
  statuses: Status[];
  members: Member[];
  createdAt: string;
}

const priorities: Record<string, { label: string; icon: typeof AlertCircle; color: string }> = {
  urgent: { label: "Urgent", icon: AlertCircle, color: "text-red-500" },
  high: { label: "High", icon: SignalHigh, color: "text-orange-500" },
  medium: { label: "Medium", icon: SignalMedium, color: "text-yellow-500" },
  low: { label: "Low", icon: SignalLow, color: "text-blue-400" },
  none: { label: "None", icon: Minus, color: "text-muted-foreground" },
};

/** Simple inline diff: show removed (red) and added (green) lines */
function TextDiff({ oldText, newText }: { oldText: string; newText: string }) {
  const oldLines = (oldText || "").split("\n");
  const newLines = (newText || "").split("\n");

  // Simple line-by-line diff
  const maxLen = Math.max(oldLines.length, newLines.length);
  const diffs: { type: "same" | "removed" | "added"; text: string }[] = [];

  for (let i = 0; i < maxLen; i++) {
    const ol = oldLines[i];
    const nl = newLines[i];
    if (ol === nl) {
      if (ol !== undefined) diffs.push({ type: "same", text: ol });
    } else {
      if (ol !== undefined) diffs.push({ type: "removed", text: ol });
      if (nl !== undefined) diffs.push({ type: "added", text: nl });
    }
  }

  return (
    <div className="mt-1 rounded-md border border-border bg-muted/30 p-2 text-xs font-mono overflow-x-auto">
      {diffs.map((d, i) => (
        <div
          key={i}
          className={cn(
            "px-1",
            d.type === "removed" && "bg-red-500/10 text-red-400 line-through",
            d.type === "added" && "bg-green-500/10 text-green-400"
          )}
        >
          {d.type === "removed" && "- "}
          {d.type === "added" && "+ "}
          {d.type === "same" && "  "}
          {d.text || "\u00A0"}
        </div>
      ))}
    </div>
  );
}

function StatusBadge({ statusId, statuses }: { statusId: string; statuses: Status[] }) {
  const status = statuses.find((s) => s.id.toString() === statusId);
  if (!status) return <span className="text-xs text-muted-foreground">{statusId}</span>;
  return (
    <span className="inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium" style={{ backgroundColor: (status.color || "#6b7280") + "18", color: status.color || "#6b7280" }}>
      <span className="h-2 w-2 rounded-full" style={{ backgroundColor: status.color || "#6b7280" }} />
      {status.name}
    </span>
  );
}

function PriorityBadge({ priority }: { priority: string }) {
  const p = priorities[priority] || priorities.none;
  const Icon = p.icon;
  return (
    <span className="inline-flex items-center gap-1 text-xs">
      <Icon className={cn("h-3 w-3", p.color)} />
      {p.label}
    </span>
  );
}

function MemberBadge({ memberId, members }: { memberId: string; members: Member[] }) {
  const member = members.find((m) => m.id.toString() === memberId);
  if (!member) return <span className="text-xs text-muted-foreground/60">Unassigned</span>;
  return (
    <span className="inline-flex items-center gap-1.5 text-xs">
      <div
        className="flex h-4 w-4 items-center justify-center rounded-full text-[8px] font-semibold text-white"
        style={{ backgroundColor: member.avatar_color }}
      >
        {(member.display_name || member.name).charAt(0).toUpperCase()}
      </div>
      {member.display_name || member.name}
    </span>
  );
}

function ActorAvatar({ entry }: { entry: IssueHistoryEntry }) {
  if (!entry.actor_name) {
    return (
      <div className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-muted text-[9px] font-medium text-muted-foreground">
        S
      </div>
    );
  }
  return (
    <div
      className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full text-[9px] font-semibold text-white"
      style={{ backgroundColor: entry.actor_avatar_color || "#6366f1" }}
      title={entry.actor_name}
    >
      {entry.actor_name.charAt(0).toUpperCase()}
    </div>
  );
}

function ChangeDetail({ entry, statuses, members }: { entry: IssueHistoryEntry; statuses: Status[]; members: Member[] }) {
  const { field_changed, old_value, new_value } = entry;

  if (field_changed === "status_id") {
    return (
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-xs text-muted-foreground">Changed status</span>
        {old_value && <StatusBadge statusId={old_value} statuses={statuses} />}
        <ArrowRight className="h-3 w-3 text-muted-foreground/40" />
        {new_value && <StatusBadge statusId={new_value} statuses={statuses} />}
      </div>
    );
  }

  if (field_changed === "priority") {
    return (
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-xs text-muted-foreground">Changed priority</span>
        {old_value && <PriorityBadge priority={old_value} />}
        <ArrowRight className="h-3 w-3 text-muted-foreground/40" />
        {new_value && <PriorityBadge priority={new_value} />}
      </div>
    );
  }

  if (field_changed === "assignee_id") {
    return (
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-xs text-muted-foreground">Changed assignee</span>
        {old_value ? <MemberBadge memberId={old_value} members={members} /> : <span className="text-xs text-muted-foreground/60">Unassigned</span>}
        <ArrowRight className="h-3 w-3 text-muted-foreground/40" />
        {new_value ? <MemberBadge memberId={new_value} members={members} /> : <span className="text-xs text-muted-foreground/60">Unassigned</span>}
      </div>
    );
  }

  if (field_changed === "description" || field_changed === "title") {
    const isLong = ((old_value || "").length + (new_value || "").length) > 80;
    if (isLong && field_changed === "description") {
      return (
        <div>
          <span className="text-xs text-muted-foreground">Updated {field_changed}</span>
          <TextDiff oldText={old_value || ""} newText={new_value || ""} />
        </div>
      );
    }
    return (
      <div className="text-xs">
        <span className="text-muted-foreground">Changed {field_changed} </span>
        {old_value && <><span className="text-red-400 line-through">{old_value}</span> </>}
        <ArrowRight className="inline h-3 w-3 text-muted-foreground/40" />
        {new_value && <> <span className="text-green-400">{new_value}</span></>}
      </div>
    );
  }

  // Generic fallback
  return (
    <div className="text-xs">
      <span className="text-muted-foreground">Changed {field_changed}</span>
      {old_value && <span className="text-muted-foreground/60"> from <span className="text-foreground/80">{old_value}</span></span>}
      {new_value && <span className="text-muted-foreground/60"> to <span className="text-foreground/80">{new_value}</span></span>}
    </div>
  );
}

/** Group entries by timestamp (same second = same group) */
function groupByTimestamp(entries: IssueHistoryEntry[]): IssueHistoryEntry[][] {
  const groups: IssueHistoryEntry[][] = [];
  let currentGroup: IssueHistoryEntry[] = [];
  let currentTs = "";

  for (const entry of entries) {
    const ts = entry.timestamp.slice(0, 19); // trim to seconds
    if (ts !== currentTs && currentGroup.length > 0) {
      groups.push(currentGroup);
      currentGroup = [];
    }
    currentTs = ts;
    currentGroup.push(entry);
  }
  if (currentGroup.length > 0) groups.push(currentGroup);
  return groups;
}

export function IssueHistoryPanel({ issueId, statuses, members, createdAt }: IssueHistoryPanelProps) {
  const [history, setHistory] = useState<IssueHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    api.getIssueHistory(issueId)
      .then(setHistory)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [issueId]);

  if (loading) {
    return <div className="py-4 text-center text-xs text-muted-foreground/40">Loading history...</div>;
  }

  const groups = groupByTimestamp(history);

  return (
    <div className="space-y-0">
      {/* Timeline entries */}
      {groups.map((group, gi) => {
        const first = group[0];
        return (
          <div key={gi} className="relative flex gap-3 pb-4">
            {/* Timeline line */}
            <div className="flex flex-col items-center">
              <ActorAvatar entry={first} />
              {(gi < groups.length - 1 || createdAt) && (
                <div className="mt-1 w-px flex-1 bg-border/50" />
              )}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0 pt-0.5">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-[13px] font-medium">
                  {first.actor_name || "System"}
                </span>
                <span className="text-[11px] text-muted-foreground/40">
                  {first.timestamp.slice(0, 16).replace("T", " ")}
                </span>
              </div>
              <div className="space-y-1.5">
                {group.map((entry) => (
                  <ChangeDetail key={entry.id} entry={entry} statuses={statuses} members={members} />
                ))}
              </div>
            </div>
          </div>
        );
      })}

      {/* Created entry */}
      <div className="relative flex gap-3">
        <div className="flex flex-col items-center">
          <div className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-primary/20 text-[9px] font-semibold text-primary">
            +
          </div>
        </div>
        <div className="flex-1 pt-0.5">
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Issue created</span>
            <span className="text-[11px] text-muted-foreground/40">
              {createdAt.slice(0, 16).replace("T", " ")}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
