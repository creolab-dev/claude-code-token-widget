---
document: "技術ガイドライン"
project: "Claude Code Token Widget"
version: "1.0"
created: "2026-03-20"
updated: "2026-03-21"
status: draft
---

# 技術ガイドライン: Claude Code Token Widget

本ドキュメントは Tauri 2.0 / Svelte 5 / Rust のベストプラクティスをまとめたリファレンスである。
実装中に繰り返し参照することを想定している。

---

## 目次

1. [プロジェクト構成](#1-プロジェクト構成)
2. [Tauri 2.0](#2-tauri-20)
3. [Rust バックエンド](#3-rust-バックエンド)
4. [Svelte 5 フロントエンド](#4-svelte-5-フロントエンド)
5. [IPC（Rust ↔ Svelte 通信）](#5-ipcrust--svelte-通信)
6. [パフォーマンス最適化](#6-パフォーマンス最適化)
7. [セキュリティ](#7-セキュリティ)
8. [テスト](#8-テスト)
9. [ビルド・配布](#9-ビルド配布)

---

## 1. プロジェクト構成

```
claude-code-token-widget/
├── docs/                         # 設計書・ガイドライン
├── package.json
├── svelte.config.js              # adapter-static, ssr = false
├── vite.config.ts
├── tsconfig.json
├── src/                          # Svelte フロントエンド
│   ├── app.html
│   ├── app.css
│   ├── lib/
│   │   ├── components/           # UIコンポーネント
│   │   ├── state/                # リアクティブ状態（.svelte.ts）
│   │   ├── tauri/                # Tauri IPC ラッパー
│   │   │   ├── commands.ts       # invoke の型安全ラッパー
│   │   │   └── events.ts         # listen ヘルパー
│   │   ├── types/                # 共有型定義
│   │   └── styles/               # グローバルCSS・テーマトークン
│   └── routes/
│       ├── +layout.svelte
│       ├── +layout.ts            # export const ssr = false;
│       └── +page.svelte
├── static/
└── src-tauri/                    # Rust バックエンド
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.rs
    ├── tauri.conf.json
    ├── rustfmt.toml
    ├── capabilities/
    │   └── default.json
    ├── icons/
    └── src/
        ├── main.rs               # エントリーポイント（薄いシム）
        ├── lib.rs                 # アプリセットアップ・コマンド登録
        ├── commands.rs            # Tauri コマンド
        ├── error.rs              # エラー型（thiserror）
        ├── models.rs             # serde 構造体
        ├── state.rs              # アプリ状態管理
        ├── watcher.rs            # ファイル監視（notify）
        └── tray.rs               # システムトレイ
```

### SvelteKit 設定のポイント

- `@sveltejs/adapter-static` + `fallback: 'index.html'` でSPAモード
- `src/routes/+layout.ts` で `export const ssr = false;` — SSR無効化でTauri APIが直接使える
- `tauri.conf.json` の `frontendDist` を `"../build"` に設定

---

## 2. Tauri 2.0

### 2.1 tauri.conf.json（ウィジェット向け設定）

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: data:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost",
      "freezePrototype": true
    },
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
    ]
  }
}
```

| プロパティ | 値 | 理由 |
|-----------|-----|------|
| `decorations: false` | フレームレス | ウィジェットらしい見た目 |
| `transparent: true` | 背景透過 | フロントエンド側でもbody/htmlを透過設定 |
| `alwaysOnTop: true` | 常に最前面 | ウィジェット必須 |
| `visible: false` | 初期非表示 | 起動時のフラッシュ防止。初期化後にshowする |
| `skipTaskbar: true` | タスクバー非表示 | ウィジェット挙動 |
| `resizable: false` | 固定サイズ | ウィジェットは固定 |

### 2.2 Capabilities（権限設定）

`src-tauri/capabilities/default.json` で最小限の権限のみ付与:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Permissions for the main widget window",
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

**原則**: 各ウィンドウに必要な権限のみ付与（最小権限の原則）。

### 2.3 システムトレイ

`Cargo.toml` の feature 設定:
```toml
tauri = { version = "2", features = ["tray-icon", "image-png"] }
```

実装のポイント:
- 左クリック: ウィジェットの表示/非表示トグル
- 右クリック: コンテキストメニュー（Show / always-on-top切替 / Quit）
- `menu_on_left_click(false)` でメニューは右クリックのみに

### 2.4 ウィンドウ位置の永続化

`tauri-plugin-window-state` を使用:

```bash
cargo add tauri-plugin-window-state
npm install @tauri-apps/plugin-window-state
```

`visible: false` と組み合わせることで、デフォルト位置でフラッシュしてから復元位置に移動する問題を防ぐ。

---

## 3. Rust バックエンド

### 3.1 main.rs（薄いシムに保つ）

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}
```

実際のロジックは全て `lib.rs` に書く。

### 3.2 エラーハンドリング（thiserror）

**`anyhow` ではなく `thiserror` を使う**。理由: Tauri コマンドの戻り値は `serde::Serialize` が必須。`anyhow::Error` は `Serialize` を実装できない。

```rust
// error.rs
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("File watch error: {0}")]
    Watch(#[from] notify::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

**絶対にコマンド内で panic しない**。sync コマンドの panic はアプリ全体をクラッシュさせる。

### 3.3 シリアライゼーション（serde）

**注**: データモデル（`TokenUsageData` 等）は `rename_all = "camelCase"` を付けていない。
Claude Code ステータスラインの JSON がそのまま snake_case で提供されるため。
`WidgetSettings` のみ `rename_all = "camelCase"` を使用。

```rust
// models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageData {
    pub context_window: ContextWindow,
    pub cost: CostInfo,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
    #[serde(default)]
    pub model: Option<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: u64,
    #[serde(default)]
    pub total_api_duration_ms: u64,
}
```

| serde属性 | 用途 |
|-----------|------|
| `rename_all = "camelCase"` | JSON(camelCase) ↔ Rust(snake_case) 変換 |
| `#[serde(default)]` | フィールド欠損時に `Default::default()` |
| `#[serde(skip_serializing_if = "Option::is_none")]` | None のフィールドを省略 |
| `#[serde(alias = "...")]` | 別名のキーも受け入れ |

**パフォーマンス**: 小さいファイル（<1MB）は `serde_json::from_str` を使用（`from_reader` より割り当てが少ない）。

### 3.4 ファイル監視（notify）

`notify-debouncer-mini` を使用（生の `notify` より簡潔）:

```toml
notify-debouncer-mini = "0.5"
```

ポイント:
- **デバウンス間隔**: 300-500ms（設定ファイルの書き込みに適切）
- **親ディレクトリを監視**: ファイルがまだ存在しない場合に備えて `RecursiveMode::NonRecursive` で親を監視
- **ウォッチャーの保持**: `setup()` で生成した `Debouncer` はマネージド状態に保存。ドロップすると監視が停止する
- **イベントフィルタ**: 監視対象ファイルのパスに一致するイベントのみ処理

### 3.5 状態管理

```rust
// state.rs
#[derive(Default)]
pub struct AppStateInner {
    pub token_data: Option<TokenUsageData>,
    pub watcher_active: bool,
}

// Arc は不要 — Tauri の State が内部で Arc を管理
pub type AppState = std::sync::Mutex<AppStateInner>;
```

| 状況 | 使うもの |
|------|---------|
| コマンド内での状態アクセス | `tauri::State<'_, AppState>` |
| コマンド外（ウォッチャースレッド等） | `app_handle.state::<AppState>()` |
| Mutex | `std::sync::Mutex`（`.await` をまたがない限り） |

### 3.6 Clippy & rustfmt

**Cargo.toml の lint 設定**:
```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
must_use_candidate = "allow"
unwrap_used = "warn"

[lints.rust]
unsafe_code = "forbid"
```

**rustfmt.toml**:
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

コミット前に実行:
```bash
cargo fmt --check
cargo clippy -- -D warnings
```

---

## 4. Svelte 5 フロントエンド

### 4.1 Runes クイックリファレンス

| 状況 | 使うもの |
|------|---------|
| 時間とともに変化する値 | `$state` |
| Tauriから来る大きなデータ（丸ごと置換） | `$state.raw` |
| 他の状態から計算する値 | `$derived` |
| 複雑な計算 | `$derived.by(() => { ... })` |
| 副作用（DOM操作、外部購読） | `$effect` + クリーンアップ return |
| コンポーネント間の状態共有 | `.svelte.ts` ファイル内のクラス |

### 4.2 $state

```svelte
<script lang="ts">
  let count = $state(0);
  let items = $state<string[]>([]);

  // Tauriから来るデータはraw（プロキシ不要）
  let tokenData = $state.raw<TokenUsageData | null>(null);
</script>
```

**アンチパターン**:
- `$state` 変数を直接 export しない（インポート先でフリーズする）
- クラスインスタンスを外側から `$state()` で包まない（フィールド内部に `$state` を置く）

### 4.3 $derived

```ts
let percentage = $derived(tokenData?.context_window.used_percentage ?? 0);
let statusColor = $derived(
  percentage > 90 ? 'red' :
  percentage > 75 ? 'orange' :
  percentage > 50 ? 'yellow' : 'green'
);
```

**重要**: オブジェクト全体ではなく、必要なプロパティだけ抽出して `$derived` に渡す。オブジェクト参照の変更で不要な再計算が走るのを防ぐ。

```ts
// BAD: tokenData の参照が変わるたびに再計算
let cost = $derived(formatCost(tokenData?.cost));

// GOOD: 特定プロパティだけを依存に
let costUsd = $derived(tokenData?.cost.total_cost_usd ?? 0);
let costFormatted = $derived(costUsd.toFixed(4));
```

### 4.4 $effect

**副作用のみに使う**。状態の同期には `$derived` を使うこと。

```svelte
<script lang="ts">
  import { listen } from '@tauri-apps/api/event';

  let data = $state.raw<TokenUsageData | null>(null);

  // Tauri イベント購読 + クリーンアップ
  $effect(() => {
    const promise = listen<TokenUsageData>('token-updated', (event) => {
      data = event.payload;
    });
    return () => {
      promise.then(unlisten => unlisten());
    };
  });
</script>
```

**アンチパターン**:
- `$effect` 内で読んだ状態と同じ状態を変更しない（無限ループ）
- `await` の後の依存は追跡されない
- `$inspect()` を本番に残さない

### 4.5 コンポーネントパターン

**Props**: `$props()` を使用（`export let` はSvelte 5で非推奨）

```svelte
<script lang="ts">
  interface Props {
    label: string;
    value: number;
    unit?: string;
  }
  let { label, value, unit = 'tokens' }: Props = $props();
</script>
```

**Snippets**: slots の代わりに使用

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';
  let { children }: { children: Snippet } = $props();
</script>

<div class="card">{@render children()}</div>
```

**イベント**: 標準DOM属性を使用（`on:click` ではなく `onclick`）

```svelte
<button onclick={() => count++}>Click</button>
```

### 4.6 状態共有（.svelte.ts）

Runes を使うファイルは **`.svelte.ts`** 拡張子が必須（`.ts` ではコンパイルエラー）。

```ts
// lib/state/token-state.svelte.ts
import type { TokenUsageData } from '$lib/types';

class TokenState {
  data = $state.raw<TokenUsageData | null>(null);
  connected = $state(false);

  get usedPercentage() {
    return this.data?.context_window.used_percentage ?? 0;
  }

  get costUsd() {
    return this.data?.cost.total_cost_usd ?? 0;
  }
}

export const tokenState = new TokenState();
```

### 4.7 スタイリング

**CSS カスタムプロパティでテーマ管理**:

```css
/* lib/styles/globals.css */
:root {
  --radius: 6px;
  --transition-speed: 150ms;
}

.dark {
  color-scheme: dark;
  --bg-primary: #1a1a1a;
  --bg-secondary: #2a2a2a;
  --text-primary: #f0f0f0;
  --text-secondary: #999999;
  --border: #333333;
  --accent: #60a5fa;
  --status-green: #22c55e;
  --status-yellow: #eab308;
  --status-orange: #f97316;
  --status-red: #ef4444;
}
```

ウィジェットは固定サイズなので:
- メディアクエリは不要（ビューポートが既知）
- ピクセル指定で問題なし
- `overflow: hidden` をルートに設定

---

## 5. IPC（Rust ↔ Svelte 通信）

### 通信方式の使い分け

| 方式 | 方向 | 用途 |
|------|------|------|
| **Commands** (`invoke`) | Frontend → Rust → Frontend | リクエスト/レスポンス。初期データ取得、設定変更 |
| **Events** (`emit`/`listen`) | Rust → Frontend | プッシュ通知。ファイル変更通知、状態更新 |
| **Channels** | Rust → Frontend（コマンド内） | 高スループットストリーミング |

### 本プロジェクトでの使い方

- **初期データ取得**: コマンド（`invoke('get_current_token')`）
- **リアルタイム更新**: イベント（ウォッチャーから `app.emit("token-updated", &data)`）
- **設定変更**: コマンド（`invoke('update_settings', { ... })`）

### 型安全なラッパー

```ts
// lib/tauri/commands.ts
import { invoke } from '@tauri-apps/api/core';
import type { TokenUsageData } from '$lib/types';

export async function getCurrentToken(): Promise<TokenUsageData | null> {
  return invoke<TokenUsageData | null>('get_current_token');
}

export async function updateSettings(settings: WidgetSettings): Promise<void> {
  return invoke('update_settings', { settings });
}
```

**注意**: `invoke` のパラメータは TypeScript 側で **camelCase**、Rust 側で **snake_case**。Tauri が自動変換する。

---

## 6. パフォーマンス最適化

### 6.1 Cargo.toml リリースプロファイル

```toml
[profile.dev]
incremental = true

[profile.release]
codegen-units = 1    # LLVM最適化向上（コンパイル遅くなる）
lto = true           # リンク時最適化
opt-level = "s"      # サイズ最適化（"3"は速度最適化）
panic = "abort"      # アンワインドなし
strip = true         # デバッグシンボル除去
```

### 6.2 Tauri 設定

```json
{
  "build": {
    "removeUnusedCommands": true
  }
}
```
Tauri 2.4+ で利用可能。Capability に参照されていないコマンドをバイナリから除去。

### 6.3 メモリ削減

- Rust: `Vec::with_capacity(n)` で事前確保、参照と借用を優先、不要な clone を避ける
- Svelte: `$state.raw` で Tauri からのデータにプロキシオーバーヘッドを避ける
- 1回のファイル変更イベントにつき、ファイル読み取り1回、パース1回、状態更新 + emit を1パスで

### 6.4 起動時間

- `visible: false` で即座にウィンドウ表示、初期化後に `show()`
- 非必須の初期化（ファイルウォッチャー、ネットワーク）は非同期で
- フロントエンドの依存を最小限に

---

## 7. セキュリティ

### やるべきこと

- CSP を必ず設定（`default-src 'self'` をベースライン）
- `freezePrototype: true` でプロトタイプ汚染を防止
- Capability で最小権限のみ付与
- IPC 境界で全入力をバリデーション
- `cargo audit` と `npm audit` を定期実行

### やってはいけないこと

- CSP を `null` のまま本番リリース
- `'unsafe-eval'` を CSP に含める
- CDN から外部スクリプトを読み込む
- ファイルシステムへの広範なアクセスを許可
- フロントエンドを信頼する（WebView はサンドボックス内）
- `dangerousDisableAssetCspModification: true` を設定

### 信頼モデル

```
Rust バックエンド（信頼境界） ← システム全アクセス
       ↕ IPC（攻撃面）
WebView フロントエンド（制限あり） ← IPC 経由のみ通信
```

---

## 8. テスト

### 8.1 Rust テスト

**ビジネスロジックをコマンドデコレータから分離**して単体テスト:

```rust
// commands.rs
pub fn parse_token_file(contents: &str) -> AppResult<TokenUsageData> {
    serde_json::from_str(contents).map_err(AppError::from)
}

#[tauri::command]
fn get_token_from_file(path: String) -> AppResult<TokenUsageData> {
    let contents = std::fs::read_to_string(&path)?;
    parse_token_file(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json() {
        let json = r#"{"contextWindow":{"totalInputTokens":100,"totalOutputTokens":50,"usedPercentage":5},"cost":{"totalCostUsd":0.01,"totalDurationMs":1000,"totalApiDurationMs":500}}"#;
        let result = parse_token_file(json);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse_token_file("not json");
        assert!(result.is_err());
    }
}
```

Tauri モックランタイムでの統合テスト:
```toml
[dev-dependencies]
tauri = { version = "2", features = ["test"] }
```

### 8.2 Svelte テスト（Vitest）

```bash
npm install -D vitest jsdom @testing-library/svelte @testing-library/user-event
```

Runes を使うテストファイルは **`.svelte.test.ts`** 拡張子:

```ts
// token-state.svelte.test.ts
import { flushSync } from 'svelte';
import { test, expect } from 'vitest';

test('derived percentage updates', () => {
  const cleanup = $effect.root(() => {
    let percentage = $state(0);
    let status = $derived(percentage > 90 ? 'red' : 'green');

    expect(status).toBe('green');
    percentage = 95;
    flushSync();
    expect(status).toBe('red');
  });
  cleanup();
});
```

**方針**: ビジネスロジックを `.svelte.ts` に抽出してユニットテスト。コンポーネントテストはインタラクションとレンダリングに限定。

---

## 9. ビルド・配布

### 9.1 ビルドコマンド

```bash
# 開発
npm run tauri dev

# リリースビルド
npm run tauri build
```

### 9.2 インストーラー

Tauri が自動生成:
- **MSI**: Windows Installer
- **NSIS**: より柔軟なインストーラー
- ポータブル exe も出力可能

### 9.3 Windows コード署名

2023年6月以降、新しいOV証明書はHSMが必須。選択肢:

| 方式 | 用途 |
|------|------|
| Legacy OV証明書（.pfx） | `bundle.windows.certificateThumbprint` で設定 |
| Azure Key Vault | `bundle.windows.signCommand` で `relic` を使用 |
| Azure Trusted Signing | `bundle.windows.signCommand` で `trusted-signing-cli` を使用 |

**MVP段階では署名なしで自分用にビルド**。OSS公開時に検討する。

### 9.4 自動アップデート

`tauri-plugin-updater` で実現可能（将来対応）:
- 署名検証が必須（秘密鍵の紛失厳禁）
- GitHub Releases をエンドポイントにできる

---

## 参考リンク

### Tauri 2.0
- [プロジェクト構造](https://v2.tauri.app/start/project-structure/)
- [設定リファレンス](https://v2.tauri.app/reference/config/)
- [セキュリティ](https://v2.tauri.app/security/)
- [CSP設定](https://v2.tauri.app/security/csp/)
- [Capabilities & Permissions](https://v2.tauri.app/security/capabilities/)
- [Rust呼び出し](https://v2.tauri.app/develop/calling-rust/)
- [フロントエンド呼び出し](https://v2.tauri.app/develop/calling-frontend/)
- [状態管理](https://v2.tauri.app/develop/state-management/)
- [ウィンドウカスタマイズ](https://v2.tauri.app/learn/window-customization/)
- [システムトレイ](https://v2.tauri.app/learn/system-tray/)
- [Window State Plugin](https://v2.tauri.app/plugin/window-state/)
- [サイズ最適化](https://v2.tauri.app/concept/size/)
- [SvelteKit連携](https://v2.tauri.app/start/frontend/sveltekit/)
- [Windows署名](https://v2.tauri.app/distribute/sign/windows/)

### Svelte 5
- [$state ドキュメント](https://svelte.dev/docs/svelte/$state)
- [$derived ドキュメント](https://svelte.dev/docs/svelte/$derived)
- [$effect ドキュメント](https://svelte.dev/docs/svelte/$effect)
- [Snippets](https://svelte.dev/docs/svelte/snippet)
- [テスト](https://svelte.dev/docs/svelte/testing)
- [v5 移行ガイド](https://svelte.dev/docs/svelte/v5-migration-guide)

### Rust
- [notify crate](https://docs.rs/notify/latest/notify/)
- [serde 属性リファレンス](https://serde.rs/attributes.html)
- [Clippy lint一覧](https://doc.rust-lang.org/stable/clippy/lints.html)
- [Rust パフォーマンスブック](https://nnethercote.github.io/perf-book/build-configuration.html)
