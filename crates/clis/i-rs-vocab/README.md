# i-rs-vocab

Vocabulary learning CLI tool for managing and reviewing vocabulary words.

## Features

- Add vocabulary words with definitions and examples
- Track learning status (new/learning/mastered)
- Quiz mode for active recall practice
- Statistics showing learning progress
- Tag support for organization
- Review count tracking for mastery progression

## Install

```bash
npm install -g @i-rs/i-rs-vocab
# or
brew install i-rs/homebrew-tap/i-rs-vocab
```

## Quick Start

```bash
# Add a new word
i-rs-vocab add hello "greeting; hello world" --tag basic --example "Hello, how are you?"

# List all words
i-rs-vocab list

# Get word details
i-rs-vocab get hello

# Start a quiz
i-rs-vocab quiz

# View statistics
i-rs-vocab stats
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new vocabulary word |
| `list` | List vocabulary words |
| `get` | Show word details |
| `update` | Update a word |
| `delete` | Delete a word |
| `quiz` | Practice vocabulary |
| `stats` | Show learning statistics |
| `example` | Show usage examples |
| `skill` | Show AI skill document |

## Data Storage

- macOS: `~/.config/i-rs/vocab.json`
- Linux: `~/.config/i-rs/vocab.json`
- Windows: `~\AppData\Roaming\i-rs\vocab.json`

## License

MIT OR Apache-2.0
