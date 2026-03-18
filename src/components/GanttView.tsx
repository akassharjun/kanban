import { useState, useMemo, useRef, useCallback } from "react";
import { cn } from "@/lib/utils";
import { AlertCircle, SignalHigh, SignalMedium, SignalLow, Minus } from "lucide-react";
import type { Issue, Status, Member, Label } from "@/types";

interface GanttViewProps {
  issues: Issue[];
  statuses: Status[];
  members: Member[];
  getLabelsForIssue: (issueId: number) => Label[];
  onClickIssue: (issue: Issue) => void;
}

type ZoomLevel = "day" | "week" | "month";

const priorityColors: Record<string, string> = {
  urgent: "#ef4444",
  high: "#f97316",
  medium: "#3b82f6",
  low: "#9ca3af",
  none: "#6b7280",
};

const priorityIcons: Record<string, React.ElementType> = {
  urgent: AlertCircle,
  high: SignalHigh,
  medium: SignalMedium,
  low: SignalLow,
  none: Minus,
};

const HEADER_HEIGHT = 48;
const ROW_HEIGHT = 40;
const LEFT_PANEL_WIDTH = 300;

function addDays(d: Date, n: number): Date {
  const r = new Date(d);
  r.setDate(r.getDate() + n);
  return r;
}

function diffDays(a: Date, b: Date): number {
  return Math.round((b.getTime() - a.getTime()) / 86400000);
}

function toLocalDate(s: string): Date {
  const [y, m, d] = s.slice(0, 10).split("-").map(Number);
  return new Date(y, m - 1, d);
}

function formatHeaderDate(d: Date, zoom: ZoomLevel): string {
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  if (zoom === "day") return `${d.getDate()}`;
  if (zoom === "week") return `${months[d.getMonth()]} ${d.getDate()}`;
  return `${months[d.getMonth()]} ${d.getFullYear()}`;
}

function getColumnWidth(zoom: ZoomLevel): number {
  if (zoom === "day") return 36;
  if (zoom === "week") return 80;
  return 120;
}

export function GanttView({ issues, statuses, members, onClickIssue }: GanttViewProps) {
  const [zoom, setZoom] = useState<ZoomLevel>("day");
  const [tooltip, setTooltip] = useState<{ issue: Issue; x: number; y: number } | null>(null);
  const timelineRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const completedStatusIds = useMemo(() => {
    return new Set(statuses.filter(s => s.category === "completed").map(s => s.id));
  }, [statuses]);

  const sortedIssues = useMemo(() => {
    return [...issues].sort((a, b) => {
      // Sort by created_at
      return a.created_at.localeCompare(b.created_at);
    });
  }, [issues]);

  // Compute timeline range
  const { startDate, endDate, columns } = useMemo(() => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    let minDate = new Date(today);
    let maxDate = new Date(today);

    for (const issue of sortedIssues) {
      const created = toLocalDate(issue.created_at);
      if (created < minDate) minDate = new Date(created);
      if (issue.due_date) {
        const due = toLocalDate(issue.due_date);
        if (due > maxDate) maxDate = new Date(due);
      }
    }

    // Add padding
    const start = addDays(minDate, -3);
    const end = addDays(maxDate, 7);

    // Generate columns based on zoom
    const cols: { date: Date; label: string }[] = [];
    const cursor = new Date(start);

    if (zoom === "day") {
      while (cursor <= end) {
        cols.push({ date: new Date(cursor), label: formatHeaderDate(cursor, zoom) });
        cursor.setDate(cursor.getDate() + 1);
      }
    } else if (zoom === "week") {
      // Align to Monday
      cursor.setDate(cursor.getDate() - ((cursor.getDay() + 6) % 7));
      while (cursor <= end) {
        cols.push({ date: new Date(cursor), label: formatHeaderDate(cursor, zoom) });
        cursor.setDate(cursor.getDate() + 7);
      }
    } else {
      // Month
      cursor.setDate(1);
      while (cursor <= end) {
        cols.push({ date: new Date(cursor), label: formatHeaderDate(cursor, zoom) });
        cursor.setMonth(cursor.getMonth() + 1);
      }
    }

    return { startDate: start, endDate: end, columns: cols };
  }, [sortedIssues, zoom]);

  const colWidth = getColumnWidth(zoom);
  const totalWidth = columns.length * colWidth;

  // Calculate position of today marker
  const todayOffset = useMemo(() => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const totalDays = diffDays(startDate, endDate);
    if (totalDays <= 0) return 0;
    const dayOffset = diffDays(startDate, today);
    return (dayOffset / totalDays) * totalWidth;
  }, [startDate, endDate, totalWidth]);

  // Compute bar position for an issue
  const getBar = useCallback((issue: Issue) => {
    const totalDays = diffDays(startDate, endDate);
    if (totalDays <= 0) return null;

    const created = toLocalDate(issue.created_at);
    const startOffset = diffDays(startDate, created);
    const leftPct = startOffset / totalDays;

    if (!issue.due_date) {
      // Show as a dot
      return { left: leftPct * totalWidth, width: 0, isDot: true };
    }

    const due = toLocalDate(issue.due_date);
    const duration = diffDays(created, due);
    const widthPct = Math.max(duration, 1) / totalDays;

    return {
      left: leftPct * totalWidth,
      width: widthPct * totalWidth,
      isDot: false,
    };
  }, [startDate, endDate, totalWidth]);

  const getMember = (id: number | null) => (id ? members.find(m => m.id === id) : undefined);

  // Sync vertical scrolling between panels
  const handleTimelineScroll = () => {
    if (timelineRef.current && listRef.current) {
      listRef.current.scrollTop = timelineRef.current.scrollTop;
    }
  };

  const handleListScroll = () => {
    if (listRef.current && timelineRef.current) {
      timelineRef.current.scrollTop = listRef.current.scrollTop;
    }
  };

  const handleBarHover = (e: React.MouseEvent, issue: Issue) => {
    setTooltip({ issue, x: e.clientX, y: e.clientY });
  };

  const handleBarLeave = () => {
    setTooltip(null);
  };

  // Month label row for day/week zoom
  const monthHeaders = useMemo(() => {
    if (zoom === "month") return [];
    const headers: { label: string; span: number }[] = [];
    let currentLabel = "";
    const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    for (const col of columns) {
      const label = `${months[col.date.getMonth()]} ${col.date.getFullYear()}`;
      if (label !== currentLabel) {
        headers.push({ label, span: 1 });
        currentLabel = label;
      } else {
        headers[headers.length - 1].span++;
      }
    }
    return headers;
  }, [columns, zoom]);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Controls */}
      <div className="flex items-center gap-2 border-b border-border px-4 py-2">
        <span className="text-xs font-medium text-muted-foreground">Zoom:</span>
        {(["day", "week", "month"] as ZoomLevel[]).map(level => (
          <button
            key={level}
            onClick={() => setZoom(level)}
            className={cn(
              "rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
              zoom === level
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
          >
            {level.charAt(0).toUpperCase() + level.slice(1)}
          </button>
        ))}
        <span className="ml-auto text-xs text-muted-foreground">
          {sortedIssues.length} issues
        </span>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Left panel - issue list */}
        <div className="flex flex-col border-r border-border" style={{ width: LEFT_PANEL_WIDTH, minWidth: LEFT_PANEL_WIDTH }}>
          {/* Left header */}
          <div
            className="flex items-center border-b border-border px-3 text-xs font-medium text-muted-foreground"
            style={{ height: HEADER_HEIGHT, minHeight: HEADER_HEIGHT }}
          >
            Issue
          </div>

          {/* Left issue list */}
          <div
            ref={listRef}
            onScroll={handleListScroll}
            className="flex-1 overflow-y-auto overflow-x-hidden"
            style={{ scrollbarWidth: "none" }}
          >
            {sortedIssues.map(issue => {
              const member = getMember(issue.assignee_id);
              const PIcon = priorityIcons[issue.priority] ?? priorityIcons.none;
              const pColor = priorityColors[issue.priority] ?? priorityColors.none;

              return (
                <div
                  key={issue.id}
                  onClick={() => onClickIssue(issue)}
                  className="flex cursor-pointer items-center gap-2 border-b border-border/30 px-3 hover:bg-accent/10 transition-colors"
                  style={{ height: ROW_HEIGHT }}
                >
                  <PIcon className="h-3.5 w-3.5 flex-shrink-0" style={{ color: pColor }} />
                  <span className="min-w-0 flex-1 truncate text-xs">{issue.title}</span>
                  {member && (
                    <span
                      className="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full text-[9px] font-medium text-white"
                      style={{ backgroundColor: member.avatar_color }}
                    >
                      {(member.display_name || member.name).charAt(0).toUpperCase()}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Right panel - timeline */}
        <div className="flex flex-1 flex-col overflow-hidden">
          {/* Timeline header */}
          <div
            className="overflow-hidden border-b border-border"
            style={{ height: HEADER_HEIGHT, minHeight: HEADER_HEIGHT }}
          >
            <div style={{ width: totalWidth }} className="h-full">
              {/* Month row (for day/week zoom) */}
              {monthHeaders.length > 0 && (
                <div className="flex h-1/2 border-b border-border/30">
                  {monthHeaders.map((mh, i) => (
                    <div
                      key={i}
                      className="flex items-center border-r border-border/30 px-2 text-[10px] font-medium text-muted-foreground"
                      style={{ width: mh.span * colWidth }}
                    >
                      {mh.label}
                    </div>
                  ))}
                </div>
              )}
              {/* Column labels */}
              <div className={cn("flex", monthHeaders.length > 0 ? "h-1/2" : "h-full")}>
                {columns.map((col, i) => {
                  const isWeekend = col.date.getDay() === 0 || col.date.getDay() === 6;
                  return (
                    <div
                      key={i}
                      className={cn(
                        "flex items-center justify-center border-r border-border/30 text-[10px] text-muted-foreground",
                        isWeekend && "bg-muted/30"
                      )}
                      style={{ width: colWidth, minWidth: colWidth }}
                    >
                      {col.label}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* Timeline body */}
          <div
            ref={timelineRef}
            onScroll={handleTimelineScroll}
            className="flex-1 overflow-auto"
          >
            <div className="relative" style={{ width: totalWidth, height: sortedIssues.length * ROW_HEIGHT }}>
              {/* Grid lines */}
              {columns.map((col, i) => {
                const isWeekend = col.date.getDay() === 0 || col.date.getDay() === 6;
                return (
                  <div
                    key={i}
                    className={cn(
                      "absolute top-0 bottom-0 border-r border-border/20",
                      isWeekend && "bg-muted/15"
                    )}
                    style={{ left: i * colWidth, width: colWidth }}
                  />
                );
              })}

              {/* Today marker */}
              {todayOffset > 0 && todayOffset < totalWidth && (
                <div
                  className="absolute top-0 bottom-0 z-10 w-px border-l-2 border-dashed border-red-500/70"
                  style={{ left: todayOffset }}
                />
              )}

              {/* Issue bars */}
              {sortedIssues.map((issue, idx) => {
                const bar = getBar(issue);
                if (!bar) return null;
                const color = priorityColors[issue.priority] ?? priorityColors.none;
                const isCompleted = completedStatusIds.has(issue.status_id);
                const top = idx * ROW_HEIGHT;

                if (bar.isDot) {
                  return (
                    <div
                      key={issue.id}
                      className="absolute z-20 cursor-pointer"
                      style={{
                        left: bar.left - 5,
                        top: top + ROW_HEIGHT / 2 - 5,
                        width: 10,
                        height: 10,
                      }}
                      onClick={() => onClickIssue(issue)}
                      onMouseEnter={(e) => handleBarHover(e, issue)}
                      onMouseLeave={handleBarLeave}
                    >
                      <div
                        className="h-full w-full rounded-full"
                        style={{ backgroundColor: color }}
                      />
                    </div>
                  );
                }

                return (
                  <div
                    key={issue.id}
                    className="absolute z-20 cursor-pointer"
                    style={{
                      left: bar.left,
                      top: top + 8,
                      width: Math.max(bar.width, 8),
                      height: ROW_HEIGHT - 16,
                    }}
                    onClick={() => onClickIssue(issue)}
                    onMouseEnter={(e) => handleBarHover(e, issue)}
                    onMouseLeave={handleBarLeave}
                  >
                    <div
                      className="h-full w-full rounded-md transition-opacity hover:opacity-80"
                      style={{
                        backgroundColor: color,
                        opacity: isCompleted ? 1 : 0.7,
                      }}
                    >
                      {isCompleted && (
                        <div
                          className="h-full rounded-md"
                          style={{
                            width: "100%",
                            backgroundColor: color,
                            opacity: 0.3,
                            backgroundImage: `repeating-linear-gradient(
                              45deg,
                              transparent,
                              transparent 3px,
                              rgba(255,255,255,0.2) 3px,
                              rgba(255,255,255,0.2) 6px
                            )`,
                          }}
                        />
                      )}
                    </div>
                  </div>
                );
              })}

              {/* Row dividers */}
              {sortedIssues.map((_, idx) => (
                <div
                  key={idx}
                  className="absolute left-0 right-0 border-b border-border/20"
                  style={{ top: (idx + 1) * ROW_HEIGHT }}
                />
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Tooltip */}
      {tooltip && (
        <div
          className="pointer-events-none fixed z-50 max-w-xs rounded-lg border border-border bg-popover px-3 py-2 text-xs text-popover-foreground shadow-lg"
          style={{ left: tooltip.x + 12, top: tooltip.y + 12 }}
        >
          <div className="font-medium">{tooltip.issue.title}</div>
          <div className="mt-1 flex flex-col gap-0.5 text-muted-foreground">
            <span>{tooltip.issue.identifier} &middot; {tooltip.issue.priority}</span>
            <span>Created: {tooltip.issue.created_at.slice(0, 10)}</span>
            {tooltip.issue.due_date && <span>Due: {tooltip.issue.due_date}</span>}
            {tooltip.issue.assignee_id && (
              <span>Assignee: {getMember(tooltip.issue.assignee_id)?.display_name ?? getMember(tooltip.issue.assignee_id)?.name ?? "Unassigned"}</span>
            )}
          </div>
        </div>
      )}

      {sortedIssues.length === 0 && (
        <div className="flex flex-1 items-center justify-center text-muted-foreground">
          No issues found
        </div>
      )}
    </div>
  );
}
