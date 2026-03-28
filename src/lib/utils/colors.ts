/** 使用率に応じた色コードを返す */
export function getStatusColor(percentage: number, isDark: boolean): string {
  if (isDark) {
    if (percentage > 90) return '#ef4444';
    if (percentage > 75) return '#f97316';
    if (percentage > 50) return '#eab308';
    return '#22c55e';
  } else {
    if (percentage > 90) return '#dc2626';
    if (percentage > 75) return '#ea580c';
    if (percentage > 50) return '#ca8a04';
    return '#16a34a';
  }
}
