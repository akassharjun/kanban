import { useState } from "react";
import { DiffPreview } from "./DiffPreview";
import type { ExecutionLog } from "@/types";

interface AttemptComparisonProps {
  logsByAttempt: Map<number, ExecutionLog[]>;
}

export function AttemptComparison({ logsByAttempt }: AttemptComparisonProps) {
  const attemptNumbers = Array.from(logsByAttempt.keys()).sort();
  const [left, setLeft] = useState(attemptNumbers[0] || 1);
  const [right, setRight] = useState(attemptNumbers[attemptNumbers.length - 1] || 2);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-4 p-3 border-b border-border">
        <label className="text-xs text-muted-foreground">
          Left:
          <select
            value={left}
            onChange={(e) => setLeft(Number(e.target.value))}
            className="ml-1 bg-secondary text-foreground border border-border rounded px-2 py-1 text-xs"
          >
            {attemptNumbers.map((n) => (
              <option key={n} value={n}>
                Attempt {n}
              </option>
            ))}
          </select>
        </label>
        <label className="text-xs text-muted-foreground">
          Right:
          <select
            value={right}
            onChange={(e) => setRight(Number(e.target.value))}
            className="ml-1 bg-secondary text-foreground border border-border rounded px-2 py-1 text-xs"
          >
            {attemptNumbers.map((n) => (
              <option key={n} value={n}>
                Attempt {n}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="flex flex-1 divide-x divide-border overflow-auto">
        <div className="flex-1 overflow-auto">
          <div className="text-xs font-medium text-muted-foreground p-2 border-b border-border">
            Attempt {left}
          </div>
          <DiffPreview logs={logsByAttempt.get(left) || []} />
        </div>
        <div className="flex-1 overflow-auto">
          <div className="text-xs font-medium text-muted-foreground p-2 border-b border-border">
            Attempt {right}
          </div>
          <DiffPreview logs={logsByAttempt.get(right) || []} />
        </div>
      </div>
    </div>
  );
}
