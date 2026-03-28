import type { WidgetSettings, AlertSettings, ScheduleSettings } from '$lib/types';
import { getSettings, updateSettings, setAlwaysOnTopCmd } from '$lib/tauri/commands';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export interface DisplayItems {
  rateLimits: boolean;
  contextWindow: boolean;
  tokenStats: boolean;
  cost: boolean;
  [key: string]: boolean;
}

class SettingsState {
  currency = $state<'usd' | 'jpy'>('usd');
  jpyRate = $state(150.0);
  opacity = $state(0.9);
  alwaysOnTop = $state(true);
  theme = $state<'dark' | 'light'>('dark');
  fontSize = $state(13);
  displayItems = $state<DisplayItems>({
    rateLimits: true,
    contextWindow: true,
    tokenStats: true,
    cost: true,
  });
  alertSettings = $state<AlertSettings>({
    enabled: true,
    thresholds: [50, 75, 90],
  });
  scheduleSettings = $state<ScheduleSettings>({
    enabled: false,
    wakeHour: 7,
    wakeMinute: 0,
    prompt: 'hello',
    command: 'claude',
  });
  async load(): Promise<void> {
    try {
      const settings = await getSettings();
      this.currency = settings.currency;
      this.jpyRate = settings.jpy_rate;
      this.opacity = settings.opacity;
      this.alwaysOnTop = settings.always_on_top;
      if (settings.theme) {
        this.theme = settings.theme;
      }
      if (settings.font_size) {
        this.fontSize = settings.font_size;
      }
      if (settings.display_items) {
        this.displayItems = { ...this.displayItems, ...settings.display_items };
      }
      if (settings.alert_settings) {
        this.alertSettings = { ...this.alertSettings, ...settings.alert_settings };
      }
      if (settings.schedule_settings) {
        this.scheduleSettings = { ...this.scheduleSettings, ...settings.schedule_settings };
      }
      // 起動時: ウィンドウの実際の状態を読み取ってトグルを同期
      try {
        const win = getCurrentWebviewWindow();
        this.alwaysOnTop = await win.isAlwaysOnTop();
      } catch {
        // フォールバック: 設定値を適用
        await setAlwaysOnTopCmd(this.alwaysOnTop);
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async save(): Promise<void> {
    await updateSettings({
      currency: this.currency,
      jpy_rate: this.jpyRate,
      opacity: this.opacity,
      always_on_top: this.alwaysOnTop,
      theme: this.theme,
      font_size: this.fontSize,
      display_items: this.displayItems,
      alert_settings: this.alertSettings,
      schedule_settings: this.scheduleSettings,
    });
  }
}

export const settingsState = new SettingsState();
