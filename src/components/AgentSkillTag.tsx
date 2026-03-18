import { cn } from "@/lib/utils";

interface AgentSkillTagProps {
  agentType: "claude" | "codex" | "gemini" | "custom";
  name?: string;
}

export function AgentSkillTag({ agentType, name }: AgentSkillTagProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded px-1.5 py-0.5 text-[9px] font-medium bg-secondary text-muted-foreground",
        agentType === "codex" && "text-blue-400/80",
        agentType === "gemini" && "text-teal-400/80",
      )}
    >
      {name || agentType}
    </span>
  );
}
