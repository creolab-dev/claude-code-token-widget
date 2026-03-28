import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { TokenUsageData, WatcherStatus } from '$lib/types';

export function onTokenUpdated(
  callback: (data: TokenUsageData) => void
): Promise<UnlistenFn> {
  return listen<TokenUsageData>('token-updated', (event) => {
    callback(event.payload);
  });
}

export function onWatcherStatus(
  callback: (status: WatcherStatus) => void
): Promise<UnlistenFn> {
  return listen<WatcherStatus>('watcher-status', (event) => {
    callback(event.payload);
  });
}

export function onWatcherError(
  callback: (error: string) => void
): Promise<UnlistenFn> {
  return listen<string>('watcher-error', (event) => {
    callback(event.payload);
  });
}
