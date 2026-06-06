# i-rs-claw

AI personal assistant — Web Dashboard + TUI terminal interface, powered by 70+ i-rs CLI tools.

i-rs-claw is an AI assistant that understands natural language and manages your personal data through the i-rs CLI toolset — health, finance, tasks, media, household, and more.

## Quick Start

```bash
# Default: start HTTP API server + Web Dashboard
cargo run -p i-rs-claw --features dashboard

# TUI terminal mode (debug/power-user)
cargo run -p i-rs-claw -- tui

# API only, no Web UI
cargo run -p i-rs-claw --features dashboard -- serve --api-only
```

Open `http://localhost:3000` in a browser. Enter the token displayed at startup.

## Modes

| Command | Description |
|---------|-------------|
| `claw serve` | **Default** — HTTP API + Web Dashboard (port 3000) |
| `claw serve --api-only` | API only, no SPA frontend |
| `claw tui` | Terminal UI (ratatui) for debugging / power users |
| `claw ask "..."` | Non-interactive single-turn chat |
| `claw config` | Interactive setup wizard |
| `claw session --list` | List all sessions |
| `claw stats` | Token usage statistics |
| `claw gateway` | Social platform adapters (Telegram/WeChat) |
| `claw plugin --list` | Manage MCP plugins |
| `claw skill --list` | Manage custom skills |

## Features

- **Natural language interface**: "Record 75kg weight" or "How's my running this week?" — just type it
- **70+ i-rs tools**: Full access to all i-rs CLI tools for personal data management
- **Web Dashboard**: Chat + Agents + Token Stats + Session management (React SPA)
- **HTTP API**: REST + SSE for programmatic access (used by Web/MiniProgram/iOS clients)
- **Streaming responses**: Real-time token-by-token AI response display
- **Tool call transparency**: See exactly which tools are being called and their results
- **Multi-turn conversations**: Context preserved across sessions
- **Multi-user support**: Token-based auth with per-user session/agent/data isolation
- **Multi-agent**: Configure multiple AI agents with different models/prompts/tools
- **MCP plugins**: Extend with external tools via Model Context Protocol

## Installation

### Prerequisites

- Rust toolchain (see `rust-toolchain.toml` at project root)
- All `i-rs` CLI tools installed and in PATH (see project root `scripts/link_to_path.sh`)

### Build

```bash
# Full build with all features
cargo build -p i-rs-claw --features dashboard --release

# Minimal build (TUI only, no web server)
cargo build -p i-rs-claw --release

# With specific storage backend
cargo build -p i-rs-claw --features "dashboard,sqlite" --release
cargo build -p i-rs-claw --features "dashboard,postgres" --release
```

### Feature Flags

| Feature | Description | Deps |
|---------|-------------|------|
| `dashboard` | HTTP API + Web Dashboard SPA | axum, tower-http, rust-embed |
| *(none)* | TUI-only mode (ratatui terminal UI) | — |
| `sqlite` | SQLite storage backend (via i-rs-claw-core) | sqlx/sqlite |
| `mysql` | MySQL storage backend (via i-rs-claw-core) | sqlx/mysql |
| `postgres` | PostgreSQL storage backend (via i-rs-claw-core) | sqlx/postgres |
| `mongo` | MongoDB storage backend (via i-rs-claw-core) | mongodb |
| `redis` | Redis storage backend (via i-rs-claw-core) | redis |

All storage features forward to `i-rs-claw-core`. The default `file` backend needs no extra flags.

## Configuration

Create `~/.i-rs/claw/config.toml`:

```toml
[providers.default]
provider = "openai"
api_key = "sk-..."
base_url = "https://api.deepseek.com"
model = "deepseek-v4-flash"
```

Supports any OpenAI-compatible API (DeepSeek, OpenRouter, Groq, etc.):

```toml
[providers.default]
provider = "openai"
api_key = "sk-..."
base_url = "https://openrouter.ai/api/v1"
model = "deepseek/deepseek-chat"
```

### Dashboard / Multi-user

```toml
[dashboard]
host = "0.0.0.0"
port = 3000
# auth_token = "my-secret"  # unset → auto-generated UUID

# Multi-user mode (optional)
[[dashboard.users]]
id = "alice"
token = "alice-token-xxx"

[[dashboard.users]]
id = "bob"
token = "bob-token-yyy"
```

Without `users`, single-user mode: all requests belong to `default` user.
With `users`, each token maps to a distinct user with isolated sessions, agents, and stats.

See [config.example.toml](config.example.toml) for all options (Agents, MCP, Gateway, Stats, Storage backends, etc.).

## Usage

### Web Dashboard

```
http://localhost:3000#<auth-token>
```

Pages: Chat, Sessions, Agents, Usage, Config, Tools, Plugins, Skills.

### TUI

```bash
claw tui                    # new session
claw tui --session <ID>    # resume specific session
claw tui --user alice      # multi-tenant mode
```

Type your request and press Enter:

```
> 记录体重75kg
> 这个月跑步情况如何？
> 帮我看看最近的支出
> How is my health today?
```

**Controls:**

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Ctrl+Q` / `Ctrl+C` | Quit |
| `Ctrl+N` | New session |
| `Ctrl+L` | Session list |
| `Ctrl+P` | Switch agent |
| `Ctrl+H` | Help |
| `Ctrl+E` | Export session |

## Voice Input (macOS)

i-rs-claw supports hands-free voice input via [VoiceInput](https://github.com/shibing624/VoiceInput), a lightweight menu bar tool for macOS.

### Setup

1. Download the DMG from [releases](https://github.com/shibing624/VoiceInput/releases)
2. Drag to Applications folder and launch
3. Grant required permissions on first run:
   - **Microphone** — for audio capture
   - **Speech Recognition** — for on-device transcription (Apple Speech)
   - **Accessibility** — for text injection into terminal

### Usage

While the i-rs-claw input field is focused, hold **Fn** and speak. Release the key — transcribed text is injected at cursor position.

## Architecture

```
claw binary (one binary, two modes)
├── claw serve  ──→  HTTP API (axum) + Web Dashboard (React SPA)
│   ├── /api/chat        POST → SSE stream
│   ├── /api/sessions    CRUD
│   ├── /api/agents      CRUD
│   ├── /api/stats       Token usage
│   └── /api/tools       Tool registry
│
├── claw tui   ──→  Terminal UI (ratatui)
│   ├── Direct claw-core access (zero HTTP overhead)
│   └── Same storage backend as serve
│
└── claw-core (shared lib)
    ├── AI engine (chat_loop, ReAct cycle)
    ├── LLM providers (OpenAI, Anthropic, Ollama, Zhipu)
    ├── Tools (70+ i-rs, web_search, file_ops, MCP, delegate, vision)
    ├── Storage (File, SQLite, MySQL, Postgres, Mongo, Redis)
    ├── Session + Memory
    └── Config + Stats
```

| Component | Role | User |
|-----------|------|------|
| **claw serve** | Main product: API gateway + Web Dashboard | All users |
| **claw tui** | Subcommand: terminal debug/power-user mode | Developers |
| **claw-core** | Pure lib: AI engine (no main.rs) | Shared by both modes |
| **dashboard-ui** | React SPA: Chat + Data + Agents + Usage + Settings | Web users |
| **小程序/App** | Independent clients via HTTP API to serve | Mobile users |

## Tech Stack

- **UI (TUI)**: [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm)
- **UI (Web)**: React 19 + Vite + wouter + react-markdown
- **API**: [axum](https://github.com/tokio-rs/axum) + SSE streaming
- **LLM**: OpenAI-compatible chat completion API
- **Backend**: i-rs CLI tools via subprocess (`Command::new("i-rs")`)
- **Storage**: File (default) / SQLite / MySQL / PostgreSQL / MongoDB / Redis

## Clients

### iOS App (IrsClawApp)

**Build:**

```bash
cd apps/IrsClawApp
xcodebuild -project IrsClawApp.xcodeproj -scheme IrsClawApp -destination 'platform=iOS Simulator,name=iPhone 17' -allowProvisioningUpdates build
```

**API Endpoint:** `/api/chat` (POST, returns SSE stream directly — unified endpoint)

### MiniProgram (IrsClawMiniProgram)

Uses `sendMessageAndStream()` from `utils/api.js` for unified SSE streaming.

**API Endpoint:** `/api/chat` (POST, returns SSE stream directly — unified endpoint)
