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
