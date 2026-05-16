# i-rs-claw

TUI intelligent personal data assistant powered by i-rs CLI tools.

i-rs-claw is a terminal-based AI assistant that understands natural language and manages your personal data through the i-rs CLI toolset — health, finance, tasks, media, household, and more.

## Features

- **Natural language interface**: "Record 75kg weight" or "How's my running this week?" — just type it
- **70+ i-rs tools**: Full access to all i-rs CLI tools for personal data management
- **Streaming responses**: Real-time token-by-token AI response display
- **Tool call transparency**: See exactly which tools are being called and their results
- **Multi-turn conversations**: Context preserved across the session
- **First-learn-then-execute**: AI automatically learns tool syntax via `skill teach` before operating data
- **Voice input**: macOS-native speech recognition via [VoiceInput](https://github.com/shibing624/VoiceInput) (see setup below)

## Installation

### Prerequisites

- Rust toolchain (see `rust-toolchain.toml` at project root)
- All `i-rs` CLI tools installed and in PATH (see project root `scripts/link_to_path.sh`)

### Build

```bash
cargo build -p i-rs-claw --release
```

The binary will be at `target/release/i-rs-claw`.

## Configuration

Create `~/.i-rs-claw/config.toml`:

```toml
[config]
api_key = "sk-..."                          # Required: LLM API key
base_url = "https://api.openai.com/v1"      # Optional: defaults to OpenAI
model = "gpt-4o-mini"                       # Optional: defaults to gpt-4o-mini
```

Supports any OpenAI-compatible API (OpenAI, OpenRouter, DeepSeek, etc.):

```toml
[config]
api_key = "sk-..."
base_url = "https://openrouter.ai/api/v1"
model = "deepseek/deepseek-chat"
```

## Usage

```bash
i-rs-claw
```

Type your request at the prompt and press Enter. Examples:

```
> 记录体重75kg
> 这个月跑步情况如何？
> 帮我看看最近的支出
> 该换牙刷了吗？
> 今天心情怎么样？
```

**Controls:**
| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Ctrl+Q` / `Ctrl+C` | Quit |
| `Backspace` | Delete character |
| `[Fn]` | Voice input (see below) |

## Voice Input (macOS)

i-rs-claw supports hands-free voice input via [VoiceInput](https://github.com/shibing624/VoiceInput), a lightweight menu bar tool for macOS.

### Setup

1. Download the DMG from [releases](https://github.com/shibing624/VoiceInput/releases)
2. Drag to Applications folder and launch
3. Grant required permissions on first run:
   - **Microphone** — for audio capture
   - **Speech Recognition** — for on-device transcription (Apple Speech)
   - **Accessibility** — for text injection into terminal
4. (Optional) Switch language in menu bar (Simplified Chinese / English, etc.)

### Usage

While the i-rs-claw input field is focused, hold **Fn** (or right **Command**, configurable) and speak. Release the key — the transcribed text is automatically injected at the cursor position. Works in any terminal (Terminal.app, iTerm2, Ghostty, etc.) and any app.

### How It Works

```
Hold Fn  →  Microphone on  →  Apple Speech (local)  →  Release Fn  →  Text at cursor
```

- Real-time waveform preview in menu bar
- Full offline support (on-device recognition)
- Optional LLM-based text correction
- Supports Chinese, English, and 50+ languages

### Tips

- For best Chinese recognition, select Simplified Chinese in the VoiceInput menu bar
- The `[Fn] 语音输入` hint at the bottom of the input box indicates voice input is available

## Architecture

```
┌─────────────────────────────────────────┐
│              main.rs                     │
│  ┌────────────┐  ┌──────────────────┐   │
│  │   ui.rs    │  │    llm.rs        │   │
│  │  (TUI)     │  │  (SSE streaming) │   │
│  └─────┬──────┘  └────────┬─────────┘   │
│        │                  │             │
│  ┌─────┴──────────────────┴──────────┐  │
│  │          app.rs (State)           │  │
│  └─────┬──────────────────┬─────────┘  │
│        │                  │            │
│  ┌─────┴──────┐   ┌──────┴────────┐   │
│  │ config.rs  │   │  tools/       │   │
│  │            │   │  ├ i_rs_cmd.rs│   │
│  │            │   │  └ search.rs  │   │
│  └────────────┘   └───────────────┘   │
└─────────────────────────────────────────┘
```

## Tech Stack

- **UI**: [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm)
- **LLM**: OpenAI-compatible chat completion API (streaming SSE)
- **Backend**: i-rs CLI tools via subprocess (`Command::new("i-rs")`)
