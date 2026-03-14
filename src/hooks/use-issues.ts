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
    const issue = await api.createIssue(input);
    await refresh();
    return issue;
  };

  const update = async (id: number, input: Parameters<typeof api.updateIssue>[1]) => {
    const issue = await api.updateIssue(id, input);
    await refresh();
    return issue;
  };

  const remove = async (id: number) => {
    await api.deleteIssue(id);
    await refresh();
  };

  const duplicate = async (id: number) => {
    const issue = await api.duplicateIssue(id);
    await refresh();
    return issue;
  };

  return { issues, loading, refresh, create, update, remove, duplicate };
}
