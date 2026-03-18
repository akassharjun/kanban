import { useState, useEffect } from "react";
import * as api from "@/tauri/commands";
import { listen } from "@/tauri/events";
import type { TaskCostSummary } from "@/types";

export function useTaskCosts(taskIdentifier: string | null): TaskCostSummary | null {
  const [costs, setCosts] = useState<TaskCostSummary | null>(null);

  useEffect(() => {
    if (taskIdentifier === null) return;

    const fetchCosts = async () => {
      try {
        const result = await api.getTaskCostSummary(taskIdentifier);
        setCosts(result);
      } catch {
        setCosts(null);
      }
    };

    fetchCosts();
    const unlisten = listen("db-changed", () => fetchCosts());
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [taskIdentifier]);

  return costs;
}
