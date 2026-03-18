import { useState, useEffect, useCallback } from "react";
import type { AutomationRule, AutomationLogEntry } from "@/types";
import * as api from "@/tauri/commands";

export function useAutomations(projectId: number | null) {
  const [rules, setRules] = useState<AutomationRule[]>([]);
  const [log, setLog] = useState<AutomationLogEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!projectId) {
      setRules([]);
      setLog([]);
      setLoading(false);
      return;
    }
    try {
      const [r, l] = await Promise.all([
        api.listAutomationRules(projectId),
        api.listAutomationLog(projectId, 50),
      ]);
      setRules(r);
      setLog(l);
    } catch (e) {
      console.error("Failed to load automations", e);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const create = async (input: Parameters<typeof api.createAutomationRule>[0]) => {
    const rule = await api.createAutomationRule(input);
    setRules(prev => [rule, ...prev]);
    return rule;
  };

  const update = async (id: number, input: Parameters<typeof api.updateAutomationRule>[1]) => {
    const rule = await api.updateAutomationRule(id, input);
    setRules(prev => prev.map(r => r.id === id ? rule : r));
    return rule;
  };

  const remove = async (id: number) => {
    await api.deleteAutomationRule(id);
    setRules(prev => prev.filter(r => r.id !== id));
  };

  const toggle = async (id: number, enabled: boolean) => {
    const rule = await api.toggleAutomationRule(id, enabled);
    setRules(prev => prev.map(r => r.id === id ? rule : r));
    return rule;
  };

  return { rules, log, loading, refresh, create, update, remove, toggle };
}
