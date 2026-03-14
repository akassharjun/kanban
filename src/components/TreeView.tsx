import { useMemo, useState } from "react";
import { ChevronRight, ChevronDown, AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Issue, Status, Member } from "@/types";

interface TreeViewProps {
  issues: Issue[];
  statuses: Status[];
  members: Member[];
  onClickIssue: (issue: Issue) => void;
}

const priorityConfig: Record<string, { icon: React.ElementType; color: string }> = {
  urgent: { icon: AlertCircle, color: "text-red-500" },
  high: { icon: SignalHigh, color: "text-orange-500" },
  medium: { icon: SignalMedium, color: "text-yellow-500" },
  low: { icon: SignalLow, color: "text-blue-500" },
  none: { icon: Minus, color: "text-muted-foreground" },
};

function TreeNode({
  issue,
  children,
  statuses,
  members,
  depth,
  onClickIssue,
}: {
  issue: Issue;
  children: Issue[];
  statuses: Status[];
  members: Member[];
  depth: number;
  onClickIssue: (issue: Issue) => void;
}) {
  const [expanded, setExpanded] = useState(true);
  const status = statuses.find(s => s.id === issue.status_id);
  const member = members.find(m => m.id === issue.assignee_id);
  const p = priorityConfig[issue.priority] || priorityConfig.none;
  const PIcon = p.icon;
  const hasChildren = children.length > 0;

  return (
    <div>
      <div
        className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent/30 cursor-pointer transition-colors"
        style={{ paddingLeft: `${depth * 24 + 8}px` }}
      >
        {hasChildren ? (
          <button onClick={() => setExpanded(!expanded)} className="p-0.5">
            {expanded ? <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" /> : <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />}
          </button>
        ) : (
          <span className="w-[18px]" />
        )}
        <PIcon className={cn("h-3.5 w-3.5 flex-shrink-0", p.color)} />
        <span className="text-xs text-muted-foreground flex-shrink-0">{issue.identifier}</span>
        <span className="text-sm truncate flex-1 cursor-pointer" onClick={() => onClickIssue(issue)}>{issue.title}</span>
        <span className="h-2 w-2 rounded-full flex-shrink-0" style={{ backgroundColor: status?.color || "#6b7280" }} />
        {member && (
          <div
            className="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-medium text-white"
            style={{ backgroundColor: member.avatar_color }}
          >
            {(member.display_name || member.name).charAt(0).toUpperCase()}
          </div>
        )}
      </div>
      {expanded && hasChildren && (
        <div>
          {children.map(child => (
            <TreeNodeWrapper key={child.id} issue={child} allIssues={[...children]} statuses={statuses} members={members} depth={depth + 1} onClickIssue={onClickIssue} />
          ))}
        </div>
      )}
    </div>
  );
}

function TreeNodeWrapper({
  issue,
  allIssues: _allIssues,
  statuses,
  members,
  depth,
  onClickIssue,
}: {
  issue: Issue;
  allIssues: Issue[];
  statuses: Status[];
  members: Member[];
  depth: number;
  onClickIssue: (issue: Issue) => void;
}) {
  const status = statuses.find(s => s.id === issue.status_id);
  const member = members.find(m => m.id === issue.assignee_id);
  const p = priorityConfig[issue.priority] || priorityConfig.none;
  const PIcon = p.icon;

  return (
    <div
      className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent/30 cursor-pointer transition-colors"
      style={{ paddingLeft: `${depth * 24 + 8 + 18}px` }}
      onClick={() => onClickIssue(issue)}
    >
      <PIcon className={cn("h-3.5 w-3.5 flex-shrink-0", p.color)} />
      <span className="text-xs text-muted-foreground flex-shrink-0">{issue.identifier}</span>
      <span className="text-sm truncate flex-1">{issue.title}</span>
      <span className="h-2 w-2 rounded-full flex-shrink-0" style={{ backgroundColor: status?.color || "#6b7280" }} />
      {member && (
        <div
          className="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full text-[10px] font-medium text-white"
          style={{ backgroundColor: member.avatar_color }}
        >
          {(member.display_name || member.name).charAt(0).toUpperCase()}
        </div>
      )}
    </div>
  );
}

export function TreeView({ issues, statuses, members, onClickIssue }: TreeViewProps) {
  const { roots, childrenMap } = useMemo(() => {
    const childrenMap = new Map<number, Issue[]>();
    const roots: Issue[] = [];

    for (const issue of issues) {
      if (!issue.parent_id) {
        roots.push(issue);
      } else {
        const existing = childrenMap.get(issue.parent_id) || [];
        existing.push(issue);
        childrenMap.set(issue.parent_id, existing);
      }
    }

    roots.sort((a, b) => a.position - b.position);
    for (const [, children] of childrenMap) {
      children.sort((a, b) => a.position - b.position);
    }

    return { roots, childrenMap };
  }, [issues]);

  return (
    <div className="flex-1 overflow-auto p-4">
      {roots.length === 0 ? (
        <div className="py-12 text-center text-muted-foreground">No issues found</div>
      ) : (
        <div className="space-y-0.5">
          {roots.map(issue => (
            <TreeNode
              key={issue.id}
              issue={issue}
              children={childrenMap.get(issue.id) || []}
              statuses={statuses}
              members={members}
              depth={0}
              onClickIssue={onClickIssue}
            />
          ))}
        </div>
      )}
    </div>
  );
}
