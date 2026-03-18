import { useState, useEffect, useCallback } from "react";
import type { Issue } from "@/types";
import * as api from "@/tauri/commands";

export function useRecentlyViewed(memberId: number | null) {
  const [recentIssues, setRecentIssues] = useState<Issue[]>([]);

  const refresh = useCallback(async () => {
    if (!memberId) { setRecentIssues([]); return; }
    try {
      const data = await api.listRecentlyViewed(memberId, 10);
      setRecentIssues(data);
    } catch (e) {
      console.error("Failed to load recently viewed", e);
    }
  }, [memberId]);

  useEffect(() => { refresh(); }, [refresh]);

  const recordView = async (issueId: number) => {
    if (!memberId) return;
    try {
      await api.recordView(issueId, memberId);
      await refresh();
    } catch (e) {
      console.error("Failed to record view", e);
    }
  };

  return { recentIssues, recordView, refresh };
}
