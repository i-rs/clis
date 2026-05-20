# i-rs-mood

Mood tracking CLI tool for recording and visualizing your daily mood.

## Features

- Record daily mood (7 levels: Amazing, Great, Good, Okay, Poor, Bad, Terrible)
- Number and word support
- View mood history with statistics
- Mood calendar visualization
- Tag and note support

## Install

```bash
npm install -g @i-rs/i-rs-mood
# or
brew install i-rs/homebrew-tap/i-rs-mood
```

## Mood Levels

| Level | Word | Emoji |
|-------|------|-------|
| 7 | Amazing | 🤩 |
| 6 | Great | 😊 |
| 5 | Good | 🙂 |
| 4 | Okay | 😐 |
| 3 | Poor | 😕 |
| 2 | Bad | 😔 |
| 1 | Terrible | 😢 |

## Quick Start

```bash
# Record today's mood
i-rs-mood add good

# Record mood for a specific date
i-rs-mood add great --date 2025-01-15

# List all mood records
i-rs-mood list

# View last 7 days with calendar
i-rs-mood list --days 7 --calendar
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/mood.json`
- Linux: `~/.config/i-rs/mood.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0
