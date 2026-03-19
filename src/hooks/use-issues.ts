import { useState, useEffect, useCallback } from "react";
import type { Issue } from "@/types";
import * as api from "@/tauri/commands";

export function useIssues(projectId: number | null) {
  const [issues, setIssues] = useState<Issue[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) { setIssues([]); setLoading(false); return; }
    try {
      const data = await api.listIssues({ project_id: projectId });
      setIssues(data);
    } catch (e) {
      console.error("Failed to load issues", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createIssue>[0]) => {
    try {
      const issue = await api.createIssue(input);
      setIssues(prev => [...prev, issue]);
      return issue;
    } catch (e) {
      console.error("Failed to create issue:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateIssue>[1]) => {
    try {
      const issue = await api.updateIssue(id, input);
      setIssues(prev => prev.map(i => i.id === id ? issue : i));
      return issue;
    } catch (e) {
      console.error("Failed to update issue:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteIssue(id);
      setIssues(prev => prev.filter(i => i.id !== id));
    } catch (e) {
      console.error("Failed to delete issue:", e);
      throw e;
    }
  };

  const duplicate = async (id: number) => {
    try {
      const issue = await api.duplicateIssue(id);
      setIssues(prev => [...prev, issue]);
      return issue;
    } catch (e) {
      console.error("Failed to duplicate issue:", e);
      throw e;
    }
  };

  return { issues, loading, refresh, create, update, remove, duplicate };
}
