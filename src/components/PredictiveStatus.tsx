import { useState, useEffect } from "react";
import * as api from "@/tauri/commands";
import { Clock } from "lucide-react";

interface PredictiveStatusProps {
  issueId: number;
  dueDate: string | null;
  agentId: string | null;
}

export function PredictiveStatus({
  dueDate,
  agentId,
}: PredictiveStatusProps) {
  const [daysLate, setDaysLate] = useState<number | null>(null);

  useEffect(() => {
    if (!dueDate || !agentId) return;

    const estimate = async () => {
      try {
        const perf = await api.getAgentPerformance(agentId);
        if (perf.tasks_completed < 3) return;

        const due = new Date(dueDate).getTime();
        const now = Date.now();
        const estimatedMs = perf.avg_duration_minutes * 60 * 1000 * 1.2;
        const estimatedCompletion = now + estimatedMs;

        if (estimatedCompletion > due) {
          setDaysLate(
            Math.ceil((estimatedCompletion - due) / (1000 * 60 * 60 * 24)),
          );
        }
      } catch {
        // Silently fail
      }
    };

    estimate();
  }, [dueDate, agentId]);

  if (daysLate === null) return null;

  return (
    <span
      className="inline-flex items-center gap-0.5 text-orange-400"
      title={`Based on agent velocity, likely ~${daysLate} day${daysLate > 1 ? "s" : ""} late`}
    >
      <Clock className="h-3 w-3" />
    </span>
  );
}
