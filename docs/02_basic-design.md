---
document: "基本設計書"
project: "Claude Code Token Widget"
version: "1.0"
created: "2026-03-20"
updated: "2026-03-21"
status: draft
---

# 基本設計書: Claude Code Token Widget

---

## 1. システム全体構成

### 1.1 アーキテクチャ概要

本アプリは3つのレイヤーで構成される。

```
┌─────────────────────────────────────────────────────────┐
│                    Claude Code (外部)                     │
│  ステータスライン機能 → statusline.sh を呼び出し           │
└──────────────────────────┬──────────────────────────────┘
                           │ stdin (JSON)
                           ▼
┌─────────────────────────────────────────────────────────┐
│              データ中継レイヤー (Shell Script)             │
│  statusline.sh: stdin → ~/.claude/token-usage.json       │
└──────────────────────────┬──────────────────────────────┘
                           │ ファイル書き込み
                           ▼
┌─────────────────────────────────────────────────────────┐
│               Tauri アプリケーション                       │
│                                                          │
│  ┌─────────────────────────────────────────────┐        │
│  │          Rust バックエンド                     │        │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────┐ │        │
│  │  │ Watcher  │→│  Parser  │→│   State   │ │        │
│  │  │ (notify) │  │ (serde)  │  │ (Mutex)   │ │        │
│  │  └──────────┘  └──────────┘  └─────┬─────┘ │        │
│  │                                     │       │        │
│  │  ┌──────────┐                ┌─────┴─────┐ │        │
│  │  │ Commands │                │  Emitter  │ │        │
│  │  │ (invoke) │                │  (events) │ │        │
│  │  └────┬─────┘                └─────┬─────┘ │        │
│  └───────┼────────────────────────────┼───────┘        │
│          │ IPC                        │ IPC             │
│  ┌───────┼────────────────────────────┼───────┐        │
│  │       ▼      Svelte フロントエンド   ▼       │        │
│  │  ┌──────────┐                ┌───────────┐ │        │
│  │  │ Commands │                │  Events   │ │        │
│  │  │ (invoke) │                │ (listen)  │ │        │
│  │  └────┬─────┘                └─────┬─────┘ │        │
│  │       └──────────┬─────────────────┘       │        │
│  │                  ▼                          │        │
│  │          ┌──────────────┐                   │        │
│  │          │ TokenState   │                   │        │
│  │          │ (.svelte.ts) │                   │        │
│  │          └──────┬───────┘                   │        │
│  │                 ▼                           │        │
│  │          ┌──────────────┐                   │        │
│  │          │  UI Components│                  │        │
│  │          │  (Svelte)    │                   │        │
│  │          └──────────────┘                   │        │
│  └─────────────────────────────────────────────┘        │
│                                                          │
│  ┌─────────────────────────────────────────────┐        │
│  │  システムトレイ (Rust)                        │        │
│  │  表示/非表示、always-on-top切替、終了          │        │
│  └─────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────┘
```

### 1.2 通信フロー

```
[起動時]
  Svelte → invoke('get_current_token') → Rust → State → レスポンス → Svelte表示

[リアルタイム更新]
  Claude Code → statusline.sh → token-usage.json
    → Watcher(検知) → Parser → State更新 + emit('token-updated') → Svelte更新

[設定変更]
  Svelte → invoke('update_settings') → Rust → Store保存
```

---

## 2. Rust バックエンド設計

### 2.1 モジュール構成

```
src-tauri/src/
├── main.rs          # エントリーポイント（lib::run() を呼ぶだけ）
├── lib.rs           # アプリビルダー、プラグイン登録、セットアップ
├── commands.rs      # Tauri コマンド定義
├── error.rs         # AppError 型定義
├── models.rs        # データモデル（serde 構造体）
├── state.rs         # AppState 定義
├── watcher.rs       # ファイル監視ロジック
└── tray.rs          # システムトレイ構築
```

### 2.2 モジュール責務

| モジュール | 責務 | 依存 |
|-----------|------|------|
| `main.rs` | プロセス起動 | `lib` |
| `lib.rs` | アプリ構築・初期化 | 全モジュール |
| `commands.rs` | IPC コマンド処理 | `state`, `models`, `error` |
| `error.rs` | エラー型定義・シリアライズ | なし |
| `models.rs` | データ構造定義 | なし |
| `state.rs` | アプリケーション状態 | `models` |
| `watcher.rs` | ファイル変更検知・データ送信 | `models`, `error` |
| `tray.rs` | トレイメニュー・イベント | なし |

### 2.3 依存クレート

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-opener = "2"
tauri-plugin-window-state = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
notify-debouncer-mini = "0.5"
dirs = "6"                          # ホームディレクトリ解決
tokio = { version = "1", features = ["time"] }  # 非同期タイマー
```

### 2.4 lib.rs の初期化フロー

```
run()
  │
  ├─ Builder::default()
  │    ├─ .plugin(opener)
  │    ├─ .plugin(window-state)
  │    ├─ .plugin(store)
  │    ├─ .manage(AppState::default())
  │    ├─ .invoke_handler(6 commands)
  │    └─ .setup(|app| {
  │         ├─ setup_tray(app)
  │         ├─ resolve_token_file_path()
  │         ├─ initial_read_and_parse() → State更新
  │         ├─ async spawn: 500ms後に初期データemit
  │         ├─ start_file_watcher(app_handle, path)
  │         │    └─ Debouncer を State に保管
  │         └─ show_main_window()
  │       })
  │
  └─ .run(tauri::generate_context!())
```

---

## 3. データモデル設計

### 3.1 入力データ（Claude Code ステータスライン JSON）

```
TokenUsageData
├── context_window: ContextWindow
│   ├── total_input_tokens: u64
│   ├── total_output_tokens: u64
│   ├── used_percentage: u8
│   └── current_usage: Option<CurrentUsage>
│       ├── input_tokens: u64
│       ├── output_tokens: u64
│       ├── cache_creation_input_tokens: u64
│       └── cache_read_input_tokens: u64
├── cost: CostInfo
│   ├── total_cost_usd: f64
│   ├── total_duration_ms: u64
│   └── total_api_duration_ms: u64
├── session_id: Option<String>
├── rate_limits: Option<RateLimits>
│   ├── five_hour: Option<RateLimitEntry>
│   │   ├── used_percentage: u8
│   │   └── resets_at: Option<u64>
│   └── seven_day: Option<RateLimitEntry>
│       ├── used_percentage: u8
│       └── resets_at: Option<u64>
└── model: Option<ModelInfo>
    ├── id: Option<String>
    └── display_name: Option<String>
```

全フィールドに `#[serde(default)]` を付与し、フィールド欠損に対して堅牢にする。

### 3.2 アプリケーション状態

```
AppStateInner
├── token_data: Option<TokenUsageData>   # 最新のトークンデータ
└── watcher_active: bool                  # ウォッチャー稼働状態

AppState = Mutex<AppStateInner>
```

### 3.3 ウィジェット設定

```
WidgetSettings
├── currency: Currency (USD | JPY)
├── jpy_rate: f64                  # JPY/USD レート（手動設定）
├── opacity: f64                   # 透過度 (0.0 - 1.0)
├── always_on_top: bool
├── theme: String                  # テーマ（"dark" | "light"）
└── display_items: Option<DisplayItems>
    ├── rate_limits: bool
    ├── context_window: bool
    ├── token_stats: bool
    └── cost: bool
```

設定は `tauri-plugin-store` でJSONファイルに永続化:
- 保存先: `~/.claude/token-widget-settings.json`

### 3.4 フロントエンド向けイベントペイロード

| イベント名 | ペイロード | 発火タイミング |
|-----------|----------|--------------|
| `token-updated` | `TokenUsageData` | ファイル変更検知時 |
| `watcher-status` | `{ active: bool }` | ウォッチャー状態変化時 |
| `watcher-error` | `string` | ファイル監視エラー時 |

---

## 4. ファイル監視設計

### 4.1 監視対象

- **ファイルパス**: `~/.claude/token-usage.json`
- **監視方式**: 親ディレクトリ（`~/.claude/`）を `RecursiveMode::NonRecursive` で監視
- **理由**: ファイルがまだ存在しない状態でも監視を開始できる

### 4.2 処理フロー

```
notify-debouncer-mini (400ms debounce)
  │
  ├─ イベント受信
  │   ├─ パスが token-usage.json に一致？
  │   │   ├─ YES → ファイル読み取り
  │   │   │        ├─ 成功 → JSON パース
  │   │   │        │         ├─ 成功 → State 更新 + emit("token-updated")
  │   │   │        │         └─ 失敗 → warn ログ（UI に影響なし）
  │   │   │        └─ 失敗 → warn ログ
  │   │   └─ NO  → 無視
  │   └─ エラー → error ログ + emit("watcher-error")
  │
  └─ チャネル切断 → ループ終了（アプリ終了時）
```

### 4.3 エラーハンドリング

| 状態 | 対応 |
|------|------|
| ファイルが存在しない | 「未接続」表示。ウォッチャーは継続（ファイル作成を待つ） |
| JSON パースエラー | 直前のデータを維持。ログに記録 |
| ファイル読み取りエラー | 直前のデータを維持。ログに記録 |
| ウォッチャー自体のエラー | エラーイベントをフロントエンドに通知 |

---

## 5. Tauri コマンド設計

### 5.1 コマンド一覧

| コマンド名 | 引数 | 戻り値 | 用途 |
|-----------|------|--------|------|
| `get_current_token` | なし | `Option<TokenUsageData>` | 初期データ取得（State→ファイルフォールバック） |
| `get_settings` | なし | `WidgetSettings` | 現在の設定取得 |
| `update_settings` | `settings: WidgetSettings` | `()` | 設定変更・保存 |
| `get_watcher_status` | なし | `bool` | ウォッチャー稼働確認 |
| `resize_window` | `width: f64, height: f64` | `()` | ウィンドウサイズ変更（resizable:falseでも動作） |
| `set_always_on_top` | `value: bool` | `()` | Always on Top 切り替え |

### 5.2 コマンドの同期/非同期

| コマンド | 方式 | 理由 |
|---------|------|------|
| `get_current_token` | sync | State の Mutex ロック + ファイルフォールバック |
| `get_settings` | sync | StoreExt 経由の読み取り |
| `update_settings` | sync | StoreExt 経由の書き込み |
| `get_watcher_status` | sync | State の読み取りのみ |
| `resize_window` | sync | ウィンドウAPI呼び出し |
| `set_always_on_top` | sync | ウィンドウAPI呼び出し |

---

## 6. Svelte フロントエンド設計

### 6.1 コンポーネントツリー

```
+page.svelte (ルート)
└── Widget.svelte (ウィジェット本体)
    ├── DragHandle.svelte (ドラッグ可能エリア / タイトル)
    ├── ContextBar.svelte (コンテキスト使用率プログレスバー)
    ├── TokenStats.svelte (トークン数表示)
    │   └── StatRow.svelte (1行の統計表示) × 3
    ├── CostDisplay.svelte (コスト・時間表示)
    └── StatusIndicator.svelte (接続状態インジケータ)
```

### 6.2 コンポーネント責務

| コンポーネント | 責務 | 受け取るデータ |
|--------------|------|--------------|
| `Widget.svelte` | レイアウト管理、テーマ適用 | — |
| `DragHandle.svelte` | ウィンドウドラッグ、タイトル表示 | — |
| `ContextBar.svelte` | 使用率のプログレスバー・色変化 | `usedPercentage` |
| `TokenStats.svelte` | トークン数の一覧表示 | `inputTokens`, `outputTokens`, `cacheTokens` |
| `StatRow.svelte` | ラベル + 値の1行表示 | `label`, `value`, `unit` |
| `CostDisplay.svelte` | コスト表示（USD/JPY）、経過時間 | `costUsd`, `durationMs` |
| `StatusIndicator.svelte` | 接続状態のドットインジケータ | `connected` |

### 6.3 状態管理

**グローバル状態**（`lib/state/token-state.svelte.ts`）:

```
TokenState (class)
├── data: $state.raw<TokenUsageData | null>    # Tauri イベントで更新
├── connected: $state<boolean>                  # ウォッチャー接続状態
├── lastUpdated: $state<Date | null>            # 最終更新時刻
│
├── get usedPercentage(): number                # derived的なgetter
├── get totalInputTokens(): number
├── get totalOutputTokens(): number
├── get cacheReadTokens(): number
├── get costUsd(): number
├── get durationMs(): number
└── get sessionId(): string
```

**設定状態**（`lib/state/settings-state.svelte.ts`）:

```
SettingsState (class)
├── currency: $state<'USD' | 'JPY'>
├── jpyRate: $state<number>
├── opacity: $state<number>
└── alwaysOnTop: $state<boolean>
```

### 6.4 データフロー

```
[初期化]
+page.svelte onMount
  ├─ invoke('get_current_token') → tokenState.data = result
  ├─ invoke('get_settings') → settingsState に反映
  └─ listen('token-updated') → tokenState.data = event.payload

[リアルタイム更新]
Rust emit('token-updated', data)
  → events.ts の listen
    → tokenState.data = payload（$state.raw 再代入でUI更新）

[設定変更]
SettingsPanel → invoke('update_settings', newSettings)
  → Rust で Store 保存
  → settingsState 更新
```

---

## 7. UI 設計

### 7.1 ウィジェットレイアウト

```
┌──────────────────────────────────┐
│ ● Token Widget         ─ ×     │ ← DragHandle (12px)
├──────────────────────────────────┤
│                                  │
│  Context  ████████████░░  78%   │ ← ContextBar (28px)
│                                  │
│  Input      12,345  tokens      │ ← StatRow
│  Output      3,456  tokens      │ ← StatRow
│  Cache       8,901  tokens      │ ← StatRow
│                                  │
│  $0.0234         00:15:32       │ ← CostDisplay (20px)
│                                  │
└──────────────────────────────────┘
       300px × 200px
```

### 7.2 色設計（ダークテーマ）

| 要素 | 色 | 用途 |
|------|-----|------|
| 背景 | `#1a1a1a` (90% opacity) | ウィジェット背景 |
| テキスト（主） | `#f0f0f0` | 数値、ラベル |
| テキスト（副） | `#999999` | 単位、補足 |
| アクセント | `#60a5fa` | タイトル、ハイライト |
| ボーダー | `#333333` | セクション区切り |

### 7.3 コンテキスト使用率の色

| 使用率 | バーの色 | 背景グロー |
|--------|---------|-----------|
| 0-50% | `#22c55e` (緑) | なし |
| 51-75% | `#eab308` (黄) | なし |
| 76-90% | `#f97316` (橙) | 薄い橙グロー |
| 91-100% | `#ef4444` (赤) | 薄い赤グロー |

### 7.4 数値フォーマット

| データ | フォーマット | 例 |
|--------|------------|-----|
| トークン数 | 3桁カンマ区切り | `12,345` |
| コスト (USD) | $小数4桁 | `$0.0234` |
| コスト (JPY) | ¥整数カンマ区切り | `¥3` |
| 使用率 | 整数% | `78%` |
| 経過時間 | HH:MM:SS | `00:15:32` |
| セッションID | 先頭8文字 | `abc12345` |

---

## 8. 設定管理設計

### 8.1 設定の永続化

| 項目 | 方式 | 保存先 |
|------|------|--------|
| ウィンドウ位置・サイズ | `tauri-plugin-window-state` | Tauri自動管理 |
| ウィジェット設定 | `tauri-plugin-store` | `~/.claude/token-widget-settings.json` |

### 8.2 設定ファイル構造

```json
{
  "currency": "USD",
  "jpyRate": 150.0,
  "opacity": 0.9,
  "alwaysOnTop": true
}
```

### 8.3 設定変更のフロー

```
UI操作 → invoke('update_settings') → Rust
  ├─ Store にJSONとして保存
  ├─ AppState 更新
  └─ ウィンドウプロパティに反映（opacity, alwaysOnTop）
```

---

## 9. データ中継スクリプト設計

### 9.1 statusline.sh

Claude Code のステータスライン機能が呼び出すスクリプト。

**入力**: stdin から JSON（Claude Code が提供）
**出力**: `~/.claude/token-usage.json` にアトミック書き込み

```bash
#!/bin/bash
# ~/.claude/statusline.sh

# stdin から JSON を読み取り、ファイルに書き出す
INPUT=$(cat)

# アトミック書き込み（一時ファイル → リネーム）
TMPFILE="$HOME/.claude/token-usage.json.tmp"
OUTFILE="$HOME/.claude/token-usage.json"

echo "$INPUT" > "$TMPFILE"
mv "$TMPFILE" "$OUTFILE"
```

**アトミック書き込みの理由**: Watcher がファイルの書き込み途中を読むのを防ぐ。

### 9.2 Claude Code 設定

`~/.claude/settings.json` に追加:

```json
{
  "statusline": {
    "command": "~/.claude/statusline.sh"
  }
}
```

---

## 10. 状態遷移設計

### 10.1 ウィジェット接続状態

```
        ┌───────────┐
        │ 起動中     │
        └─────┬─────┘
              │ setup 完了
              ▼
  ┌─ ファイル存在? ─┐
  │ YES             │ NO
  ▼                 ▼
┌──────┐     ┌──────────┐
│ 接続  │     │ 未接続    │
│ 済み  │     │ (待機中)  │
└──┬───┘     └─────┬────┘
   │               │ ファイル作成検知
   │               ▼
   │         ┌──────────┐
   │         │ 接続済み   │
   │         └──────────┘
   │
   │ ファイル削除
   ▼
┌──────────┐
│ 未接続    │ ← ファイル再作成で復帰
│ (切断)    │
└──────────┘
```

### 10.2 UI状態による表示

| 状態 | StatusIndicator | データ表示 |
|------|----------------|-----------|
| 起動中 | 黄色点滅 | ローディング |
| 接続済み | 緑色 | リアルタイムデータ |
| 未接続（待機中） | グレー | 「Claude Code を起動してください」 |
| 未接続（切断） | 赤色 | 最終データを薄く表示 |
| エラー | 赤色点滅 | エラーメッセージ |

---

## 11. 外部依存関係

### 11.1 Rust クレート

| クレート | バージョン | 用途 |
|---------|----------|------|
| `tauri` | 2.x | アプリフレームワーク |
| `tauri-plugin-opener` | 2.x | URL/ファイルオープナー |
| `tauri-plugin-window-state` | 2.x | ウィンドウ位置永続化 |
| `tauri-plugin-store` | 2.x | 設定永続化 |
| `serde` + `serde_json` | 1.x | シリアライゼーション |
| `thiserror` | 2.x | エラー型 |
| `notify-debouncer-mini` | 0.5.x | ファイル監視 |
| `dirs` | 6.x | ホームディレクトリ解決 |
| `tokio` | 1.x | 非同期タイマー（初期データemit待ち） |

### 11.2 npm パッケージ

| パッケージ | 用途 |
|-----------|------|
| `@tauri-apps/api` | Tauri IPC |
| `@tauri-apps/plugin-window-state` | ウィンドウ状態 |
| `@tauri-apps/plugin-store` | 設定ストア |
| `@sveltejs/adapter-static` | SPA ビルド |

### 11.3 システム依存

| 依存 | バージョン | 備考 |
|------|----------|------|
| WebView2 | OS同梱 | Windows 11 プリインストール |
| Rust toolchain | stable | ビルド時のみ |
| Node.js | 20+ | ビルド時のみ |

---

## 12. 関連ドキュメント

| ドキュメント | パス | 状態 |
|------------|------|------|
| 要件定義書 | `docs/00_requirements.md` | 完了 |
| 技術ガイドライン | `docs/01_tech-guidelines.md` | 完了 |
| 詳細設計書 | `docs/03_detailed-design.md` | 完了 |
| 技術調査レポート | `eal-cc-company/.company/research/topics/claude-code-token-widget.md` | 完了 |
| PMプロジェクト | `eal-cc-company/.company/pm/projects/claude-code-token-widget.md` | 進行中 |
