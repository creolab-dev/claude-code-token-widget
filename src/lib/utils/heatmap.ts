export function getMaxCost(entries: { total_cost_usd: number }[]): number {
  if (entries.length === 0) return 1;
  return Math.max(...entries.map((e) => e.total_cost_usd), 0.01);
}

export function getIntensity(cost: number, maxCost: number): number {
  if (cost <= 0) return 0;
  const ratio = cost / maxCost;
  if (ratio > 0.75) return 4;
  if (ratio > 0.5) return 3;
  if (ratio > 0.25) return 2;
  return 1;
}
