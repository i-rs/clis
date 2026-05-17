# i-rs-mood

Mood tracking CLI tool for recording and visualizing your daily mood.

## Features

- Record daily mood (5 levels: Great, Good, Okay, Bad, Terrible)
- Support multiple input formats (number, word, emoji)
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
| 5 | Great | 😊 |
| 4 | Good | 🙂 |
| 3 | Okay | 😐 |
| 2 | Bad | 😔 |
| 1 | Terrible | 😢 |

## Quick Start

```bash
# Record today's mood
i-rs-mood add 2025-01-15 good

# List all mood records
i-rs-mood list

# View last 7 days with calendar
i-rs-mood list --days 7 --calendar
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/moods.json`
- Linux: `~/.config/i-rs/moods.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0
