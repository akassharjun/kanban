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
    try {
      const epic = await api.createEpic(input);
      await refresh();
      return epic;
    } catch (e) {
      console.error("Failed to create epic:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateEpic>[1]) => {
    try {
      const epic = await api.updateEpic(id, input);
      await refresh();
      return epic;
    } catch (e) {
      console.error("Failed to update epic:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteEpic(id);
      await refresh();
    } catch (e) {
      console.error("Failed to delete epic:", e);
      throw e;
    }
  };

  return { epics, loading, refresh, create, update, remove };
}
