# Claude Code Token Widget

Real-time desktop widget for monitoring Claude Code token usage, rate limits, and costs.

Built with **Tauri 2.0** (Rust) + **Svelte 5** for minimal resource footprint.

![Windows](https://img.shields.io/badge/platform-Windows-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Rate Limit Tracking** - 5-hour and 7-day usage with reset countdown
- **Context Window** - Progress bar with color-coded usage levels
- **Token Stats** - Input, output, cache read, and cache creation tokens
- **Cost Display** - USD/JPY with configurable exchange rate
- **OS Alerts** - Native notifications at 50%, 75%, 90% thresholds
- **Heatmap** - 26-week GitHub-style usage history
- **Wake-Up Scheduler** - Auto-launch Claude CLI at a set time
- **System Tray** - Tooltip with quick stats, show/hide toggle
- **Themes** - Dark (default) and Light
- **Customizable** - Font size (10-18px), opacity (30-100%), display items toggle

## Install

### Download (Recommended)

Download the latest release from [Releases](https://github.com/creolab-dev/claude-code-token-widget/releases):

| File | Description |
|------|-------------|
| `Claude Code Token Widget_0.1.0_x64-setup.exe` | NSIS installer (recommended) |
| `Claude Code Token Widget_0.1.0_x64_en-US.msi` | MSI installer |
| `claude-code-token-widget.exe` | Portable executable |

> Windows 11 recommended. Windows 10 requires [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Build from Source

Prerequisites: Rust 1.94+, Node.js 18+

```bash
git clone https://github.com/creolab-dev/claude-code-token-widget.git
cd claude-code-token-widget
npm install
npm run tauri build
```

Binaries output to `src-tauri/target/release/bundle/`.

## Setup

The widget reads token data from `~/.claude/token-usage.json`, written by Claude Code's status line feature.

### 1. Create the statusline script

Create `~/.claude/statusline.sh`:

```bash
#!/bin/bash
OUTFILE="$HOME/.claude/token-usage.json"
TMPFILE="${OUTFILE}.tmp"

INPUT=$(cat)

if [ -n "$INPUT" ]; then
    echo "$INPUT" > "$TMPFILE" && mv "$TMPFILE" "$OUTFILE"
fi
```

### 2. Enable status line in Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "bash ~/.claude/statusline.sh"
  }
}
```

### 3. Launch the widget

Run the installed app or portable `.exe`. The widget appears as a frameless, always-on-top window.

## Usage

The widget has 3 views, accessible via the icons at the bottom:

| View | Content |
|------|---------|
| **Main** | Rate limits, context window, tokens, cost, session info |
| **Settings** | Theme, font size, opacity, currency, alerts, display toggles |
| **Heatmap** | 26-week daily usage history (GitHub-style grid) |

### Settings

| Setting | Default | Range |
|---------|---------|-------|
| Theme | Dark | Dark / Light |
| Font Size | 12px | 10-18px |
| Opacity | 100% | 30-100% |
| Currency | USD | USD / JPY |
| JPY Rate | 150 | Manual input |
| Always on Top | On | On / Off |
| Alert Thresholds | 50%, 75%, 90% | Toggle each |

### System Tray

Right-click the tray icon for:
- Show / Hide widget
- Always on Top toggle
- Quit

Tooltip displays: `5H: X% | 7D: X% | Ctx: X%`

## Performance

| Metric | Value |
|--------|-------|
| Idle Memory | < 40 MB |
| Idle CPU | < 1% |
| Update Latency | < 1 second |
| Installer Size | < 10 MB |

## Tech Stack

- **Backend**: Rust + Tauri 2.0
- **Frontend**: SvelteKit + Svelte 5 (Runes)
- **File Watching**: notify-debouncer-mini (100ms) with 5s polling fallback
- **Storage**: tauri-plugin-store (settings + heatmap)
- **Notifications**: tauri-plugin-notification

## Development

```bash
# Dev mode (hot reload)
npm run tauri dev

# Type check
npm run check

# Run all tests (55 Rust + 55 vitest)
npm run test:all

# Rust tests only
cd src-tauri && cargo test

# Frontend tests only
npm test
```

## License

MIT
