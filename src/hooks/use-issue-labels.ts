import { useState, useEffect, useCallback, useMemo } from "react";
import type { Label } from "@/types";
import * as api from "@/tauri/commands";

/**
 * Fetches issue-label mappings for a project and provides a lookup function.
 * Returns a map of issue_id -> Label[] for efficient rendering.
 */
export function useIssueLabelMap(projectId: number | null, allLabels: Label[]) {
  const [mappings, setMappings] = useState<{ issue_id: number; label_id: number }[]>([]);

  const refresh = useCallback(async () => {
    if (!projectId) { setMappings([]); return; }
    try {
      const data = await api.listIssueLabelMappings(projectId);
      setMappings(data);
    } catch (e) {
      console.error("Failed to load issue label mappings", e);
    }
  }, [projectId]);

  useEffect(() => { refresh(); }, [refresh]);

  const labelMap = useMemo(() => {
    const map = new Map<number, Label[]>();
    const labelById = new Map(allLabels.map(l => [l.id, l]));
    for (const { issue_id, label_id } of mappings) {
      const label = labelById.get(label_id);
      if (label) {
        const list = map.get(issue_id) ?? [];
        list.push(label);
        map.set(issue_id, list);
      }
    }
    return map;
  }, [mappings, allLabels]);

  const getLabelsForIssue = useCallback((issueId: number): Label[] => {
    return labelMap.get(issueId) ?? [];
  }, [labelMap]);

  return { labelMap, getLabelsForIssue, refresh };
}
