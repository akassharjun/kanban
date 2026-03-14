import { useState, useEffect, useCallback } from "react";
import type { Status } from "@/types";
import * as api from "@/tauri/commands";

export function useStatuses(projectId: number | null) {
  const [statuses, setStatuses] = useState<Status[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) { setStatuses([]); setLoading(false); return; }
    try {
      const data = await api.listStatuses(projectId);
      setStatuses(data);
    } catch (e) {
      console.error("Failed to load statuses", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  return { statuses, loading, refresh };
}
