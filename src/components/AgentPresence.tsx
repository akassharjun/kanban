import { cn } from "@/lib/utils";
import type { AgentPresenceData, ExecutionEntryType } from "@/types";

interface AgentPresenceProps {
  agents: AgentPresenceData[];
  maxVisible?: number;
}

function actionLabel(type?: ExecutionEntryType, action?: string): string {
  if (!type || !action) return "";
  switch (type) {
    case "file_read": return `Reading ${action}`;
    case "file_edit": return `Editing ${action}`;
    case "command": return `Running ${action}`;
    case "reasoning": return "Thinking...";
    case "checkpoint": return action;
    case "error": return `Error: ${action}`;
    default: return action;
  }
}

export function AgentPresence({ agents, maxVisible = 3 }: AgentPresenceProps) {
  if (agents.length === 0) return null;

  const visible = agents.slice(0, maxVisible);
  const overflow = agents.length - maxVisible;
  const activeAgent = agents.find((a) => a.status === "active");

  return (
    <div className="flex items-center gap-1.5 mt-1">
      <div className="flex items-center -space-x-1">
        {visible.map((agent) => (
          <div
            key={agent.agentId}
            className={cn(
              "flex h-4 w-4 items-center justify-center rounded-full bg-secondary text-[6px] border-2 border-card",
              agent.status === "active" && "border-green-400 agent-ring-active",
              agent.status === "error" && "border-red-400",
              agent.status === "idle" && "border-border",
            )}
            title={agent.agentName}
          >
            🤖
          </div>
        ))}
        {overflow > 0 && (
          <span className="text-[9px] text-muted-foreground ml-1">
            +{overflow}
          </span>
        )}
      </div>
      {activeAgent?.lastAction && (
        <span className="text-[10px] text-muted-foreground truncate max-w-[120px]">
          {actionLabel(activeAgent.lastActionType, activeAgent.lastAction)}
        </span>
      )}
    </div>
  );
}
