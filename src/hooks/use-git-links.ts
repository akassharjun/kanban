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
    const link = await api.createGitLink(input);
    setGitLinks(prev => [link, ...prev]);
    return link;
  };

  const update = async (id: number, input: Parameters<typeof api.updateGitLink>[1]) => {
    const link = await api.updateGitLink(id, input);
    setGitLinks(prev => prev.map(l => l.id === id ? link : l));
    return link;
  };

  const remove = async (id: number) => {
    await api.deleteGitLink(id);
    setGitLinks(prev => prev.filter(l => l.id !== id));
  };

  return { gitLinks, loading, refresh, create, update, remove };
}
