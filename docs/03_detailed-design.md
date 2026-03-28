---
document: "詳細設計書"
project: "Claude Code Token Widget"
version: "1.0"
created: "2026-03-21"
updated: "2026-03-21"
status: draft
---

# 詳細設計書: Claude Code Token Widget

---

## 1. Rust バックエンド詳細

### 1.1 main.rs

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    claude_code_token_widget_lib::run();
}
```

他のロジックは一切書かない。

---

### 1.2 error.rs

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("File watch error: {0}")]
    Watch(#[from] notify_debouncer_mini::notify::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Store error: {0}")]
    Store(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

---

### 1.3 models.rs

```rust
use serde::{Deserialize, Serialize};

/// Claude Code ステータスラインから受信するトークン使用量データ
/// 注: rename_all なし — JSONフィールド名はそのまま snake_case で送受信
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageData {
    #[serde(default)]
    pub context_window: ContextWindow,
    #[serde(default)]
    pub cost: CostInfo,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
    #[serde(default)]
    pub model: Option<ModelInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextWindow {
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub total_output_tokens: u64,
    #[serde(default)]
    pub used_percentage: u8,
    #[serde(default)]
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrentUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: u64,
    #[serde(default)]
    pub total_api_duration_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<RateLimitEntry>,
    #[serde(default)]
    pub seven_day: Option<RateLimitEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitEntry {
    #[serde(default)]
    pub used_percentage: u8,
    #[serde(default)]
    pub resets_at: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// ウィジェット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSettings {
    #[serde(default = "default_currency")]
    pub currency: Currency,
    #[serde(default = "default_jpy_rate")]
    pub jpy_rate: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub display_items: Option<DisplayItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayItems {
    #[serde(default = "default_true")]
    pub rate_limits: bool,
    #[serde(default = "default_true")]
    pub context_window: bool,
    #[serde(default = "default_true")]
    pub token_stats: bool,
    #[serde(default = "default_true")]
    pub cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    #[default]
    Usd,
    Jpy,
}

impl Default for WidgetSettings {
    fn default() -> Self {
        Self {
            currency: Currency::Usd,
            jpy_rate: 150.0,
            opacity: 0.9,
            always_on_top: true,
            theme: "dark".to_string(),
            display_items: None,
        }
    }
}

fn default_currency() -> Currency { Currency::Usd }
fn default_jpy_rate() -> f64 { 150.0 }
fn default_opacity() -> f64 { 0.9 }
fn default_true() -> bool { true }
fn default_theme() -> String { "dark".to_string() }

/// ウォッチャーステータスのイベントペイロード
#[derive(Debug, Clone, Serialize)]
pub struct WatcherStatus {
    pub active: bool,
}
```

---

### 1.4 state.rs

```rust
use crate::models::{TokenUsageData, WidgetSettings};
use std::sync::Mutex;

#[derive(Default)]
pub struct AppStateInner {
    pub token_data: Option<TokenUsageData>,
    pub watcher_active: bool,
}

/// Tauri managed state（Arc は Tauri が内部で管理）
pub type AppState = Mutex<AppStateInner>;
```

---

### 1.5 watcher.rs

```rust
use crate::error::AppResult;
use crate::models::{TokenUsageData, WatcherStatus};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const DEBOUNCE_MS: u64 = 400;

/// token-usage.json のパスを解決
pub fn resolve_token_file_path() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to resolve home directory");
    home.join(".claude").join("token-usage.json")
}

/// ファイルを読み取り、パースして返す
pub fn read_and_parse(path: &Path) -> AppResult<TokenUsageData> {
    let contents = std::fs::read_to_string(path)?;
    let data = serde_json::from_str::<TokenUsageData>(&contents)?;
    Ok(data)
}

/// ファイルウォッチャーを起動し、変更時にイベントを emit する
///
/// 戻り値の Debouncer を保持すること（ドロップすると監視停止）
pub fn start_file_watcher(
    app_handle: AppHandle,
    token_file_path: PathBuf,
) -> AppResult<notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>> {
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        move |result: Result<Vec<DebouncedEvent>, notify_debouncer_mini::notify::Error>| {
            let _ = tx.send(result);
        },
    )?;

    // 親ディレクトリを監視（ファイル未存在でも対応可能）
    let watch_dir = token_file_path
        .parent()
        .expect("Token file path must have a parent directory");
    debouncer
        .watcher()
        .watch(watch_dir, RecursiveMode::NonRecursive)?;

    // イベントループをバックグラウンドスレッドで開始
    let handle = app_handle.clone();
    let path = token_file_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // ウォッチャー開始通知
        let _ = handle.emit("watcher-status", WatcherStatus { active: true });

        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    let relevant = events.iter().any(|e| e.path == path);
                    if relevant {
                        match read_and_parse(&path) {
                            Ok(data) => {
                                // State 更新
                                if let Some(state) =
                                    handle.try_state::<crate::state::AppState>()
                                {
                                    if let Ok(mut guard) = state.lock() {
                                        guard.token_data = Some(data.clone());
                                    }
                                }
                                // フロントエンドに通知
                                let _ = handle.emit("token-updated", &data);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse token file: {e}");
                            }
                        }
                    }
                }
                Ok(Err(errors)) => {
                    eprintln!("Watch errors: {errors:?}");
                    let _ = handle.emit("watcher-error", format!("{errors:?}"));
                }
                Err(_) => {
                    // チャネル切断 = アプリ終了
                    break;
                }
            }
        }

        let _ = handle.emit("watcher-status", WatcherStatus { active: false });
    });

    Ok(debouncer)
}
```

---

### 1.6 commands.rs

```rust
use crate::error::{AppError, AppResult};
use crate::models::{TokenUsageData, WidgetSettings};
use crate::state::AppState;
use crate::watcher;
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;

/// 現在のトークンデータを取得（初期ロード用）
/// State にデータがなければファイルから直接読み取る
#[tauri::command]
pub fn get_current_token(state: State<'_, AppState>) -> AppResult<Option<TokenUsageData>> {
    let guard = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    if guard.token_data.is_some() {
        return Ok(guard.token_data.clone());
    }
    drop(guard);

    // State にデータがない場合、ファイルから直接読み取り
    let path = watcher::resolve_token_file_path();
    if path.exists() {
        match watcher::read_and_parse(&path) {
            Ok(data) => {
                // State にも保存
                if let Ok(mut guard) = state.lock() {
                    guard.token_data = Some(data.clone());
                }
                Ok(Some(data))
            }
            Err(_) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// ウォッチャーの稼働状態を取得
#[tauri::command]
pub fn get_watcher_status(state: State<'_, AppState>) -> AppResult<bool> {
    let guard = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(guard.watcher_active)
}

/// 現在のウィジェット設定を取得
#[tauri::command]
pub fn get_settings(app_handle: tauri::AppHandle) -> AppResult<WidgetSettings> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| AppError::Store(e.to_string()))?;

    let settings = match store.get("settings") {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => WidgetSettings::default(),
    };

    Ok(settings)
}

/// Always on Top を切り替え
#[tauri::command]
pub fn set_always_on_top(app_handle: tauri::AppHandle, value: bool) -> AppResult<()> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.set_always_on_top(value).map_err(|e| AppError::Store(e.to_string()))?;
    }
    Ok(())
}

/// ウィンドウサイズを変更（resizable:false でも動作するように Rust 側で実行）
#[tauri::command]
pub fn resize_window(app_handle: tauri::AppHandle, width: f64, height: f64) -> AppResult<()> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_resizable(true);
        let size = tauri::LogicalSize::new(width, height);
        let _ = window.set_size(tauri::Size::Logical(size));
        let _ = window.set_resizable(false);
    }
    Ok(())
}

/// ウィジェット設定を更新・保存
#[tauri::command]
pub fn update_settings(
    app_handle: tauri::AppHandle,
    settings: WidgetSettings,
) -> AppResult<()> {
    let value =
        serde_json::to_value(&settings).map_err(|e| AppError::Store(e.to_string()))?;

    let store = app_handle
        .store("settings.json")
        .map_err(|e| AppError::Store(e.to_string()))?;
    store.set("settings", value);
    store.save().map_err(|e| AppError::Store(e.to_string()))?;

    Ok(())
}
```

---

### 1.7 tray.rs

```rust
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
    let on_top = MenuItem::with_id(app, "on_top", "Always on Top", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &on_top, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Claude Code Token Widget")
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let window = app.get_webview_window("main");
            match event.id.as_ref() {
                "show" => {
                    if let Some(w) = window {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
                "on_top" => {
                    if let Some(w) = window {
                        if let Ok(current) = w.is_always_on_top() {
                            let _ = w.set_always_on_top(!current);
                        }
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // 左クリックで表示/非表示トグル
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

---

### 1.8 lib.rs

```rust
mod commands;
mod error;
mod models;
mod state;
mod tray;
mod watcher;

use state::AppState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_current_token,
            commands::get_watcher_status,
            commands::get_settings,
            commands::update_settings,
            commands::resize_window,
            commands::set_always_on_top,
        ])
        .setup(|app| {
            // システムトレイ
            tray::setup_tray(app)?;

            let app_handle = app.handle().clone();
            let token_file_path = watcher::resolve_token_file_path();

            // 初期データ読み取り
            if token_file_path.exists() {
                match watcher::read_and_parse(&token_file_path) {
                    Ok(data) => {
                        if let Ok(mut guard) = app.state::<AppState>().lock() {
                            guard.token_data = Some(data.clone());
                            guard.watcher_active = true;
                        }
                        // 初期データをフロントエンドにも emit（setup完了後に遅延送信）
                        let handle = app.handle().clone();
                        let init_data = data;
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            let _ = handle.emit("token-updated", &init_data);
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to parse initial token file: {e}");
                    }
                }
            }

            // ウォッチャー起動（戻り値を State に保管して drop を防ぐ）
            match watcher::start_file_watcher(app_handle, token_file_path) {
                Ok(debouncer) => {
                    app.manage(Mutex::new(Some(debouncer)));
                }
                Err(e) => {
                    eprintln!("Failed to start file watcher: {e}");
                }
            }

            // ウィンドウ表示
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### 1.9 Cargo.toml

```toml
[package]
name = "claude-code-token-widget"
version = "0.1.0"
description = "Real-time Claude Code token usage desktop widget"
authors = ["you"]
edition = "2021"

[lib]
name = "claude_code_token_widget_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-opener = "2"
tauri-plugin-window-state = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
notify-debouncer-mini = "0.5"
dirs = "6"
tokio = { version = "1", features = ["time"] }

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
must_use_candidate = "allow"
unwrap_used = "warn"

[lints.rust]
unsafe_code = "forbid"

[profile.dev]
incremental = true

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
panic = "abort"
strip = true
```

---

### 1.10 tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Claude Code Token Widget",
  "version": "0.1.0",
  "identifier": "com.claude-code-token-widget",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Claude Code Token Widget",
        "width": 300,
        "height": 340,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "visible": false,
        "skipTaskbar": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: data:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost",
      "freezePrototype": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.comodoca.com"
    }
  }
}
```

---

### 1.11 capabilities/default.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Main widget window permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "core:window:allow-start-dragging",
    "core:window:allow-set-position",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-is-visible",
    "core:window:allow-set-focus",
    "core:window:allow-set-always-on-top",
    "core:window:allow-is-always-on-top",
    "core:window:allow-set-size",
    "core:window:allow-set-resizable",
    "core:window:allow-close",
    "core:window:allow-destroy",
    "window-state:default",
    "store:default"
  ]
}
```

---

## 2. データ中継スクリプト詳細

### 2.1 statusline.sh

```bash
#!/bin/bash
# ~/.claude/statusline.sh
#
# Claude Code のステータスラインスクリプト
# stdin から JSON を受け取り、token-usage.json にアトミック書き出し

OUTFILE="$HOME/.claude/token-usage.json"
TMPFILE="${OUTFILE}.tmp"

# stdin を読み取り
INPUT=$(cat)

# 空でなければ書き出し
if [ -n "$INPUT" ]; then
    echo "$INPUT" > "$TMPFILE" && mv "$TMPFILE" "$OUTFILE"
fi
```

### 2.2 Claude Code settings.json への追加設定

```json
{
  "statusline": {
    "command": "bash ~/.claude/statusline.sh"
  }
}
```

---

## 3. Svelte フロントエンド詳細

### 3.1 TypeScript 型定義

注: Rust 側のデータモデル（TokenUsageData 等）は `rename_all = "camelCase"` を付けていないため、
TypeScript 側のフィールド名も snake_case のまま送受信される。

```ts
// src/lib/types/index.ts

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

export interface WidgetSettings {
  currency: 'usd' | 'jpy';
  jpy_rate: number;
  opacity: number;
  always_on_top: boolean;
  theme?: 'dark' | 'light';
  display_items?: DisplayItems;
}

export interface WatcherStatus {
  active: boolean;
}

export type ConnectionStatus = 'loading' | 'connected' | 'disconnected' | 'error';
```

---

### 3.2 Tauri IPC ラッパー

```ts
// src/lib/tauri/commands.ts
import { invoke } from '@tauri-apps/api/core';
import type { TokenUsageData, WidgetSettings } from '$lib/types';

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
```

```ts
// src/lib/tauri/events.ts
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
```

---

### 3.3 グローバル状態

```ts
// src/lib/state/token-state.svelte.ts
import type { TokenUsageData, ConnectionStatus } from '$lib/types';

class TokenState {
  data = $state.raw<TokenUsageData | null>(null);
  status = $state<ConnectionStatus>('loading');
  lastUpdated = $state<Date | null>(null);
  error = $state<string | null>(null);

  // --- Derived getters ---

  get usedPercentage(): number {
    return this.data?.contextWindow.usedPercentage ?? 0;
  }

  get totalInputTokens(): number {
    return this.data?.contextWindow.totalInputTokens ?? 0;
  }

  get totalOutputTokens(): number {
    return this.data?.contextWindow.totalOutputTokens ?? 0;
  }

  get cacheReadTokens(): number {
    return this.data?.contextWindow.currentUsage?.cacheReadInputTokens ?? 0;
  }

  get costUsd(): number {
    return this.data?.cost.totalCostUsd ?? 0;
  }

  get durationMs(): number {
    return this.data?.cost.totalDurationMs ?? 0;
  }

  get sessionId(): string {
    const id = this.data?.sessionId ?? '';
    return id.length > 8 ? id.slice(0, 8) : id;
  }

  // --- Status color ---

  get statusColor(): string {
    const p = this.usedPercentage;
    if (p > 90) return 'var(--status-red)';
    if (p > 75) return 'var(--status-orange)';
    if (p > 50) return 'var(--status-yellow)';
    return 'var(--status-green)';
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
```

```ts
// src/lib/state/settings-state.svelte.ts
import type { WidgetSettings } from '$lib/types';
import { getSettings, updateSettings } from '$lib/tauri/commands';

class SettingsState {
  currency = $state<'usd' | 'jpy'>('usd');
  jpyRate = $state(150.0);
  opacity = $state(0.9);
  alwaysOnTop = $state(true);

  async load(): Promise<void> {
    const settings = await getSettings();
    this.currency = settings.currency;
    this.jpyRate = settings.jpyRate;
    this.opacity = settings.opacity;
    this.alwaysOnTop = settings.alwaysOnTop;
  }

  async save(): Promise<void> {
    await updateSettings({
      currency: this.currency,
      jpyRate: this.jpyRate,
      opacity: this.opacity,
      alwaysOnTop: this.alwaysOnTop,
    });
  }
}

export const settingsState = new SettingsState();
```

---

### 3.4 ユーティリティ関数

```ts
// src/lib/utils/format.ts

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
  return `¥${jpy.toLocaleString('en-US')}`;
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
```

---

### 3.5 コンポーネント詳細

#### +layout.ts

```ts
// src/routes/+layout.ts
export const ssr = false;
export const prerender = true;
```

#### +layout.svelte

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import '$lib/styles/globals.css';
  let { children } = $props();
</script>

<div class="app dark">
  {@render children()}
</div>

<style>
  .app {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }
</style>
```

#### +page.svelte

```svelte
<!-- src/routes/+page.svelte -->
<script lang="ts">
  import Widget from '$lib/components/Widget.svelte';
  import { tokenState } from '$lib/state/token-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { getCurrentToken } from '$lib/tauri/commands';
  import { onTokenUpdated, onWatcherStatus, onWatcherError } from '$lib/tauri/events';

  // 初期化
  $effect(() => {
    // 初期データ取得
    getCurrentToken().then((data) => {
      if (data) {
        tokenState.update(data);
      } else {
        tokenState.setDisconnected();
      }
    });

    // 設定読み込み
    settingsState.load();

    // イベントリスナー登録
    const unlistenToken = onTokenUpdated((data) => {
      tokenState.update(data);
    });
    const unlistenStatus = onWatcherStatus((status) => {
      if (!status.active) tokenState.setDisconnected();
    });
    const unlistenError = onWatcherError((error) => {
      tokenState.setError(error);
    });

    // クリーンアップ
    return () => {
      unlistenToken.then((fn) => fn());
      unlistenStatus.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  });
</script>

<Widget />
```

#### Widget.svelte

```svelte
<!-- src/lib/components/Widget.svelte -->
<script lang="ts">
  import DragHandle from './DragHandle.svelte';
  import ContextBar from './ContextBar.svelte';
  import TokenStats from './TokenStats.svelte';
  import CostDisplay from './CostDisplay.svelte';
  import StatusIndicator from './StatusIndicator.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
</script>

<div class="widget" style="opacity: {settingsState.opacity}">
  <DragHandle />
  <div class="widget-body">
    <ContextBar />
    <TokenStats />
    <CostDisplay />
  </div>
  <StatusIndicator />
</div>

<style>
  .widget {
    width: 300px;
    height: 200px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    font-family: 'Segoe UI', system-ui, sans-serif;
    user-select: none;
    position: relative;
  }

  .widget-body {
    flex: 1;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
```

#### DragHandle.svelte

```svelte
<!-- src/lib/components/DragHandle.svelte -->
<script lang="ts">
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  async function startDrag() {
    const window = getCurrentWebviewWindow();
    await window.startDragging();
  }
</script>

<div class="drag-handle" onmousedown={startDrag} role="banner">
  <span class="title">Token Widget</span>
</div>

<style>
  .drag-handle {
    height: 24px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    cursor: grab;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    border-radius: var(--radius) var(--radius) 0 0;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .title {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.3px;
  }
</style>
```

#### ContextBar.svelte

```svelte
<!-- src/lib/components/ContextBar.svelte -->
<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';

  let percentage = $derived(tokenState.usedPercentage);
  let color = $derived(tokenState.statusColor);
  let glowActive = $derived(percentage > 75);
</script>

<div class="context-bar">
  <span class="label">Context</span>
  <div class="bar-container">
    <div
      class="bar-fill"
      class:glow={glowActive}
      style="width: {percentage}%; background: {color}"
    ></div>
  </div>
  <span class="value" style="color: {color}">{percentage}%</span>
</div>

<style>
  .context-bar {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .label {
    font-size: 11px;
    color: var(--text-secondary);
    width: 48px;
    flex-shrink: 0;
  }

  .bar-container {
    flex: 1;
    height: 8px;
    background: var(--bg-secondary);
    border-radius: 4px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease, background 0.3s ease;
  }

  .bar-fill.glow {
    box-shadow: 0 0 6px currentColor;
  }

  .value {
    font-size: 12px;
    font-weight: 600;
    width: 36px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
```

#### TokenStats.svelte

```svelte
<!-- src/lib/components/TokenStats.svelte -->
<script lang="ts">
  import StatRow from './StatRow.svelte';
  import { tokenState } from '$lib/state/token-state.svelte';
  import { formatTokens } from '$lib/utils/format';

  let inputFormatted = $derived(formatTokens(tokenState.totalInputTokens));
  let outputFormatted = $derived(formatTokens(tokenState.totalOutputTokens));
  let cacheFormatted = $derived(formatTokens(tokenState.cacheReadTokens));
</script>

<div class="token-stats">
  <StatRow label="Input" value={inputFormatted} unit="tokens" />
  <StatRow label="Output" value={outputFormatted} unit="tokens" />
  <StatRow label="Cache" value={cacheFormatted} unit="tokens" />
</div>

<style>
  .token-stats {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
```

#### StatRow.svelte

```svelte
<!-- src/lib/components/StatRow.svelte -->
<script lang="ts">
  interface Props {
    label: string;
    value: string;
    unit: string;
  }
  let { label, value, unit }: Props = $props();
</script>

<div class="stat-row">
  <span class="label">{label}</span>
  <span class="value">{value}</span>
  <span class="unit">{unit}</span>
</div>

<style>
  .stat-row {
    display: flex;
    align-items: baseline;
    gap: 4px;
    font-size: 12px;
    line-height: 1.6;
  }

  .label {
    color: var(--text-secondary);
    width: 48px;
    flex-shrink: 0;
  }

  .value {
    color: var(--text-primary);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    flex: 1;
    text-align: right;
  }

  .unit {
    color: var(--text-secondary);
    font-size: 10px;
    width: 40px;
    flex-shrink: 0;
  }
</style>
```

#### CostDisplay.svelte

```svelte
<!-- src/lib/components/CostDisplay.svelte -->
<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { formatCostUsd, formatCostJpy, formatDuration } from '$lib/utils/format';

  let costFormatted = $derived(
    settingsState.currency === 'jpy'
      ? formatCostJpy(tokenState.costUsd, settingsState.jpyRate)
      : formatCostUsd(tokenState.costUsd)
  );

  let durationFormatted = $derived(formatDuration(tokenState.durationMs));
</script>

<div class="cost-display">
  <span class="cost">{costFormatted}</span>
  <span class="duration">{durationFormatted}</span>
</div>

<style>
  .cost-display {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-top: 4px;
    border-top: 1px solid var(--border);
    font-size: 13px;
  }

  .cost {
    color: var(--text-primary);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .duration {
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }
</style>
```

#### StatusIndicator.svelte

```svelte
<!-- src/lib/components/StatusIndicator.svelte -->
<script lang="ts">
  import { tokenState } from '$lib/state/token-state.svelte';
  import type { ConnectionStatus } from '$lib/types';

  let status = $derived(tokenState.status);

  const colorMap: Record<ConnectionStatus, string> = {
    loading: 'var(--status-yellow)',
    connected: 'var(--status-green)',
    disconnected: '#666666',
    error: 'var(--status-red)',
  };

  let color = $derived(colorMap[status]);
  let pulsing = $derived(status === 'loading' || status === 'error');
</script>

<div class="status-indicator" title={status}>
  <div class="dot" class:pulse={pulsing} style="background: {color}"></div>
</div>

<style>
  .status-indicator {
    position: absolute;
    top: 7px;
    right: 8px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    transition: background 0.3s ease;
  }

  .dot.pulse {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
</style>
```

---

### 3.6 グローバルCSS

```css
/* src/lib/styles/globals.css */

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: transparent;
}

:root {
  --radius: 8px;
  --transition-speed: 150ms;
}

.dark {
  color-scheme: dark;
  --bg-primary: rgba(26, 26, 26, 0.92);
  --bg-secondary: rgba(42, 42, 42, 0.95);
  --text-primary: #f0f0f0;
  --text-secondary: #999999;
  --border: rgba(51, 51, 51, 0.8);
  --accent: #60a5fa;

  --status-green: #22c55e;
  --status-yellow: #eab308;
  --status-orange: #f97316;
  --status-red: #ef4444;
}
```

---

## 4. ファイル一覧（全体）

```
claude-code-token-widget/
├── STATUS.md
├── docs/
│   ├── 00_requirements.md
│   ├── 01_tech-guidelines.md
│   ├── 02_basic-design.md
│   └── 03_detailed-design.md
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── app.html
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Widget.svelte
│   │   │   ├── DragHandle.svelte
│   │   │   ├── ContextBar.svelte
│   │   │   ├── TokenStats.svelte
│   │   │   ├── StatRow.svelte
│   │   │   ├── CostDisplay.svelte
│   │   │   └── StatusIndicator.svelte
│   │   ├── state/
│   │   │   ├── token-state.svelte.ts
│   │   │   └── settings-state.svelte.ts
│   │   ├── tauri/
│   │   │   ├── commands.ts
│   │   │   └── events.ts
│   │   ├── types/
│   │   │   └── index.ts
│   │   ├── utils/
│   │   │   └── format.ts
│   │   └── styles/
│   │       └── globals.css
│   └── routes/
│       ├── +layout.ts
│       ├── +layout.svelte
│       └── +page.svelte
├── static/
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── rustfmt.toml
    ├── capabilities/
    │   └── default.json
    ├── icons/
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── commands.rs
        ├── error.rs
        ├── models.rs
        ├── state.rs
        ├── watcher.rs
        └── tray.rs
```

---

## 5. 関連ドキュメント

| ドキュメント | パス | 状態 |
|------------|------|------|
| 要件定義書 | `docs/00_requirements.md` | 完了 |
| 技術ガイドライン | `docs/01_tech-guidelines.md` | 完了 |
| 基本設計書 | `docs/02_basic-design.md` | 完了 |
| 詳細設計書 | `docs/03_detailed-design.md` | 本書 |
