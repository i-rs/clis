# i-rs-code Phase 2: 让 Agent 写得靠谱

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 5 项重要改进，让 agent 写代码更可靠、更安全

**Architecture:** 工具失败反馈 → diff 预览 → edit replaceAll → 动态上下文注入 → TUI 取消机制

**Tech Stack:** Rust, tokio, serde_json, similar crate

---

### Task 7: 工具调用失败时反馈给 LLM

**Problem:** `crates/code/src/agent/engine.rs` 中 `Ok(Err(_)) => continue` 静默跳过 tokio spawn panic，`Err(_) => continue` 跳过 join error。LLM 不知道工具失败了，不会重试或换方案。

**Files:**
- Modify: `crates/code/src/agent/engine.rs`

- [ ] **Step 1: 在两个 react_loop 中正确处理工具错误**

在 `react_loop` 和 `react_loop_streaming` 中，将 `Ok(Err(_)) => continue` 改为把 error 信息作为 Tool result 返回给 LLM：

```rust
let (tc, result) = match tokio::time::timeout(std::time::Duration::from_secs(120), handle).await {
    Ok(Ok(inner)) => inner,
    Ok(Err(join_err)) => {
        let result_str = format!("Error: tool task panicked: {}", join_err);
        messages.push(LlmMessage::Tool {
            name: tc.name.clone(),
            content: result_str,
            call_id: tc.id.clone(),
        });
        continue;
    }
    Err(_) => {
        let result_str = "Error: tool execution timed out (120s)".to_string();
        messages.push(LlmMessage::Tool {
            name: tc.name.clone(),
            content: result_str,
            call_id: tc.id.clone(),
        });
        continue;
    }
};
```

对于 streaming 版本，同步发送 `AgentEvent::ToolCallEnd`：
```rust
Ok(Err(join_err)) => {
    let result_str = format!("Error: tool task panicked: {}", join_err);
    event_tx.send(AgentEvent::ToolCallEnd {
        id: tc.id.clone(),
        name: tc.name.clone(),
        result: result_str.clone(),
    }).await.ok();
    messages.push(LlmMessage::Tool {
        name: tc.name.clone(),
        content: result_str,
        call_id: tc.id.clone(),
    });
    continue;
}
```

- [ ] **Step 2: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 3: Commit**

```
fix(code): 工具调用失败不再静默跳过 — 错误信息反馈给 LLM
```

---

### Task 8: Write/Edit 工具集成 git diff 预览

**Problem:** `write`/`edit` 直接改文件，没有预览和 rollback。`diff.rs` 存在但从未使用。

**Files:**
- Modify: `crates/code/src/tools/filesystem.rs`

- [ ] **Step 1: WriteTool 改动前生成 diff**

```rust
async fn call(&self, args: &Map<String, Value>) -> ToolResult {
    let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let content = args.get("content").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("content required"))?;
    check_path(path)?;

    let old_content = if std::path::Path::new(path).exists() {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;

    if old_content.is_empty() {
        Ok(format!("Created {} ({} bytes)\n```{}\n```", path, content.len(), content))
    } else {
        let diff = crate::diff::diff_text(&old_content, content);
        Ok(format!(
            "Modified {} (+{} -{})\n```diff\n{}\n```",
            path, diff.lines_added, diff.lines_removed, diff.patch
        ))
    }
}
```

- [ ] **Step 2: EditTool 改动前生成 diff**

```rust
async fn call(&self, args: &Map<String, Value>) -> ToolResult {
    let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let old = args.get("old_string").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("old_string required"))?;
    let new = args.get("new_string").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("new_string required"))?;
    let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);
    check_path(path)?;

    let content = std::fs::read_to_string(path)?;

    let count = content.matches(old).count();
    if count == 0 {
        anyhow::bail!("old_string not found in {}", path);
    }
    if count > 1 && !replace_all {
        anyhow::bail!("Found {} matches for old_string. Use replace_all=true or provide more context.", count);
    }

    let new_content = if replace_all {
        content.replace(old, new)
    } else {
        content.replacen(old, new, 1)
    };

    let diff = crate::diff::diff_text(&content, &new_content);
    std::fs::write(path, &new_content)?;

    Ok(format!(
        "Edited {} (+{} -{}{})\n```diff\n{}\n```",
        path,
        diff.lines_added,
        diff.lines_removed,
        if replace_all { format!(" ({} replacements)", count) } else { String::new() },
        diff.patch
    ))
}
```

同步更新 EditTool schema，添加 `replace_all` 参数：
```rust
"replace_all": {"type": "boolean", "description": "Replace all occurrences (default: false)"}
```

- [ ] **Step 3: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 4: Commit**

```
feat(code): write/edit 工具 — 集成 git diff 预览 + edit 支持 replaceAll
```

---

### Task 9: TUI 取消机制 (Ctrl+C 中断当前 agent)

**Problem:** Agent 开始工作后只能杀进程退出，没法中断当前 tool call。

**Files:**
- Modify: `crates/code/src/app.rs`
- Modify: `crates/code/src/tui.rs`

- [ ] **Step 1: App 添加取消 token**

```rust
// app.rs
use tokio::sync::oneshot;

pub struct App {
    // ... existing fields
    pub cancel_tx: Option<oneshot::Sender<()>>,
}
```

在 `App::new()` 中初始化为 `None`。

- [ ] **Step 2: spawn agent 时创建 cancel token**

修改 `tui.rs` 的 Enter 处理和 `Waiting` 模式的 Ctrl+C：

```rust
// Enter 分支
let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
app.cancel_tx = Some(cancel_tx);
// ... spawn
```

在 `Waiting` 模式的 key handler 中添加：
```rust
AppMode::Waiting => {
    match key.code {
        KeyCode::Char('c') | KeyCode::Esc if key.modifiers == KeyModifiers::CONTROL => {
            if let Some(tx) = app.cancel_tx.take() {
                tx.send(()).ok();
            }
            app.mode = AppMode::Idle;
            let content = app.finish_streaming();
            if !content.is_empty() {
                app.messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: format!("{}\n\n[Cancelled]", content),
                });
            }
            return;
        }
        _ => {}
    }
}
```

- [ ] **Step 3: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 4: Commit**

```
feat(code): TUI 取消机制 — Ctrl+C 中断当前 agent 执行
```

---

### Task 10: 动态上下文注入 (git status + 工作目录)

**Problem:** LLM 不知道当前项目状态（git branch、未提交更改、项目类型）。

**Files:**
- Modify: `crates/code/src/prompt.rs`
- Modify: `crates/code/src/agent/mod.rs`

- [ ] **Step 1: 添加上下文收集函数**

在 `crates/code/src/prompt.rs` 中添加：

```rust
pub fn build_context() -> String {
    let mut ctx = String::new();

    if let Ok(dir) = std::env::current_dir() {
        ctx.push_str(&format!("Working directory: {}\n", dir.display()));
    }

    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            ctx.push_str(&format!("Git branch: {}\n", branch));
        }
    }

    if let Ok(output) = std::process::Command::new("git")
        .args(["status", "--short"])
        .output()
    {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !status.is_empty() {
            let lines: Vec<&str> = status.lines().take(20).collect();
            ctx.push_str(&format!("Git status:\n{}\n", lines.join("\n")));
        }
    }

    if std::path::Path::new("Cargo.toml").exists() {
        ctx.push_str("Project type: Rust (Cargo)\n");
    } else if std::path::Path::new("package.json").exists() {
        ctx.push_str("Project type: Node.js\n");
    } else if std::path::Path::new("pyproject.toml").exists() {
        ctx.push_str("Project type: Python\n");
    }

    ctx
}
```

- [ ] **Step 2: 在 build_messages 中注入上下文**

修改 `crates/code/src/agent/mod.rs` 的 `build_messages`：

```rust
fn build_messages(
    history: &[LlmMessage],
    system_text: &str,
    prompt: &str,
) -> Vec<LlmMessage> {
    let mut msgs = Vec::new();
    let context = crate::prompt::build_context();
    let system = if context.is_empty() {
        system_text.to_string()
    } else {
        format!("{}\n\n## Current Context\n{}", system_text, context)
    };
    msgs.push(LlmMessage::System(system));
    // ... rest unchanged
}
```

- [ ] **Step 3: Compile and verify**

Run: `cargo check -p i-rs-code`
Expected: 0 errors

- [ ] **Step 4: Commit**

```
feat(code): 动态上下文注入 — git branch/status + 项目类型
```

---

### Task 11: 全量 cargo check + clippy

- [ ] **Step 1: 全量编译**

Run: `cargo check --workspace`
Expected: 0 errors, 0 warnings

- [ ] **Step 2: i-rs-code clippy**

Run: `cargo clippy -p i-rs-code -- -D warnings`
Expected: 0 errors

- [ ] **Step 3: Final commit (if needed)**

```
chore(code): phase 2 全量检查通过
```
