# i-rs-claw Examples

## Basic TUI Usage

### Starting the Assistant

```bash
# Just launch
i-rs-claw

# Launch with a specific session
i-rs-claw tui --session "550e8400-e29b-41d4-a716-446655440000"
```

### Natural Language Queries

Once in the TUI, type your request and press Enter:

```
> 记录体重75kg
> 这个月跑步情况如何？
> 帮我看看最近的支出
> 该换牙刷了吗？
> 今天心情怎么样？
```

## Non-Interactive Mode

### Ask Questions Directly

```bash
# Record a weight entry
i-rs-claw ask "记录体重75kg"

# Ask about recent data
i-rs-claw ask "这个月跑步情况如何？"

# Continue a previous conversation
i-rs-claw ask "上次说到哪里了？" --session "550e8400-e29b-41d4-a716-446655440000"
```

## Session Management

```bash
# List all sessions
i-rs-claw session --list

# Export a session as Markdown
i-rs-claw session --export-md "550e8400-e29b-41d4-a716-446655440000"

# Export as JSON
i-rs-claw session --export-json "550e8400-e29b-41d4-a716-446655440000"
```

## Configuration

### Interactive Setup

```bash
i-rs-claw config
```

This interactive wizard will prompt for:
1. Provider (openai/anthropic/ollama)
2. API Key
3. Base URL
4. Model
5. Search API Key (optional)
6. MCP Servers (optional)

### Tool Management

```bash
# Launch the interactive tool manager
i-rs-claw tools
```

Navigate with arrow keys, toggle with Space, select all with `a`.

## Plugin Management

```bash
# List discovered plugins
i-rs-claw plugin --list

# Show plugin details
i-rs-claw plugin --info my-plugin

# Enable a plugin
i-rs-claw plugin --enable my-plugin

# Disable a plugin
i-rs-claw plugin --disable my-plugin
```

## Voice Input (macOS)

Requires [VoiceInput](https://github.com/shibing624/VoiceInput):

1. Install VoiceInput from the DMG
2. Grant Microphone, Speech Recognition, and Accessibility permissions
3. With the TUI input focused, hold **Fn** key and speak
4. Release the key — transcribed text is injected at cursor

## Gateway (Social Platform Integration)

### Configuration

```toml
# config.toml
[gateway]
enabled = true

[gateway.telegram]
enabled = true
token = "123456:ABC-DEF..."
```

Build and run the gateway server:

```bash
cargo build -p i-rs-claw --release
i-rs-claw gateway
```
i-rs-claw gateway
```

## Dashboard (Web UI)

### Build and Run

```bash
cargo build -p i-rs-claw --release --features dashboard
i-rs-claw dashboard
```

Configuration via `config.toml`:

```toml
[dashboard]
enabled = true
host = "127.0.0.1"
port = 3000
```

### Dashboard API Examples

```bash
# Health check
curl http://127.0.0.1:3000/api/health

# Get configuration
curl http://127.0.0.1:3000/api/config

# List sessions
curl http://127.0.0.1:3000/api/sessions

# List tools
curl http://127.0.0.1:3000/api/tools

# List plugins
curl http://127.0.0.1:3000/api/plugins

# Send a message
curl -X POST http://127.0.0.1:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "记录体重75kg"}'

# SSE stream for chat responses
curl -N http://127.0.0.1:3000/api/chat/stream/&lt;SESSION_ID&gt;

# Get session messages
curl http://127.0.0.1:3000/api/sessions/&lt;SESSION_ID&gt;

# Delete a session
curl -X DELETE http://127.0.0.1:3000/api/sessions/&lt;SESSION_ID&gt;
```

## MCP (Model Context Protocol)

### Adding External Tool Servers

```bash
# Via config wizard
i-rs-claw config

# Or directly in config.toml
[[mcp_servers]]
name = "filesystem"
transport_type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed"]

[[mcp_servers]]
name = "remote-api"
transport_type = "sse"
url = "https://my-mcp-server.example.com/tools"
```

## Provider Examples

### OpenAI (default)

```toml
[config]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
```

### Anthropic

```toml
[config]
provider = "anthropic"
api_key = "sk-ant-..."
base_url = "https://api.anthropic.com/v1"
model = "claude-3-haiku-20240307"
```

### Ollama (local)

```toml
[config]
provider = "ollama"
base_url = "http://localhost:11434/v1"
model = "llama3.2"
# No API key needed for local Ollama
```

### DeepSeek via OpenRouter

```toml
[config]
api_key = "sk-..."
base_url = "https://openrouter.ai/api/v1"
model = "deepseek/deepseek-chat"
```
