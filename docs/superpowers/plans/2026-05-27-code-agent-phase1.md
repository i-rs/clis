# i-rs-code Phase 1: 让 Agent 能真正写代码

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 5 个关键阻塞项，让 i-rs-code 能作为独立写代码 agent 正常工作

**Architecture:** TUI 多轮对话状态持久化 → bash 工具放行编译命令 → read 工具支持分页 → ContextManager 集成到 ReAct 循环 → 系统提示词增强为专业编码 agent

**Tech Stack:** Rust, tokio, ratatui/crossterm, serde_json

---

### Task 1: TUI 多轮对话 — Agent 历史消息持久化

**Problem:** `tui.rs:283-293` 每次 Enter 都 `tokio::spawn` 创建新 `Agent`，消息历史丢失。LLM 每轮只看到当前 prompt + system prompt，完全无记忆。

**Files:**
- Modify: `crates/code/src/app.rs`
- Modify: `crates/code/src/tui.rs`
- Modify: `crates/code/src/agent/mod.rs`
- Modify: `crates/code/src/agent/event.rs`

- [ ] **Step 1: App 新增 `Agent` 持久化字段**

在 `crates/code/src/app.rs` 的 `App` struct 中添加：
```rust
pub agent_messages: Vec<crate::provider::LlmMessage>,
```
在 `App::new()` 初始化为 `Vec::new()`。

- [ ] **Step 2: AgentEvent 新增 ToolCall 事件（带完整 LlmMessage）**

修改 `crates/code/src/agent/event.rs`，让 `AgentEvent` 携带足够信息让 App 重建 `LlmMessage`：

在 `AgentEvent::ToolCallStart` 中已经有 `id, name, args`。在 `AgentEvent::ToolCallEnd` 中已经有 `id, name, result`。

新增 `AgentEvent::MessagesUpdate` 事件用于同步完整的 messages 列表：
```rust
pub enum AgentEvent {
    Token(String),
    ToolCallStart { id: String, name: String, args: Value },
    ToolCallEnd { id: String, name: String, result: String },
    Done { usage: Option<crate::provider::Usage>, messages: Vec<crate::provider::LlmMessage> },
    Error(String),
}
```

- [ ] **Step 3: engine::react_loop_streaming 返回 messages**

修改 `crates/code/src/agent/engine.rs` 的 `react_loop_streaming`，在 `AgentEvent::Done` 中附带 `messages.clone()`：

```rust
event_tx.send(AgentEvent::Done { usage: Some(total_usage.clone()), messages: messages.clone() }).await.ok();
```

同步修改 `react_loop` 的 `Done` 处理，在返回前也 clone messages（非 TUI 模式也需要）。

- [ ] **Step 4: Agent::run_once_streaming 接收历史消息**

修改 `crates/code/src/agent/mod.rs`：
```rust
pub async fn run_once_streaming(
    &mut self,
    prompt: &str,
    event_tx: mpsc::Sender<event::AgentEvent>,
    history: Vec<LlmMessage>,
) -> anyhow::Result<String> {
    let system_text = crate::prompt::SYSTEM;
    let tool_defs = self.tools.schemas();
    let msgs = build_messages(&history, system_text, prompt);
    // ... rest unchanged
}
```

`build_messages` 已经接受 `&[LlmMessage]` 作为 history，无需修改。

- [ ] **Step 5: tui.rs 保持 Agent 跨轮存活**

修改 `crates/code/src/tui.rs`：

1. 添加 `App` 上的 `CancelToken` 用于中断：
```rust
// app.rs
pub cancel_token: Option<tokio::sync::oneshot::Sender<()>>,
```

2. `handle_key` Enter 分支改为在 `App` 上存储 agent 配置，spawn 时传入 `agent_messages`：
```rust
KeyCode::Enter if !app.input.is_empty() => {
    let prompt = std::mem::take(&mut app.input);
    app.cursor_pos = 0;
    app.messages.push(ChatMessage {
        role: "user".into(),
        content: prompt.clone(),
    });
    app.start_streaming();
    app.mode = AppMode::Waiting;

    let config = app.config.clone();
    let tx = event_tx.clone();
    let history = app.agent_messages.clone();
    tokio::spawn(async move {
        if let Err(e) = run_streaming_agent(&config, &prompt, tx.clone(), history).await {
            tx.send(AgentEvent::Error(e.to_string())).await.ok();
        }
    });
}
```

3. `run_streaming_agent` 接收 history 并传给 Agent：
```rust
async fn run_streaming_agent(
    config: &crate::config::Config,
    prompt: &str,
    event_tx: mpsc::Sender<AgentEvent>,
    mut history: Vec<crate::provider::LlmMessage>,
) -> anyhow::Result<()> {
    let provider = crate::provider::create_provider(config)?;
    let tools = crate::tools::ToolRegistry::new(config)?;
    let mut agent = crate::agent::Agent::new(config.clone(), provider, tools, false);
    agent.run_once_streaming(prompt, event_tx.clone()).await
}
```

4. `handle_event` 的 `Done` 分支提取 messages 并存到 App：
```rust
AgentEvent::Done { usage, messages } => {
    let content = app.finish_streaming();
    if let Some(msgs) = messages {
        app.agent_messages = msgs;
    }
    // ... rest unchanged
}
```

- [ ] **Step 6: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors, 0 warnings

- [ ] **Step 7: Commit**

```
fix(code): TUI 多轮对话 — Agent messages 跨轮持久化
```

---

### Task 2: Bash 工具放行编译命令

**Problem:** `crates/code/src/tools/bash.rs:33` 把 `curl`, `wget`, `python`, `ruby`, `perl`, `node -e`, `bash -c`, `sh -c`, `eval`, `exec`, `source` 全杀了。`cargo build/test/check` 碰到 `sh -c` 前缀被误杀（因为 `sh -c cmd` 是 bash tool 自己的调用方式，但 `cmd` 内容以 `cargo` 等开头不应被杀）。

**Files:**
- Modify: `crates/code/src/tools/bash.rs`

- [ ] **Step 1: 改为 allowlist 模式**

将 blocked 列表改为 allowlist + 真正危险的命令黑名单：

```rust
async fn call(&self, args: &Map<String, Value>) -> ToolResult {
    let cmd = args.get("command").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("command required"))?;
    let desc = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

    let blocked_patterns = [
        "rm -rf /",
        "mkfs",
        "dd if=",
        ":(){ :|:& };:",
        "> /dev/sd",
        "chmod -R 777 /",
    ];
    for b in &blocked_patterns {
        if cmd.contains(b) {
            anyhow::bail!("Command contains dangerous pattern: {}", b);
        }
    }

    let output = tokio::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = format!("$ {}\n{}\n", desc, cmd);
    if !stdout.is_empty() {
        result.push_str(&format!("stdout:\n{}", stdout));
    }
    if !stderr.is_empty() {
        result.push_str(&format!("stderr:\n{}", stderr));
    }
    if !output.status.success() {
        result.push_str(&format!("exit code: {}", output.status.code().unwrap_or(-1)));
    }
    Ok(result)
}
```

- [ ] **Step 2: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 3: Commit**

```
fix(code): bash 工具 — 改为危险模式黑名单，放行 cargo/python 等编译命令
```

---

### Task 3: Read 工具支持 offset/limit

**Problem:** `crates/code/src/tools/filesystem.rs:41-51` 一次读取整个文件。大文件吃光 context window。

**Files:**
- Modify: `crates/code/src/tools/filesystem.rs`

- [ ] **Step 1: 添加 offset 和 limit 参数**

```rust
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read a file with line numbers. Use offset and limit for large files." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a file with line numbers. Use offset and limit for large files.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the file"},
                        "offset": {"type": "integer", "description": "Line number to start reading from (1-indexed, default: 1)"},
                        "limit": {"type": "integer", "description": "Max number of lines to read (default: 200)"}
                    },
                    "required": ["file_path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        check_path(path)?;
        let content = std::fs::read_to_string(path)?;

        let total_lines = content.lines().count();
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

        let lines: Vec<&str> = content.lines().collect();
        let end = (offset + limit - 1).min(lines.len());
        let selected: Vec<&str> = lines[(offset - 1)..end].to_vec();

        let max_digits = end.to_string().len();
        let numbered: Vec<String> = selected.iter().enumerate()
            .map(|(i, l)| format!("{:>width$}: {}", offset + i, l, width = max_digits))
            .collect();

        let header = if offset > 1 || end < total_lines {
            format!("{} (lines {}-{} of {})\n```\n{}\n```", path, offset, end, total_lines, numbered.join("\n"))
        } else {
            format!("{}\n```\n{}\n```", path, numbered.join("\n"))
        };
        Ok(header)
    }
}
```

- [ ] **Step 2: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 3: Commit**

```
feat(code): read 工具 — 支持 offset/limit 分页读取大文件
```

---

### Task 4: ContextManager 集成到 ReAct 循环

**Problem:** `crates/code/src/agent/context.rs` 的 `compress()` 从未在 engine 中调用。长对话超 token 时直接报错。

**Files:**
- Modify: `crates/code/src/agent/context.rs`
- Modify: `crates/code/src/agent/engine.rs`

- [ ] **Step 1: 改进 ContextManager 使用字符估算 token**

修改 `crates/code/src/agent/context.rs`，4 字符 ≈ 1 token 的粗略估算：

```rust
pub struct ContextManager {
    max_tokens: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { max_tokens: 128_000 }
    }

    pub fn with_max_tokens(max: usize) -> Self {
        Self { max_tokens: max }
    }

    fn estimate_tokens(messages: &[LlmMessage]) -> usize {
        let total_chars: usize = messages.iter().map(|m| match m {
            LlmMessage::System(s) => s.len(),
            LlmMessage::User(s) => s.len(),
            LlmMessage::Assistant(s) => s.len(),
            LlmMessage::Tool { content, .. } => content.len(),
            LlmMessage::ToolCall { args, .. } => args.to_string().len(),
        }).sum();
        (total_chars + 3) / 4
    }

    pub fn should_compress(&self, messages: &[LlmMessage]) -> bool {
        Self::estimate_tokens(messages) > self.max_tokens * 80 / 100
    }

    pub fn compress(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        if !self.should_compress(messages) {
            return messages.to_vec();
        }

        let mut compressed = Vec::new();
        let mut tool_call_pairs: Vec<(LlmMessage, LlmMessage)> = Vec::new();

        for (i, msg) in messages.iter().enumerate() {
            match msg {
                LlmMessage::System(s) => {
                    compressed.push(LlmMessage::System(s.clone()));
                }
                LlmMessage::ToolCall { .. } => {
                    if let Some(next) = messages.get(i + 1) {
                        if let LlmMessage::Tool { .. } = next {
                            tool_call_pairs.push((msg.clone(), next.clone()));
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        let summary = if !tool_call_pairs.is_empty() {
            let count = tool_call_pairs.len();
            let last_few: Vec<LlmMessage> = tool_call_pairs.drain(..tool_call_pairs.len().saturating_sub(3))
                .flat_map(|(call, result)| vec![call, result]).collect();
            let truncated_pairs: Vec<LlmMessage> = tool_call_pairs.into_iter()
                .flat_map(|(call, result)| {
                    let truncated_result = LlmMessage::Tool {
                        name: match &result {
                            LlmMessage::Tool { name, .. } => name.clone(),
                            _ => String::new(),
                        },
                        content: "[compressed]".to_string(),
                        call_id: match &result {
                            LlmMessage::Tool { call_id, .. } => call_id.clone(),
                            _ => String::new(),
                        },
                    };
                    vec![call, truncated_result]
                }).collect();

            let mut msg = vec![
                LlmMessage::System("[Context compressed]".to_string()),
                LlmMessage::Assistant(format!("(Previous conversation had {} tool calls, compressed for context. Recent calls preserved.)", count)),
            ];
            msg.extend(truncated_pairs);
            msg.extend(last_few);
            msg
        } else {
            messages.to_vec()
        };

        summary
    }
}
```

- [ ] **Step 2: 在 ReAct 循环中调用 compress**

修改 `crates/code/src/agent/engine.rs`，在两个循环的每轮开始前调用 compress：

在 `react_loop` 和 `react_loop_streaming` 的 `for _round in 0..max_rounds` 循环体开头：

```rust
let ctx = crate::agent::context::ContextManager::new();
if ctx.should_compress(&messages) {
    messages = ctx.compress(&messages);
}
```

注意：`messages` 参数需要从 `Vec<LlmMessage>` 改为 `mut`（已经是 `mut` 了）。

- [ ] **Step 3: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 4: Commit**

```
feat(code): ContextManager 集成到 ReAct 循环 — 自动压缩长对话
```

---

### Task 5: 系统提示词增强

**Problem:** `crates/code/src/prompt.rs` 只有 41 行，缺少项目感知、Rust 工作流、错误修复模式等关键指导。

**Files:**
- Modify: `crates/code/src/prompt.rs`

- [ ] **Step 1: 重写系统提示词**

```rust
pub const SYSTEM: &str = "\
You are i-rs-code, an expert coding AI agent.

## CRITICAL RULE — You MUST use tools
You are a tool-using AI. Every action goes through a tool call. If you only respond with text, nothing happens.

Available tools: read, write, edit, grep, glob, ls, bash, git, web_fetch, web_search.

## Workflow
1. UNDERSTAND — use read/glob/grep/ls to understand current code
2. PLAN — think about what changes are needed, which files are affected
3. EXECUTE — use write/edit/bash to make changes
4. VERIFY — use `bash cargo check` to verify compilation
5. FIX — if errors, read the error output and fix iteratively

## Tool Usage Patterns
- `read <path> [offset] [limit]` — read file contents (use offset/limit for large files)
- `write <path> <content>` — create or overwrite a file
- `edit <path> <old_string> <new_string>` — surgical edit (preferred over write)
- `grep <pattern> [path] [include]` — search file contents with regex
- `glob <pattern> [path]` — find files by pattern
- `ls [path]` — list directory
- `bash <command> <description>` — run shell command

## Rust Project Rules
- After any code change, run `cargo check` to verify
- Run `cargo clippy -- -D warnings` if available
- Run `cargo test` to verify tests pass
- Read `Cargo.toml` to understand dependencies before adding new ones
- Follow existing code patterns and style in the project
- Workspace: use `--workspace` or `-p <package>` flags as needed

## Multi-File Changes
- Change interfaces first (struct/trait/enum), then update implementations
- Read all affected files before making changes
- Use `grep` to find all references before renaming
- Verify each file compiles before moving to the next

## Error Recovery
- Read compiler errors carefully, they tell you exactly what's wrong
- Fix one error at a time, re-check after each fix
- If a tool fails, read the error and try a different approach
- Use `bash git diff` to review your changes before committing

## NEVER
- Do NOT say \"let me\" or \"I'll\" — call the tool directly
- Do NOT respond with text when you should be using a tool
- Do NOT ask the user to run commands — use bash yourself
- Do NOT explain what you're about to do — just do it
- Do NOT read entire large files when offset/limit would suffice

After completing changes, briefly summarize what was done in Chinese.";
```

- [ ] **Step 2: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 3: Commit**

```
feat(code): 增强系统提示词 — 添加 Rust 工作流、错误恢复、多文件修改指导
```

---

### Task 6: cargo clippy 全量通过

**Files:**
- Modify: `crates/code/src/app.rs`
- Modify: `crates/code/src/session.rs`
- Modify: `crates/code/src/tools/web.rs`
- Modify: `crates/code/src/protocol/handler.rs`
- Modify: `crates/code/src/main.rs`
- Modify: `crates/code/src/tui.rs`

- [ ] **Step 1: 修复 app.rs**

`TokenUsage` 加 `#[derive(Default)]` 并删除手写 impl：
```rust
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
}
```
删除 `impl Default for TokenUsage { ... }` 块（行 16-23）。

修复 `finish_streaming` 的 let-and-return：
```rust
pub fn finish_streaming(&mut self) -> String {
    self.streaming.take().map(|s| s.content).unwrap_or_default()
}
```

- [ ] **Step 2: 修复 session.rs**

合并嵌套 if：
```rust
if entry.file_type()?.is_file()
    && let Some(name) = entry.file_name().to_str()
    && let Some(id) = name.strip_suffix(".json") {
        ids.push(id.to_string());
    }
```

- [ ] **Step 3: 修复 tools/web.rs**

将 regex 构建移到循环外：
```rust
let re = regex::Regex::new(r###"<a[^>]*class=\"result__a\"[^>]*>(.*?)</a>"###).unwrap();
let strip_re = regex::Regex::new("<[^>]*>").unwrap();
for cap in re.captures_iter(&html) {
    // use strip_re instead of building inline
}
```

- [ ] **Step 4: 修复 main.rs**

将 `map_or(true, |k| k.trim().is_empty())` 改为 `is_none_or(|k| k.trim().is_empty())`（3 处）。

- [ ] **Step 5: 修复 protocol/handler.rs**

合并嵌套 if-let 链。

- [ ] **Step 6: 修复 tui.rs**

合并可合并的 if 语句和 match guard。

- [ ] **Step 7: Verify**

Run: `cargo clippy -p i-rs-code -- -D warnings`
Expected: 0 errors

- [ ] **Step 8: Commit**

```
chore(code): 修复全部 clippy warnings
```
