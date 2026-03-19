import { useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { useGlobalExecutionLogs } from "@/hooks/use-execution-logs";
import type { TickerEntry } from "@/types";
import { motion, AnimatePresence } from "framer-motion";

function dotColor(type: string): string {
  switch (type) {
    case "error":
    case "fail":
      return "bg-red-400";
    case "complete":
      return "bg-green-400";
    case "timeout":
      return "bg-orange-400";
    default:
      return "bg-green-400";
  }
}

function actionText(entry: TickerEntry): string {
  switch (entry.entryType) {
    case "file_edit":
      return `Edited ${entry.action}`;
    case "file_read":
      return `Read ${entry.action}`;
    case "command":
      return `Running ${entry.action}`;
    case "complete":
      return `Completed ${entry.issueIdentifier || "task"}`;
    case "fail":
      return `Failed ${entry.issueIdentifier || "task"}`;
    case "claim":
      return `Claimed ${entry.issueIdentifier || "task"}`;
    case "start":
      return `Started ${entry.issueIdentifier || "task"}`;
    case "reasoning":
      return "Thinking...";
    default:
      return entry.action;
  }
}

function relativeTime(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  return `${Math.floor(mins / 60)}h`;
}

interface ActivityTickerProps {
  projectId: number | null;
  onClickEntry?: (issueId: number) => void;
}

export function ActivityTicker({ projectId, onClickEntry }: ActivityTickerProps) {
  const entries = useGlobalExecutionLogs(projectId, 50);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollLeft = 0;
    }
  }, [entries.length]);

  if (entries.length === 0) {
    return null;
  }

  return (
    <div className="h-9 border-t border-border bg-ticker flex items-center overflow-hidden group hover:h-20 transition-[height] duration-200">
      <div
        ref={scrollRef}
        className="flex items-center gap-4 px-3 overflow-x-auto scrollbar-none w-full"
      >
        <AnimatePresence mode="popLayout">
          {entries.map((entry) => (
            <motion.div
              key={entry.id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className={cn(
                "flex items-center gap-1.5 shrink-0 cursor-pointer",
                "hover:opacity-100",
              )}
              onClick={() => onClickEntry?.(entry.issueId)}
            >
              <div className={cn("w-1 h-1 rounded-full", dotColor(entry.entryType))} />
              <span className="text-[11px] font-medium text-muted-foreground">
                {entry.agentName}
              </span>
              <span className="text-[11px] text-muted-foreground/60">
                {actionText(entry)}
              </span>
              <span className="text-[10px] text-muted-foreground/30">
                {relativeTime(entry.timestamp)}
              </span>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  );
}
