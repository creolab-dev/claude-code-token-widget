export interface CurrentUsage {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
}

export interface ContextWindow {
  total_input_tokens: number;
  total_output_tokens: number;
  used_percentage: number;
  current_usage: CurrentUsage | null;
}

export interface CostInfo {
  total_cost_usd: number;
  total_duration_ms: number;
  total_api_duration_ms: number;
}

export interface RateLimitEntry {
  used_percentage: number;
  resets_at: number | null;
}

export interface RateLimits {
  five_hour: RateLimitEntry | null;
  seven_day: RateLimitEntry | null;
}

export interface ModelInfo {
  id: string | null;
  display_name: string | null;
}

export interface TokenUsageData {
  context_window: ContextWindow;
  cost: CostInfo;
  session_id: string | null;
  rate_limits: RateLimits | null;
  model: ModelInfo | null;
}

export interface DisplayItems {
  rateLimits: boolean;
  contextWindow: boolean;
  tokenStats: boolean;
  cost: boolean;
  [key: string]: boolean;
}

export interface AlertSettings {
  enabled: boolean;
  thresholds: number[];
}

export interface ScheduleSettings {
  enabled: boolean;
  wakeHour: number;
  wakeMinute: number;
  prompt: string;
  command: string;
}

export interface WidgetSettings {
  currency: 'usd' | 'jpy';
  jpy_rate: number;
  opacity: number;
  always_on_top: boolean;
  theme?: 'dark' | 'light';
  font_size?: number;
  display_items?: DisplayItems;
  alert_settings?: AlertSettings;
  schedule_settings?: ScheduleSettings;
}

export interface HeatmapEntry {
  date: string;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cost_usd: number;
  session_count: number;
}

export interface WatcherStatus {
  active: boolean;
}

export type ConnectionStatus = 'loading' | 'connected' | 'disconnected' | 'error';
