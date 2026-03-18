import type { TaskCostSummary } from "@/types";

interface CostBadgeProps {
  costs: TaskCostSummary;
}

function formatTokens(tokens: number): string {
  if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`;
  if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}k`;
  return String(tokens);
}

export function CostBadge({ costs }: CostBadgeProps) {
  if (costs.total_tokens === 0 && costs.total_cost_dollars === 0) return null;

  return (
    <span
      className="inline-flex items-center rounded px-1.5 py-0.5 font-mono text-[9px] text-muted-foreground"
      title={`Compute: ${costs.total_compute_minutes.toFixed(1)}min · Tokens: ${costs.total_tokens.toLocaleString()} · Cost: $${costs.total_cost_dollars.toFixed(2)}`}
    >
      {formatTokens(costs.total_tokens)} tok
    </span>
  );
}
