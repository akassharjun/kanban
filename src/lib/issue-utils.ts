export function getAgentType(
  memberName: string,
): { type: string; color: string } | null {
  if (memberName.startsWith("[claude]"))
    return { type: "claude", color: "bg-orange-500/20 text-orange-400" };
  if (memberName.startsWith("[codex]"))
    return { type: "codex", color: "bg-green-500/20 text-green-400" };
  if (memberName.startsWith("[gemini]"))
    return { type: "gemini", color: "bg-blue-500/20 text-blue-400" };
  return null;
}

export function safeJsonParse<T>(str: string | null | undefined, fallback: T): T {
  if (str == null) return fallback;
  try {
    return JSON.parse(str) as T;
  } catch {
    return fallback;
  }
}
