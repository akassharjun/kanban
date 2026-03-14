import { useState, useEffect, useCallback } from "react";
import type { Project } from "@/types";
import * as api from "@/tauri/commands";

export function useProjects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await api.listProjects();
      setProjects(data);
    } catch (e) {
      console.error("Failed to load projects", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: { name: string; description?: string; icon?: string; prefix: string }) => {
    const project = await api.createProject(input);
    await refresh();
    return project;
  };

  const update = async (id: number, input: { name?: string; description?: string; icon?: string; status?: string }) => {
    const project = await api.updateProject(id, input);
    await refresh();
    return project;
  };

  const remove = async (id: number) => {
    await api.deleteProject(id);
    await refresh();
  };

  return { projects, loading, refresh, create, update, remove };
}
