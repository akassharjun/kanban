import { cn } from "@/lib/utils";

interface ConfidenceBadgeProps {
  score: number | null;
}

export function ConfidenceBadge({ score }: ConfidenceBadgeProps) {
  if (score === null) return null;

  const isHigh = score >= 0.85;
  const isMedium = score >= 0.5 && score < 0.85;

  const colorClass = isHigh
    ? "text-green-400 bg-green-400/15"
    : isMedium
      ? "text-yellow-400 bg-yellow-400/15"
      : "text-red-400 bg-red-400/15";

  const icon = isHigh ? "✓" : isMedium ? "⟳" : "✗";

  return (
    <span
      className={cn(
        "inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 font-mono text-[10px] font-semibold",
        colorClass,
      )}
    >
      {score.toFixed(2)} <span>{icon}</span>
    </span>
  );
}
