// Fractional sort keys: drop an item by computing a key between its neighbours.
// f64 precision affords ~50 reorders before adjacent sort_keys collide; a
// `RebalanceColumn` operation lands in Spec #3.
export function midpoint(before?: number, after?: number): number {
  if (before != null && after != null) return (before + after) / 2;
  if (before != null) return before + 1024; // dropped at the bottom
  if (after != null) return after - 1024; // dropped at the top
  return 1024; // empty column
}
