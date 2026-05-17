# i-rs-claw Usage

## Global Options

- `--help` — Show help information
- `--version` — Show version information

## Commands

### TUI (default)

Launch the interactive TUI assistant.

```bash
i-rs-claw
i-rs-claw tui
i-rs-claw tui --session <SESSION_ID>
```

Options:
- `--session <SESSION_ID>` — Resume a specific session by ID

### config

Interactive configuration wizard.

```bash
i-rs-claw config
```

Prompts for:
- **Provider** — `openai` (default), `anthropic`, `ollama`, etc.
- **API Key** — Your LLM API key
- **Base URL** — API endpoint (default: `https://api.openai.com/v1`)
- **Model** — Model name (default: `gpt-4o-mini`)
- **Search API Key** — Custom search engine key (optional, falls back to DuckDuckGo)
- **Search Base URL** — Custom search endpoint (optional)
- **MCP Servers** — External tool servers via Model Context Protocol

### tools

Interactive tool enable/disable manager.

```bash
i-rs-claw tools
```

Controls:
- `↑`/`↓` — Navigate
- `Space` — Toggle tool
- `a` — Select all
- `n` — Clear selection
- `Enter` — Save
- `Esc`/`q` — Cancel

Default tools enabled: `kv`, `weight`, `water`, `sleep`, `meal`, `pig`, `mood`, `sit`, `spark`, `todo`

### session

List and manage conversation sessions.

```bash
i-rs-claw session --list
i-rs-claw session --export-md <SESSION_ID>
i-rs-claw session --export-json <SESSION_ID>
```

Options:
- `--list` — List all sessions
- `--export-md <SESSION_ID>` — Export session as Markdown
- `--export-json <SESSION_ID>` — Export session as JSON

### ask

Send a message and print the response (non-interactive).

```bash
i-rs-claw ask "记录体重75kg"
i-rs-claw ask "这个月跑步情况如何？" --session <SESSION_ID>
```

Options:
- `--session <SESSION_ID>` — Session ID for context continuity

### gateway

Start the gateway server for social platform integration.

```bash
i-rs-claw gateway
```

Requires configuration in `config.toml`:
- `[gateway]` — Master switch
- `[gateway.telegram]` — Telegram bot token
- `[gateway.discord]` — Discord bot token
- `[gateway.slack]` — Slack bot token + app token
- `[gateway.wechat]` — WeChat webhook URL + secret

Requires feature flags: `gateway-telegram`, `gateway-discord`, `gateway-slack`, `gateway-wechat`, or `gateway-all`.

### dashboard

Start the dashboard web server.

```bash
i-rs-claw dashboard
```

Requires feature flag `dashboard`. Defaults to `http://127.0.0.1:3000`.

API endpoints:
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/config` | Current configuration (sanitized) |
| GET | `/api/sessions` | List all sessions |
| GET | `/api/sessions/:id` | Get session messages |
| DELETE | `/api/sessions/:id` | Delete a session |
| GET | `/api/tools` | List available tools with schemas |
| GET | `/api/plugins` | List installed plugins |
| POST | `/api/chat` | Send a message |
| GET | `/api/chat/stream/:session_id` | SSE stream for chat responses |

### plugin

List and manage plugins.

```bash
i-rs-claw plugin --list
i-rs-claw plugin --info <NAME>
i-rs-claw plugin --enable <NAME>
i-rs-claw plugin --disable <NAME>
```

Options:
- `--list` — List all discovered plugins
- `--info <NAME>` — Show detailed plugin info
- `--enable <NAME>` — Enable a plugin
- `--disable <NAME>` — Disable a plugin

## Configuration File

Location: `~/.i-rs-claw/config.toml`

```toml
[config]
# Required: LLM API key (can also set via I_RS_CLAW_API_KEY env var)
api_key = "sk-..."

# Provider: "openai" (default), "anthropic", "ollama"
provider = "openai"

# API base URL (supports any OpenAI-compatible API)
base_url = "https://api.openai.com/v1"

# Model name
model = "gpt-4o-mini"

# Search API key (optional, falls back to DuckDuckGo)
search_api_key = "..."
search_base_url = "https://api.custom-search.com/v1"

# Allowed directories for file operations
allowed_dirs = ["~/Documents", "~/Downloads"]

# MCP server connections
[[mcp_servers]]
name = "filesystem"
transport_type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path"]

# Plugin auto-discovery (default: true)
plugins_auto_discover = true

[gateway]
enabled = true

[gateway.telegram]
enabled = true
token = "123456:ABC-DEF..."

[dashboard]
enabled = true
host = "127.0.0.1"
port = 3000
```

### Environment Variables

- `I_RS_CLAW_API_KEY` — Override API key from environment
- `CONFIG_DIR` — Override config directory (not yet implemented for claw)

## Data Storage

- Config: `~/.i-rs-claw/config.toml`
- Sessions: `~/.i-rs-claw/claw/sessions/*.jsonl`
- Session index: `~/.i-rs-claw/claw/index.json`
- Cross-session memory: `~/.i-rs-claw/claw/memory.json`
- Tool cache: `~/.i-rs-claw/claw/tool_cache.json`
- Skills: `~/.i-rs-claw/claw/skills/`
- Plugins: `~/.i-rs-claw/plugins/<name>/plugin.toml`
- Plugin state: `~/.i-rs-claw/plugins/state.json`
- Theme: `~/.i-rs-claw/theme.json`
