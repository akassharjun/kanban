import { useState, useEffect, useCallback } from "react";
import type { GitLink } from "@/types";
import * as api from "@/tauri/commands";

export function useGitLinks(issueId: number | null) {
  const [gitLinks, setGitLinks] = useState<GitLink[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!issueId) { setGitLinks([]); return; }
    setLoading(true);
    try {
      const data = await api.listGitLinks(issueId);
      setGitLinks(data);
    } catch (e) {
      console.error("Failed to load git links", e);
    } finally {
      setLoading(false);
    }
  }, [issueId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createGitLink>[0]) => {
    try {
      const link = await api.createGitLink(input);
      setGitLinks(prev => [link, ...prev]);
      return link;
    } catch (e) {
      console.error("Failed to create git link:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateGitLink>[1]) => {
    try {
      const link = await api.updateGitLink(id, input);
      setGitLinks(prev => prev.map(l => l.id === id ? link : l));
      return link;
    } catch (e) {
      console.error("Failed to update git link:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteGitLink(id);
      setGitLinks(prev => prev.filter(l => l.id !== id));
    } catch (e) {
      console.error("Failed to delete git link:", e);
      throw e;
    }
  };

  return { gitLinks, loading, refresh, create, update, remove };
}
