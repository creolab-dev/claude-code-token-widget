import type { TokenUsageData, ConnectionStatus } from '$lib/types';
import { getStatusColor } from '$lib/utils/colors';

class TokenState {
  data = $state.raw<TokenUsageData | null>(null);
  status = $state<ConnectionStatus>('loading');
  lastUpdated = $state<Date | null>(null);
  error = $state<string | null>(null);
  isDark = $state(true);

  // --- Rate Limits ---

  get fiveHourUsed(): number {
    return this.data?.rate_limits?.five_hour?.used_percentage ?? 0;
  }

  get sevenDayUsed(): number {
    return this.data?.rate_limits?.seven_day?.used_percentage ?? 0;
  }

  get fiveHourResetsAt(): Date | null {
    const ts = this.data?.rate_limits?.five_hour?.resets_at;
    return ts ? new Date(ts * 1000) : null;
  }

  get sevenDayResetsAt(): Date | null {
    const ts = this.data?.rate_limits?.seven_day?.resets_at;
    return ts ? new Date(ts * 1000) : null;
  }

  // --- Context Window ---

  get usedPercentage(): number {
    return this.data?.context_window.used_percentage ?? 0;
  }

  get totalInputTokens(): number {
    return this.data?.context_window.total_input_tokens ?? 0;
  }

  get totalOutputTokens(): number {
    return this.data?.context_window.total_output_tokens ?? 0;
  }

  get cacheReadTokens(): number {
    return this.data?.context_window.current_usage?.cache_read_input_tokens ?? 0;
  }

  // --- Cost ---

  get costUsd(): number {
    return this.data?.cost.total_cost_usd ?? 0;
  }

  get durationMs(): number {
    return this.data?.cost.total_duration_ms ?? 0;
  }

  // --- Model ---

  get modelName(): string {
    return this.data?.model?.display_name ?? '';
  }

  get sessionId(): string {
    const id = this.data?.session_id ?? '';
    return id.length > 8 ? id.slice(0, 8) : id;
  }

  // --- Status colors (direct hex) ---

  get contextColor(): string {
    return getStatusColor(this.usedPercentage, this.isDark);
  }

  get fiveHourColor(): string {
    return getStatusColor(this.fiveHourUsed, this.isDark);
  }

  get sevenDayColor(): string {
    return getStatusColor(this.sevenDayUsed, this.isDark);
  }

  // --- Actions ---

  update(data: TokenUsageData): void {
    this.data = data;
    this.status = 'connected';
    this.lastUpdated = new Date();
    this.error = null;
  }

  setDisconnected(): void {
    this.status = 'disconnected';
  }

  setError(message: string): void {
    this.status = 'error';
    this.error = message;
  }
}

export const tokenState = new TokenState();
