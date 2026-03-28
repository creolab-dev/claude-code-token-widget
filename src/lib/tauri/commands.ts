import { invoke } from '@tauri-apps/api/core';
import type { TokenUsageData, WidgetSettings, HeatmapEntry } from '$lib/types';

export async function getCurrentToken(): Promise<TokenUsageData | null> {
  return invoke<TokenUsageData | null>('get_current_token');
}

export async function getWatcherStatus(): Promise<boolean> {
  return invoke<boolean>('get_watcher_status');
}

export async function getSettings(): Promise<WidgetSettings> {
  return invoke<WidgetSettings>('get_settings');
}

export async function updateSettings(settings: WidgetSettings): Promise<void> {
  return invoke('update_settings', { settings });
}

export async function resizeWindow(width: number, height: number): Promise<void> {
  return invoke('resize_window', { width, height });
}

export async function setAlwaysOnTopCmd(value: boolean): Promise<void> {
  return invoke('set_always_on_top', { value });
}

export async function getHeatmapData(days?: number): Promise<HeatmapEntry[]> {
  return invoke<HeatmapEntry[]>('get_heatmap_data', { days: days ?? 365 });
}
