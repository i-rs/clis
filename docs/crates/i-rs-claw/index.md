# i-rs-claw

TUI intelligent personal data assistant powered by i-rs CLI tools.

## Overview

i-rs-claw is a terminal-based AI assistant that understands natural language and manages your personal data through the i-rs CLI toolset — health, finance, tasks, media, household, and more. It supports multiple interaction modes: a full TUI, a one-shot command-line ask mode, and optional social platform integration via the Gateway system.

## Quick Start

```bash
# Configure (interactive wizard)
i-rs-claw config

# Launch the TUI assistant
i-rs-claw
```

## Key Capabilities

| Feature | Description |
|---------|-------------|
| **Natural Language Interface** | "Record 75kg weight" or "How's my running this week?" — just type it |
| **70+ i-rs Tools** | Full access to all i-rs CLI tools for personal data management |
| **Streaming Responses** | Real-time token-by-token AI response display |
| **Tool Call Transparency** | See exactly which tools are being called and their results |
| **Multi-turn Conversations** | Context preserved across the session via JSONL persistence |
| **First-Learn-Then-Execute** | AI automatically learns tool syntax via `skill teach` before operating data |
| **Plugin System** | MCP-based plugin discovery with automatic tool integration |
| **Gateway** | Social platform integration (Telegram, Discord, Slack, WeChat) |
| **Dashboard** | Web-based REST API and SSE streaming (feature-gated) |
| **Voice Input** | macOS-native speech recognition via VoiceInput |
| **HTTP Debug Sidebar** | Inspect LLM request/response details in real-time |
| **Proactive Reminders** | i-rs remind integration for timely notifications |

## Installation

### Prerequisites

- Rust toolchain (see `rust-toolchain.toml` at project root)
- All `i-rs` CLI tools installed and in PATH (see project root `scripts/link_to_path.sh`)
- (For voice input) [VoiceInput](https://github.com/shibing624/VoiceInput) on macOS

### Build

```bash
# Standard build (TUI only)
cargo build -p i-rs-claw --release

# With Dashboard web UI
cargo build -p i-rs-claw --release --features dashboard

# Gateway is always included — configure platforms in config.toml
```

The binary will be at `target/release/i-rs-claw`.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      main.rs                             │
│  ┌────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   TUI      │  │   Gateway    │  │   Dashboard      │  │
│  │  (ui.rs)   │  │  (gateway/)  │  │  (dashboard/)    │  │
│  └──────┬─────┘  └──────┬───────┘  └────────┬────────┘  │
│         │               │                    │           │
│  ┌──────┴───────────────┴────────────────────┴────────┐ │
│  │                   AppCore (core/)                   │ │
│  │  ┌──────────┐ ┌─────────┐ ┌──────┐ ┌───────────┐  │ │
│  │  │ config   │ │ session │ │memory│ │tool_cache │  │ │
│  │  │ (toml)   │ │ (JSONL) │ │(JSON)│ │ (JSON)    │  │ │
│  │  └──────────┘ └─────────┘ └──────┘ └───────────┘  │ │
│  └────────────────────────────────────────────────────┘ │
│         │                                               │
│  ┌──────┴────────────────────────────────────────────┐  │
│  │              provider.rs (LLM)                     │  │
│  │   ┌────────┐ ┌────────┐ ┌────────┐ ┌─────────┐   │  │
│  │   │OpenAI  │ │Anthropic│ │ Ollama │ │ DeepSeek│   │  │
│  │   └────────┘ └────────┘ └────────┘ └─────────┘   │  │
│  └────────────────────────────────────────────────────┘  │
│         │                                               │
│  ┌──────┴────────────────────────────────────────────┐  │
│  │                    tools/                          │  │
│  │  ┌───────┐ ┌────────┐ ┌───────┐ ┌──────────┐    │  │
│  │  │ i-rs  │ │search  │ │ MCP   │ │  plugin  │    │  │
│  │  │ CLI   │ │ tools  │ │ tools │ │  (MCP)   │    │  │
│  │  └───────┘ └────────┘ └───────┘ └──────────┘    │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Data Location

All data is stored under `~/.i-rs-claw/`:

```
~/.i-rs-claw/
├── config.toml         # Main configuration
├── theme.json          # Custom color theme (optional)
├── claw/
│   ├── index.json      # Session index
│   ├── conv_cache.json # Conversation cache
│   ├── skills/         # User-defined skills
│   ├── memory.json     # Cross-session memory
│   ├── tool_cache.json # Tool documentation cache
│   └── sessions/       # Session data files (*.jsonl, *_api.json, *_plan.json)
└── plugins/
    ├── state.json      # Plugin enabled/disabled state
    └── <name>/
        └── plugin.toml # Plugin manifest
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `dashboard` | Disabled | Axum web server with REST API + SSE streaming |

Gateway (Telegram + WeChat) is always included — enable platforms via `config.toml`.

## Configuration

See [Usage](./usage.md) for detailed configuration reference.

## Commands

- [Usage](./usage.md) — Detailed command reference
- [Examples](./examples.md) — Extensive usage examples
- [Test](./test.md) — Test records
