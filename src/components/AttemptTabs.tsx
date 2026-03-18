import { cn } from "@/lib/utils";
import { ConfidenceBadge } from "./ConfidenceBadge";

interface Attempt {
  number: number;
  confidence: number | null;
  agentName: string;
}

interface AttemptTabsProps {
  attempts: Attempt[];
  activeAttempt: number;
  onSelect: (attempt: number) => void;
  onCompare?: () => void;
}

export function AttemptTabs({
  attempts,
  activeAttempt,
  onSelect,
  onCompare,
}: AttemptTabsProps) {
  return (
    <div className="flex items-center gap-1 border-b border-border px-3 overflow-x-auto">
      {attempts.map((attempt) => (
        <button
          key={attempt.number}
          onClick={() => onSelect(attempt.number)}
          className={cn(
            "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-colors",
            activeAttempt === attempt.number
              ? "border-foreground text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground",
          )}
        >
          Attempt {attempt.number}
          <ConfidenceBadge score={attempt.confidence} />
        </button>
      ))}
      {attempts.length > 1 && onCompare && (
        <button
          onClick={onCompare}
          className="px-3 py-2 text-xs font-medium text-muted-foreground hover:text-foreground border-b-2 border-transparent"
        >
          Compare
        </button>
      )}
    </div>
  );
}
