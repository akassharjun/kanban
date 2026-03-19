import { useState, useEffect, useCallback } from "react";
import type { Label } from "@/types";
import * as api from "@/tauri/commands";

export function useLabels(projectId: number | null) {
  const [labels, setLabels] = useState<Label[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) { setLabels([]); setLoading(false); return; }
    try {
      const data = await api.listLabels(projectId);
      setLabels(data);
    } catch (e) {
      console.error("Failed to load labels", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createLabel>[0]) => {
    try {
      const label = await api.createLabel(input);
      await refresh();
      return label;
    } catch (e) {
      console.error("Failed to create label:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateLabel>[1]) => {
    try {
      const label = await api.updateLabel(id, input);
      await refresh();
      return label;
    } catch (e) {
      console.error("Failed to update label:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteLabel(id);
      await refresh();
    } catch (e) {
      console.error("Failed to delete label:", e);
      throw e;
    }
  };

  return { labels, loading, refresh, create, update, remove };
}
