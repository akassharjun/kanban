interface EpicArcRingProps {
  total: number;
  completed: number;
  blocked?: number;
  size?: "sm" | "md" | "lg";
}

const sizeConfig = {
  sm: { width: 16, radius: 6, stroke: 2, fontSize: "4px", fontWeight: 700 },
  md: { width: 32, radius: 13, stroke: 2.5, fontSize: "8px", fontWeight: 700 },
  lg: { width: 40, radius: 17, stroke: 3, fontSize: "10px", fontWeight: 700 },
};

export function EpicArcRing({
  total,
  completed,
  blocked: _blocked = 0,
  size = "lg",
}: EpicArcRingProps) {
  const config = sizeConfig[size];
  const cx = config.width / 2;
  const circumference = 2 * Math.PI * config.radius;
  const progress = total > 0 ? completed / total : 0;
  const isComplete = total > 0 && completed === total;
  const offset = circumference * (1 - progress);

  return (
    <div
      style={{
        position: "relative",
        width: config.width,
        height: config.width,
        flexShrink: 0,
      }}
    >
      <svg
        width={config.width}
        height={config.width}
        viewBox={`0 0 ${config.width} ${config.width}`}
      >
        <circle
          cx={cx}
          cy={cx}
          r={config.radius}
          fill="none"
          stroke="hsl(0 0% 15%)"
          strokeWidth={config.stroke}
        />
        <circle
          cx={cx}
          cy={cx}
          r={config.radius}
          fill="none"
          stroke={isComplete ? "hsl(142 71% 65%)" : "hsl(0 0% 98%)"}
          strokeWidth={config.stroke}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
          transform={`rotate(-90 ${cx} ${cx})`}
          style={{ transition: "stroke-dashoffset 0.5s ease" }}
        />
      </svg>
      {size !== "sm" && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: config.fontSize,
            fontWeight: config.fontWeight,
            color: isComplete ? "hsl(142 71% 65%)" : "hsl(0 0% 98%)",
            fontFamily: '"Geist Mono", monospace',
          }}
        >
          {isComplete ? "✓" : `${Math.round(progress * 100)}%`}
        </div>
      )}
    </div>
  );
}
