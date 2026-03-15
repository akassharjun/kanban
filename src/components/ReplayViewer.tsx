import { useState, useEffect } from "react";
import { X } from "lucide-react";
import { useTaskReplay } from "@/hooks/use-agents";
import * as api from "@/tauri/commands";
import type { FullTaskContract } from "@/types";

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

export function ReplayViewer({ identifier, onClose }: ReplayViewerProps) {
  const { logs, loading } = useTaskReplay(identifier);
  const [contract, setContract] = useState<FullTaskContract | null>(null);

  useEffect(() => {
    api.getTaskContract(identifier).then(setContract).catch(() => {});
  }, [identifier]);

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

      {/* Loading state */}
      {loading && (
        <div className="text-zinc-500 text-sm">Loading execution logs...</div>
      )}

      {/* Timeline */}
      {!loading && logs.length === 0 && (
        <div className="text-zinc-500 text-sm">No execution logs found.</div>
      )}

      {!loading && logs.length > 0 && (
        <div className="space-y-0">
          {logs.map((log, index) => {
            const colorClass = entryTypeColors[log.entry_type] || "bg-zinc-500/20 text-zinc-400";
            const isLast = index === logs.length - 1;

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

                {/* Entry type badge + message */}
                <div className="flex-1 pb-4">
                  <div className="flex items-center gap-2 mb-0.5">
                    <span className={`px-1.5 py-0.5 rounded text-xs font-medium ${colorClass}`}>
                      {log.entry_type}
                    </span>
                  </div>
                  <p className="text-sm text-zinc-300 leading-relaxed">{log.message}</p>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
