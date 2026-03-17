import type { Agent, ExecutionLog } from "@/types";
import { normalizeAgentType } from "./format-utils";

export function filterAgents(
  agents: Agent[],
  showInactive: boolean,
  agentTypeFilter: string,
): Agent[] {
  return agents.filter((agent) => {
    if (!showInactive && agent.status === "offline") return false;
    if (agentTypeFilter === "all") return true;
    return normalizeAgentType(agent.agent_type) === agentTypeFilter;
  });
}

export function filterLogs(
  logs: ExecutionLog[],
  entryTypeFilter: string,
): ExecutionLog[] {
  if (entryTypeFilter === "all") return logs;
  return logs.filter((l) => l.entry_type === entryTypeFilter);
}
