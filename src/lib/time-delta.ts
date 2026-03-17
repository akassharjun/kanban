export function computeTimeDelta(
  prevTimestamp: string,
  currTimestamp: string,
): string | null {
  const toMs = (ts: string) => new Date(ts.replace(" ", "T") + "Z").getTime();
  const prev = toMs(prevTimestamp);
  const curr = toMs(currTimestamp);
  const delta = curr - prev;
  if (delta >= 1000) {
    return delta < 60000
      ? `+${Math.round(delta / 1000)}s`
      : `+${Math.round(delta / 60000)}m`;
  }
  return null;
}
