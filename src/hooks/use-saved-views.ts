import { useState, useEffect, useCallback } from "react";
import type { SavedView } from "@/types";
import * as api from "@/tauri/commands";

export function useSavedViews(projectId: number | null) {
  const [savedViews, setSavedViews] = useState<SavedView[]>([]);

  const refresh = useCallback(async () => {
    if (!projectId) { setSavedViews([]); return; }
    try {
      const data = await api.listSavedViews(projectId);
      setSavedViews(data);
    } catch (e) {
      console.error("Failed to load saved views", e);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createSavedView>[0]) => {
    try {
      const view = await api.createSavedView(input);
      setSavedViews(prev => [...prev, view]);
      return view;
    } catch (e) {
      console.error("Failed to create saved view:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateSavedView>[1]) => {
    try {
      const view = await api.updateSavedView(id, input);
      setSavedViews(prev => prev.map(v => v.id === id ? view : v));
      return view;
    } catch (e) {
      console.error("Failed to update saved view:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteSavedView(id);
      setSavedViews(prev => prev.filter(v => v.id !== id));
    } catch (e) {
      console.error("Failed to delete saved view:", e);
      throw e;
    }
  };

  return { savedViews, refresh, create, update, remove };
}
