import { useState, useEffect, useCallback } from "react";
import type { Agent, AgentMetrics, ProjectMetrics, ExecutionLog } from "@/types";
import * as api from "@/tauri/commands";

export function useAgents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await api.listAgents();
      setAgents(data);
    } catch (e) {
      console.error("Failed to load agents", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  return { agents, loading, refresh };
}

export function useProjectMetrics(projectId: number | null) {
  const [metrics, setMetrics] = useState<ProjectMetrics | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (projectId === null) { setLoading(false); return; }
    try {
      const data = await api.projectMetrics(projectId);
      setMetrics(data);
    } catch (e) {
      console.error("Failed to load project metrics", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  return { metrics, loading, refresh };
}

export function useAgentMetrics(agentId: string | null) {
  const [metrics, setMetrics] = useState<AgentMetrics | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!agentId) return;
    try {
      const data = await api.getAgentStats(agentId);
      setMetrics(data);
    } catch (e) {
      console.error("Failed to load agent metrics", e);
    } finally {
      setLoading(false);
    }
  }, [agentId]);

  useEffect(() => { refresh(); }, [refresh]);

  return { metrics, loading, refresh };
}

export function useTaskReplay(identifier: string | null) {
  const [logs, setLogs] = useState<ExecutionLog[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!identifier) return;
    setLoading(true);
    try {
      const data = await api.taskReplay(identifier);
      setLogs(data);
    } catch (e) {
      console.error("Failed to load task replay", e);
    } finally {
      setLoading(false);
    }
  }, [identifier]);

  useEffect(() => { refresh(); }, [refresh]);

  return { logs, loading, refresh };
}
