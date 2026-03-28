# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A desktop widget that monitors Claude Code's token usage in real time. Built with **Tauri v2** (Rust backend) + **SvelteKit** (Svelte 5 frontend). The widget watches `~/.claude/token-usage.json` for changes and displays context window usage, rate limits, cost, and token stats in an always-on-top frameless window.

Features: OS alert notifications on rate limit thresholds, GitHub-style usage heatmap, wake-up scheduler, tray tooltip with usage percentages, configurable font size.

## Commands

```bash
# Development (starts both Vite dev server and Tauri window)
npm run tauri dev

# Build production binary
npm run tauri build

# Frontend only (no Tauri window)
npm run dev

# Type checking
npm run check

# Frontend tests (vitest)
npm test

# Rust tests
cd src-tauri && cargo test

# All tests (frontend + Rust)
npm run test:all
```

## Architecture

### Data Flow

1. Claude Code writes token usage data to `~/.claude/token-usage.json` via its status line feature
2. Rust backend (`watcher.rs`) watches the `~/.claude/` directory with `notify-debouncer-mini` (100ms debounce)
3. On file change, Rust parses the JSON into `TokenUsageData` and:
   - Emits `token-updated` event to frontend
   - Checks alert thresholds and sends OS notifications (`alerts.rs`)
   - Records daily usage for heatmap (`heatmap.rs`)
   - Updates tray tooltip (`tray.rs`)
4. Frontend listens for events in `+page.svelte` and updates `tokenState` (Svelte 5 runes-based reactive state)
5. Frontend polls `get_current_token` every 5 seconds as fallback only when watcher is disconnected; polling stops on first `token-updated` event
6. Components re-render from the shared state singleton

### Backend (src-tauri/src/)

| File | Role |
|------|------|
| `lib.rs` | App entry: plugin registration, initial data load, watcher/scheduler startup, window setup |
| `models.rs` | All data types: `TokenUsageData`, `WidgetSettings`, `AlertSettings`, `ScheduleSettings`, `HeatmapEntry`, etc. |
| `watcher.rs` | File watcher: resolves token file path, reads/parses JSON, emits events, triggers alerts/heatmap/tray |
| `commands.rs` | Tauri commands: `get_current_token`, `get_settings`, `update_settings`, `resize_window` (clamped 100-800w, 50-2000h), `set_always_on_top`, `get_heatmap_data` (max 3650 days) |
| `state.rs` | `AppState` = `Mutex<AppStateInner>` holding cached token data, notification tracking, heatmap delta state, scheduler handle |
| `tray.rs` | System tray: show/hide, always-on-top toggle, quit. Tooltip shows `5H: X% | 7D: X% | Ctx: X%` |
| `alerts.rs` | OS notification on rate limit threshold crossing. Collects notifications under lock, sends after lock release to avoid blocking |
| `heatmap.rs` | Daily usage accumulation via `tauri-plugin-store` (`heatmap-data.json`). Delta tracking per session with persistence across restarts (`_last_session_id`, `_last_session_cost` keys) |
| `scheduler.rs` | Wake-up scheduler: spawns Claude CLI at configured time. Command field validated against path traversal/injection |
| `error.rs` | `AppError` enum (`JsonParse`, `Watch`, `Io`, `Lock`, `Store`, `Window`) with `thiserror`, implements `Serialize` for Tauri IPC |

### Frontend (src/)

- **State management**: Svelte 5 runes (`$state`, `$state.raw`, `$derived`) in class singletons under `src/lib/state/`
  - `token-state.svelte.ts` — token data, connection status, computed colors
  - `settings-state.svelte.ts` — persisted settings (currency, opacity, theme, font size, display items, alerts, schedule). Reads actual `isAlwaysOnTop()` window state on load
  - `heatmap-state.svelte.ts` — heatmap data loading and intensity calculation
- **Tauri bridge**: `src/lib/tauri/commands.ts` (invoke wrappers) and `events.ts` (event listeners)
- **Types**: `src/lib/types/index.ts` — mirrors Rust `models.rs` structs
- **Components**: `src/lib/components/` — Widget (3-view: widget/settings/heatmap), DragHandle, RateLimitBar, ContextBar, TokenStats, CostDisplay, SettingsPanel, HeatmapView, etc.

### Key Design Decisions

- **SPA mode**: SSR disabled (`+layout.ts` exports `ssr = false`), static adapter with `index.html` fallback — required for Tauri
- **Frameless window**: `decorations: false`, `transparent: true` in `tauri.conf.json`. Custom drag handle on all 3 views (widget/settings/heatmap headers)
- **Window resizing**: Since `resizable: false`, the `resize_window` command temporarily enables resizing, sets size (clamped), then disables. Only triggers on initial data load and view/setting changes — not on every data update
- **Settings persistence**: Uses `tauri-plugin-store` (`settings.json` in app data dir), not the token file
- **Heatmap persistence**: Separate store file `heatmap-data.json` keyed by ISO date strings. Session tracking (`_last_session_id`, `_last_session_cost`) also persisted to prevent double-counting on widget restart
- **Alert dedup**: Per-threshold `HashSet` in `AppState`, cleared when `resets_at` changes (new rate limit window). Notifications collected under lock, sent after lock release
- **Theming**: CSS custom properties in `globals.css` toggled via `.dark`/`.light` class on root element. Background colors are fully opaque — transparency is controlled solely by CSS `opacity` property
- **Font sizing**: CSS custom properties (`--font-label`, `--font-value`, `--font-small`, `--font-tiny`) computed from `fontSize` setting (10-18px) and injected via inline style in `+layout.svelte`
- **Polling fallback**: Frontend polls `get_current_token` every 5 seconds only when disconnected; stops when watcher events arrive
- **Reset countdown**: `RateLimitBar` uses a `tick` counter (60s interval) to keep the countdown live between data updates

## Important Type Notes

- `used_percentage` fields in `ContextWindow` and `RateLimitEntry` are **`f64`** (not integer). Claude Code writes values like `57.99999999999999` which would fail `u8` parsing
- `DisplayItems`, `AlertSettings`, `ScheduleSettings`, `WidgetSettings` all use `#[serde(rename_all = "camelCase")]` in Rust to match TypeScript field names
- Percentages displayed with 1 decimal place (`.toFixed(1)`)

## Rust Lints

Configured in `Cargo.toml`:
- `clippy::pedantic` = warn (with some allows: `module_name_repetitions`, `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`)
- `clippy::unwrap_used` = warn
- `unsafe_code` = **forbid**

## Tauri Events (Rust → Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `token-updated` | `TokenUsageData` | Fired on file change or initial load |
| `watcher-status` | `WatcherStatus` | Watcher started/stopped |
| `watcher-error` | `String` | Watch error message |

## Tauri Plugins

| Plugin | Purpose |
|--------|---------|
| `tauri-plugin-store` | Settings and heatmap data persistence |
| `tauri-plugin-window-state` | Window position/size restore |
| `tauri-plugin-notification` | OS native alert notifications |
| `tauri-plugin-opener` | Default link/file opening |

## Frontend Conventions

- Svelte 5 syntax: `$state()`, `$derived()`, `$effect()`, `$props()` — no legacy `$:` or stores
- TypeScript interfaces use `snake_case` field names to match Rust serde serialization (except structs with `#[serde(rename_all = "camelCase")]`: `DisplayItems`, `AlertSettings`, `ScheduleSettings`, `WidgetSettings`)
- Color utility (`colors.ts`) returns hex strings directly based on usage percentage thresholds
- Formatting utilities (`format.ts`) handle tokens, cost (USD/JPY), and duration display
- Widget has 3 view modes: `widget` (main), `settings`, `heatmap` — toggled via `viewMode` state
- All 3 views have draggable headers for window movement
