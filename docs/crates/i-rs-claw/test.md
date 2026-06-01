# i-rs-claw Test Records

## Basic Commands

```bash
# Launch TUI
i-rs-claw

# Config wizard
i-rs-claw config

# Tool management
i-rs-claw tools

# Non-interactive ask
i-rs-claw ask "Hello, who are you?"

# Session management
i-rs-claw session --list
```

## Feature Flags

```bash
# Build with dashboard
cargo build -p i-rs-claw --features dashboard
i-rs-claw dashboard

# Gateway (always included)
i-rs-claw gateway
```

## Plugin System

```bash
# Create a test plugin directory
mkdir -p ~/.i-rs/claw/plugins/test-plugin

# Create plugin manifest
cat <<EOF > ~/.i-rs/claw/plugins/test-plugin/plugin.toml
[plugin]
name = "test-plugin"
version = "1.0.0"
description = "Test plugin for development"

[transport]
transport_type = "stdio"
command = "echo"
args = ["hello"]
EOF

# List plugins
i-rs-claw plugin --list

# Enable/disable
i-rs-claw plugin --enable test-plugin
i-rs-claw plugin --disable test-plugin

# Show info
i-rs-claw plugin --info test-plugin
```

## Session CRUD

```bash
# Create a session via ask
i-rs-claw ask "Hello"

# List sessions
i-rs-claw session --list

# Export session
i-rs-claw session --export-md <SESSION_ID>
i-rs-claw session --export-json <SESSION_ID>
```

## Dashboard API

```bash
# Start dashboard
cargo build -p i-rs-claw --features dashboard
i-rs-claw dashboard &

# Test endpoints
curl http://127.0.0.1:3000/api/health
curl http://127.0.0.1:3000/api/sessions

# Kill dashboard
kill %1
```

## MCP Server (stdio)

```bash
# Add in config.toml:
# [[mcp_servers]]
# name = "test"
# transport_type = "stdio"
# command = "echo"
# args = ["{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"tools\":[]}}"]
```

## Voice Input (macOS Only)

```bash
# Setup
brew install --cask voiceinput

# Grant permissions:
# - Microphone
# - Speech Recognition
# - Accessibility

# Usage: Hold Fn key and speak in the TUI input field
```
