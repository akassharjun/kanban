import { useState, useEffect, useCallback } from "react";
import type { GithubConfig } from "@/types";
import * as api from "@/tauri/commands";

export function useGithubConfig(projectId: number | null) {
  const [config, setConfig] = useState<GithubConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) {
      setConfig(null);
      setLoading(false);
      return;
    }
    try {
      const data = await api.getGithubConfig(projectId);
      setConfig(data);
    } catch (e) {
      console.error("Failed to load GitHub config", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const save = useCallback(async (input: {
    repo_owner: string;
    repo_name: string;
    access_token?: string | null;
    branch_pattern?: string;
    auto_link_prs?: boolean;
    auto_transition_on_merge?: boolean;
    merge_target_status_id?: number | null;
  }) => {
    if (!projectId) return null;
    try {
      const result = await api.setGithubConfig(projectId, input);
      setConfig(result);
      return result;
    } catch (e) {
      console.error("Failed to save GitHub config:", e);
      throw e;
    }
  }, [projectId]);

  return { config, loading, refresh, save };
}
