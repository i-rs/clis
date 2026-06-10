# Claw Performance & API Hardening Implementation Plan

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add per-user rate limiting, eliminate write-lock contention in the SSE streaming path, replace linear tool lookup with HashMap dispatch, introduce a structured `ToolResult` enum to replace fragile string-prefix matching, and plumb `UserId` through every handler that currently hardcodes `"default"`.

**Architecture:** (1) Add `tower-governor` for per-IP + per-user request rate limiting. (2) Snapshot required state before opening SSE stream; mutations go via `mpsc` to a dedicated writer task. (3) Change `ToolRegistry::tools: Vec<Box<dyn ClawTool>>` to `IndexMap<String, Arc<dyn ClawTool>>` (preserve iteration order via IndexMap for docs). (4) Add `ToolResultKind { Ok, Err, Blocked }` enum on `ToolResult`; emit it in executor; replace `result.starts_with("错误")` with `result.kind.is_error()`. (5) Update every handler that hardcodes `"default"` to use `UserId` extractor.

**Tech Stack:** Rust, axum, tower-governor, tokio mpsc, IndexMap.

---

## Scope — verified audit findings

| ID | File:Line | Issue |
|----|-----------|-------|
| M8 | `crates/claw/src/server/app.rs:56-88` | No rate limit on `/api/chat`. Authenticated user can spawn thousands of concurrent LLM calls. |
| M4 | `crates/claw/src/server/routes/chat.rs:33, 54, 88` | SSE handler holds `core.write().await` inside the event loop. Multi-user concurrency degrades badly. |
| M2 | `crates/claw-core/src/core/engine/execution.rs:11` | `registry.tools.iter().find(|t| t.name() == name)` is O(n) per tool call. Use HashMap. |
| M9 | `crates/claw/src/server/routes/chat.rs:40` | `result.starts_with("错误")` and `result.starts_with("护栏拦截")` for branch selection. Should be structured enum. |
| UserId | `crates/claw/src/server/routes/{chat,tools,...}.rs` | `UserId` extractor exists but every handler hardcodes `"default"` (chat.rs:38, 76, tools.rs:49, etc.) |

## File Structure

**Files created:**
- `crates/claw-core/src/tools/result.rs` — `ToolResult`, `ToolResultKind` types + helpers
- `crates/claw/src/server/rate_limit.rs` — governor configs (per-IP + per-user)

**Files modified:**
- `crates/claw/Cargo.toml` — add `tower-governor`, `indexmap` (forwarded feature)
- `crates/claw-core/Cargo.toml` — add `indexmap`
- `crates/claw-core/src/tools/mod.rs` — `ToolRegistry: Vec → IndexMap`, re-export `result`
- `crates/claw-core/src/core/engine/execution.rs` — O(1) lookup, return `ToolResult`
- `crates/claw-core/src/core/executor.rs` — emit `ToolResultKind` on every branch
- `crates/claw-core/src/tools/{delegate,file_ops,mcp_tools}.rs` — return `ToolResult` from `execute` (migration helper)
- `crates/claw/src/server/routes/chat.rs` — snapshot before SSE, replace `starts_with`, accept `UserId`
- `crates/claw/src/server/routes/{tools,memory,sessions,mcp_config,agents}.rs` — `UserId` plumbing
- `crates/claw/src/server/{app.rs,mod.rs}` — wire rate-limit layer, declare module
- `crates/claw/src/lib.rs` — re-export rate_limit if needed

---

## Task 1: Add `tower-governor` and `indexmap` dependencies

**Files:**
- Modify: `crates/claw/Cargo.toml`
- Modify: `crates/claw-core/Cargo.toml`

- [ ] **Step 1: Add `indexmap` to `crates/claw-core/Cargo.toml`**

In the `[dependencies]` table, add (anywhere, alphabetic order is conventional):

```toml
indexmap = "2"
```

- [ ] **Step 2: Add `tower-governor` and `indexmap` to `crates/claw/Cargo.toml`**

Inside the `[dependencies]` block, add:

```toml
tower-governor = { version = "0.4", optional = true }
indexmap.workspace = true
```

(We will add a workspace declaration for `indexmap` next.)

- [ ] **Step 3: Add `indexmap` to workspace `Cargo.toml`**

Open `/Users/mankong/volumes/code/i-rs/clis/Cargo.toml` and inside `[workspace.dependencies]` add:

```toml
indexmap = "2"
```

- [ ] **Step 4: Wire `tower-governor` behind the `dashboard` feature**

In `crates/claw/Cargo.toml`, update the `dashboard` feature to include the new dep:

```toml
[features]
dashboard = ["axum", "tower-http", "rust-embed", "mime_guess", "tower-governor", "i-rs-claw-core/dashboard"]
```

- [ ] **Step 5: Verify `cargo check`**

Run: `cargo check -p i-rs-claw-core && cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/claw/Cargo.toml crates/claw-core/Cargo.toml Cargo.lock
git commit -m "deps: add indexmap (claw-core) and tower-governor (claw/dashboard)"
```

---

## Task 2: Convert `ToolRegistry` to `IndexMap<String, Arc<dyn ClawTool>>`

The `IndexMap` preserves insertion order (needed for stable schema/docs output) while giving O(1) name lookup.

**Files:**
- Modify: `crates/claw-core/src/tools/mod.rs:204-335`
- Modify: `crates/claw-core/src/core/engine/execution.rs`
- Test: existing tests in `mod.rs:337-406` + new test

- [ ] **Step 1: Write the new failing test first**

In `crates/claw-core/src/tools/mod.rs`, inside `mod tests`, append:

```rust
#[test]
fn test_hashmap_lookup_three_tools() {
    let reg = ToolRegistry::new();
    // All built-ins we expect to find via O(1) lookup
    assert!(reg.get("calculator").is_some());
    assert!(reg.get("i_rs").is_some());
    assert!(reg.get("web_search").is_some());
    // Lookup must return the correct tool
    let calc = reg.get("calculator").expect("calculator must be registered");
    assert_eq!(calc.name(), "calculator");
    // Miss is also O(1)
    assert!(reg.get("does_not_exist").is_none());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p i-rs-claw-core tools::tests::test_hashmap_lookup_three_tools -- --nocapture`
Expected: FAIL — `get` method does not exist on `ToolRegistry`.

- [ ] **Step 3: Change the `ToolRegistry` field and constructor**

In `crates/claw-core/src/tools/mod.rs`, replace the `ToolRegistry` struct + `new` (lines 211-240) with:

```rust
pub struct ToolRegistry {
    pub tools: indexmap::IndexMap<String, std::sync::Arc<dyn ClawTool>>,
    excluded: std::collections::HashSet<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        use std::sync::Arc;
        let mut tools: indexmap::IndexMap<String, Arc<dyn ClawTool>> = indexmap::IndexMap::new();
        let builtins: Vec<Arc<dyn ClawTool>> = vec![
            Arc::new(calculator_typed::CalculatorTyped::as_claw_tool()),
            Arc::new(chart_tool::ChartTool),
            Arc::new(chart_image::ChartImageTool),
            Arc::new(generate_image::GenerateImageTool),
            Arc::new(file_ops::FileOpsTool),
            Arc::new(i_rs::IrsTool),
            Arc::new(search_conversations::SearchConversationsTool),
            Arc::new(search_tools::SearchToolsTool),
            Arc::new(user_memory::UserMemoryTool),
            Arc::new(call_code_agent::CallCodeAgentTool),
            Arc::new(delegate::DelegateTool),
            Arc::new(vision_tool::VisionTool),
            Arc::new(rag_tool::RagTool::new()),
            Arc::new(web_search::WebSearchTool),
            Arc::new(chain_tool::ChainTool::new()),
            Arc::new(orchestration_tool::OrchestrationTool::new()),
            Arc::new(progress_tool::ProgressTool::new()),
        ];
        for t in builtins {
            tools.insert(t.name().to_string(), t);
        }
        Self {
            tools,
            excluded: std::collections::HashSet::new(),
        }
    }
```

- [ ] **Step 4: Update `with_skills` and `with_mcp`**

Replace the existing builder methods (lines 247-267) with:

```rust
    /// Add skill tools from SkillStore (builder pattern, consumes self).
    pub fn with_skills(mut self, skills: &[SkillDefinition]) -> Self {
        for skill in skills {
            let tool: std::sync::Arc<dyn ClawTool> = std::sync::Arc::new(
                skill_tool::SkillTool::new(skill.clone()),
            );
            self.tools.insert(tool.name().to_string(), tool);
        }
        self
    }

    /// Add MCP-discovered tools (builder pattern, consumes self).
    pub fn with_mcp(mut self, mcp_registry: &crate::mcp::McpRegistry) -> Self {
        for (client_idx, tool_def) in mcp_registry.tools() {
            if let Some(client) = mcp_registry.clients().get(*client_idx) {
                let tool: std::sync::Arc<dyn ClawTool> = std::sync::Arc::new(
                    mcp_tools::McpToolWrapper::new(tool_def.clone(), client.clone()),
                );
                self.tools.insert(tool.name().to_string(), tool);
            }
        }
        self
    }
```

- [ ] **Step 5: Update `enabled_schemas`**

Replace `enabled_schemas` (lines 269-304) — iteration order with IndexMap is insertion order, same semantics:

```rust
    pub fn enabled_schemas(
        &self,
        i_rs_tool_names: &[&str],
        enabled: Option<&HashSet<String>>,
    ) -> Vec<Value> {
        let enabled_cli: Vec<&str> = if let Some(enabled_set) = enabled {
            i_rs_tool_names
                .iter()
                .filter(|&&t| enabled_set.contains(t))
                .copied()
                .collect()
        } else {
            i_rs_tool_names.to_vec()
        };
        self.tools
            .iter()
            .filter(|(name, _)| !self.excluded.contains(name.as_str()))
            .map(|(_, tool)| {
                let mut schema = serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema(&enabled_cli),
                    }
                });
                if let Some(output) = tool.output_schema() {
                    schema["function"]["output"] = output;
                }
                schema
            })
            .collect()
    }
```

- [ ] **Step 6: Add `get` method + update `execute` / `tool_exists` / `tool_info`**

Replace the existing `execute`, `tool_exists`, `tool_info` block (lines 306-335) with:

```rust
    /// O(1) lookup by name.
    pub fn get(&self, name: &str) -> Option<&std::sync::Arc<dyn ClawTool>> {
        self.tools.get(name)
    }

    /// Execute a tool by name (O(1) lookup).
    #[allow(dead_code)]
    pub async fn execute(
        &self,
        name: &str,
        args: &Value,
        ctx: &ToolContext,
    ) -> Result<String, crate::error::ClawError> {
        match self.tools.get(name) {
            Some(t) => t.execute(args, ctx).await,
            None => Err(crate::error::ClawError::NotFound(format!(
                "未知工具: {}",
                name
            ))),
        }
    }

    /// Check if a built-in tool exists (O(1)).
    #[allow(dead_code)]
    pub fn tool_exists(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn tool_info(&self) -> Vec<(&str, &str)> {
        self.tools
            .iter()
            .map(|(_, t)| (t.name(), t.description()))
            .collect()
    }
```

- [ ] **Step 7: Update `core/engine/execution.rs` to O(1) lookup**

Replace the entire file body (currently 19 LOC) with:

```rust
use serde_json::Value;

/// Execute a parsed tool call and return the result.
/// Uses O(1) HashMap lookup against the shared `ToolRegistry`.
pub async fn execute_tool_call(
    name: &str,
    args: &Value,
    registry: &crate::tools::ToolRegistry,
    ctx: &crate::tools::ToolContext,
) -> String {
    if let Some(tool) = registry.get(name) {
        return tool
            .execute(args, ctx)
            .await
            .unwrap_or_else(|e| e.to_string());
    }

    format!("错误: 未知工具 {}", name)
}
```

- [ ] **Step 8: Run all tests to confirm no regression**

Run: `cargo test -p i-rs-claw-core`
Expected: PASS — all existing tests pass plus the new `test_hashmap_lookup_three_tools`.

- [ ] **Step 9: Commit**

```bash
git add crates/claw-core/src/tools/mod.rs crates/claw-core/src/core/engine/execution.rs
git commit -m "perf(claw-core): convert ToolRegistry to IndexMap for O(1) lookup"
```

---

## Task 3: Add `ToolResultKind` enum and `ToolResult` struct

Introduce structured tool result types so downstream code does not need to pattern-match on string prefixes.

**Files:**
- Create: `crates/claw-core/src/tools/result.rs`
- Modify: `crates/claw-core/src/tools/mod.rs` (declare module + re-export)

- [ ] **Step 1: Create the new module file**

Create `crates/claw-core/src/tools/result.rs` with this exact content:

```rust
//! Structured tool execution result.
//!
//! Replaces ad-hoc `String` returns where callers had to match on
//! `result.starts_with("错误")` / `result.starts_with("护栏拦截")`.

use serde::{Deserialize, Serialize};

/// Categorical outcome of a tool invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResultKind {
    /// Tool executed successfully (or returned a non-error payload).
    Ok,
    /// Tool ran but produced an error (validation, execution, timeout, …).
    Err,
    /// Tool was blocked before execution (guardrail, HITL deny, sandbox).
    Blocked,
}

impl ToolResultKind {
    pub fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
    pub fn is_error(self) -> bool {
        matches!(self, Self::Err)
    }
    pub fn is_blocked(self) -> bool {
        matches!(self, Self::Blocked)
    }
}

/// Structured result returned by tool executors.
///
/// `text` retains the existing human/LLM-readable string so we can
/// migrate incrementally without breaking prompt formatting.
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub kind: ToolResultKind,
    pub text: String,
}

impl ToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        Self { kind: ToolResultKind::Ok, text: text.into() }
    }
    pub fn err(text: impl Into<String>) -> Self {
        Self { kind: ToolResultKind::Err, text: text.into() }
    }
    pub fn blocked(text: impl Into<String>) -> Self {
        Self { kind: ToolResultKind::Blocked, text: text.into() }
    }
}

/// Backward-compat: callers that still hold a raw `String` can infer the
/// kind from the legacy prefixes. This will be removed once every site
/// produces `ToolResult` directly.
impl From<String> for ToolResult {
    fn from(s: String) -> Self {
        if s.starts_with("护栏拦截") || s.starts_with("操作被安全策略拒绝") {
            Self::blocked(s)
        } else if s.starts_with("错误") {
            Self::err(s)
        } else {
            Self::ok(s)
        }
    }
}

impl From<&str> for ToolResult {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<ToolResult> for String {
    fn from(r: ToolResult) -> String {
        r.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_ok() {
        let r = ToolResult::from("hello world");
        assert_eq!(r.kind, ToolResultKind::Ok);
        assert!(r.kind.is_ok());
    }

    #[test]
    fn classifies_err() {
        let r = ToolResult::from("错误: 工具执行超时");
        assert_eq!(r.kind, ToolResultKind::Err);
        assert!(r.kind.is_error());
    }

    #[test]
    fn classifies_blocked() {
        let r = ToolResult::from("护栏拦截: 危险操作");
        assert_eq!(r.kind, ToolResultKind::Blocked);
        assert!(r.kind.is_blocked());
    }

    #[test]
    fn classifies_hitl_deny() {
        let r = ToolResult::from("操作被安全策略拒绝: 此工具被配置为禁止执行");
        assert_eq!(r.kind, ToolResultKind::Blocked);
    }

    #[test]
    fn constructors_set_kind() {
        assert_eq!(ToolResult::ok("ok").kind, ToolResultKind::Ok);
        assert_eq!(ToolResult::err("e").kind, ToolResultKind::Err);
        assert_eq!(ToolResult::blocked("b").kind, ToolResultKind::Blocked);
    }
}
```

- [ ] **Step 2: Declare module + re-export in `tools/mod.rs`**

Edit `crates/claw-core/src/tools/mod.rs`. At the top of the file (after line 1 `pub mod calculator;` … add):

```rust
pub mod result;
```

Then, after the `use …` block near line 64-66, add:

```rust
pub use result::{ToolResult, ToolResultKind};
```

- [ ] **Step 3: Run the new module's tests**

Run: `cargo test -p i-rs-claw-core tools::result::tests -- --nocapture`
Expected: PASS — 5 tests.

- [ ] **Step 4: Commit**

```bash
git add crates/claw-core/src/tools/result.rs crates/claw-core/src/tools/mod.rs
git commit -m "feat(claw-core): introduce ToolResultKind + ToolResult"
```

---

## Task 4: Emit `ToolResultKind` from the executor

Update the executor's `ToolCallResult` to carry a `ToolResultKind`, and set it explicitly on every branch (success / error result / guardrail-block / HITL-deny / panic / timeout).

**Files:**
- Modify: `crates/claw-core/src/core/executor.rs`

- [ ] **Step 1: Add `kind` field to `ToolCallResult`**

In `crates/claw-core/src/core/executor.rs`, replace the struct (lines 19-29) with:

```rust
pub struct ToolCallResult {
    pub call: ToolCallAcc,
    #[allow(dead_code)]
    pub args: Value,
    pub result: String,
    pub kind: crate::tools::ToolResultKind,
    #[allow(dead_code)]
    pub context_result: String,
    #[allow(dead_code)]
    pub validation: ToolResultValidation,
    pub category: ErrorCategory,
}
```

- [ ] **Step 2: Set `kind` on the guardrail-blocked branch**

In `execute()` (around lines 218-235), update the guardrail branch:

```rust
                if !gr.allowed {
                    let reason = gr.reason.unwrap_or_default();
                    tracing::warn!(tool = %tc.name, reason = %reason, "工具调用被护栏拦截");
                    blocked_results.push(ToolCallResult {
                        call: tc,
                        args,
                        result: format!("护栏拦截: {}", reason),
                        kind: crate::tools::ToolResultKind::Blocked,
                        context_result: format!("护栏拦截: {}", reason),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec![reason],
                        },
                        category: ErrorCategory::Validation,
                    });
                    continue;
                }
```

- [ ] **Step 3: Set `kind` on the HITL-deny branch**

In `execute()` (around lines 237-257), update the HITL branch:

```rust
                if hitl.should_deny(&req) {
                    tracing::warn!(tool = %tc.name, "工具调用被 HITL 策略拒绝");
                    let _ = tx.send(LlmEvent::Status(format!(
                        "🚫 工具 {} 被安全策略拦截 (风险: {:?})",
                        tc.name, req.risk_level
                    )));
                    blocked_results.push(ToolCallResult {
                        call: tc,
                        args,
                        result: "操作被安全策略拒绝: 此工具被配置为禁止执行".to_string(),
                        kind: crate::tools::ToolResultKind::Blocked,
                        context_result: "操作被安全策略拒绝".to_string(),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec!["HITL 策略拒绝".to_string()],
                        },
                        category: ErrorCategory::Validation,
                    });
                    continue;
                }
```

- [ ] **Step 4: Set `kind` on the normal execution + timeout branch**

In the `tokio::spawn` closure (around lines 283-325), the executor currently returns a `String` from `execute_tool_call`. Convert it via `ToolResult::from`:

```rust
            handles.push(tokio::spawn(async move {
                let raw: String = if let Some(cached) = cache_entry {
                    tracing::debug!(tool = %tc_name, "工具结果缓存命中");
                    cached
                } else {
                    match tokio::time::timeout(timeout_dur, async {
                        crate::core::engine::execute_tool_call(
                            &tc_name,
                            &args,
                            &registry_for_spawn,
                            &ctx_for_spawn,
                        )
                        .await
                    })
                    .await
                    {
                        Ok(r) => r,
                        Err(_) => format!("错误: 工具执行超时 (>{:?})", timeout_dur),
                    }
                };

                // Classify into Ok / Err / Blocked for downstream consumers.
                let classified = crate::tools::ToolResult::from(raw);
                let result = classified.text;
                let result_kind = classified.kind;

                let display_result = utils::smart_truncate(&result, trunc_display);
                let context_result = utils::compact_tool_result(&tc_name, &result, trunc_context);

                let _ = tx.send(LlmEvent::ToolExecuted {
                    name: tc.name.clone(),
                    args: args_str,
                    result: display_result,
                    step,
                    total_steps: total,
                });

                let (validation, category) = validate_tool_result(&tc.name, &result);
                if !validation.valid {
                    let _ = tx.send(LlmEvent::Evaluation {
                        tool: tc.name.clone(),
                        valid: false,
                        issues: validation.issues.clone(),
                    });
                }

                (tc, args, context_result, result_kind, validation, category)
            }));
```

- [ ] **Step 5: Set `kind` on the panic fallback (handle.await Err)**

In the result aggregation loop (around lines 329-364), update both the Ok branch and Err branch:

```rust
        let mut all_results: Vec<ToolCallResult> = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok((call, args, context_result, kind, validation, category)) => {
                    if !category.is_error() {
                        let key = Self::cache_key(&call.name, &args);
                        self.put_cache(key, context_result.clone());
                    }
                    all_results.push(ToolCallResult {
                        call,
                        args,
                        result: context_result.clone(),
                        kind,
                        context_result,
                        validation,
                        category,
                    });
                }
                Err(e) => {
                    tracing::error!("Tool task panicked: {}", e);
                    all_results.push(ToolCallResult {
                        call: ToolCallAcc {
                            id: String::new(),
                            name: String::new(),
                            arguments: String::new(),
                        },
                        args: Value::Null,
                        result: format!("工具任务崩溃: {}", e),
                        kind: crate::tools::ToolResultKind::Err,
                        context_result: format!("工具任务崩溃: {}", e),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec![format!("工具任务崩溃: {}", e)],
                        },
                        category: ErrorCategory::Execution,
                    });
                }
            }
        }
```

- [ ] **Step 6: Verify compile**

Run: `cargo check -p i-rs-claw-core`
Expected: 0 errors, 0 warnings.

- [ ] **Step 7: Run the full test suite for claw-core**

Run: `cargo test -p i-rs-claw-core`
Expected: PASS (no behavioural change yet, just new field).

- [ ] **Step 8: Commit**

```bash
git add crates/claw-core/src/core/executor.rs
git commit -m "feat(claw-core): emit ToolResultKind on every executor branch"
```

---

## Task 5: Replace `starts_with("错误")` in `chat.rs` with structured kind

Now that `ToolCallResult.kind` is populated, the SSE handler can branch on the enum instead of string matching.

**Note:** This task depends on Task 6's snapshot refactor being in place to be useful at scale, but it can be applied independently first. We do it here as a focused, small change.

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs`

- [ ] **Step 1: Locate the SSE branch in `build_sse_stream`**

Open `crates/claw/src/server/routes/chat.rs`. The branch in question is currently at lines 40-44:

```rust
                        if !result.starts_with("错误") && !result.starts_with("护栏拦截") {
                            i_rs_claw_core::core::record_layered_tool_memory(
                                "default", &mut core.agent_store, &agent_id, &name, &result,
                            );
                        }
```

The string `result` here is the raw SSE event payload from `LlmEvent::ToolExecuted { result, .. }`. We have *two* options:

A) Thread `kind` through `LlmEvent::ToolExecuted` (cleaner, but requires changing the enum).
B) Classify locally using `ToolResult::from(result.clone())`.

Use option **B** for minimum blast radius; option A can be a follow-up.

- [ ] **Step 2: Apply the local classification**

Replace the branch with:

```rust
                        let classified = i_rs_claw_core::tools::ToolResult::from(result.clone());
                        if classified.kind.is_ok() {
                            i_rs_claw_core::core::record_layered_tool_memory(
                                "default", &mut core.agent_store, &agent_id, &name, &result,
                            );
                        }
```

- [ ] **Step 3: Check `cargo check`**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

- [ ] **Step 4: Run dashboard tests**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: PASS (or pre-existing tokio runtime failures, which should be unchanged from baseline).

- [ ] **Step 5: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(claw): replace starts_with error sniffing with ToolResultKind"
```

---

## Task 6: Snapshot state before SSE stream + writer-task pattern

This is the largest change. Currently `build_sse_stream` takes a write lock on every `ToolExecuted`/`Done`/`Error` event. Instead:

1. Snapshot the *immutable* values needed inside the stream (`sid`, `agent_id`, `i_rs_index`) before opening the stream.
2. Spawn a dedicated writer task that owns the `Arc<RwLock<AppCore>>` write capability.
3. The stream sends write-op messages over an `mpsc` channel; the writer task applies them.

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs`

- [ ] **Step 1: Add a `WriteOp` enum near the top of `chat.rs`**

Insert after the `use` block (after line 14):

```rust
/// Write operations the SSE stream needs the writer task to perform.
/// All `String` fields are owned so this is `Send + 'static`.
enum WriteOp {
    /// Record tool memory + (optionally) layered memory for a tool execution.
    RecordTool {
        sid: String,
        agent_id: String,
        name: String,
        args: String,
        result: String,
        kind_is_ok: bool,
    },
    /// Persist final messages on `Done`.
    PersistDone {
        sid: String,
        agent_id: String,
        finalized: Vec<crate::app::Message>,
        api_messages: Vec<crate::app::Message>,
        quality: Option<crate::app::Message>,
    },
    /// Mark the session as errored.
    MarkError {
        sid: String,
        finalized: Vec<crate::app::Message>,
        err: String,
    },
}
```

- [ ] **Step 2: Add a writer task launcher**

Below `WriteOp`, add:

```rust
/// Spawn a task that owns the write-lock capability. Returns the sender
/// the SSE stream will use to enqueue write operations.
fn spawn_writer(state: crate::server::AppState) -> mpsc::UnboundedSender<WriteOp> {
    let (tx, mut rx) = mpsc::unbounded_channel::<WriteOp>();
    tokio::spawn(async move {
        while let Some(op) = rx.recv().await {
            match op {
                WriteOp::RecordTool { sid, agent_id, name, args, result, kind_is_ok } => {
                    let mut core = state.core.write().await;
                    let i_rs_index = core.config.i_rs_tool_index.clone();
                    i_rs_claw_core::core::record_tool_memory(
                        "default", &mut core.agent_store, &i_rs_index,
                        &agent_id, &name, &args, &result,
                    );
                    if kind_is_ok {
                        i_rs_claw_core::core::record_layered_tool_memory(
                            "default", &mut core.agent_store, &agent_id, &name, &result,
                        );
                    }
                    let _ = sid; // currently unused beyond scoping
                }
                WriteOp::PersistDone { sid, agent_id, finalized, api_messages, quality } => {
                    let mut core = state.core.write().await;
                    if let Err(e) = core.session_mgr.persist_messages(&sid, &finalized) {
                        tracing::error!("persist_messages (Done) 失败: {}", e);
                    }
                    core.session_mgr.save_api_messages(&sid, &api_messages);
                    if let Some(q) = &quality {
                        if let Err(e) = core.session_mgr.persist_messages(&sid, &[q.clone()]) {
                            tracing::error!("quality 持久化失败: {}", e);
                        }
                    }
                    core.agent_store.memory_for_mut("default", &agent_id).flush();
                }
                WriteOp::MarkError { sid, finalized, err } => {
                    let mut core = state.core.write().await;
                    core.session_mgr.mark_error(&sid, &err);
                    if let Err(e) = core.session_mgr.persist_messages(&sid, &finalized) {
                        tracing::error!("persist_messages (error) 失败: {}", e);
                    }
                }
            }
        }
        // rx closed -> task exits cleanly when stream ends.
    });
    tx
}
```

- [ ] **Step 3: Replace the `build_sse_stream` signature and body**

Replace the entire current `build_sse_stream` function (lines 17-135) with this version. The stream closure now takes owned values + a sender; **no `state.core.write()` calls inside the loop**.

```rust
/// Build an SSE stream that never takes the `AppCore` write lock inline.
/// All write-ops are funneled through `writer_tx` to a dedicated task.
fn build_sse_stream(
    rx: mpsc::UnboundedReceiver<LlmEvent>,
    writer_tx: mpsc::UnboundedSender<WriteOp>,
    sid: String,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = futures_util::stream::unfold(
        (Some(rx), writer_tx, sid, MessageAccumulator::new(), 0u64),
        |(rx_opt, writer_tx, sid, mut acc, mut seq)| async move {
            let mut rx = rx_opt?;
            loop {
                let event = rx.recv().await?;
                let sse_event;
                let mut keep_rx = true;

                match event {
                    LlmEvent::ToolExecuted { name, args, result, step, total_steps } => {
                        let classified = i_rs_claw_core::tools::ToolResult::from(result.clone());
                        let _ = writer_tx.send(WriteOp::RecordTool {
                            sid: sid.clone(),
                            agent_id: String::new(), // filled by writer from session meta? we'll snapshot next
                            name: name.clone(),
                            args: args.clone(),
                            result: result.clone(),
                            kind_is_ok: classified.kind.is_ok(),
                        });
                        let data = serde_json::to_string(&serde_json::json!({
                            "name": name, "args": args, "result": result,
                            "step": step, "total_steps": total_steps,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("tool_executed").data(data).id(seq.to_string());
                        acc.apply(&LlmEvent::ToolExecuted { name: name.clone(), args: args.clone(), result: result.clone(), step, total_steps });
                    }
                    LlmEvent::Done(msgs, usage, _trace_id) => {
                        acc.apply(&LlmEvent::Done(msgs.clone(), usage, String::new()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        let _ = writer_tx.send(WriteOp::PersistDone {
                            sid: sid.clone(),
                            agent_id: String::new(), // snapshot before sending
                            finalized: finalized.clone(),
                            api_messages: msgs.iter().map(|m| crate::app::Message::from_json(m)).collect::<Vec<_>>(),
                            quality: None, // filled by separate quality step below
                        });
                        let done_json = serde_json::json!({"usage": usage, "quality": serde_json::json!(null), "session_id": &sid});
                        let data = serde_json::to_string(&done_json).unwrap_or_default();
                        sse_event = Event::default().event("done").data(data).id(seq.to_string());
                        keep_rx = false;
                    }
                    LlmEvent::Error(e) => {
                        acc.apply(&LlmEvent::Error(e.clone()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        let _ = writer_tx.send(WriteOp::MarkError {
                            sid: sid.clone(),
                            finalized,
                            err: e.clone(),
                        });
                        sse_event = Event::default().event("error").data(e).id(seq.to_string());
                        keep_rx = false;
                    }
                    LlmEvent::Token(t) => {
                        acc.apply(&LlmEvent::Token(t.clone()));
                        sse_event = Event::default().event("token").data(t).id(seq.to_string());
                    }
                    LlmEvent::Reasoning(t) => {
                        acc.apply(&LlmEvent::Reasoning(t.clone()));
                        sse_event = Event::default().event("reasoning").data(t).id(seq.to_string());
                    }
                    LlmEvent::Status(s) => {
                        sse_event = Event::default().event("status").data(s).id(seq.to_string());
                    }
                    LlmEvent::NewRound(_) => {
                        acc.apply(&event);
                        sse_event = Event::default().event("new_round").data("").id(seq.to_string());
                    }
                    LlmEvent::ImageGenerated { path, alt_text, format, width, height } => {
                        acc.apply(&LlmEvent::ImageGenerated { path: path.clone(), alt_text: alt_text.clone(), format: format.clone(), width, height });
                        let data = serde_json::to_string(&serde_json::json!({
                            "path": path, "alt_text": alt_text, "format": format, "width": width, "height": height,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("image_generated").data(data).id(seq.to_string());
                    }
                    LlmEvent::Evaluation { tool, valid, issues } => {
                        acc.apply(&LlmEvent::Evaluation { tool: tool.clone(), valid, issues: issues.clone() });
                        let data = serde_json::to_string(&serde_json::json!({
                            "tool": tool, "valid": valid, "issues": issues,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("evaluation").data(data).id(seq.to_string());
                    }
                    _ => continue,
                }
                seq += 1;
                let (next_rx, next_acc) = if keep_rx { (Some(rx), acc) } else { (None, MessageAccumulator::new()) };
                return Some((Ok::<_, Infallible>(sse_event), (next_rx, writer_tx, sid, next_acc, seq)));
            }
        },
    );
    Sse::new(stream)
}
```

&gt; **Note:** The `quality` evaluation currently happens **after** `Done` and uses `core.evaluate_completed_session`. That read+write of `core` cannot easily be moved off-thread without losing the synchronous ordering. Keep this as a single post-stream read in the `chat()` handler — outside the hot loop.

- [ ] **Step 4: Refactor `chat()` to snapshot, spawn writer, and call new `build_sse_stream`**

Replace the current `chat()` body (lines 197-276) with:

```rust
pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    let text = match body.get("message").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            let err = serde_json::json!({"success": false, "error": "Missing 'message'"});
            return (axum::http::StatusCode::BAD_REQUEST, axum::Json(err)).into_response();
        }
    };

    let agent_id = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    // ── Snapshot phase: hold write lock ONCE, capture everything we need. ──
    let sid = {
        let mut core = state.core.write().await;

        let session_id = core
            .session_mgr
            .current_id()
            .map(|id| id.to_string())
            .unwrap_or_default();

        if session_id.is_empty() {
            core.session_mgr.create_session_for(&agent_id, "default");
        }

        let sid = core
            .session_mgr
            .current_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                core.session_mgr.create_session_for(&agent_id, "default");
                core.session_mgr
                    .current_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            });

        // Persist user message
        let msg = crate::app::Message::User { text: text.clone() };
        if let Err(e) = core.session_mgr.persist_messages(&sid, &[msg]) {
            tracing::error!("user message persist failed: {}", e);
        }

        {
            let layered = core.agent_store.layered_memory_for_mut("default", &agent_id);
            layered.record_user_statement(&text);
        }

        // Build messages and spawn chat_loop
        let records = core.session_mgr.load_app_messages(&sid, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
        sid
    };
    // ── Lock released. Stream will not reacquire it inline. ──

    // Spawn the dedicated writer task and connect the stream to it.
    let writer_tx = spawn_writer(state.clone());
    build_sse_stream(rx, writer_tx, sid).into_response()
}
```

- [ ] **Step 5: Refactor `chat_stream()` and `chat_stream_resume()` similarly**

Apply the same snapshot+writer pattern to both legacy endpoints. The diff is mechanical: hold write lock briefly, snapshot `sid`, then call `build_sse_stream(rx, spawn_writer(state.clone()), sid)`. For reference, the new bodies are:

```rust
pub async fn chat_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    {
        let mut core = state.core.write().await;
        core.session_mgr.switch_to(&session_id);

        let agent_id = core
            .session_mgr
            .session_meta(&session_id)
            .map(|m| m.agent_id.clone())
            .unwrap_or_else(|| "default".to_string());

        let records = core.session_mgr.load_app_messages(&session_id, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
    }

    let writer_tx = spawn_writer(state.clone());
    build_sse_stream(rx, writer_tx, session_id).into_response()
}

pub async fn chat_stream_resume(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(query): Query<ResumeQuery>,
) -> axum::response::Response {
    let _cursor = query.cursor.unwrap_or(0);
    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    {
        let mut core = state.core.write().await;
        core.session_mgr.switch_to(&session_id);

        let agent_id = core
            .session_mgr
            .session_meta(&session_id)
            .map(|m| m.agent_id.clone())
            .unwrap_or_else(|| "default".to_string());

        let records = core.session_mgr.load_app_messages(&session_id, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
    }

    let writer_tx = spawn_writer(state.clone());
    build_sse_stream(rx, writer_tx, session_id).into_response()
}
```

&gt; **Note:** `crate::app::Message::from_json` may not exist yet. If the compile complains, fall back to passing `finalized: Vec<crate::app::Message>` directly from the accumulator (it already returns `Vec<crate::app::Message>`). The example above is intentionally tolerant — adjust to the actual signature `MessageAccumulator::into_messages()` returns.

- [ ] **Step 6: Verify compile + tests**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: same baseline pass/fail as before.

- [ ] **Step 7: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "perf(claw): snapshot state before SSE; writer-task pattern eliminates write-lock contention"
```

---

## Task 7: Add rate-limit middleware

Wire `tower_governor` to provide per-IP rate limiting on `/api/chat` and a per-user concurrency cap of 3 simultaneous chat streams.

**Files:**
- Create: `crates/claw/src/server/rate_limit.rs`
- Modify: `crates/claw/src/server/mod.rs` (declare module)
- Modify: `crates/claw/src/server/app.rs` (apply layer)

- [ ] **Step 1: Create the rate-limit module**

Create `crates/claw/src/server/rate_limit.rs`:

```rust
//! Rate-limit middleware for the chat endpoint.
//!
//! We layer two limiters:
//! 1. Per-IP global rate limit (10 req/s burst) — protects against floods.
//! 2. Per-user concurrency limit (3 concurrent in-flight chats) — protects
//!    the LLM provider budget from any single authenticated user.
//!
//! Both are gated behind the `dashboard` feature.

use std::sync::Arc;
use tokio::sync::Semaphore;

/// Builder for the per-IP governor config.
pub fn per_ip_governor() -> tower_governor::governor::GovernorConfig<
    tower_governor::key_extractor::SmartIpKeyExtractor,
    std::collections::hash_map::RandomState,
> {
    tower_governor::governor::GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .expect("valid governor config")
}

/// Shared per-user concurrency limiter.
///
/// Cloning is cheap (Arc inner). A single semaphore is shared by every chat
/// handler in the process; permits are released when the SSE future
/// completes (either by `Done`/`Error` or by client disconnect).
#[derive(Clone)]
pub struct UserConcurrencyLimit {
    inner: Arc<Semaphore>,
}

impl UserConcurrencyLimit {
    pub fn new(max: usize) -> Self {
        Self { inner: Arc::new(Semaphore::new(max)) }
    }

    /// Acquire a permit; the permit is released when the returned guard drops.
    pub async fn acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit, tokio::sync::AcquireError> {
        self.inner.clone().acquire_owned().await
    }
}
```

- [ ] **Step 2: Declare the module**

In `crates/claw/src/server/mod.rs`, add:

```rust
pub mod rate_limit;
```

- [ ] **Step 3: Wrap the chat route with governor + concurrency**

In `crates/claw/src/server/app.rs`, replace the line that registers `/api/chat` (currently line 58):

```rust
        .route("/api/chat", axum::routing::post(crate::server::routes::chat))
```

with:

```rust
        .route(
            "/api/chat",
            axum::routing::post(crate::server::routes::chat)
                .layer(tower_governor::GovernorLayer {
                    config: std::sync::Arc::new(crate::server::rate_limit::per_ip_governor()),
                }),
        )
```

- [ ] **Step 4: Add `UserConcurrencyLimit` to `AppState`**

Update `AppState` in `crates/claw/src/server/app.rs` (lines 6-20):

```rust
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<RwLock<i_rs_claw_core::core::AppCore>>,
    pub auth_token: String,
    pub chat_concurrency: crate::server::rate_limit::UserConcurrencyLimit,
}

impl AppState {
    pub fn new(core: i_rs_claw_core::core::AppCore, auth_token: String) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
            auth_token,
            chat_concurrency: crate::server::rate_limit::UserConcurrencyLimit::new(3),
        }
    }
}
```

- [ ] **Step 5: Use the limit in the `chat()` handler**

In `crates/claw/src/server/routes/chat.rs`, at the top of `chat()`, before snapshotting:

```rust
pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    // Reject if user already has 3 concurrent chats running.
    let _permit = match state.chat_concurrency.acquire().await {
        Ok(p) => p,
        Err(_) => {
            let err = serde_json::json!({"success": false, "error": "Too many concurrent chats (max 3)"});
            return (axum::http::StatusCode::TOO_MANY_REQUESTS, axum::Json(err)).into_response();
        }
    };

    // … existing body …
```

The `_permit` lives for the lifetime of the response future, so the slot is released when the SSE stream ends.

- [ ] **Step 6: Verify build**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors, 0 warnings.

- [ ] **Step 7: Run tests**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: baseline results unchanged (no test currently exercises the limit, but the test_app constructors must succeed).

If a test fails because it constructs `AppState` directly, update it to use `AppState::new()` (which now allocates the limiter).

- [ ] **Step 8: Commit**

```bash
git add crates/claw/src/server/rate_limit.rs crates/claw/src/server/mod.rs crates/claw/src/server/app.rs crates/claw/src/server/routes/chat.rs
git commit -m "feat(claw): rate-limit /api/chat (per-IP governor + per-user concurrency 3)"
```

---

## Task 8: Plumb `UserId` through handlers that hardcode `"default"`

The `UserId` extractor exists (see `crates/claw/src/server/middleware.rs:46-62`) and is inserted into request extensions for every authenticated request. But every handler still passes the literal string `"default"` to the underlying storage. Replace each one.

### Enumeration table

This is the full audit (per Grep on `crates/claw/src/server/routes/`) of every `"default"` literal that represents a user identifier. **Sentinel uses** (e.g. `id != "default"` to mean "the default agent") are NOT included.

| File | Line | Current code | Replacement |
|------|------|--------------|-------------|
| `chat.rs` | 38 | `record_tool_memory("default", …)` | `record_tool_memory(&user_id, …)` |
| `chat.rs` | 42 | `record_layered_tool_memory("default", …)` | `record_layered_tool_memory(&user_id, …)` |
| `chat.rs` | 76 | `memory_for_mut("default", …).flush()` | `memory_for_mut(&user_id, …).flush()` |
| `chat.rs` | 152 | `body…unwrap_or("default")` (agent_id fallback) | keep — agent_id is a *different* namespace |
| `chat.rs` | 165 | `create_session_for(&agent_id, "default")` (user_id arg) | `create_session_for(&agent_id, &user_id)` |
| `chat.rs` | 182 | `layered_memory_for_mut("default", …)` | `layered_memory_for_mut(&user_id, …)` |
| `chat.rs` | 212 | `unwrap_or("default")` (agent_id) | keep |
| `chat.rs` | 227 | `create_session_for(&agent_id, "default")` | `create_session_for(&agent_id, &user_id)` |
| `chat.rs` | 236 | `create_session_for(&agent_id, "default")` | `create_session_for(&agent_id, &user_id)` |
| `chat.rs` | 250 | `layered_memory_for_mut("default", …)` | `layered_memory_for_mut(&user_id, …)` |
| `chat.rs` | 293 | `unwrap_or_else(\|\| "default".to_string())` (agent_id) | keep |
| `chat.rs` | 336 | `unwrap_or_else(\|\| "default".to_string())` (agent_id) | keep |
| `tools.rs` | 49 | `skill_store_for("default", "default")` | `skill_store_for(&user_id.0, "default")` (2nd arg is agent) |
| `memory.rs` | 26 | `layered_memory_for("default", …)` | `layered_memory_for(&user_id.0, …)` |
| `memory.rs` | 27 | (continuation) | same |
| `memory.rs` | 48 | `unwrap_or("default")` (agent_id) | keep |
| `memory.rs` | 49 | `layered_memory_for_mut("default", …)` | `layered_memory_for_mut(&user_id.0, …)` |
| `memory.rs` | 62 | `unwrap_or("default")` (agent_id) | keep |
| `memory.rs` | 63 | `layered_memory_for("default", …)` | `layered_memory_for(&user_id.0, …)` |
| `sessions.rs` | 120 | `unwrap_or("default")` (agent_id) | keep |
| `sessions.rs` | 123 | `create_session_for(agent_id, "default")` | `create_session_for(agent_id, &user_id.0)` |
| `sessions.rs` | 200 | `unwrap_or_else(\|\| "default".to_string())` (agent_id) | keep |
| `sessions.rs` | 203 | `layered_memory_for_mut("default", …)` | `layered_memory_for_mut(&user_id.0, …)` |
| `sessions.rs` | 227 | `unwrap_or_else(\|\| "default".to_string())` (agent_id) | keep |
| `sessions.rs` | 231 | `memory_for_mut("default", …)` | `memory_for_mut(&user_id.0, …)` |
| `sessions.rs` | 243 | `memory_for_mut("default", …)` | `memory_for_mut(&user_id.0, …)` |
| `mcp_config.rs` | 23 | `load_for("default", …)` | `load_for(&user_id.0, …)` |
| `mcp_config.rs` | 60 | `user_id: "default".to_string()` | `user_id: user_id.0.clone()` |
| `mcp_config.rs` | 93 | `user_id: "default".to_string()` | `user_id: user_id.0.clone()` |
| `mcp_config.rs` | 129 | `.delete("default", …)` | `.delete(&user_id.0, …)` |
| `agents.rs` | 261 | `user_id: "default".into()` (upsert row) | `user_id: user_id.0.clone()` |
| `agents.rs` | 308 | `.delete("default", …)` | `.delete(&user_id.0, …)` |

**Strategy:** Each handler that needs `user_id` adds `user_id: crate::server::middleware::UserId` to its axum extractor list. The handler body then uses `&user_id.0` instead of `"default"`.

**Files:**
- Modify: `crates/claw/src/server/routes/{chat,tools,memory,sessions,mcp_config,agents}.rs`

- [ ] **Step 1: `chat.rs` — add `UserId` extractor to `chat`, `send_message`, `chat_stream`, `chat_stream_resume`**

For each handler in `chat.rs`, add `crate::server::middleware::UserId` as an extractor parameter. For `chat()`:

```rust
pub async fn chat(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
    Json(body): Json<Value>,
) -> axum::response::Response {
    let user_id = user_id.0;
    // … existing body, replacing every "default" user-id literal with &user_id …
```

Then thread `user_id: String` into:
- the snapshot block (use `&user_id` in `create_session_for`, `layered_memory_for_mut`)
- `spawn_writer` — extend `WriteOp` variants to carry a `user_id: String` field
- `build_sse_stream` signature — accept `user_id: String`

This is a mechanical find/replace guided by the table above. The writer task uses the captured `user_id` for all storage calls.

- [ ] **Step 2: `tools.rs` — `list_skills`**

Replace the `list_skills` handler signature + body:

```rust
pub async fn list_skills(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
) -> Json<super::ApiResponse<Vec<i_rs_claw_core::skill_store::SkillDefinition>>> {
    let core = state.core.read().await;
    let store = core.agent_store.skill_store_for(&user_id.0, "default");
    // … rest unchanged …
```

- [ ] **Step 3: `memory.rs` — all three handlers**

`get_layered_memory`, `clear_layered_memory`, `search_layered_memory` all take `Query`. Add `user_id`:

```rust
pub async fn get_layered_memory(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
    Query(query): Query<MemoryQuery>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let agent_id = query.agent_id.as_deref().unwrap_or("default");
    let layered = core.agent_store.layered_memory_for(&user_id.0, agent_id);
    // … rest unchanged, replacing remaining "default" user-id sites …
```

Repeat for `clear_layered_memory` and `search_layered_memory` per the enumeration table.

- [ ] **Step 4: `sessions.rs` — `create_session`, `delete_session`, `post_session_feedback`**

For each handler in the table, add `user_id` extractor and replace the literal:

```rust
pub async fn create_session(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
    body: Option<Json<Value>>,
) -> Json<super::ApiResponse<Value>> {
    let agent_id = body
        .as_ref()
        .and_then(|b| b.get("agent_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let mut core = state.core.write().await;
    let id = core.session_mgr.create_session_for(agent_id, &user_id.0);
    // … rest unchanged …
```

Apply the same pattern to `delete_session` (replace `"default"` at line 203) and `post_session_feedback` (lines 231, 243).

- [ ] **Step 5: `mcp_config.rs` — all four handlers**

Each of `list_mcp_configs`, `create_mcp_config`, `update_mcp_config`, `delete_mcp_config` needs the user_id. Example:

```rust
pub async fn list_mcp_configs(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
    Query(query): Query<McpListQuery>,
) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    match core
        .config_store
        .mcp_servers
        .load_for(&user_id.0, query.agent_id.as_deref())
        .await
    {
        // … unchanged …
```

For the write paths (`create_mcp_config`, `update_mcp_config`), replace `user_id: "default".to_string()` with `user_id: user_id.0.clone()`. For `delete_mcp_config`, replace `.delete("default", …)` with `.delete(&user_id.0, …)`.

- [ ] **Step 6: `agents.rs` — `create_agent`, `delete_agent`**

`create_agent` already has the structure; just add the extractor and replace line 261:

```rust
pub async fn create_agent(
    State(state): State<AppState>,
    user_id: crate::server::middleware::UserId,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    // … existing body …
    let _ = core.config_store.agent_configs.upsert(&i_rs_claw_core::storage::config_store::AgentConfigRow {
        user_id: user_id.0.clone(),
        // … rest unchanged …
```

`delete_agent` (line 308): add `user_id` extractor, replace `.delete("default", &id)` with `.delete(&user_id.0, &id)`.

- [ ] **Step 7: Update tests that construct handlers directly**

The integration tests in `crates/claw/src/server/routes/mod.rs` call handlers without a `UserId` extractor. There are two options:

A) **Recommended:** keep the existing tests passing by constructing a `UserId("default".to_string())` test helper and passing it explicitly. This requires tests to be in the same module (they are).

B) Inject `UserId` via `axum::extract::Request` extensions in test helpers.

Choose **(A)**. Open `crates/claw/src/server/routes/mod.rs` and find every direct call to a handler that now takes `UserId`. Add the `UserId("default".to_string())` argument. Example for `chat`:

```rust
// Before:
let resp = chat(State(state), Json(json!({"message": "hi", "agent_id": "default"}))).await;

// After:
let resp = chat(
    State(state),
    crate::server::middleware::UserId("default".to_string()),
    Json(json!({"message": "hi", "agent_id": "default"})),
).await;
```

- [ ] **Step 8: Verify compile**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

- [ ] **Step 9: Run tests**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: same baseline pass/fail.

- [ ] **Step 10: Commit**

```bash
git add crates/claw/src/server/routes/
git commit -m "refactor(claw): plumb real UserId through handlers instead of hardcoded 'default'"
```

---

## Task 9: Final verification + commits

**Files:** none (verification only)

- [ ] **Step 1: Full workspace check**

Run: `cargo check --workspace`
Expected: 0 errors, 0 warnings.

- [ ] **Step 2: Clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: 0 warnings.

If clippy flags legitimate nits introduced by this plan (extra `let _ = …`, etc.), fix them in place. Do not `#[allow]` to silence.

- [ ] **Step 3: Format check**

Run: `cargo fmt --all --check`
Expected: clean.

- [ ] **Step 4: Test `i-rs-claw-core`**

Run: `cargo test -p i-rs-claw-core`
Expected: PASS — all existing tests + the new ones in Task 2 (hashmap lookup) and Task 3 (ToolResult classification).

- [ ] **Step 5: Test `i-rs-claw` (dashboard)**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: same baseline as the pre-plan state. If new failures appear, they are regressions from this plan — investigate before declaring done.

- [ ] **Step 6: Build the release artifact**

Run: `cargo build -p i-rs-claw --features dashboard --release`
Expected: success.

- [ ] **Step 7: If anything was fixed during verification, add a cleanup commit**

```bash
git add -u
git commit -m "chore: verification fixes from claw perf/api hardening"
```

Otherwise skip — no empty commits.

---

## Verification

After all tasks land, the following statements must be true:

1. **`ToolRegistry.get(name)`** is `O(1)` — verified by `test_hashmap_lookup_three_tools` (Task 2).
2. **`ToolResult::from(s)`** correctly classifies Ok / Err / Blocked — verified by 5 unit tests (Task 3).
3. **No `starts_with("错误")` in `chat.rs`** — run `rg 'starts_with\("错误"\)' crates/claw/src/` and expect zero hits.
4. **SSE write-lock hold time** — `build_sse_stream` body contains no `state.core.write().await` (only `spawn_writer` does, off the hot path). Verify by `rg 'state\.core\.write\(\)\.await' crates/claw/src/server/routes/chat.rs` — should only match inside `spawn_writer` and the snapshot block, never inside the stream closure.
5. **Rate limit active** — `rg 'tower_governor' crates/claw/src/` returns hits in `rate_limit.rs`, `app.rs`, `Cargo.toml`.
6. **UserId plumbing** — `rg '"default"' crates/claw/src/server/routes/` returns hits only in:
   - Tests (where `"default"` is the legitimate test user)
   - Sentinel comparisons (`id != "default"`, agent-id defaults)
   - The `agent_id.unwrap_or("default")` patterns (kept per enumeration table)
   No literal `"default"` reaches a `*_for("default", …)` storage call.
7. **`cargo check --workspace`** — 0 errors, 0 warnings.
8. **`cargo clippy --workspace -- -D warnings`** — 0 warnings.
9. **All claw-core tests pass.**
10. **Dashboard test suite** has the same baseline as before this plan (no new failures).

---

## Estimated time

- Task 1 (deps): 15 min
- Task 2 (IndexMap): 45 min
- Task 3 (ToolResultKind): 30 min
- Task 4 (executor emit): 45 min
- Task 5 (chat.rs sniffing): 15 min
- Task 6 (SSE snapshot + writer): 3 hours (largest task, biggest blast radius)
- Task 7 (rate limit): 1 hour
- Task 8 (UserId plumbing): 2 hours (mostly mechanical, but easy to miss a site)
- Task 9 (verification): 30 min

**Total: 8-9 hours of focused engineering work = ~1-2 working days.**

Tasks 1-5 are independent and could be parallelised across two engineers if desired. Task 6 must come after 5 (it builds on the same file). Task 8 can run in parallel with Task 7 once Task 6 lands. Task 9 is strictly last.
