# i-rs-code: Code Editor AI Agent

## Overview

`i-rs-code` is a TUI code editor AI agent in the i-rs ecosystem, similar to Claude Code, Codex CLI, and OpenCode. Built in Rust with ratatui.

## Key Design Decisions

1. **Independent crate** — does NOT depend on `i-rs-claw`, references its patterns
2. **Dual-mode binary** — TUI mode for interactive use, CLI mode for claw subprocess calls
3. **MVP: Layer 1 + Layer 2** — ReAct loop + file tools + search tools + shell
4. **Claw integration** — `i-rs-code chat --json "prompt"` for subprocess consumption

## Subcommands

```
i-rs-code                  # Enter TUI full-screen mode
i-rs-code chat "msg"       # One-shot conversation (streaming, for claw)
i-rs-code chat --json "msg" # One-shot conversation (JSON lines)
i-rs-code config            # View/edit config
```

## Architecture

```
src/
├── main.rs            # Entry: clap subcommand dispatch
├── cli.rs             # Cli definition
├── app.rs             # App state (message list, input, sessions)
├── config.rs          # ~/.config/i-rs-code/config.toml
├── agent/
│   ├── mod.rs         # Agent main loop
│   ├── loop.rs        # ReAct loop (stream → tool_calls → execute → loop → done)
│   └── context.rs     # Context management + auto-compaction
├── tools/
│   ├── mod.rs         # Tool trait + ToolRegistry
│   ├── filesystem.rs  # Read, Write, Edit, Glob, Grep, Ls
│   ├── bash.rs        # Shell command execution
│   └── git.rs         # Git operations (commit, diff, log)
├── provider/
│   ├── mod.rs         # LlmProvider trait
│   ├── openai.rs      # OpenAI-compatible API
│   └── anthropic.rs   # Anthropic API
├── diff.rs            # Unified diff parse/apply + checkpoint snapshots
├── session.rs         # Session persistence (JSONL)
├── tui.rs             # TUI event loop
├── ui.rs              # ratatui rendering
└── utils.rs           # Helper functions
```

## Dependencies

| Category | Crates |
|----------|--------|
| CLI | `clap` (derive) |
| TUI | `ratatui`, `crossterm` |
| Async | `tokio` |
| LLM | `reqwest`, `serde_json`, `serde`, `futures` |
| Config | `toml`, `dirs` |
| Filesystem | `ignore` (gitignore-aware), `globset` |
| Diff | `similar` |
| UUID | `uuid` |
| Color | `owo-colors` |

## TUI Layout (MVP)

```
┌──────────────────────────────────────┐
│  File Browser  │  Conversation Panel │
│                │                     │
│  src/          │  [AI] Let me check  │
│  ├── main.rs   │  the file...        │
│  ├── lib.rs    │                     │
│  └── utils/    │  [Tool] Read file   │
│                │  src/main.rs (240b) │
│                │                     │
│                │  [AI] I see the     │
│                │  issue, let me fix  │
│                │  it...              │
├──────────────────────────────────────┤
│  > Type your prompt...               │
└──────────────────────────────────────┘
```

## ReAct Loop

```
User Input → build_messages() → LLM stream → tool_calls → parallel execute → inject results → loop → text response
```

- Pure ReAct (single-threaded while-loop)
- Tool execution in parallel via `tokio::spawn`
- Auto-compaction at ~80% context window
- Checkpoints before every file edit (non-git snapshots)

## Tool Definition

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> serde_json::Value;  // JSON Schema for LLM
    async fn call(&self, args: &Map<String, Value>) -> ToolResult;
}
```

## File Security

- **Read-before-edit enforcement** — tools require file to be read before modification
- **Uniqueness validation** — Edit verifies old_string appears exactly once
- **Path traversal protection** — reject paths with `../` escaping workspace
- **Blocklist** — sensitive files (`.env`, SSH keys) gated

## Claw Integration

Claw invokes i-rs-code as a subprocess with `--json` flag:
```
i-rs-code chat --json "fix the bug in src/main.rs"
```

Output: JSON lines streaming (token, tool_call, tool_result, error, done)
Consumed by claw's `tools/i_rs.rs` (same pattern as existing CLI tool calls).

## Future (Post-MVP)

- LSP diagnostics integration
- MCP protocol extension
- Sub-agent delegation
- Git worktree isolation
- Sandboxed shell (bubblewrap/seatbelt)
