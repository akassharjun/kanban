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

  const create = async (input: { name: string; description?: string; icon?: string; prefix: string; path: string }) => {
    try {
      const project = await api.createProject(input);
      setProjects(prev => [...prev, project]);
      return project;
    } catch (e) {
      console.error("Failed to create project:", e);
      throw e;
    }
  };

  const update = async (id: number, input: { name?: string; description?: string; icon?: string; status?: string }) => {
    try {
      const project = await api.updateProject(id, input);
      setProjects(prev => prev.map(p => p.id === id ? project : p));
      return project;
    } catch (e) {
      console.error("Failed to update project:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteProject(id);
      setProjects(prev => prev.filter(p => p.id !== id));
    } catch (e) {
      console.error("Failed to delete project:", e);
      throw e;
    }
  };

  return { projects, loading, refresh, create, update, remove };
}
