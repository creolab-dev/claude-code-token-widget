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

## Prerequisites

- **Claude Code** installed and working ([install guide](https://docs.anthropic.com/en/docs/claude-code/overview))
- **Windows 10 or 11** (x64)
  - Windows 10 users: install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if not already present (pre-installed on Windows 11)

## Getting Started

### Step 1: Download and install the widget

1. Go to [Releases](https://github.com/creolab-dev/claude-code-token-widget/releases)
2. Download one of the following:

   | File | Description |
   |------|-------------|
   | `Claude Code Token Widget_0.1.0_x64-setup.exe` | Installer (recommended) |
   | `Claude Code Token Widget_0.1.0_x64_en-US.msi` | MSI installer |
   | `claude-code-token-widget.exe` | Portable (no install needed) |

3. Run the installer and follow the prompts, or place the portable `.exe` anywhere you like

### Step 2: Create the statusline script

The widget reads token data from a JSON file that Claude Code writes via its status line feature. You need to create a small script that receives this data.

1. Open a terminal (Git Bash, WSL, or similar) and run:

   ```bash
   cat > ~/.claude/statusline.sh << 'EOF'
   #!/bin/bash
   OUTFILE="$HOME/.claude/token-usage.json"
   TMPFILE="${OUTFILE}.tmp"

   INPUT=$(cat)

   if [ -n "$INPUT" ]; then
       echo "$INPUT" > "$TMPFILE" && mv "$TMPFILE" "$OUTFILE"
   fi
   EOF
   ```

2. Make the script executable:

   ```bash
   chmod +x ~/.claude/statusline.sh
   ```

> **Windows path note:** `~/.claude/` corresponds to `C:\Users\<your-username>\.claude\` on Windows.

### Step 3: Enable the status line in Claude Code

Add the `statusLine` setting to your Claude Code config.

1. Open `~/.claude/settings.json` in a text editor
2. Add the following (merge with your existing settings if the file already exists):

   ```json
   {
     "statusLine": {
       "type": "command",
       "command": "bash ~/.claude/statusline.sh"
     }
   }
   ```

   If you already have other settings in the file, just add the `"statusLine"` key inside the existing `{}`:

   ```json
   {
     "existing_setting": "...",
     "statusLine": {
       "type": "command",
       "command": "bash ~/.claude/statusline.sh"
     }
   }
   ```

### Step 4: Launch and verify

1. Start the widget (from Start Menu or the portable `.exe`)
2. Open Claude Code in a terminal and start a conversation
3. The widget should display token usage data within a few seconds

If the widget shows no data, check that:
- Claude Code is running and you have an active conversation
- `~/.claude/token-usage.json` exists (it's created after the first Claude Code message)
- The `statusLine` setting in `settings.json` is correct

### Build from Source (optional)

Prerequisites: [Rust 1.94+](https://rustup.rs/), [Node.js 18+](https://nodejs.org/)

```bash
git clone https://github.com/creolab-dev/claude-code-token-widget.git
cd claude-code-token-widget
npm install
npm run tauri build
```

Binaries output to `src-tauri/target/release/bundle/`.

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
