# i-rs-code Phase 4.1: 安全沙箱 + 测试覆盖 + 代码清理 + 文件工具

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 补齐 i-rs-code 的基础安全防护、测试基础设施、代码质量和文件工具，作为 Phase 4 的第一阶段。

**Architecture:** 4 组独立修改：(1) 安全沙箱强化 Bash/文件工具的路径限制 (2) 创建 MockLlmProvider + MockTool 等测试基础设施 (3) 代码质量清理 (4) 新增 DeleteTool/RenameTool 替代 bash rm/mv。

**Tech Stack:** Rust, tokio, tower-lsp (测试 mock), serde_json, anyhow

---

## 文件变更总览

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/tools/bash.rs` | 修改 | 增加 workspace chdir 强制、扩展 blocklist |
| `src/tools/filesystem.rs` | 修改 | `check_path` 强化 + `delete_entry`/`rename_entry` 辅助 |
| `src/tools/delete.rs` | 新建 | `DeleteTool` |
| `src/tools/rename.rs` | 新建 | `RenameTool` |
| `src/tools/mod.rs` | 修改 | 注册 DeleteTool + RenameTool |
| `src/testing.rs` | 新建 | MockLlmProvider、MockTool、辅助函数 |
| `src/agent/engine.rs` | 修改 | 加测试（使用 mock） |
| `src/agent/event.rs` | 修改 | 移除 `#[allow(dead_code)]` |
| `src/main.rs` | 修改 | 移除 `#![allow(dead_code)]` |
| `src/provider/mod.rs` | 修改 | 公开 `Usage`/`ToolCall` 等类型（若尚未公开） |
| `Cargo.toml` | 修改 | 新增 `tempfile` dev-dependency |
| `src/agent/context.rs` | 修改 | 增加测试 |
| `src/debug.rs` | 修改 | 增加测试 |
| `src/diff.rs` | 修改 | 增加测试 |

---

### Task 1: 代码质量清理

**Files:**
- Modify: `src/main.rs:1`
- Modify: `src/agent/event.rs:9`
- Modify: `src/agent/engine.rs:10`
- Modify: `src/protocol/handler.rs:6`

- [ ] **Step 1: 移除 main.rs 的 `#![allow(dead_code)]`**

在 `src/main.rs` 移除第 1 行的 `#![allow(dead_code)]`。

```rust
// 从:
#![allow(dead_code)]
// 改为:
// (删除该行)
```

- [ ] **Step 2: 移除 agent/event.rs 的 `#[allow(dead_code)]`**

在 `src/agent/event.rs` 移除 `FileChanged` 上的 `#[allow(dead_code)]`。然后在 `src/tui.rs` 的 `handle_event` 函数中，在 `AgentEvent::FileChanged` 分支里真的使用该变体（比如打印到 debug log）：

```rust
// event.rs 中保留 FileChanged 但不加 allow(dead_code)
// 然后在 tui.rs handle_event 里:
AgentEvent::FileChanged { path } => {
    app.file_changes.insert(path);
    crate::debug::push_log(crate::debug::HttpLogEntry {
        url: "internal".into(),
        request_body: format!("FileChanged: {}", path),
        response_status: 200,
        response_body_preview: String::new(),
        duration_ms: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    });
}
```

- [ ] **Step 3: 移除未使用的 `DEFAULT_TOOL_TIMEOUT_SECS`**

在 `src/agent/engine.rs` 删除第 10 行：

```rust
// 删除这行:
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 120;
```

- [ ] **Step 4: 修复 `protocol/handler.rs` 的 `unsafe` env var**

```rust
// 从:
unsafe { std::env::set_var("I_RS_CODE_AGENT_MODE", "1"); }
// 改为（Rust 1.66+ 不是 unsafe）:
std::env::set_var("I_RS_CODE_AGENT_MODE", "1");
```

- [ ] **Step 5: 验证编译**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```
Expected: 编译通过 + clippy 无警告 + 21 测试通过。

- [ ] **Step 6: 提交**

```bash
git add crates/code/src/main.rs crates/code/src/agent/event.rs crates/code/src/agent/engine.rs crates/code/src/protocol/handler.rs crates/code/src/tui.rs
git commit -m "refactor(code): 清理 dead_code/unsafe/未使用常量"
```

---

### Task 2: 安全沙箱 — BashTool 强化

**Files:**
- Modify: `src/tools/bash.rs:28-65`

- [ ] **Step 1: 读取当前 BashTool 实现**

```bash
cat src/tools/bash.rs
```

- [ ] **Step 2: 扩展危险命令 blocklist**

在 `src/tools/bash.rs` 的 `blocked_patterns` 数组增加：

```rust
let blocked_patterns = [
    "rm -rf /",
    "rm -rf --no-preserve-root /",
    "rm -rf /*",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",
    "> /dev/sd",
    "chmod -R 777 /",
    "chmod 777 /",
    "sudo ",
    "wget -O /",
    "curl -o /",
    ">|",
    "echo '",
];
```

- [ ] **Step 3: 增加 workspace 目录约束**

在 `BashTool::call` 中，在执行命令前注入 `cd` 约束。从工作目录 `std::env::current_dir()` 获取 workspace，前置 `cd`：

```rust
async fn call(&self, args: &Map<String, Value>) -> ToolResult {
    let cmd = args.get("command").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("command required"))?;
    let desc = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

    // 安全检查
    let blocked_patterns = [ /* ...同上... */ ];
    for b in &blocked_patterns {
        if cmd.contains(b) {
            anyhow::bail!("Command contains dangerous pattern: {}", b);
        }
    }

    // 强制在 workspace 内执行
    let cwd = std::env::current_dir()?;
    let cmd = format!("cd {} && {}", cwd.to_string_lossy(), cmd);

    let output = tokio::process::Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .await?;
    // ...rest unchanged
}
```

- [ ] **Step 4: 更新测试覆盖新的 blocklist 项**

在 `src/tools/bash.rs` 的测试模块，更新 `blocked_patterns()` 函数和测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn blocked_patterns() -> Vec<&'static str> {
        vec![
            "rm -rf /", "rm -rf --no-preserve-root /", "rm -rf /*",
            "mkfs", "dd if=", ":(){ :|:& };:", "> /dev/sd",
            "chmod -R 777 /", "chmod 777 /", "sudo ", "wget -O /",
            "curl -o /",
        ]
    }

    #[test]
    fn test_safe_commands_not_blocked() {
        let safe = vec!["cargo check", "python main.py", "git status", "ls -la", "npm test", "echo hello"];
        let patterns = blocked_patterns();
        for cmd in &safe {
            assert!(!patterns.iter().any(|p| cmd.contains(p)), "cmd '{}' should not be blocked", cmd);
        }
    }

    #[test]
    fn test_dangerous_commands_blocked() {
        let patterns = blocked_patterns();
        let dangerous = vec![
            "rm -rf /", "rm -rf --no-preserve-root /", "rm -rf /*",
            "mkfs.ext4 /dev/sda1", "dd if=/dev/zero",
            ":(){ :|:& };:", "echo test > /dev/sda",
            "sudo apt install", "wget -O /tmp/test http://example.com",
            "chmod -R 777 /", "chmod 777 /etc",
        ];
        for cmd in &dangerous {
            assert!(patterns.iter().any(|p| cmd.contains(p)), "cmd '{}' should be blocked", cmd);
        }
    }

    #[test]
    fn test_bash_contains_blocklist() {
        // 验证 blocklist 包含要求的新模式
        let patterns = blocked_patterns();
        assert!(patterns.contains(&"sudo "), "sudo must be blocked");
        assert!(patterns.contains(&"rm -rf /*"), "rm -rf /* must be blocked");
    }
}
```

- [ ] **Step 5: 验证**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code -p i-rs-code tools::bash::tests
```
Expected: 所有 bash 测试通过（包含新增的 blocklist 测试）。

- [ ] **Step 6: 提交**

```bash
git add crates/code/src/tools/bash.rs
git commit -m "feat(code): 强化 BashTool 安全沙箱 — 扩展 blocklist + workspace chdir 约束"
```

---

### Task 3: 增强 filesystem 工具的安全检查

**Files:**
- Modify: `src/tools/filesystem.rs:13-19`

- [ ] **Step 1: 强化 `check_path`**

当前已有 `check_path` 函数。保持其逻辑，但增加写入前校验确保目标可写（通过 exists 校验）：实际上当前逻辑已经足够。只需加一个统一的 `resolve_safe_path` 辅助函数返回 canonical 路径供 delete/rename 使用：

```rust
/// 验证路径安全并返回 canonical 路径
fn resolve_safe_path(path: &str) -> anyhow::Result<std::path::PathBuf> {
    let p = std::path::Path::new(path);
    if p.components().any(|c| c.as_os_str() == "..") {
        anyhow::bail!("Path traversal detected: {}", path);
    }
    if p.is_absolute() {
        let cwd = std::env::current_dir()?;
        if !p.canonicalize()?.starts_with(&cwd) {
            anyhow::bail!("Access denied: path outside workspace: {}", path);
        }
    }
    Ok(p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
}
```

- [ ] **Step 2: 验证编译**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings
```

- [ ] **Step 3: 提交**

```bash
git add crates/code/src/tools/filesystem.rs
git commit -m "feat(code): 增强 filesystem check_path — 添加 resolve_safe_path 辅助函数"
```

---

### Task 4: DeleteTool 文件删除工具

**Files:**
- Create: `src/tools/delete.rs`
- Modify: `src/tools/mod.rs`

- [ ] **Step 1: 创建 `src/tools/delete.rs`**

```rust
use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult, resolve_safe_path};
use std::path::Path;

pub struct DeleteTool;

#[async_trait]
impl Tool for DeleteTool {
    fn name(&self) -> &str { "delete" }
    fn description(&self) -> &str { "Delete a file or empty directory. Use --recursive for directories with content." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "delete",
                "description": "Delete a file or empty directory. Use --recursive for directories with content.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to the file or directory to delete"},
                        "recursive": {"type": "boolean", "description": "Recursively delete directory contents (default: false)"}
                    },
                    "required": ["path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("path required"))?;
        let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let safe_path = resolve_safe_path(path)?;

        if !safe_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if safe_path.is_dir() {
            if recursive {
                std::fs::remove_dir_all(&safe_path)?;
                Ok(format!("Deleted directory {} (recursive)", path))
            } else {
                if safe_path.read_dir()?.next().is_some() {
                    anyhow::bail!("Directory not empty: {}. Use recursive=true to delete.", path);
                }
                std::fs::remove_dir(&safe_path)?;
                Ok(format!("Deleted empty directory {}", path))
            }
        } else {
            std::fs::remove_file(&safe_path)?;
            Ok(format!("Deleted file {}", path))
        }
    }
}
```

- [ ] **Step 2: 在 `src/tools/mod.rs` 注册 DeleteTool**

在 `src/tools/mod.rs` 中添加：

```rust
pub mod delete;

// 在 ToolRegistry::new() 中:
Box::new(delete::DeleteTool),
```

- [ ] **Step 3: 验证**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings
```

- [ ] **Step 4: 提交**

```bash
git add crates/code/src/tools/delete.rs crates/code/src/tools/mod.rs
git commit -m "feat(code): 新增 DeleteTool — 安全文件/目录删除替代 bash rm"
```

---

### Task 5: RenameTool 文件重命名工具

**Files:**
- Create: `src/tools/rename.rs`
- Modify: `src/tools/mod.rs`

- [ ] **Step 1: 创建 `src/tools/rename.rs`**

```rust
use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult, resolve_safe_path};

pub struct RenameTool;

#[async_trait]
impl Tool for RenameTool {
    fn name(&self) -> &str { "rename" }
    fn description(&self) -> &str { "Rename or move a file/directory within the workspace." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "rename",
                "description": "Rename or move a file/directory within the workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "from": {"type": "string", "description": "Current path"},
                        "to": {"type": "string", "description": "New path"}
                    },
                    "required": ["from", "to"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let from = args.get("from").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("from required"))?;
        let to = args.get("to").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("to required"))?;
        let safe_from = resolve_safe_path(from)?;
        let safe_to = resolve_safe_path(to)?;

        if !safe_from.exists() {
            anyhow::bail!("Source does not exist: {}", from);
        }
        if safe_to.exists() {
            anyhow::bail!("Destination already exists: {}", to);
        }

        // 确保目标父目录存在
        if let Some(parent) = safe_to.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(&safe_from, &safe_to)?;
        Ok(format!("Renamed {} → {}", from, to))
    }
}
```

- [ ] **Step 2: 在 `src/tools/mod.rs` 注册 RenameTool**

```rust
pub mod rename;

// 在 ToolRegistry::new() 中:
Box::new(rename::RenameTool),
```

- [ ] **Step 3: 验证**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings
```

- [ ] **Step 4: 提交**

```bash
git add crates/code/src/tools/rename.rs crates/code/src/tools/mod.rs
git commit -m "feat(code): 新增 RenameTool — 安全文件重命名/移动替代 bash mv"
```

---

### Task 6: 测试基础设施 — MockLlmProvider + MockTool

**Files:**
- Create: `src/testing.rs`
- Modify: `Cargo.toml` (新增 `tempfile` dev-dependency)

- [ ] **Step 1: 在 Cargo.toml 添加 dev-dependencies**

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: 创建 `src/testing.rs`**

```rust
//! 测试基础设施：Mock 提供者、Mock 工具、辅助函数
//!
//! 使用示例：
//! ```
//! let mock_provider = MockLlmProvider::new()
//!     .with_response("Hello")
//!     .with_tool_call("echo", r#"{"message": "hi"}"#);
//! let mut rx = mock_provider.stream(&[], &[]).await;
//! ```

use crate::provider::*;
use crate::tools::{Tool, ToolResult, ToolRegistry};
use async_trait::async_trait;
use serde_json::{json, Value, Map};
use std::sync::Arc;

// ---- Mock Provider ----

pub struct MockLlmProvider {
    events: Vec<StreamEventKind>,
    name: String,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self { events: Vec::new(), name: "mock".into() }
    }

    /// Add a token event to the stream
    pub fn with_token(mut self, token: &str) -> Self {
        self.events.push(StreamEventKind::Token(token.to_string()));
        self
    }

    /// Add a reasoning event
    pub fn with_reasoning(mut self, text: &str) -> Self {
        self.events.push(StreamEventKind::Reasoning(text.to_string()));
        self
    }

    /// Add a tool call event
    pub fn with_tool_call(mut self, name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        self.events.push(StreamEventKind::ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            args,
        });
        self
    }

    /// Build with a simple text response (token + done)
    pub fn with_response(text: &str) -> Self {
        Self {
            events: vec![
                StreamEventKind::Token(text.to_string()),
                StreamEventKind::Done { content: None, usage: None },
            ],
            name: "mock".into(),
        }
    }

    /// Build with a tool call followed by done
    pub fn with_tool_only(name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        Self {
            events: vec![
                StreamEventKind::ToolCall { id: id.to_string(), name: name.to_string(), args },
                StreamEventKind::Done { content: None, usage: None },
            ],
            name: "mock".into(),
        }
    }

    /// Build with a tool call + text + done
    pub fn with_text_and_tool(text: &str, name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        Self {
            events: vec![
                StreamEventKind::Token(text.to_string()),
                StreamEventKind::ToolCall { id: id.to_string(), name: name.to_string(), args },
                StreamEventKind::Done { content: None, usage: None },
            ],
            name: "mock".into(),
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str { &self.name }

    async fn stream(&self, _messages: &[LlmMessage], _tool_defs: &[Value]) -> StreamRx {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let events = self.events.clone();
        tokio::spawn(async move {
            for event in events {
                if tx.send(StreamEvent { kind: event }).await.is_err() {
                    break;
                }
            }
        });
        rx
    }

    async fn chat(&self, _messages: &[LlmMessage], _tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        Ok(LlmResponse {
            content: None,
            reasoning: String::new(),
            tool_calls: Vec::new(),
            usage: None,
        })
    }
}

// ---- Mock Tool ----

pub struct MockTool {
    name: String,
    response: Result<String, String>,
}

impl MockTool {
    pub fn new(name: &str, response: &str) -> Self {
        Self { name: name.to_string(), response: Ok(response.to_string()) }
    }

    pub fn with_error(name: &str, error: &str) -> Self {
        Self { name: name.to_string(), response: Err(error.to_string()) }
    }
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { "Mock tool for testing" }
    fn schema(&self) -> Value {
        json!({"type": "function", "function": {"name": self.name, "description": "mock", "parameters": {"type": "object", "properties": {}}}})
    }
    async fn call(&self, _args: &Map<String, Value>) -> ToolResult {
        match &self.response {
            Ok(s) => Ok(s.clone()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

/// Create a ToolRegistry with mock tools for testing
pub fn mock_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new_empty();
    registry.register(Arc::new(MockTool::new("read", "file content line 1\nline 2\n")));
    registry.register(Arc::new(MockTool::new("write", "Created test.txt (10 bytes)")));
    registry.register(Arc::new(MockTool::new("bash", "stdout:\nhello")));
    registry.register(Arc::new(MockTool::new("grep", "Found 2 matches:\nmain.rs:10:fn main()\nlib.rs:5:fn main()")));
    registry
}

/// Build a minimal config for testing
pub fn test_config() -> crate::config::Config {
    crate::config::Config {
        provider: Some("mock".into()),
        api_key: Some("test-key".into()),
        base_url: None,
        model: None,
        workspace: None,
        tools_dir: None,
        bin_dir: None,
        max_rounds: 5,
        max_tool_retries: 2,
        tool_timeout_secs: 30,
        agents: std::collections::HashMap::new(),
        mcp_servers: Vec::new(),
    }
}
```

- [ ] **Step 3: 需要给 `ToolRegistry` 增加 `new_empty()` 和 `register()` 方法**

在 `src/tools/mod.rs` 中：

```rust
impl ToolRegistry {
    pub fn new(config: &crate::config::Config) -> anyhow::Result<Self> {
        // ... existing code ...
    }

    /// Create empty registry (for testing)
    pub fn new_empty() -> Self {
        Self { tools: std::collections::HashMap::new() }
    }

    /// Register a tool (for testing)
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
}
```

- [ ] **Step 4: 验证编译**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings
```

- [ ] **Step 5: 提交**

```bash
git add crates/code/src/testing.rs crates/code/src/tools/mod.rs Cargo.toml
git commit -m "feat(code): 添加测试基础设施 — MockLlmProvider + MockTool + ToolRegistry.register()"
```

---

### Task 7: Agent Engine 单元测试

**Files:**
- Modify: `src/agent/engine.rs` (加测试模块)

- [ ] **Step 1: 在 `src/agent/engine.rs` 末尾添加测试模块**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use crate::tools::ToolRegistry;

    fn make_tool_def(name: &str) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": name,
                "description": "test tool",
                "parameters": {"type": "object", "properties": {}}
            }
        })
    }

    #[tokio::test]
    async fn test_react_loop_simple_response() {
        let provider = MockLlmProvider::with_response("Hello, world!");
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Say hello".into())];

        let (text, _msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 5, 30
        ).await.expect("react_loop should succeed");

        assert!(text.contains("Hello"), "Expected 'Hello' in response, got: {}", text);
    }

    #[tokio::test]
    async fn test_react_loop_tool_call_then_response() {
        let provider = MockLlmProvider::with_text_and_tool(
            "Let me check...",
            "read", "call-1", r#"{"file_path": "test.txt"}"#,
        );
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Read test.txt".into())];

        let (text, msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 5, 30
        ).await.expect("react_loop should succeed");

        assert!(text.contains("Let me check"), "Response should contain initial text");
        // verify a tool message was added to the history
        let has_tool_result = msgs.iter().any(|m| matches!(m, LlmMessage::Tool { .. }));
        assert!(has_tool_result, "Tool result should be in message history");
    }

    #[tokio::test]
    async fn test_react_loop_tool_error_triggers_over_limit() {
        // Use a tool that always errors
        let failing_tool = MockTool::with_error("read", "File not found");
        let mut registry = ToolRegistry::new_empty();
        registry.register(std::sync::Arc::new(failing_tool));
        // Also register a second tool that will be called after over_limit
        let echo_tool = MockTool::new("bash", "done");
        registry.register(std::sync::Arc::new(echo_tool));

        let provider = MockLlmProvider::with_tool_only("read", "call-1", r#"{"file_path": "missing.txt"}"#);
        let tool_defs = vec![make_tool_def("read"), make_tool_def("bash")];
        let messages = vec![LlmMessage::User("Test".into())];

        let (_text, msgs) = react_loop(
            &provider, &registry, messages, &tool_defs, false, 3, 5
        ).await.expect("react_loop should not bail on tool errors");

        // After enough retries, there should be a system message about over-limit
        // Since max_rounds=3 and max_tool_retries=2, the over-limit message should appear
        let has_over_limit = msgs.iter().any(|m| matches!(m, LlmMessage::System(s) if s.contains("工具连续")));
        assert!(has_over_limit, "Over-limit reflection prompt should be injected");
    }

    #[tokio::test]
    async fn test_react_loop_provider_retry_on_error() {
        // Mock provider that errors once then succeeds
        struct RetryProvider;
        #[async_trait]
        impl LlmProvider for RetryProvider {
            fn name(&self) -> &str { "retry" }
            async fn stream(&self, _msgs: &[LlmMessage], _tools: &[Value]) -> StreamRx {
                let (tx, rx) = tokio::sync::mpsc::channel(256);
                let mut count = 0u32;
                tokio::spawn(async move {
                    // First call: error
                    if count == 0 {
                        count += 1;
                        tx.send(StreamEvent { kind: StreamEventKind::Error("rate limit exceeded".into()) }).await.ok();
                    }
                    // Second call: success
                    tx.send(StreamEvent { kind: StreamEventKind::Token("Hello after retry".into()) }).await.ok();
                    tx.send(StreamEvent { kind: StreamEventKind::Done { content: None, usage: None } }).await.ok();
                });
                rx
            }
            async fn chat(&self, _msgs: &[LlmMessage], _tools: &[Value]) -> anyhow::Result<LlmResponse> {
                Ok(LlmResponse { content: None, reasoning: String::new(), tool_calls: Vec::new(), usage: None })
            }
        }

        let provider = RetryProvider;
        let tools = mock_tool_registry();
        let messages = vec![LlmMessage::User("Test retry".into())];

        let (text, _msgs) = react_loop(
            &provider, &tools, messages, &[], false, 5, 30
        ).await.expect("react_loop should retry and succeed");

        assert!(text.contains("retry"), "Should get success after retry");
    }
}
```

- [ ] **Step 2: 运行引擎测试**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo test -p i-rs-code agent::engine::tests -- --nocapture
```
Expected: 4 个引擎测试全部通过。

- [ ] **Step 3: 提交**

```bash
git add crates/code/src/agent/engine.rs
git commit -m "test(code): ReAct 循环单元测试 — 正常响应/工具调用/重试/错误恢复"
```

---

### Task 8: Provider 单元测试（SSE 解析 + chat）

**Files:**
- Modify: `src/provider/openai.rs` (加测试)
- Modify: `src/provider/anthropic.rs` (加测试)

- [ ] **Step 1: 为 OpenAiProvider 添加 SSE 解析测试**

在 `src/provider/openai.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_config;

    #[test]
    fn test_build_messages_system() {
        let msgs = vec![LlmMessage::System("You are a helper".into())];
        let result = OpenAiProvider::build_messages(&msgs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[0]["content"], "You are a helper");
    }

    #[test]
    fn test_build_messages_user() {
        let msgs = vec![LlmMessage::User("Hello".into())];
        let result = OpenAiProvider::build_messages(&msgs);
        assert_eq!(result[0]["role"], "user");
    }

    #[test]
    fn test_build_messages_assistant_with_reasoning() {
        let msgs = vec![LlmMessage::AssistantWithReasoning {
            content: "Answer".into(),
            reasoning: "Thinking...".into(),
            tool_calls: vec![],
        }];
        let result = OpenAiProvider::build_messages(&msgs);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["reasoning_content"], "Thinking...");
    }

    #[test]
    fn test_build_messages_tool_call() {
        let msgs = vec![LlmMessage::ToolCall {
            id: "call-1".into(),
            name: "read".into(),
            args: serde_json::json!({"file_path": "test.txt"}),
        }];
        let result = OpenAiProvider::build_messages(&msgs);
        assert_eq!(result[0]["role"], "assistant");
        assert!(result[0]["tool_calls"].is_array());
        assert_eq!(result[0]["tool_calls"][0]["function"]["name"], "read");
    }

    #[test]
    fn test_build_messages_tool_result() {
        let msgs = vec![LlmMessage::Tool {
            name: "read".into(),
            content: "file content".into(),
            call_id: "call-1".into(),
        }];
        let result = OpenAiProvider::build_messages(&msgs);
        assert_eq!(result[0]["role"], "tool");
        assert_eq!(result[0]["content"], "file content");
    }
}
```

- [ ] **Step 2: 运行测试**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo test -p i-rs-code provider::openai::tests -- --nocapture
```
Expected: 5 个构建消息测试全部通过。

- [ ] **Step 3: 提交**

```bash
git add crates/code/src/provider/openai.rs
git commit -m "test(code): OpenAiProvider build_messages 单元测试"
```

---

### Task 9: config/diff/debug 单元测试补充

**Files:**
- Modify: `src/diff.rs`
- Modify: `src/debug.rs`
- Modify: `src/config.rs`

- [ ] **Step 1: diff.rs 测试**

在 `src/diff.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_text_no_change() {
        let text = "hello\nworld\n";
        let result = diff_text(text, text);
        assert_eq!(result.lines_added, 0);
        assert_eq!(result.lines_removed, 0);
    }

    #[test]
    fn test_diff_text_add_line() {
        let old = "hello\n";
        let new = "hello\nworld\n";
        let result = diff_text(old, new);
        assert_eq!(result.lines_added, 1);
    }

    #[test]
    fn test_diff_text_remove_line() {
        let old = "hello\nworld\n";
        let new = "hello\n";
        let result = diff_text(old, new);
        assert_eq!(result.lines_removed, 1);
    }

    #[test]
    fn test_diff_text_modify() {
        let old = "hello\nworld\n";
        let new = "hello\nrust\n";
        let result = diff_text(old, new);
        assert!(result.lines_added > 0 || result.lines_removed > 0);
        assert!(result.patch.contains("rust"));
    }
}
```

- [ ] **Step 2: debug.rs 测试**

在 `src/debug.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_log_and_get_logs() {
        let entry = HttpLogEntry {
            url: "https://example.com".into(),
            request_body: "{}".into(),
            response_status: 200,
            response_body_preview: "ok".into(),
            duration_ms: 100,
            timestamp: "2024-01-01".into(),
        };
        push_log(entry);
        let logs = get_logs();
        assert!(!logs.is_empty());
        assert_eq!(logs[0].url, "https://example.com");
    }

    #[test]
    fn test_get_logs_returns_recent() {
        // push 3 entries
        for i in 0..3 {
            push_log(HttpLogEntry {
                url: format!("https://example.com/{}", i),
                request_body: String::new(),
                response_status: 200,
                response_body_preview: String::new(),
                duration_ms: 0,
                timestamp: String::new(),
            });
        }
        let logs = get_logs();
        assert!(logs.len() >= 3);
    }

    #[test]
    fn test_clear_log() {
        push_log(HttpLogEntry {
            url: "test".into(),
            request_body: String::new(),
            response_status: 200,
            response_body_preview: String::new(),
            duration_ms: 0,
            timestamp: String::new(),
        });
        clear_log();
        let logs = get_logs();
        assert!(logs.is_empty());
    }
}
```

- [ ] **Step 3: 运行所有测试**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo test -p i-rs-code
```
Expected: 所有测试通过（之前 21 + 新增引擎 4 + provider 5 + diff 4 + debug 3 = 37）。

- [ ] **Step 4: 提交**

```bash
git add crates/code/src/diff.rs crates/code/src/debug.rs
git commit -m "test(code): 补充 diff/debug 单元测试"
```

---

### Task 10: 验证最终状态

- [ ] **Step 1: 全量检查**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && cargo check -p i-rs-code && cargo clippy -p i-rs-code -- -D warnings && cargo test -p i-rs-code
```
Expected: 0 errors, 0 warnings, 37+ tests passed.

- [ ] **Step 2: 检查 git 状态**

```bash
cd /Users/mankong/volumes/code/i-rs/clis && git status
```

- [ ] **Step 3: 确认 Phase 4.1 完成**

所有 4 个目标达成：
- ✅ 安全沙箱 (bash blocklist + workspace chdir + resolve_safe_path)
- ✅ 测试覆盖 (MockLlmProvider + engine/provider/diff/debug 测试)
- ✅ 代码质量 (dead_code/unsafe/未使用常量)
- ✅ 文件工具 (DeleteTool + RenameTool)

---

## 自检清单

| Spec 需求 | 对应 Task | 完成 |
|-----------|-----------|------|
| 2.4 安全沙箱 | Task 2 (bash), Task 3 (filesystem) | ✓ |
| 2.5 测试覆盖 + Mock | Task 6 (infra), Task 7 (engine), Task 8 (provider), Task 9 (diff/debug) | ✓ |
| 4.2 代码质量清理 | Task 1 | ✓ |
| 3.3 文件工具 (delete/rename) | Task 4, Task 5 | ✓ |
