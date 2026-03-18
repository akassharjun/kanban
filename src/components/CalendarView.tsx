import { useState, useMemo } from "react";
import { cn } from "@/lib/utils";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { Issue, Member, Label } from "@/types";

interface CalendarViewProps {
  issues: Issue[];
  members: Member[];
  getLabelsForIssue: (issueId: number) => Label[];
  onClickIssue: (issue: Issue) => void;
}

const DAY_NAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

const MAX_VISIBLE_ISSUES = 3;

const priorityColors: Record<string, string> = {
  urgent: "bg-red-500",
  high: "bg-orange-500",
  medium: "bg-yellow-500",
  low: "bg-blue-500",
  none: "bg-gray-400",
};

function toDateKey(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function parseDate(s: string): Date {
  // Parse YYYY-MM-DD as local date (not UTC)
  const [y, m, d] = s.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function CalendarView({ issues, members, onClickIssue }: CalendarViewProps) {
  const [currentMonth, setCurrentMonth] = useState(() => {
    const now = new Date();
    return new Date(now.getFullYear(), now.getMonth(), 1);
  });

  const today = useMemo(() => toDateKey(new Date()), []);

  const issuesByDate = useMemo(() => {
    const map = new Map<string, Issue[]>();
    for (const issue of issues) {
      if (!issue.due_date) continue;
      const key = issue.due_date.slice(0, 10); // YYYY-MM-DD
      const list = map.get(key) ?? [];
      list.push(issue);
      map.set(key, list);
    }
    return map;
  }, [issues]);

  const calendarDays = useMemo(() => {
    const year = currentMonth.getFullYear();
    const month = currentMonth.getMonth();

    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);

    // Start from Sunday of the week containing the first day
    const startDate = new Date(firstDay);
    startDate.setDate(startDate.getDate() - firstDay.getDay());

    // End at Saturday of the week containing the last day
    const endDate = new Date(lastDay);
    endDate.setDate(endDate.getDate() + (6 - lastDay.getDay()));

    const days: { date: Date; key: string; inMonth: boolean }[] = [];
    const cursor = new Date(startDate);
    while (cursor <= endDate) {
      days.push({
        date: new Date(cursor),
        key: toDateKey(cursor),
        inMonth: cursor.getMonth() === month,
      });
      cursor.setDate(cursor.getDate() + 1);
    }
    return days;
  }, [currentMonth]);

  const goToToday = () => {
    const now = new Date();
    setCurrentMonth(new Date(now.getFullYear(), now.getMonth(), 1));
  };

  const prevMonth = () => {
    setCurrentMonth(prev => new Date(prev.getFullYear(), prev.getMonth() - 1, 1));
  };

  const nextMonth = () => {
    setCurrentMonth(prev => new Date(prev.getFullYear(), prev.getMonth() + 1, 1));
  };

  const getMember = (id: number | null) => (id ? members.find(m => m.id === id) : undefined);

  const isPastDue = (dateKey: string) => dateKey < today;

  return (
    <div className="flex flex-1 flex-col overflow-hidden p-4">
      {/* Header */}
      <div className="mb-4 flex items-center gap-3">
        <h2 className="text-lg font-semibold">
          {MONTH_NAMES[currentMonth.getMonth()]} {currentMonth.getFullYear()}
        </h2>
        <div className="flex items-center gap-1">
          <button
            onClick={prevMonth}
            className="rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <ChevronLeft className="h-4 w-4" />
          </button>
          <button
            onClick={nextMonth}
            className="rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
        <button
          onClick={goToToday}
          className="rounded-md border border-border px-2.5 py-1 text-xs font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          Today
        </button>
      </div>

      {/* Day headers */}
      <div className="grid grid-cols-7 border-b border-border">
        {DAY_NAMES.map((name, i) => (
          <div
            key={name}
            className={cn(
              "px-2 py-1.5 text-center text-xs font-medium text-muted-foreground",
              (i === 0 || i === 6) && "opacity-60"
            )}
          >
            {name}
          </div>
        ))}
      </div>

      {/* Calendar grid */}
      <div className="grid flex-1 grid-cols-7 overflow-auto auto-rows-fr">
        {calendarDays.map(({ date, key, inMonth }) => {
          const dayIssues = issuesByDate.get(key) ?? [];
          const isToday = key === today;
          const isWeekend = date.getDay() === 0 || date.getDay() === 6;
          const hasOverflow = dayIssues.length > MAX_VISIBLE_ISSUES;
          const visibleIssues = hasOverflow ? dayIssues.slice(0, MAX_VISIBLE_ISSUES - 1) : dayIssues;

          return (
            <div
              key={key}
              className={cn(
                "min-h-[100px] border-b border-r border-border/50 p-1.5",
                !inMonth && "opacity-40",
                isWeekend && inMonth && "bg-muted/30",
                isToday && "ring-2 ring-inset ring-accent/50"
              )}
            >
              <div
                className={cn(
                  "mb-1 flex h-6 w-6 items-center justify-center rounded-full text-xs font-medium",
                  isToday
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground"
                )}
              >
                {date.getDate()}
              </div>

              <div className="flex flex-col gap-0.5">
                {visibleIssues.map(issue => {
                  const member = getMember(issue.assignee_id);
                  const pastDue = isPastDue(key) && issue.status_id !== undefined;
                  return (
                    <button
                      key={issue.id}
                      onClick={() => onClickIssue(issue)}
                      className={cn(
                        "flex w-full items-center gap-1.5 rounded px-1.5 py-0.5 text-left text-[11px] leading-tight transition-colors",
                        "hover:bg-accent/20",
                        pastDue ? "bg-red-500/10" : "bg-card"
                      )}
                    >
                      <span
                        className={cn(
                          "h-1.5 w-1.5 flex-shrink-0 rounded-full",
                          priorityColors[issue.priority] ?? priorityColors.none
                        )}
                      />
                      <span className="min-w-0 flex-1 truncate">{issue.title}</span>
                      {member && (
                        <span
                          className="flex h-4 w-4 flex-shrink-0 items-center justify-center rounded-full text-[8px] font-medium text-white"
                          style={{ backgroundColor: member.avatar_color }}
                        >
                          {(member.display_name || member.name).charAt(0).toUpperCase()}
                        </span>
                      )}
                    </button>
                  );
                })}
                {hasOverflow && (
                  <span className="px-1.5 text-[10px] text-muted-foreground">
                    +{dayIssues.length - (MAX_VISIBLE_ISSUES - 1)} more
                  </span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
