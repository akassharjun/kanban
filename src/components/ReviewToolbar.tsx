import { useState } from "react";
import * as api from "@/tauri/commands";
import { Button } from "@/components/ui/button";
import { Check, X, RotateCcw } from "lucide-react";

interface ReviewToolbarProps {
  issueIdentifier: string;
  onActionComplete: () => void;
}

export function ReviewToolbar({
  issueIdentifier,
  onActionComplete,
}: ReviewToolbarProps) {
  const [loading, setLoading] = useState<string | null>(null);

  const handleAction = async (action: "approve" | "reject" | "retry") => {
    setLoading(action);
    try {
      if (action === "approve") {
        await api.approveTask(issueIdentifier);
      } else if (action === "reject") {
        await api.rejectTask(issueIdentifier, "Rejected via review toolbar");
      } else {
        await api.rejectTask(issueIdentifier, "Retry requested");
      }
      onActionComplete();
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="flex items-center gap-2 p-3 border-b border-border bg-card rounded-t-lg">
      <Button
        size="sm"
        variant="outline"
        className="text-green-400 border-green-400/30 hover:bg-green-400/10"
        onClick={() => handleAction("approve")}
        disabled={loading !== null}
      >
        <Check className="h-3.5 w-3.5 mr-1" />
        Approve
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="text-red-400 border-red-400/30 hover:bg-red-400/10"
        onClick={() => handleAction("reject")}
        disabled={loading !== null}
      >
        <X className="h-3.5 w-3.5 mr-1" />
        Reject
      </Button>
      <Button
        size="sm"
        variant="outline"
        className="text-yellow-400 border-yellow-400/30 hover:bg-yellow-400/10"
        onClick={() => handleAction("retry")}
        disabled={loading !== null}
      >
        <RotateCcw className="h-3.5 w-3.5 mr-1" />
        Retry
      </Button>
    </div>
  );
}
