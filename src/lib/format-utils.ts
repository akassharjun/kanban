export function formatTimestamp(timestamp: string): string {
  if (timestamp.length < 10) return timestamp;
  const date = timestamp.slice(0, 10);
  const time = timestamp.length >= 19 ? timestamp.slice(11, 19) : "";
  const today = new Date().toISOString().slice(0, 10);
  if (date === today) return time;
  return `${date.slice(5)} ${time}`;
}

export function formatTime(timestamp: string): string {
  if (timestamp.length >= 19) return timestamp.slice(11, 19);
  return timestamp;
}

export function normalizeAgentType(agentType: string | null): string {
  const t = agentType || "custom";
  if (t === "claude-code") return "claude";
  return t;
}
