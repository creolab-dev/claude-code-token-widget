/** トークン数を 3 桁カンマ区切りにフォーマット */
export function formatTokens(value: number): string {
  return value.toLocaleString('en-US');
}

/** USD コストを $X.XXXX 形式にフォーマット */
export function formatCostUsd(value: number): string {
  return `$${value.toFixed(4)}`;
}

/** JPY コストを ¥X,XXX 形式にフォーマット */
export function formatCostJpy(usd: number, rate: number): string {
  const jpy = Math.round(usd * rate);
  return `\u00a5${jpy.toLocaleString('en-US')}`;
}

/** ミリ秒を HH:MM:SS 形式にフォーマット */
export function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return [hours, minutes, seconds]
    .map((v) => v.toString().padStart(2, '0'))
    .join(':');
}
