import { useState, useEffect, useCallback } from "react";
import type { Issue } from "@/types";
import * as api from "@/tauri/commands";

export function useStarred(memberId: number | null) {
  const [starredIssues, setStarredIssues] = useState<Issue[]>([]);
  const [starredIds, setStarredIds] = useState<Set<number>>(new Set());

  const refresh = useCallback(async () => {
    if (!memberId) { setStarredIssues([]); setStarredIds(new Set()); return; }
    try {
      const data = await api.listStarred(memberId);
      setStarredIssues(data);
      setStarredIds(new Set(data.map(i => i.id)));
    } catch (e) {
      console.error("Failed to load starred issues", e);
    }
  }, [memberId]);

  useEffect(() => { refresh(); }, [refresh]);

  const toggle = async (issueId: number) => {
    if (!memberId) return;
    try {
      if (starredIds.has(issueId)) {
        await api.unstarIssue(issueId, memberId);
        setStarredIds(prev => { const next = new Set(prev); next.delete(issueId); return next; });
        setStarredIssues(prev => prev.filter(i => i.id !== issueId));
      } else {
        await api.starIssue(issueId, memberId);
        setStarredIds(prev => new Set(prev).add(issueId));
        await refresh();
      }
    } catch (e) {
      console.error("Failed to toggle starred issue:", e);
      throw e;
    }
  };

  const isStarred = (issueId: number) => starredIds.has(issueId);

  return { starredIssues, isStarred, toggle, refresh };
}
