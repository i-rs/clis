# i-rs-code: Code Editor AI Agent

## Overview

`i-rs-code` is a TUI code editor AI agent in the i-rs ecosystem, similar to Claude Code, Codex CLI, and OpenCode. Beyond generic code editing, it has first-class support for **creating and maintaining i-rs CLI tools**, with **bidirectional communication** to claw for multi-agent collaboration.

## Key Design Decisions

1. **Independent crate** — does NOT depend on `i-rs-claw`, references its patterns
2. **Dual-mode binary** — TUI mode for interactive use, agent mode for claw subprocess calls
3. **MVP: Layer 1 + Layer 2** — ReAct loop + file tools + search tools + shell
4. **Bidirectional JSON-RPC protocol** over stdio for claw ↔ i-rs-code communication
5. **Built-in i-rs crate templates** — knows how to create standard i-rs CLI tools

## Communication Protocol (Claw ↔ i-rs-code)

The core extension beyond a standalone code editor: a **bidirectional JSON-RPC protocol** over stdio.

### Launch Mode

Claw spawns i-rs-code with a dedicated agent flag:

```
i-rs-code agent --task-id "uuid-123"
```

- **stdin (Claw → i-rs-code)**: Commands, context, debug responses
- **stdout (i-rs-code → Claw)**: Events, results, tool definitions (JSON lines)
- **stderr**: Logging only

### Message Types

**Claw → i-rs-code (stdin):**

| Type | Purpose |
|------|---------|
| `task` | Assign a code generation task with prompt + context |
| `respond` | Respond to i-rs-code's debug/help request |
| `cancel` | Cancel current task |
| `context` | Provide file contents, project info |

```json
{"type": "task", "task_id": "uuid-123", "prompt": "创建一个 i-rs-mood CLI 工具...", "context": {"workspace": "/path/to/clis", "tools": ["i-rs-todo", "i-rs-weight"]}}
{"type": "respond", "task_id": "uuid-123", "request_id": "req-1", "content": "尝试在 Cargo.toml 添加 serde 依赖"}
{"type": "cancel", "task_id": "uuid-123"}
```

**i-rs-code → Claw (stdout):**

| Event | Purpose |
|-------|---------|
| `token` | Streaming LLM token |
| `progress` | Stage update (creating_crate, writing_code, compiling, testing) |
| `request` | Request help from claw (debug, design review) |
| `tool_created` | New CLI tool definition for claw to register |
| `tool_updated` | Existing tool schema updated |
| `error` | Error with detail |
| `done` | Task complete with summary |

```json
{"event": "token", "task_id": "uuid-123", "content": "Let me create..."}
{"event": "progress", "task_id": "uuid-123", "stage": "creating_crate", "detail": "Generating i-rs-mood crate structure"}
{"event": "request", "task_id": "uuid-123", "request_id": "req-1", "type": "debug", "content": "Build error: ...", "context": {"file": "src/main.rs", "error": "..."}}
{"event": "tool_created", "task_id": "uuid-123", "tool": {"name": "i-rs-mood", "description": "Mood tracking tool", "commands": ["add", "list", "calendar", "stats"], "path": "crates/clis/i-rs-mood"}}
{"event": "done", "task_id": "uuid-123", "result": "Created i-rs-mood with add/list/calendar/stats commands"}
```

### Lifecycle

```
User: "帮我创建一个记录心情的 CLI"

Claw: 判断需要代码生成
  → spawn i-rs-code agent --task-id "uuid-123"
  → send task with prompt + workspace context

i-rs-code: ReAct loop
  1. create_i_rs_crate() → 生成目录结构 + Cargo.toml
  2. write main.rs, commands/, models/, storage/, presentation/
  3. cargo check → build error
  4. request claw for debug help
     → Claw receives request, analyzes, responds
  5. apply fix, recheck → pass
  6. register_tool() → output tool definition
  7. done → summary

Claw: receives tool_created event
  → registers i-rs-mood in tool registry
  → informs user "工具已创建并可用"
```

## Subcommands

```
i-rs-code                        # Enter TUI full-screen mode
i-rs-code chat "msg"             # One-shot conversation (streaming output)
i-rs-code agent --task-id "id"   # Agent mode (JSON-RPC over stdio, used by claw)
i-rs-code config                 # View/edit config
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
├── protocol/
│   ├── mod.rs         # JSON-RPC message types (serde)
│   ├── transport.rs   # Stdio transport (read from stdin, write to stdout)
│   └── handler.rs     # Message dispatch (task/respond/cancel)
├── tools/
│   ├── mod.rs         # Tool trait + ToolRegistry
│   ├── filesystem.rs  # Read, Write, Edit, Glob, Grep, Ls
│   ├── bash.rs        # Shell command execution
│   ├── git.rs         # Git operations (commit, diff, log)
│   ├── create_crate.rs # i-rs crate template generator
│   ├── call_claw.rs   # Request debugging help from claw
│   └── register_tool.rs # Output tool definition for claw registration
├── templates/
│   ├── mod.rs         # Template loading
│   ├── cargo_toml.rs  # Cargo.toml template
│   ├── main_rs.rs     # main.rs template
│   ├── commands.rs    # Command module templates
│   ├── models.rs      # Data model templates
│   └── storage.rs     # Storage module templates
├── provider/
│   ├── mod.rs         # LlmProvider trait
│   ├── openai.rs      # OpenAI-compatible API
│   └── anthropic.rs   # Anthropic API
├── diff.rs            # Unified diff parse/apply + checkpoint snapshots
├── session.rs         # Session persistence (JSONL)
├── tui.rs             # TUI event loop
├── ui.rs              # ratatui rendering
└── utils.rs
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
┌──────────────────────────────────────────┐
│  File Browser  │  Conversation Panel     │
│                │                         │
│  crates/clis/  │  [AI] Let me create...  │
│  ├── i-rs-mood │                         │
│  │   ├── src/  │  [Tool] create_crate    │
│  │   ├── ...   │  → Generated i-rs-mood  │
│  │   └── ...   │                         │
│                │  [Tool] cargo check      │
│                │  → Build error: ...      │
│                │                         │
│                │  [Tool] call_claw        │
│                │  → Sent debug request    │
│                │                         │
│                │  [Tool] fix → recheck    │
│                │  → cargo check passed    │
├──────────────────────────────────────────┤
│  > Creating i-rs-mood CLI tool...        │
└──────────────────────────────────────────┘
```

## i-rs-Code Specific Tools

### create_crate
Generates a new i-rs CLI crate following project standards:

```json
{
  "name": "i-rs-mood",
  "description": "Mood tracking CLI tool",
  "special_commands": ["calendar", "stats"]
}
```

Creates:
- `crates/clis/i-rs-{name}/src/{models,storage,commands,presentation}/`
- `Cargo.toml` with workspace deps
- `src/main.rs` with clap + exit_on_error!
- `src/commands/` — add, delete, get, list, update, example, skill
- `src/models/mod.rs` — Entity + Row + Store
- `src/storage/mod.rs` — create_store!
- `src/presentation/mod.rs` — render_table
- `docs/crates/i-rs-{name}/` — markdown docs
- `skills/i-rs-{name}/SKILL.md`

Updates workspace `Cargo.toml` members list and VitePress sidebar config.

### call_claw
Sends a debug/help request to claw over stdout and waits for response on stdin:

```json
// i-rs-code → claw (stdout):
{"event": "request", "task_id": "uuid-123", "request_id": "req-1", "type": "debug", "content": "build error: ..."}

// claw → i-rs-code (stdin):
{"type": "respond", "task_id": "uuid-123", "request_id": "req-1", "content": "try adding serde..."}
```

### register_tool
Outputs the created tool's schema for claw to register in its tool registry:

```json
{"event": "tool_created", "task_id": "uuid-123", "tool": {
  "name": "i-rs-mood",
  "description": "Mood tracking CLI tool",
  "commands": ["add", "delete", "get", "list", "update", "calendar", "stats"],
  "path": "crates/clis/i-rs-mood"
}}
```

Claw ingests this and adds the tool to its ToolRegistry, making it available for future conversations.

## ReAct Loop

```
task → build_messages() → LLM stream → tool_calls → parallel execute → inject results → loop → done
```

Tool execution order for a typical crate creation:
1. `list_dir` → explore workspace structure
2. `create_crate` → scaffold i-rs crate
3. `write` → write each source file
4. `bash(cargo check)` → verify compilation
5. `read` → examine error output
6. `call_claw` → request debugging help (if stuck)
7. `edit` → fix issues
8. `bash(cargo check)` → re-verify
9. `register_tool` → output tool definition
10. done

## Human-in-the-Loop 审批

i-rs-code 本身没有权限系统，所有需要人工介入的操作通过协议发给 claw，由 claw 的 permission 系统处理。

### 审批触发节点

| 事件 | 触发条件 | 预期行为 |
|------|---------|---------|
| 创建文件 | `write` 工具调用 | 自动发 approval 请求 + diff 预览 |
| 修改文件 | `edit` 工具调用 | 自动发 approval 请求 + diff 预览 |
| 执行 shell | `bash` 工具调用（写操作） | 只读命令（cargo check/mkdir/git status）免审批，写入/删除命令需要审批 |
| git commit/push | git 操作 | 请求审批 |
| 注册新工具 | `register_tool` 调用 | 审批通过后才正式注册到 claw |
| 批量修改 | 一次超过 3 个文件 | 请求审批 |

### 协议消息

```json
// i-rs-code → claw: 请求审批
{"event": "request", "task_id": "uuid-123", "request_id": "req-2",
 "type": "approval", "stage": "write",
 "content": "Write src/main.rs (240 lines)",
 "detail": {"file": "src/main.rs", "diff": "+240 -0 lines", "diff_content": "#[derive(Parser)]..."}}

// claw 展示给用户: [Y]es / [N]o / [S]kip
// 用户确认后，claw 回复
{"type": "respond", "task_id": "uuid-123", "request_id": "req-2",
 "content": "approved"}
// 或
{"type": "respond", "task_id": "uuid-123", "request_id": "req-2",
 "content": "rejected", "reason": "不需要这个文件"}

// 如果用户请求修改后执行
{"type": "respond", "task_id": "uuid-123", "request_id": "req-2",
 "content": "modify", "modification": "把文件名改成 lib.rs"}
```

### TUI 模式

独立 TUI 模式下无审批——用户就在终端前实时观察每一步，看到不满意直接 Ctrl-C 打断用 Edit 改。但每个 `write`/`edit` 操作会 **自动 checkpoint 快照**，用户可以随时回退：

```
[Checkpoint] Saving snapshot before edit: src/main.rs (240 bytes)
[Checkpoint] 3 snapshots available. ~s to view history.
```

### Agent 模式（claw 调用）

审批流程：
1. i-rs-code 到达需要审批的步骤，发送 `type: "approval"` 请求
2. i-rs-code 暂停，等待 stdin 上的 `respond`
3. claw 收到请求后，根据用户配置的 permission level 处理：
   - **always**: 自动 approved，不打扰用户
   - **default**: 展示给用户确认
   - **strict**: 默认拒绝，用户手动批准才能执行
4. claw 回复 `respond`，i-rs-code 继续执行

## File Security

- **Read-before-edit enforcement** — tools require file to be read before modification
- **Uniqueness validation** — Edit verifies old_string appears exactly once
- **Path traversal protection** — reject paths with `../` escaping workspace
- **Blocklist** — sensitive files (`.env`, SSH keys) gated

## Claw Integration

On the claw side, a new `tools/code.rs` wraps i-rs-code:

```rust
// Claw tool: call i-rs-code for code generation
pub struct CodeTool;

#[async_trait]
impl ClawTool for CodeTool {
    fn name(&self) -> &str { "create_cli_tool" }
    fn description(&self) -> &str { "Create or modify i-rs CLI tools using i-rs-code" }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let task_id = Uuid::new_v4();
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        
        // Spawn i-rs-code as subprocess
        let mut child = Command::new("i-rs-code")
            .args(["agent", "--task-id", &task_id.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        
        // Send task
        send_json(&child.stdin, json!({"type": "task", "task_id": task_id, "prompt": prompt, "context": {...}}));
        
        // Read events, handle requests
        for event in read_events(&child.stdout) {
            match event.event.as_str() {
                "request" => {
                    // Call claw's own LLM to help debug
                    let response = claw_llm_chat(&event.content).await?;
                    send_json(&child.stdin, json!({"type": "respond", ...}));
                }
                "tool_created" => {
                    // Register new tool in registry
                    tool_registry.register_remote(event.tool);
                }
                "done" => break,
            }
        }
    }
}
```

## Future (Post-MVP)

- LSP diagnostics integration
- MCP protocol extension (i-rs-code as MCP server)
- Sub-agent delegation (i-rs-code spawns sub-agents for parallel work)
- Git worktree isolation for safe code generation
- Sandboxed shell (bubblewrap/seatbelt)
- Template customization via config
