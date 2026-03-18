import type { Epic } from "@/types";

interface EpicBadgeProps {
  epic: Epic;
}

export function EpicBadge({ epic }: EpicBadgeProps) {
  return (
    <span
      className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium"
      style={{
        backgroundColor: epic.color + "18",
        color: epic.color,
      }}
    >
      <span
        className="h-1.5 w-1.5 rounded-full"
        style={{ backgroundColor: epic.color }}
      />
      {epic.title}
    </span>
  );
}
