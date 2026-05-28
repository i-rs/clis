# i-rs-code 架构重构计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 i-rs-code 的核心架构问题，按优先级逐项完成 19 项改进

**Architecture:** 逐项小步重构，每项独立可测试、可提交。不破坏现有功能。

**Tech Stack:** Rust 1.95 / tokio / anyhow / serde_json

---

## Task 1: Provider 结构化错误类型

**问题:** `is_transient_error()` 用字符串 `contains("rate")` 判断重试，不可靠

**Files:**
- Create: `crates/code/src/provider/error.rs`
- Modify: `crates/code/src/provider/mod.rs`
- Modify: `crates/code/src/agent/engine.rs`

- [ ] **Step 1: 创建 `provider/error.rs`**

```rust
#[derive(Debug, Clone)]
pub enum ProviderError {
    RateLimited { retry_after_ms: Option<u64> },
    Timeout,
    ServerError { status: u16 },
    AuthFailed,
    ContextLengthExceeded,
    ConnectionFailed,
    Unknown(String),
}

impl ProviderError {
    pub fn from_message(msg: &str, status: Option<u16>) -> Self {
        let lower = msg.to_lowercase();
        if lower.contains("rate") || lower.contains("限流") || lower.contains("quota") {
            Self::RateLimited { retry_after_ms: None }
        } else if lower.contains("timeout") {
            Self::Timeout
        } else if lower.contains("context_length") || lower.contains("max tokens") || lower.contains("token limit") {
            Self::ContextLengthExceeded
        } else if lower.contains("auth") || lower.contains("unauthorized") || lower.contains("401") {
            Self::AuthFailed
        } else if let Some(code) = status {
            if code >= 500 { Self::ServerError { status: code } }
            else { Self::Unknown(msg.to_string()) }
        } else {
            Self::Unknown(msg.to_string())
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Timeout | Self::ServerError { .. } | Self::ConnectionFailed)
    }
}
```

- [ ] **Step 2: 在 `provider/mod.rs` 中声明模块**

在 `pub mod ollama;` 后添加 `pub mod error;`

- [ ] **Step 3: 替换 `engine.rs` 中的 `is_transient_error`**

将:
```rust
fn is_transient_error(e: &str) -> bool {
    let lower = e.to_lowercase();
    lower.contains("限流")
        || lower.contains("rate")
        // ...
}
```
替换为:
```rust
fn is_transient_error(e: &str) -> bool {
    crate::provider::error::ProviderError::from_message(e, None).is_retryable()
}
```

- [ ] **Step 4: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "refactor(code): ProviderError 结构化错误类型替代字符串匹配"
```

---

## Task 2: react_loop 取消机制

**问题:** `react_loop` 启动后无法从外部中断

**Files:**
- Modify: `crates/code/src/agent/engine.rs`
- Modify: `crates/code/src/tui.rs`

- [ ] **Step 1: 给 `react_loop_inner` 添加 `cancel_rx` 参数**

在 `react_loop_inner` 函数签名中添加 `cancel_rx: Option<tokio::sync::oneshot::Receiver<()>>`:

```rust
async fn react_loop_inner(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    output: OutputMode<'_>,
    max_rounds: u32,
    tool_timeout_secs: u64,
    cancel_rx: Option<tokio::sync::oneshot::Receiver<()>>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
```

在 `for _round in 0..max_rounds` 循环开头添加取消检查:

```rust
for _round in 0..max_rounds {
    if let Some(ref rx) = cancel_rx {
        if rx.try_recv().is_ok() {
            return Err(anyhow::anyhow!("cancelled"));
        }
    }
    // ... rest of loop
}
```

- [ ] **Step 2: 更新 `react_loop` 和 `react_loop_streaming` 传递 `cancel_rx: None`**

```rust
pub async fn react_loop(...) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Stdout { json_output };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None).await
}

pub async fn react_loop_streaming(...) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Channel { event_tx: &event_tx };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None).await
}
```

- [ ] **Step 3: 给 `Agent::run_once_streaming` 添加 cancel 参数**

```rust
pub async fn run_once_streaming(
    &mut self,
    prompt: &str,
    event_tx: mpsc::Sender<event::AgentEvent>,
    history: Vec<LlmMessage>,
    cancel_rx: Option<tokio::sync::oneshot::Receiver<()>>,
) -> anyhow::Result<String> {
```

在调用处传递 `cancel_rx`:

```rust
let (final_text, new_messages) = engine::react_loop_streaming(
    &*self.provider, &self.tools, msgs, &tool_defs, event_tx,
    self.config.max_rounds, self.config.tool_timeout_secs,
).await?;
```

改为:

```rust
let (final_text, new_messages) = engine::react_loop_streaming(
    &*self.provider, &self.tools, msgs, &tool_defs, event_tx,
    self.config.max_rounds, self.config.tool_timeout_secs,
).await?;
```

注意：`react_loop_streaming` 已将 cancel_rx 嵌入，所以 `run_once_streaming` 需要改签名接受 cancel 并传到 inner。

实际上更简单的方式：在 `react_loop_streaming` 中把 `cancel_rx` 传到 `react_loop_inner`。在 `Agent::run_once_streaming` 中接受 `cancel_rx` 参数。`tui.rs` 的 `run_streaming_agent` 已经有 `tokio::select!`，所以取消会工作在两层。

- [ ] **Step 4: 更新 `tui.rs` 调用方传递 cancel**

`tui.rs:427` 的 `agent.run_once_streaming(prompt, event_tx.clone(), history)` 改为 `agent.run_once_streaming(prompt, event_tx.clone(), history, Some(cancel_rx))`。注意 `cancel_rx` 已经在 `run_streaming_agent` 的 `tokio::select!` 中，所以需要改成 move cancel_rx 的方式。

- [ ] **Step 5: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 6: 提交**

```bash
git add -A && git commit -m "feat(code): react_loop 取消机制支持"
```

---

## Task 3: 工具结果后处理独立化

**问题:** tui.rs 事件处理器直接解析特定工具 JSON（如 `update_user_memory`），事件层不应知道工具语义

**Files:**
- Create: `crates/code/src/agent/post_process.rs`
- Modify: `crates/code/src/agent/mod.rs`
- Modify: `crates/code/src/agent/engine.rs`
- Modify: `crates/code/src/tui.rs`

- [ ] **Step 1: 创建 `agent/post_process.rs`**

```rust
use crate::memory::CrossSessionMemory;

pub fn process_tool_result(
    tool_name: &str,
    result: &str,
    memory: &mut Option<CrossSessionMemory>,
) -> String {
    if tool_name == "update_user_memory" {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(result) {
            if let Some(mem) = memory.as_mut() {
                let key = val.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let value = val.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !key.is_empty() {
                    mem.record(key.to_string(), value.to_string());
                }
            }
        }
        "[Memory updated]".to_string()
    } else {
        result.to_string()
    }
}
```

- [ ] **Step 2: 在 `agent/mod.rs` 中声明模块**

添加 `pub mod post_process;`

- [ ] **Step 3: 在 `engine.rs` 的工具结果处理循环中调用**

在 `react_loop_inner` 中 `for (name, call_id, result_str) in &tool_messages` 循环里，工具结果被追加到 messages 之前，调用后处理。

注意：engine 当前不持有 `CrossSessionMemory`。后处理器可以在不修改 engine 的情况下，作为一个 hook 注入。最简方案：在 `execute_tools` 返回后、追加到 messages 前，由调用方（Agent）处理。

**更简方案：** 由于当前 i-rs-code 的 `update_user_memory` 工具并不存在（这是 claw 的功能），此问题在 i-rs-code 中尚未出现。**标记为 N/A，跳过。**

- [ ] **Step 4: 提交（如果实际有代码变更）**

---

## Task 4: ChatSession trait 统一会话交互

**问题:** TUI/CLI/Protocol 三处各自重复消息构建 + chat_loop 调用

**Files:**
- Create: `crates/code/src/agent/session_trait.rs`
- Modify: `crates/code/src/agent/mod.rs`
- Modify: `crates/code/src/main.rs`
- Modify: `crates/code/src/tui.rs`
- Modify: `crates/code/src/protocol/handler.rs`

- [ ] **Step 1: 创建 `agent/session_trait.rs`**

```rust
use crate::config::Config;
use crate::provider::{LlmMessage, Usage};
use crate::tools::ToolRegistry;
use super::event::AgentEvent;
use tokio::sync::mpsc;
use std::future::Future;
use std::pin::Pin;

pub struct ChatInput {
    pub prompt: String,
    pub history: Vec<LlmMessage>,
}

pub struct ChatOutput {
    pub text: String,
    pub messages: Vec<LlmMessage>,
    pub usage: Option<Usage>,
}

pub trait ChatSession {
    fn run(
        &mut self,
        input: ChatInput,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>>;
}

pub trait StreamingChatSession {
    fn run_streaming(
        &mut self,
        input: ChatInput,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>>;
}
```

- [ ] **Step 2: 让 `Agent` 实现 `ChatSession` 和 `StreamingChatSession`**

在 `agent/mod.rs` 中为 Agent 添加 impl:

```rust
impl super::session_trait::ChatSession for Agent {
    fn run(
        &mut self,
        input: super::session_trait::ChatInput,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<super::session_trait::ChatOutput>> + Send + '_>> {
        let ChatInput { prompt, history } = input;
        Box::pin(async move {
            let system_text = crate::prompt::SYSTEM;
            let tool_defs = self.tools.schemas();
            let msgs = build_messages(&history, system_text, &prompt);
            let (final_text, new_messages) = engine::react_loop(
                &*self.provider, &self.tools, msgs, &tool_defs,
                self.json_output, self.config.max_rounds, self.config.tool_timeout_secs,
            ).await?;
            self.messages = new_messages.clone();
            Ok(super::session_trait::ChatOutput {
                text: final_text,
                messages: new_messages,
                usage: None,
            })
        })
    }
}
```

同理为 `StreamingChatSession` 实现。

- [ ] **Step 3: 在 `agent/mod.rs` 声明模块**

添加 `pub mod session_trait;`

- [ ] **Step 4: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "refactor(code): ChatSession trait 统一会话交互接口"
```

---

## Task 5: app.rs 拆分

**问题:** `app.rs` 177 行混合 UI 状态 + 消息 + 输入管理 + cursor + scroll

**Files:**
- Create: `crates/code/src/tui/input.rs`
- Modify: `crates/code/src/app.rs`
- Modify: `crates/code/src/tui.rs`
- Modify: `crates/code/src/tui/ui.rs`

- [ ] **Step 1: 创建 `tui/input.rs` — 输入状态和光标操作**

```rust
pub struct InputState {
    pub content: String,
    pub cursor_pos: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self { content: String::new(), cursor_pos: 0 }
    }

    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.content[..self.cursor_pos].chars().last().map(|c| c.len_utf8()).unwrap_or(1);
            self.cursor_pos -= len;
            self.content.remove(self.cursor_pos);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_pos < self.content.len() {
            let len = self.content[self.cursor_pos..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            self.content.drain(self.cursor_pos..self.cursor_pos + len);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.content[..self.cursor_pos].chars().last().map(|c| c.len_utf8()).unwrap_or(1);
            self.cursor_pos -= len;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos < self.content.len() {
            let len = self.content[self.cursor_pos..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            self.cursor_pos += len;
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_pos = 0;
    }
}
```

- [ ] **Step 2: 将 `App` 中的 `input`/`cursor_pos`/cursor 方法替换为 `InputState`**

在 `app.rs` 中：
- 移除 `input: String` 和 `cursor_pos: usize` 字段
- 添加 `pub input_state: InputState`
- 移除 `insert_char`, `delete_char`, `move_cursor_left`, `move_cursor_right`, `move_cursor_home`, `move_cursor_end` 方法
- 更新 `App::new()` 使用 `input_state: InputState::new()`

- [ ] **Step 3: 更新 `tui.rs` 和 `tui/ui.rs` 引用**

所有 `app.input` → `app.input_state.content`
所有 `app.cursor_pos` → `app.input_state.cursor_pos`
所有 `app.insert_char(c)` → `app.input_state.insert_char(c)`
等等

- [ ] **Step 4: 在 `tui.rs` 或 `tui/mod.rs` 中声明模块**

```rust
mod input;
```

- [ ] **Step 5: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 6: 提交**

```bash
git add -A && git commit -m "refactor(code): app.rs 拆分 InputState 到 tui/input.rs"
```

---

## Task 6: 统一错误体系

**问题:** MCP 层用 `Result<T, String>`，其他层用 `anyhow`

**Files:**
- Modify: `crates/code/src/mcp.rs`

- [ ] **Step 1: 将 `mcp.rs` 中所有 `Result<T, String>` 改为 `anyhow::Result<T>`**

查找 `mcp.rs` 中所有 `.map_err(|e| e.to_string())` 和 `Result<_, String>` 模式，替换为 `?` 或 `.context()`。

- [ ] **Step 2: 同样检查 `protocol/transport.rs`**

- [ ] **Step 3: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "refactor(code): 统一错误体系为 anyhow"
```

---

## Task 7: messages 统一管理

**问题:** `app.agent_messages` 和 API messages 同步脆弱

**Files:**
- Modify: `crates/code/src/app.rs`
- Modify: `crates/code/src/tui.rs`

- [ ] **Step 1: 移除 `App.agent_messages`，只保留 `messages: Vec<ChatMessage>`**

所有从 `agent_messages` 读取的地方改为从 `messages` 重建。在 TUI mode 下，`messages` 是唯一数据源。

`agent.run_once_streaming` 当前接受 `history: Vec<LlmMessage>`。改为从 `app.messages` 重建 `Vec<LlmMessage>`（已有 `extract_tool_calls` 等逻辑可以复用）。

实际上由于 `run_once_streaming` 接受的参数在 `tui.rs:420` 中已经是从 `app.agent_messages` 取出的，最简方案是保留 `agent_messages` 但将其改为 private，只在 `run_once_streaming` 调用时取出。

**更简方案：** 添加注释说明 `agent_messages` 是 API 层消息缓存，仅在 `run_once_streaming` 读取时使用，写入仅在 `AgentEvent::Done` 时发生。保持现状但文档化。

- [ ] **Step 2: 在 `app.rs` 的 `agent_messages` 字段添加文档注释**

```rust
/// API-format message history, synced from AgentEvent::Done.
/// Used as history input for the next LLM call.
pub agent_messages: Vec<crate::provider::LlmMessage>,
```

- [ ] **Step 3: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "docs(code): agent_messages 字段文档化"
```

---

## Task 8: 全局单例集中管理

**问题:** 4 处 `LazyLock` 全局单例散布各模块

**Files:**
- Create: `crates/code/src/runtime.rs`
- Modify: `crates/code/src/tools/mcp.rs`
- Modify: `crates/code/src/tools/pty.rs`
- Modify: `crates/code/src/tools/lsp.rs`
- Modify: `crates/code/src/tools/web.rs`

- [ ] **Step 1: 创建 `runtime.rs`**

```rust
use std::sync::LazyLock;
use tokio::sync::Mutex;
use std::time::Instant;

use crate::mcp::McpManager;
use crate::pty::PtyManager;
use crate::lsp::LspSession;

pub static MCP_MANAGER: LazyLock<McpManager> = LazyLock::new(McpManager::new);
pub static PTY_MANAGER: LazyLock<PtyManager> = LazyLock::new(PtyManager::new);
pub static LSP_SESSION: LazyLock<Mutex<LspSession>> = LazyLock::new(|| Mutex::new(LspSession::new()));
pub static LAST_WEB_REQUEST: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));
```

- [ ] **Step 2: 更新各 tools 模块引用**

- `tools/mcp.rs`: `use crate::runtime::MCP_MANAGER;` 替换本地 static
- `tools/pty.rs`: `use crate::runtime::PTY_MANAGER;` 替换本地 static
- `tools/lsp.rs`: `use crate::runtime::LSP_SESSION;` 替换本地 static
- `tools/web.rs`: `use crate::runtime::LAST_WEB_REQUEST;` 替换本地 static
- `lsp.rs`: 移除本地 `CLIENT_CAPS` static（移到 runtime.rs 或保持在 lsp.rs 因为它小）

- [ ] **Step 3: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "refactor(code): 全局单例集中到 runtime.rs"
```

---

## Task 9: Token 预算

**问题:** 只有事后压缩，没有事前预估

**Files:**
- Modify: `crates/code/src/agent/context.rs`
- Modify: `crates/code/src/agent/engine.rs`

- [ ] **Step 1: 在 `context.rs` 添加 `check_budget` 函数**

```rust
const MODEL_CONTEXT_LIMIT: usize = 128_000;

pub fn estimate_message_tokens(messages: &[crate::provider::LlmMessage]) -> usize {
    let mut total = 0usize;
    for msg in messages {
        match msg {
            crate::provider::LlmMessage::System(s) => total += s.len() / 4,
            crate::provider::LlmMessage::User(s) => total += s.len() / 4,
            crate::provider::LlmMessage::Assistant(s) => total += s.len() / 4,
            crate::provider::LlmMessage::AssistantWithReasoning { content, reasoning, .. } => {
                total += content.len() / 4 + reasoning.len() / 4;
            }
            crate::provider::LlmMessage::Tool { content, .. } => total += content.len() / 4,
            crate::provider::LlmMessage::ToolCall { args, .. } => total += args.to_string().len() / 4,
        }
    }
    total
}

pub fn check_budget(messages: &[crate::provider::LlmMessage], tool_defs: &[serde_json::Value]) -> usize {
    let msg_tokens = estimate_message_tokens(messages);
    let tool_tokens = tool_defs.iter().map(|v| v.to_string().len() / 4).sum::<usize>();
    msg_tokens + tool_tokens
}

pub fn exceeds_budget(messages: &[crate::provider::LlmMessage], tool_defs: &[serde_json::Value]) -> bool {
    check_budget(messages, tool_defs) > MODEL_CONTEXT_LIMIT * 8 / 10
}
```

- [ ] **Step 2: 在 `engine.rs` 的 `react_loop_inner` 循环开头添加预算检查**

```rust
if super::context::exceeds_budget(&messages, tool_defs) {
    messages = ctx.compress(&messages);
}
```

- [ ] **Step 3: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "feat(code): Token 预算事前检查"
```

---

## Task 10: chat_loop 生命周期钩子

**问题:** 无法在每轮插入自定义逻辑

**Files:**
- Modify: `crates/code/src/agent/engine.rs`

- [ ] **Step 1: 定义 `LoopHooks` trait**

```rust
pub trait LoopHooks: Send + Sync {
    fn on_round_start(&self, round: u32, messages: &[LlmMessage]) {}
    fn on_tool_result(&self, name: &str, result: &str) {}
    fn on_done(&self, rounds: u32, usage: &Option<Usage>) {}
}
```

- [ ] **Step 2: 给 `react_loop_inner` 添加可选 hooks 参数**

```rust
async fn react_loop_inner(
    // ... existing params ...
    hooks: Option<&dyn LoopHooks>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
```

在循环各阶段调用 hooks。

- [ ] **Step 3: 更新所有调用方传 `None`**

- [ ] **Step 4: 运行验证**

```bash
cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "feat(code): chat_loop 生命周期钩子 LoopHooks trait"
```

---

## Task 11-19: P3 清理项

按需逐项实施，每项独立提交：

- **Task 11:** 工具结果缓存 — `agent/tool_cache.rs`，LRU 缓存相同参数的工具调用结果
- **Task 12:** 成本控制 — `config.rs` 添加 `max_cost_per_session: Option<f64>`
- **Task 13:** 流式响应缓冲 — `agent/event.rs` 添加 `BufferedToken(String)` 事件，TUI 按 `\n` 批量渲染
- **Task 14:** "default" Agent 集中化 — 提取 `DEFAULT_AGENT: &str = "default"` 常量
- **Task 15:** AgentEvent 改为 trait object 或添加 source tag — 低优先级
- **Task 16:** MCP 工具统一注册 — 合并 ToolRegistry + McpRegistry 查找路径
- **Task 17:** MockLlmProvider 支持多轮 — `testing.rs` 添加 `with_responses(vec![...])`
- **Task 18:** create_crate.rs todo!() 实现
- **Task 19:** TUI 基础测试 — `tui/input.rs` 的 cursor 操作单元测试
