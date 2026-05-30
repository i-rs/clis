# Claw Architecture Audit Report

> Date: 2026-05-31
> Status: Pending fixes
> Scope: `crates/claw/src/` — 7-dimension architectural analysis

---

## Executive Summary

| Dimension | Grade | Key Issue |
|-----------|-------|-----------|
| Task Loop | B+ | Round counting includes transient errors; no cancellation |
| Tool Invocation | B- | **P0: MCP tool deadlock risk** |
| Context Management | C+ | ContextManager is mostly decorative |
| State Recording | B | Stats agent_id/cost hardcoded |
| Execution Logging | C | In-memory only; no response body |
| Failure Retry | B- | **P0: Error detection misses timeout/MCP/validation errors** |
| Result Evaluation | D+ | No validation, no dedup, no quality scoring |

---

## P0 Fixes (Must Fix)

### P0-1: McpToolWrapper deadlock

**File:** `tools/mcp_tools.rs:67`

```rust
// CURRENT (deadlock risk in async context):
let result = self.client.call_tool(&self.definition.name, args)?;

// FIX:
let result = self.client.call_tool_async(&self.definition.name, args).await?;
```

`call_tool` uses `rt.block_on()` inside a `tokio::spawn` task — classic deadlock. `call_tool_async` already exists using `spawn_blocking`.

### P0-2: Error detection misses most error types

**File:** `core/engine/mod.rs:125`

```rust
// CURRENT (only matches "执行错误:" and plain "错误:"):
if result.result.starts_with("错误:") {
```

`ClawError::Display` formats:
- `Execution` → `"执行错误: ..."` ← detected
- `Validation` → `"参数错误: ..."` ← **NOT detected**
- `Timeout` → `"超时: ..."` ← **NOT detected**
- `Mcp` → `"MCP 错误: ..."` ← **NOT detected**
- `Network` → `"网络错误: ..."` ← **NOT detected**
- `NotFound` → `"未找到: ..."` ← **NOT detected**

**Fix options:**
1. Add `is_error: bool` to `ToolCallResult`, set by `execute_tool_call` based on `Result` type
2. Match all prefixes: `starts_with("错误:") || starts_with("执行错误:") || starts_with("参数错误:") || starts_with("超时:") || starts_with("MCP 错误:") || starts_with("网络错误:") || starts_with("未找到:")`

### P0-3: Panicked tool task produces misaligned results

**File:** `core/executor.rs:125-128`

When a `tokio::spawn` task panics, the `JoinError` is logged but no `ToolCallResult` is added. This causes:
- `all_results` has fewer entries than input calls
- Tool call IDs don't match their results
- LLM receives broken tool_call_id → content mapping

**Fix:** Need to pass `ToolCallAcc` into the spawned task and construct a placeholder `ToolCallResult` on panic.

---

## P1 Fixes

### P1-1: Stats agent_id hardcoded "default"

**File:** `providers/sse.rs:230`, `providers/anthropic.rs:429`

```rust
agent_id: "default".to_string(), // Always "default" regardless of actual agent
```

**Fix:** Store `agent_id` in provider struct, use it in UsageRecord emission.

### P1-2: Stats estimated_cost_usd always 0.0

**File:** `providers/sse.rs:241`

```rust
estimated_cost_usd: 0.0, // Comment says "Estimated by StatsManager" but it's not
```

**Fix:** Re-estimate cost in `StatsManager::record()` before buffering:
```rust
record.estimated_cost_usd = self.pricing.estimate(&record.model, ...);
```

### P1-3: Stats react_rounds always 0

**File:** `providers/sse.rs:238`

`TokenRecord` is emitted from inside the provider, not from `chat_loop` where round count is tracked.

**Fix:** Emit `UsageRecord` from `chat_loop` instead of from provider streaming code.

---

## P2 Fixes

### P2-1: Tool retry counters never reset on success

**File:** `core/engine/mod.rs:50`

```rust
let mut retry_counts: HashMap<String, u32> = HashMap::new();
// Never cleared when a tool succeeds
```

If tool "weight" fails twice (hitting max_retries), later calls to "weight" in the same session immediately trigger the reflection prompt without retrying.

**Fix:** Remove tool name from `retry_counts` when it succeeds:
```rust
for result in &all_results {
    if !is_error(&result.result) {
        retry_counts.remove(&result.call.name);
    }
}
```

### P2-2: Round counting includes transient error rounds

**File:** `core/engine/mod.rs:64`

```rust
round_count += 1; // Increments even on transient provider errors
```

During network instability, transient errors consume round budget, potentially terminating the conversation before `max_react_rounds` real LLM calls.

**Fix:** Only increment on successful LLM responses.

### P2-3: Provider retry uses linear backoff

**File:** `core/engine/mod.rs:176`

```rust
let wait_secs = 3 * consecutive_provider_errors; // 3s, 6s — too aggressive
```

**Fix:** Use exponential backoff:
```rust
let wait_secs = 3 * (1 << consecutive_provider_errors.min(3)); // 3, 6, 12, 24s
```

### P2-4: Off-by-one in retry count semantics

**File:** `core/engine/mod.rs:128`

```rust
if *count <= max_retries { should_retry = true; }
// max_retries=2 means 3 total attempts (1 + 2 retries)
```

**Fix:** Use `<` instead of `<=`, or rename to `max_total_attempts`.

### P2-5: No cancellation support

**File:** `chat_loop` function signature

No `CancellationToken` parameter. User pressing Ctrl+C exits TUI but spawned task continues running.

**Fix:** Add `cancel: tokio_util::sync::CancellationToken` parameter, check at top of each iteration.

---

## P3 Fixes

### P3-1: ToolRegistry recreated per tool call

**File:** `core/engine/execution.rs:14`

```rust
let registry = crate::tools::ToolRegistry::with_skills(skills); // New registry every call
```

Allocates 10+ `Box<dyn ClawTool>` per invocation.

**Fix:** Cache in `ToolCallExecutor`.

### P3-2: ContextManager barely used

**Files:** `core/context.rs`, `core/engine/mod.rs`

- `is_near_limit()` never called from `chat_loop`
- `context_advisory()` never called anywhere
- `compress()` rarely triggers for 128K+ models
- Token estimation (CJK ×1.5 + ASCII /4) can be off 2-3x

**Fix:** Call `is_near_limit()` before each `stream_chat()` in `chat_loop`, with auto-compression trigger.

### P3-3: No persistent execution log

All logs in `app.http_logs` vector — lost on exit. No file-based structured logging.

**Fix:** Add `tracing_subscriber` file appender to `{claw_dir}/logs/{date}.log`.

### P3-4: No tool execution timing

`LlmEvent::ToolExecuted` doesn't include duration.

**Fix:** Add `duration_ms: u64` field.

### P3-5: Tool call argument JSON silently falls back to {}

**File:** `providers/sse.rs:257-258`

```rust
let args: Value = serde_json::from_str(&tc.arguments).unwrap_or(serde_json::json!({}));
```

**Fix:** Log warning and include parse error info in the args.

### P3-6: No tool result deduplication

LLM frequently calls the same tool with same args in ReAct loops.

**Fix:** Cache results keyed by `(tool_name, args_hash)` with short TTL.

### P3-7: UsageRecord success always true

**File:** `providers/sse.rs:239`

Failed API calls don't emit UsageRecord, so there's no record of failures in stats.

### P3-8: Session index.json rewritten every append

**File:** `session.rs:284`

Each `append_message` triggers `save_index()` — O(n) per message with many sessions.

### P3-9: Startup doesn't call stats cleanup

**File:** `core/mod.rs:172-189`

`AppCore::new` creates StatsManager but never calls `cleanup(keep_days)`.

---

## Cross-Cutting Observations

### Error Handling Inconsistency

The codebase mixes three error representations:
1. `Result<String, ClawError>` — structured, but only used at tool level
2. `Result<String, String>` — used in some tool implementations
3. Bare `String` with "错误:" prefix — used for error detection in chat_loop

This is the root cause of P0-2. **Recommendation:** Standardize on `Result<String, ClawError>` everywhere, add `is_error: bool` to `ToolCallResult`.

### No Observability Pipeline

- HTTP request/response → in-memory vector only
- Tool execution → no persistent record
- No structured error codes for filtering
- No performance metrics (latency, throughput)

### ContextManager Is Decorative

The `ContextManager` module exists with `is_near_limit()`, `compress()`, `context_advisory()` methods, but:
- None are called from the critical path (`chat_loop`)
- `max_conversation_turns` config bypasses it entirely
- Token estimation is unreliable for mixed CJK/ASCII

Either integrate it fully or remove it to avoid confusion.
