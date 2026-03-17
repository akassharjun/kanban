import { useMemo } from "react";
import {
  Bot,
  Play,
  Check,
  XCircle,
  Pencil,
  BookOpen,
  Zap,
  X,
  Brain,
  ArrowRight,
  Flag,
  User,
  Edit,
  MessageCircle,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { ActivityLogEntry, Comment, Member } from "@/types";

export interface TimelineEntry {
  id: string;
  timestamp: string;
  type: string;
  source: "execution" | "activity" | "comment";
  title: string;
  detail: string;
  metadata?: Record<string, unknown>;
  memberName?: string;
  agentType?: string;
  /** Original comment object, kept so parent can handle edit/delete */
  _comment?: Comment;
}

interface IconConfig {
  icon: LucideIcon;
  color: string;
  bg: string;
}

const EXECUTION_ICONS: Record<string, IconConfig> = {
  claim:     { icon: Bot,       color: "text-blue-400",   bg: "bg-blue-400/10" },
  start:     { icon: Play,      color: "text-blue-400",   bg: "bg-blue-400/10" },
  complete:  { icon: Check,     color: "text-green-400",  bg: "bg-green-400/10" },
  fail:      { icon: X,         color: "text-red-400",    bg: "bg-red-400/10" },
  file_edit: { icon: Pencil,    color: "text-yellow-400", bg: "bg-yellow-400/10" },
  file_read: { icon: BookOpen,  color: "text-cyan-400",   bg: "bg-cyan-400/10" },
  command:   { icon: Zap,       color: "text-orange-400", bg: "bg-orange-400/10" },
  error:     { icon: XCircle,   color: "text-red-400",    bg: "bg-red-400/10" },
  reasoning: { icon: Brain,     color: "text-purple-400", bg: "bg-purple-400/10" },
};

const ACTIVITY_ICONS: Record<string, IconConfig> = {
  status_id:   { icon: ArrowRight,  color: "text-zinc-400", bg: "bg-zinc-400/10" },
  priority:    { icon: Flag,        color: "text-zinc-400", bg: "bg-zinc-400/10" },
  assignee_id: { icon: User,        color: "text-zinc-400", bg: "bg-zinc-400/10" },
};

const COMMENT_ICON: IconConfig = {
  icon: MessageCircle, color: "text-indigo-400", bg: "bg-indigo-400/10",
};

const DEFAULT_ACTIVITY_ICON: IconConfig = {
  icon: Edit, color: "text-zinc-400", bg: "bg-zinc-400/10",
};

function getIconConfig(entry: TimelineEntry): IconConfig {
  if (entry.source === "execution") {
    return EXECUTION_ICONS[entry.type] || EXECUTION_ICONS.command;
  }
  if (entry.source === "comment") {
    return COMMENT_ICON;
  }
  return ACTIVITY_ICONS[entry.type] || DEFAULT_ACTIVITY_ICON;
}

function sourceLabel(source: TimelineEntry["source"]): string {
  switch (source) {
    case "execution": return "Execution";
    case "activity":  return "Activity";
    case "comment":   return "Comment";
  }
}

function sourceBadgeColor(source: TimelineEntry["source"]): string {
  switch (source) {
    case "execution": return "bg-blue-500/15 text-blue-400";
    case "activity":  return "bg-zinc-500/15 text-zinc-400";
    case "comment":   return "bg-indigo-500/15 text-indigo-400";
  }
}

function formatTimestamp(ts: string): string {
  // Show date + time truncated to minute
  return ts.slice(0, 16).replace("T", " ");
}

// -- normalizers --

export function normalizeActivity(entries: ActivityLogEntry[]): TimelineEntry[] {
  return entries.map((e) => {
    const isComment = e.field_changed === "comment";
    return {
      id: `activity-${e.id}`,
      timestamp: e.timestamp,
      type: isComment ? "comment_log" : e.field_changed,
      source: "activity" as const,
      title: isComment
        ? "Comment"
        : `Changed ${e.field_changed}`,
      detail: isComment
        ? (e.new_value || "")
        : [
            e.old_value ? `from ${e.old_value}` : "",
            e.new_value ? `to ${e.new_value}` : "",
          ].filter(Boolean).join(" "),
    };
  });
}

export function normalizeComments(
  comments: Comment[],
  members: Member[],
): TimelineEntry[] {
  return comments.map((c) => {
    const member = members.find((m) => m.id === c.member_id);
    return {
      id: `comment-${c.id}`,
      timestamp: c.created_at,
      type: "comment",
      source: "comment" as const,
      title: member ? (member.display_name || member.name) : "System",
      detail: c.content,
      memberName: member ? (member.display_name || member.name) : undefined,
      _comment: c,
    };
  });
}

// -- component --

interface UnifiedTimelineProps {
  activity: ActivityLogEntry[];
  comments: Comment[];
  members: Member[];
  /** Optional execution logs when they become available */
  executionLogs?: TimelineEntry[];
  /** Callbacks for comment actions */
  onEditComment?: (comment: Comment) => void;
  onDeleteComment?: (commentId: number) => void;
}

export function UnifiedTimeline({
  activity,
  comments,
  members,
  executionLogs = [],
  onEditComment,
  onDeleteComment,
}: UnifiedTimelineProps) {
  const entries = useMemo(() => {
    const normalized = [
      ...normalizeActivity(activity),
      ...normalizeComments(comments, members),
      ...executionLogs,
    ];
    // Sort ascending by timestamp
    normalized.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    return normalized;
  }, [activity, comments, members, executionLogs]);

  if (entries.length === 0) {
    return (
      <p className="py-4 text-center text-xs text-muted-foreground">
        No activity yet.
      </p>
    );
  }

  return (
    <div className="relative pl-4">
      {/* Vertical timeline line */}
      <div className="absolute left-[21px] top-0 bottom-0 w-px bg-zinc-700" />

      <div className="space-y-3">
        {entries.map((entry) => {
          const { icon: Icon, color, bg } = getIconConfig(entry);
          const isComment = entry.source === "comment" && entry._comment;

          return (
            <div key={entry.id} className="group relative flex gap-3">
              {/* Icon dot */}
              <div
                className={cn(
                  "relative z-10 flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full",
                  bg,
                )}
              >
                <Icon className={cn("h-3 w-3", color)} />
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0 pb-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className={cn("rounded-full px-1.5 py-0.5 text-[10px] font-medium", sourceBadgeColor(entry.source))}>
                    {sourceLabel(entry.source)}
                  </span>
                  <span className="text-xs font-medium text-foreground">
                    {entry.title}
                  </span>
                  <span className="ml-auto text-[10px] text-muted-foreground whitespace-nowrap">
                    {formatTimestamp(entry.timestamp)}
                  </span>
                  {/* Comment edit/delete actions */}
                  {isComment && (onEditComment || onDeleteComment) && (
                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      {onEditComment && (
                        <button
                          onClick={() => onEditComment(entry._comment!)}
                          className="rounded p-0.5 hover:bg-accent"
                        >
                          <Pencil className="h-3 w-3 text-muted-foreground" />
                        </button>
                      )}
                      {onDeleteComment && (
                        <button
                          onClick={() => onDeleteComment(entry._comment!.id)}
                          className="rounded p-0.5 hover:bg-destructive/20"
                        >
                          <X className="h-3 w-3 text-muted-foreground" />
                        </button>
                      )}
                    </div>
                  )}
                </div>
                {entry.detail && (
                  <p className="mt-0.5 text-xs text-muted-foreground break-words">
                    {entry.detail}
                  </p>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
