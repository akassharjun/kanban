import { useState, useEffect, useCallback } from "react";
import type { Epic } from "@/types";
import * as api from "@/tauri/commands";

export function useEpics(projectId: number | null) {
  const [epics, setEpics] = useState<Epic[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) { setEpics([]); setLoading(false); return; }
    try {
      const data = await api.listEpics(projectId);
      setEpics(data);
    } catch (e) {
      console.error("Failed to load epics", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createEpic>[0]) => {
    const epic = await api.createEpic(input);
    await refresh();
    return epic;
  };

  const update = async (id: number, input: Parameters<typeof api.updateEpic>[1]) => {
    const epic = await api.updateEpic(id, input);
    await refresh();
    return epic;
  };

  const remove = async (id: number) => {
    await api.deleteEpic(id);
    await refresh();
  };

  return { epics, loading, refresh, create, update, remove };
}
