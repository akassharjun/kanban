import { useState, useEffect, useCallback } from "react";
import * as api from "@/tauri/commands";
import { listen } from "@/tauri/events";
import type { ExecutionLog, TickerEntry } from "@/types";

// Module-level cache for deduplication
const logCache = new Map<string, ExecutionLog[]>();
const subscribers = new Map<string, Set<() => void>>();

function notifySubscribers(identifier: string) {
  const subs = subscribers.get(identifier);
  if (subs) subs.forEach((cb) => cb());
}

async function fetchLogs(identifier: string): Promise<ExecutionLog[]> {
  try {
    const logs = await api.taskReplay(identifier);
    logCache.set(identifier, logs);
    notifySubscribers(identifier);
    return logs;
  } catch {
    return logCache.get(identifier) || [];
  }
}

export function useExecutionLogs(identifier: string | null) {
  const [logs, setLogs] = useState<ExecutionLog[]>([]);

  useEffect(() => {
    if (identifier === null) return;

    const cached = logCache.get(identifier);
    if (cached) setLogs(cached);

    const cb = () => setLogs(logCache.get(identifier) || []);
    if (!subscribers.has(identifier)) subscribers.set(identifier, new Set());
    subscribers.get(identifier)!.add(cb);

    fetchLogs(identifier);

    let timeout: ReturnType<typeof setTimeout>;
    const unlisten = listen("db-changed", () => {
      clearTimeout(timeout);
      timeout = setTimeout(() => fetchLogs(identifier), 2000);
    });

    return () => {
      subscribers.get(identifier)?.delete(cb);
      clearTimeout(timeout);
      unlisten.then((fn) => fn());
    };
  }, [identifier]);

  return logs;
}

export function useGlobalExecutionLogs(projectId: number | null, limit = 50) {
  const [entries, setEntries] = useState<TickerEntry[]>([]);

  const fetchGlobal = useCallback(async () => {
    if (projectId === null) return;
    try {
      const logs = await api.recentActivity(projectId, limit);
      const mapped: TickerEntry[] = logs.map((log) => ({
        id: log.id,
        agentName: log.agent_id || "unknown",
        agentId: log.agent_id || "",
        action: log.message,
        entryType: log.entry_type,
        issueIdentifier: null,
        issueId: log.issue_id,
        timestamp: log.timestamp,
      }));
      setEntries(mapped);
    } catch {
      // Silently fail — ticker is non-critical
    }
  }, [projectId, limit]);

  useEffect(() => {
    fetchGlobal();

    let timeout: ReturnType<typeof setTimeout>;
    const unlisten = listen("db-changed", () => {
      clearTimeout(timeout);
      timeout = setTimeout(fetchGlobal, 2000);
    });

    return () => {
      clearTimeout(timeout);
      unlisten.then((fn) => fn());
    };
  }, [fetchGlobal]);

  return entries;
}
