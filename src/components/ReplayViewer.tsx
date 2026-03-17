import { useState, useEffect, useMemo } from "react";
import { X, ChevronRight, ChevronDown, Search } from "lucide-react";
import { useTaskReplay } from "@/hooks/use-agents";
import { safeJsonParse } from "@/lib/issue-utils";
import * as api from "@/tauri/commands";
import type { FullTaskContract, ExecutionLog } from "@/types";

interface ReplayViewerProps {
  identifier: string;
  onClose: () => void;
}

const entryTypeColors: Record<string, string> = {
  claim: "bg-blue-500/20 text-blue-400",
  start: "bg-blue-500/20 text-blue-400",
  reasoning: "bg-purple-500/20 text-purple-400",
  file_read: "bg-cyan-500/20 text-cyan-400",
  file_edit: "bg-yellow-500/20 text-yellow-400",
  command: "bg-orange-500/20 text-orange-400",
  discovery: "bg-emerald-500/20 text-emerald-400",
  error: "bg-red-500/20 text-red-400",
  result: "bg-green-500/20 text-green-400",
  complete: "bg-green-500/20 text-green-400",
  checkpoint: "bg-zinc-500/20 text-zinc-400",
  timeout: "bg-red-500/20 text-red-400",
  unblocked: "bg-emerald-500/20 text-emerald-400",
};

const FILTER_TYPES = ["all", "file_edit", "command", "error", "result"] as const;
type FilterType = (typeof FILTER_TYPES)[number];

const FILTER_LABELS: Record<FilterType, string> = {
  all: "All",
  file_edit: "File Edit",
  command: "Command",
  error: "Error",
  result: "Result",
};

function formatTime(iso: string): string {
  try {
    const date = new Date(iso);
    return date.toISOString().substring(11, 19);
  } catch {
    return "—";
  }
}

function stateColor(state: string): string {
  switch (state) {
    case "completed":
      return "bg-green-500/20 text-green-400";
    case "executing":
      return "bg-blue-500/20 text-blue-400";
    case "blocked":
      return "bg-red-500/20 text-red-400";
    case "queued":
      return "bg-zinc-500/20 text-zinc-400";
    case "validating":
      return "bg-yellow-500/20 text-yellow-400";
    case "cancelled":
      return "bg-red-500/20 text-red-400";
    default:
      return "bg-zinc-500/20 text-zinc-400";
  }
}

function MetadataSection({ metadata }: { metadata: string | null }) {
  const [expanded, setExpanded] = useState(false);
  const parsed = safeJsonParse<Record<string, unknown>>(metadata, {});

  if (!metadata || Object.keys(parsed).length === 0) return null;

  const file = parsed.file as string | undefined;
  const exitCode = parsed.exit_code as number | undefined;
  const command = parsed.command as string | undefined;
  const otherKeys = Object.keys(parsed).filter(
    (k) => k !== "file" && k !== "exit_code" && k !== "command"
  );

  return (
    <div className="mt-1.5">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1 text-[10px] font-mono text-zinc-500 hover:text-zinc-300 transition-colors"
      >
        {expanded ? (
          <ChevronDown className="w-3 h-3" />
        ) : (
          <ChevronRight className="w-3 h-3" />
        )}
        metadata
      </button>
      {expanded && (
        <div className="mt-1 ml-4 space-y-1 text-xs font-mono">
          {file && (
            <div>
              <span className="inline-block bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 rounded px-1.5 py-0.5 text-[10px]">
                {file}
              </span>
            </div>
          )}
          {exitCode !== undefined && (
            <div className="text-zinc-400">
              exit_code:{" "}
              <span
                className={
                  exitCode === 0 ? "text-green-400" : "text-red-400"
                }
              >
                {exitCode}
              </span>
            </div>
          )}
          {command && (
            <div className="bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-zinc-300 whitespace-pre-wrap break-all">
              {command}
            </div>
          )}
          {otherKeys.map((key) => (
            <div key={key} className="text-zinc-400">
              {key}: <span className="text-zinc-300">{String(parsed[key])}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function ReplayViewer({ identifier, onClose }: ReplayViewerProps) {
  const { logs, loading } = useTaskReplay(identifier);
  const [contract, setContract] = useState<FullTaskContract | null>(null);
  const [activeAttempt, setActiveAttempt] = useState<number | null>(null);
  const [activeFilter, setActiveFilter] = useState<FilterType>("all");
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    api.getTaskContract(identifier).then(setContract).catch(() => {});
  }, [identifier]);

  // Group logs by attempt_number
  const attempts = useMemo(() => {
    const map = new Map<number, ExecutionLog[]>();
    for (const log of logs) {
      const existing = map.get(log.attempt_number) || [];
      existing.push(log);
      map.set(log.attempt_number, existing);
    }
    return Array.from(map.keys()).sort((a, b) => a - b);
  }, [logs]);

  // Default to latest attempt when logs load
  useEffect(() => {
    if (attempts.length > 0 && activeAttempt === null) {
      setActiveAttempt(attempts[attempts.length - 1]);
    }
  }, [attempts, activeAttempt]);

  // Filter logs by attempt, entry type, and search
  const filteredLogs = useMemo(() => {
    let filtered = logs;

    // Filter by attempt (only if multiple attempts)
    if (attempts.length > 1 && activeAttempt !== null) {
      filtered = filtered.filter((l) => l.attempt_number === activeAttempt);
    }

    // Filter by entry type
    if (activeFilter !== "all") {
      filtered = filtered.filter((l) => l.entry_type === activeFilter);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter((l) =>
        l.message.toLowerCase().includes(q)
      );
    }

    return filtered;
  }, [logs, attempts, activeAttempt, activeFilter, searchQuery]);

  return (
    <div className="bg-zinc-950 text-zinc-100 h-full overflow-y-auto p-6 relative">
      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-4 right-4 p-1.5 rounded-md text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors"
      >
        <X className="w-5 h-5" />
      </button>

      {/* Header */}
      {contract && (
        <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-4 mb-6">
          <div className="flex items-center gap-3 flex-wrap">
            <span className="font-mono text-amber-400 text-sm">{contract.identifier}</span>
            <span className="font-semibold text-white">{contract.title}</span>
          </div>
          <div className="flex items-center gap-2 mt-2 flex-wrap">
            <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${stateColor(contract.task_state)}`}>
              {contract.task_state}
            </span>
            <span className="text-xs text-zinc-500">
              {contract.attempt_count} attempt{contract.attempt_count !== 1 ? "s" : ""}
            </span>
          </div>
        </div>
      )}

      {/* Controls: Search, Attempt Tabs, Filter Pills */}
      <div className="space-y-3 mb-6">
        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search log messages..."
            className="w-full bg-zinc-900 border border-zinc-700 rounded-lg pl-9 pr-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500 focus:outline-none focus:border-zinc-500"
          />
        </div>

        {/* Attempt tabs */}
        {attempts.length > 1 && (
          <div className="flex gap-1">
            {attempts.map((attemptNum) => (
              <button
                key={attemptNum}
                onClick={() => setActiveAttempt(attemptNum)}
                className={`px-3 py-1 rounded text-xs font-mono transition-colors ${
                  activeAttempt === attemptNum
                    ? "bg-amber-500/20 text-amber-400 border border-amber-500/30"
                    : "bg-zinc-800 text-zinc-400 border border-zinc-700 hover:border-zinc-500"
                }`}
              >
                Attempt {attemptNum}
              </button>
            ))}
          </div>
        )}

        {/* Entry type filter pills */}
        <div className="flex gap-1 flex-wrap">
          {FILTER_TYPES.map((type) => (
            <button
              key={type}
              onClick={() => setActiveFilter(type)}
              className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
                activeFilter === type
                  ? type === "all"
                    ? "bg-zinc-600/30 text-zinc-100 border border-zinc-500"
                    : `${entryTypeColors[type] || "bg-zinc-500/20 text-zinc-400"} border border-current/20`
                  : "bg-zinc-800/50 text-zinc-500 border border-zinc-700 hover:border-zinc-500 hover:text-zinc-300"
              }`}
            >
              {FILTER_LABELS[type]}
            </button>
          ))}
        </div>
      </div>

      {/* Loading state */}
      {loading && (
        <div className="text-zinc-500 text-sm">Loading execution logs...</div>
      )}

      {/* Timeline */}
      {!loading && logs.length === 0 && (
        <div className="text-zinc-500 text-sm">No execution logs found.</div>
      )}

      {!loading && logs.length > 0 && filteredLogs.length === 0 && (
        <div className="text-zinc-500 text-sm">No logs match the current filters.</div>
      )}

      {!loading && filteredLogs.length > 0 && (
        <div className="space-y-0">
          {filteredLogs.map((log, index) => {
            const colorClass = entryTypeColors[log.entry_type] || "bg-zinc-500/20 text-zinc-400";
            const isLast = index === filteredLogs.length - 1;

            return (
              <div key={log.id} className="flex gap-4">
                {/* Timestamp */}
                <div className="w-16 flex-shrink-0 text-right">
                  <span className="font-mono text-xs text-zinc-500">
                    {formatTime(log.timestamp)}
                  </span>
                </div>

                {/* Timeline line + dot */}
                <div className="flex flex-col items-center flex-shrink-0">
                  <div className="w-2 h-2 rounded-full bg-zinc-600 mt-1.5" />
                  {!isLast && <div className="w-px flex-1 border-l-2 border-zinc-700" />}
                </div>

                {/* Entry type badge + message + metadata */}
                <div className="flex-1 pb-4">
                  <div className="flex items-center gap-2 mb-0.5">
                    <span className={`px-1.5 py-0.5 rounded text-xs font-medium ${colorClass}`}>
                      {log.entry_type}
                    </span>
                  </div>
                  <p className="text-sm text-zinc-300 leading-relaxed">{log.message}</p>
                  <MetadataSection metadata={log.metadata} />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
