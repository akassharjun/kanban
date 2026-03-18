import { useState, useEffect, useCallback } from "react";
import type { MilestoneWithProgress } from "@/types";
import * as api from "@/tauri/commands";

export function useMilestones(projectId: number | null) {
  const [milestones, setMilestones] = useState<MilestoneWithProgress[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) { setMilestones([]); setLoading(false); return; }
    try {
      const data = await api.listMilestones(projectId);
      setMilestones(data);
    } catch (e) {
      console.error("Failed to load milestones", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createMilestone>[0]) => {
    const milestone = await api.createMilestone(input);
    await refresh();
    return milestone;
  };

  const update = async (id: number, input: Parameters<typeof api.updateMilestone>[1]) => {
    const milestone = await api.updateMilestone(id, input);
    await refresh();
    return milestone;
  };

  const remove = async (id: number) => {
    await api.deleteMilestone(id);
    await refresh();
  };

  return { milestones, loading, refresh, create, update, remove };
}
