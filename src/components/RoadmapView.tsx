import { useState, useMemo } from "react";
import { cn } from "@/lib/utils";
import type { Issue, Epic, MilestoneWithProgress, Status } from "@/types";

type ZoomLevel = "week" | "month" | "quarter";

interface RoadmapViewProps {
  issues: Issue[];
  epics: Epic[];
  milestones: MilestoneWithProgress[];
  statuses: Status[];
  onClickIssue: (issue: Issue) => void;
}

function startOfWeek(date: Date): Date {
  const d = new Date(date);
  d.setDate(d.getDate() - d.getDay() + 1);
  d.setHours(0, 0, 0, 0);
  return d;
}

function addDays(date: Date, days: number): Date {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d;
}

function formatDate(d: Date): string {
  return d.toISOString().split("T")[0];
}

function getColumnWidth(zoom: ZoomLevel): number {
  switch (zoom) {
    case "week": return 120;
    case "month": return 160;
    case "quarter": return 200;
  }
}

function getColumnDays(zoom: ZoomLevel): number {
  switch (zoom) {
    case "week": return 7;
    case "month": return 30;
    case "quarter": return 91;
  }
}

function getColumnLabel(date: Date, zoom: ZoomLevel): string {
  if (zoom === "week") {
    const month = date.toLocaleString("default", { month: "short" });
    return `${month} ${date.getDate()}`;
  }
  if (zoom === "month") {
    return date.toLocaleString("default", { month: "long", year: "numeric" });
  }
  const q = Math.floor(date.getMonth() / 3) + 1;
  return `Q${q} ${date.getFullYear()}`;
}

function generateColumns(zoom: ZoomLevel, totalColumns: number): Date[] {
  const now = new Date();
  const start = startOfWeek(now);
  // Go back a few columns so we show some past context
  const offset = zoom === "week" ? 2 : 1;
  const begin = addDays(start, -offset * getColumnDays(zoom));
  const cols: Date[] = [];
  for (let i = 0; i < totalColumns; i++) {
    cols.push(addDays(begin, i * getColumnDays(zoom)));
  }
  return cols;
}

interface BarInfo {
  type: "epic" | "milestone";
  id: number;
  title: string;
  color: string;
  startDate: string;
  endDate: string;
  issues: Issue[];
  progress?: number; // 0-1 for milestones
}

export function RoadmapView({ issues, epics, milestones, statuses, onClickIssue }: RoadmapViewProps) {
  const [zoom, setZoom] = useState<ZoomLevel>("month");
  const totalColumns = zoom === "week" ? 16 : zoom === "month" ? 8 : 6;
  const columns = useMemo(() => generateColumns(zoom, totalColumns), [zoom, totalColumns]);
  const colWidth = getColumnWidth(zoom);
  const colDays = getColumnDays(zoom);
  const rowHeight = 56;
  const headerHeight = 40;
  const labelWidth = 200;

  const completedCategories = useMemo(() => {
    const cats = new Set<number>();
    statuses.filter(s => s.category === "completed" || s.category === "discarded").forEach(s => cats.add(s.id));
    return cats;
  }, [statuses]);

  // Build bars from epics and milestones
  const bars: BarInfo[] = useMemo(() => {
    const result: BarInfo[] = [];

    // Epics - span from earliest issue created_at to latest due_date or now+30d
    for (const epic of epics) {
      const epicIssues = issues.filter(i => i.epic_id === epic.id);
      if (epicIssues.length === 0 && epic.status === "closed") continue;

      const dates = epicIssues
        .map(i => i.due_date || i.created_at.split("T")[0])
        .filter(Boolean)
        .sort();

      const startDate = epic.created_at.split("T")[0];
      const endDate = dates.length > 0
        ? dates[dates.length - 1]
        : formatDate(addDays(new Date(), 30));

      result.push({
        type: "epic",
        id: epic.id,
        title: epic.title,
        color: epic.color,
        startDate,
        endDate: endDate > startDate ? endDate : formatDate(addDays(new Date(startDate), 14)),
        issues: epicIssues,
      });
    }

    // Milestones
    for (const ms of milestones) {
      const msIssues = issues.filter(i => i.milestone_id === ms.id);
      const startDate = ms.created_at.split("T")[0];
      const endDate = ms.due_date || formatDate(addDays(new Date(), 30));
      const progress = ms.total_issues > 0 ? ms.completed_issues / ms.total_issues : 0;

      result.push({
        type: "milestone",
        id: ms.id,
        title: ms.title,
        color: ms.status === "closed" ? "#22c55e" : "#3b82f6",
        startDate,
        endDate,
        issues: msIssues,
        progress,
      });
    }

    return result;
  }, [epics, milestones, issues]);

  const timelineStart = columns[0];
  const totalDays = totalColumns * colDays;

  const getBarPosition = (startDate: string, endDate: string) => {
    const start = new Date(startDate);
    const end = new Date(endDate);
    const startOffset = (start.getTime() - timelineStart.getTime()) / (1000 * 60 * 60 * 24);
    const endOffset = (end.getTime() - timelineStart.getTime()) / (1000 * 60 * 60 * 24);
    const pxPerDay = colWidth / colDays;

    return {
      left: Math.max(0, startOffset * pxPerDay),
      width: Math.max(40, (Math.min(endOffset, totalDays) - Math.max(startOffset, 0)) * pxPerDay),
    };
  };

  // Today marker
  const todayOffset = (() => {
    const now = new Date();
    const diff = (now.getTime() - timelineStart.getTime()) / (1000 * 60 * 60 * 24);
    return diff * (colWidth / colDays);
  })();

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center gap-3 px-4 py-2 border-b border-border/50">
        <span className="text-xs text-muted-foreground/60">Zoom:</span>
        <div className="flex items-center rounded-lg bg-muted/50 p-0.5">
          {(["week", "month", "quarter"] as ZoomLevel[]).map(z => (
            <button
              key={z}
              onClick={() => setZoom(z)}
              className={cn(
                "rounded-md px-2.5 py-1 text-xs font-medium transition-all capitalize",
                zoom === z
                  ? "bg-card text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {z}
            </button>
          ))}
        </div>
        <div className="ml-4 flex items-center gap-3 text-xs text-muted-foreground/60">
          <span className="flex items-center gap-1.5">
            <span className="h-2.5 w-6 rounded-sm bg-indigo-500/60" />
            Epics
          </span>
          <span className="flex items-center gap-1.5">
            <span className="h-2.5 w-6 rounded-sm bg-blue-500/60" />
            Milestones
          </span>
        </div>
      </div>

      {/* Timeline */}
      <div className="flex-1 overflow-auto">
        <div className="relative" style={{ minWidth: labelWidth + colWidth * totalColumns }}>
          {/* Header row */}
          <div className="sticky top-0 z-10 flex bg-background border-b border-border/50">
            <div className="flex-shrink-0 border-r border-border/30 bg-background" style={{ width: labelWidth, height: headerHeight }}>
              <div className="flex items-center h-full px-3 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/50">
                Item
              </div>
            </div>
            {columns.map((col, i) => (
              <div
                key={i}
                className="flex-shrink-0 border-r border-border/20 flex items-center justify-center text-[11px] text-muted-foreground/60"
                style={{ width: colWidth, height: headerHeight }}
              >
                {getColumnLabel(col, zoom)}
              </div>
            ))}
          </div>

          {/* Rows */}
          {bars.length === 0 ? (
            <div className="flex items-center justify-center py-16 text-sm text-muted-foreground/50">
              No epics or milestones to display. Create some in Project Settings.
            </div>
          ) : (
            bars.map((bar) => {
              const pos = getBarPosition(bar.startDate, bar.endDate);
              const completedCount = bar.issues.filter(i => completedCategories.has(i.status_id)).length;

              return (
                <div key={`${bar.type}-${bar.id}`} className="flex" style={{ height: rowHeight }}>
                  {/* Label */}
                  <div
                    className="flex-shrink-0 border-r border-border/30 flex items-center px-3 gap-2"
                    style={{ width: labelWidth }}
                  >
                    <span
                      className="h-2.5 w-2.5 rounded-full flex-shrink-0"
                      style={{ backgroundColor: bar.color }}
                    />
                    <span className="text-[13px] font-medium truncate">{bar.title}</span>
                    <span className="ml-auto text-[10px] text-muted-foreground/40 flex-shrink-0">
                      {completedCount}/{bar.issues.length}
                    </span>
                  </div>

                  {/* Timeline area */}
                  <div
                    className="relative flex-1 border-b border-border/10"
                    style={{ minWidth: colWidth * totalColumns }}
                  >
                    {/* Column grid lines */}
                    {columns.map((_, i) => (
                      <div
                        key={i}
                        className="absolute top-0 bottom-0 border-r border-border/10"
                        style={{ left: i * colWidth, width: colWidth }}
                      />
                    ))}

                    {/* Today line */}
                    {todayOffset > 0 && todayOffset < colWidth * totalColumns && (
                      <div
                        className="absolute top-0 bottom-0 w-px bg-red-500/40 z-[5]"
                        style={{ left: todayOffset }}
                      />
                    )}

                    {/* Bar */}
                    <div
                      className="absolute top-2.5 rounded-md flex items-center overflow-hidden group cursor-default"
                      style={{
                        left: pos.left,
                        width: pos.width,
                        height: rowHeight - 20,
                        backgroundColor: bar.color + "25",
                        borderLeft: `3px solid ${bar.color}`,
                      }}
                      title={`${bar.title} (${completedCount}/${bar.issues.length} done)`}
                    >
                      {/* Progress fill for milestones */}
                      {bar.type === "milestone" && bar.progress !== undefined && (
                        <div
                          className="absolute inset-0 rounded-r-md"
                          style={{
                            width: `${bar.progress * 100}%`,
                            backgroundColor: bar.color + "30",
                          }}
                        />
                      )}

                      {/* Issue dots */}
                      <div className="relative flex items-center gap-0.5 px-2 z-10">
                        {bar.issues.slice(0, 12).map(issue => (
                          <div
                            key={issue.id}
                            onClick={() => onClickIssue(issue)}
                            className="h-2 w-2 rounded-full cursor-pointer hover:ring-2 hover:ring-white/50 transition-all"
                            style={{
                              backgroundColor: completedCategories.has(issue.status_id)
                                ? "#22c55e"
                                : bar.color,
                            }}
                            title={`${issue.identifier}: ${issue.title}`}
                          />
                        ))}
                        {bar.issues.length > 12 && (
                          <span className="text-[9px] text-muted-foreground/60 ml-1">
                            +{bar.issues.length - 12}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
