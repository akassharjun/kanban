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
    const label = await api.createLabel(input);
    await refresh();
    return label;
  };

  const update = async (id: number, input: Parameters<typeof api.updateLabel>[1]) => {
    const label = await api.updateLabel(id, input);
    await refresh();
    return label;
  };

  const remove = async (id: number) => {
    await api.deleteLabel(id);
    await refresh();
  };

  return { labels, loading, refresh, create, update, remove };
}
