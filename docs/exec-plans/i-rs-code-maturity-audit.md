# i-rs-code Maturity Fix Plan

> 44 issues identified. Grouped by severity. Each item has a unique ID, file location, and clear acceptance criteria.

## Status: In Progress

---

## CRITICAL (2 items) — Blocks core functionality

### C1. MCP tool schemas never sent to LLM
- **File**: `tools/mod.rs:99-101`, `tools/mcp.rs:87-115`, `mcp.rs:19-24`
- **Problem**: `ToolRegistry::schemas()` only returns built-in tool schemas. After `mcp_connect` adds MCP tools to `MCP_TOOL_REGISTRY`, the LLM never receives their schemas. `McpToolWrapper::schema()` returns a generic `{"input": {"type": "object"}}` instead of the actual `input_schema` from `McpToolDef`.
- **Fix**:
  1. Store `input_schema` from `McpToolDef` in `McpToolWrapper`
  2. `McpToolWrapper::schema()` returns actual schema using `input_schema`
  3. `ToolRegistry::schemas()` calls `all_tools()` instead of only iterating `self.tools`
- **Accept**: After `mcp_connect`, `tool_defs` in the next LLM request includes MCP tool schemas with real parameters.

### C2. Global mutable singleton architecture (Runtime)
- **File**: `runtime.rs`, `mcp.rs:76`, `lsp.rs`, `pty.rs`, `tools/web.rs:44-49`
- **Problem**: `LazyLock<Mutex<Option<RuntimeInner>>>` + separate `LazyLock` for mcp/pty/lsp/web. Cannot test in isolation, cannot run multi-agent, `RuntimeInner` fields `mcp_manager`/`pty_manager` are dead (lines 11-12 never accessed).
- **Fix**:
  1. Remove dead fields from `RuntimeInner` (mcp_manager, pty_manager, lsp_session, lsp_initialized)
  2. Keep the single `RUNTIME` mutex but consolidate the scattered `LazyLock` singletons
  3. Add `McpManager`/`PtyManager`/`LspSession` into `RuntimeInner` as `AsyncMutex` wrappers
  4. `reset_for_testing()` already resets all state — just clean up the duplication
- **Accept**: One `RUNTIME` holds all state. No duplicate singletons. `cargo test -p i-rs-code` passes.

---

## HIGH (8 items) — Seriously affects functionality

### H1. Session resume loses LLM conversation history
- **File**: `main.rs:59-63`, `session.rs:6-11`, `tui.rs:512-532`
- **Problem**: `Session` only stores `AgentMessage` (display). On resume, `app.agent_messages` is empty so the LLM gets no history. `run_streaming_agent` creates fresh `Agent` + `ToolRegistry` per prompt.
- **Fix**:
  1. Add `agent_messages: Vec<LlmMessage>` field to `Session` struct (serialize/deserialize)
  2. On save: store both `messages` (display) and `agent_messages` (protocol)
  3. On resume in `main.rs`: load `agent_messages` into `app.agent_messages`
  4. Reuse `ToolRegistry` across the session instead of creating new one per prompt
- **Accept**: `--session <id>` resumes with full LLM context. Conversation continues correctly.

### H2. Sequential tool execution (no parallelism)
- **File**: `agent/tool_exec.rs:21-57`
- **Problem**: `for tc in pending_tool_calls` awaits each tool one at a time. Independent reads/greps/globs could run concurrently.
- **Fix**:
  1. Spawn all tool tasks via `tokio::spawn` upfront (already done at line 33)
  2. Collect all `JoinHandle`s and `join_all()` instead of awaiting each immediately
  3. Reconstruct results in original order
  4. Keep retry logic: only retry individual failed tools, not the whole batch
- **Accept**: Multiple independent tool calls execute concurrently. Results returned in original order. Tests pass.

### H3. Bash whitelist bypasses
- **File**: `tools/bash.rs:63-73, 109-124`
- **Problem**: Pattern matching checks exact substrings — `rm  -rf  /` (extra spaces) bypasses. `while` loop stripping `./` can be tricked. Local scripts (`./script.sh`) allowed with no content validation.
- **Fix**:
  1. Normalize command: collapse multiple spaces, strip leading/trailing whitespace before matching
  2. Use regex-based pattern matching for dangerous patterns (e.g., `rm\s+-[a-zA-Z]*f\s+/`)
  3. Block `rm` with `-f` or `-rf` flags targeting `/`, `~`, `*`
  4. Keep local script allowance but add a comment noting it's intentional
- **Accept**: `rm  -rf  /` blocked. `rm -r -f /` blocked. All existing tests pass + new bypass tests.

### H4. API key stored in plaintext
- **File**: `main.rs:149-160`, `config.rs:212-220`
- **Problem**: Config wizard shows masked key but writes plaintext to `config.toml`. No keychain integration.
- **Fix**:
  1. Add warning when saving API key to config: "API key stored in plaintext. Consider using env var I_RS_CODE_API_KEY."
  2. Support `I_RS_CODE_API_KEY` environment variable override in `Config::load()`
  3. In `save()`, mask the api_key field if it came from env var
- **Accept**: Env var override works. Warning printed on config init. Key not written to disk if from env.

### H5. Sync/Async mixing blocks tokio runtime
- **File**: `tools/filesystem.rs:46-69,146-148`, `config.rs:93-146`, `tui.rs:554`
- **Problem**: `std::fs::read_to_string`, `std::fs::canonicalize`, `std::path::Path::exists` inside async functions. Blocks the tokio runtime thread.
- **Fix**:
  1. Replace `std::fs::read_to_string` → `tokio::fs::read_to_string`
  2. Replace `std::fs::canonicalize` → `tokio::fs::canonicalize` (or wrap in `spawn_blocking`)
  3. Replace `std::fs::write` → `tokio::fs::write`
  4. `expand_file_refs` in `tui.rs:554`: wrap `std::fs::read_to_string` in `spawn_blocking`
  5. `config.rs:93-146` `ProjectInfo::detect()`: wrap sync I/O in `spawn_blocking` or make `detect()` async
- **Accept**: No `std::fs` calls in async functions. Tests pass.

### H6. Context compression loses tool details
- **File**: `agent/context.rs:144-205`
- **Problem**: `compress_core()` strips all `ToolCall`/`Tool` messages (line 185). LLM loses exact file content, error messages, and file lists.
- **Fix**:
  1. Keep last 8 tool call/result pairs fully intact (already partially done)
  2. For older pairs: preserve tool name + first 500 chars of result (not just "5000 chars" summary)
  3. Always preserve error lines fully in compressed summaries
  4. Keep file paths as a cross-reference list at the top of compressed context
- **Accept**: After compression, LLM can still reference exact error messages and file paths from earlier tool results.

### H7. TUI event channel drops events silently
- **File**: `tui.rs:93-95`
- **Problem**: `event_rx.try_recv()` only processes one event per frame. With channel capacity 256, high-throughput streaming can drop events.
- **Fix**:
  1. Drain all pending events in the event poll loop: `while let Ok(event) = event_rx.try_recv()`
  2. Increase channel capacity to 512 or 1024
  3. Terminal render at 20fps (50ms poll) is sufficient for draining
- **Accept**: No events dropped during fast streaming. All tokens appear in final output.

### H8. Undo only stores content, no git baseline
- **File**: `tui.rs:373-383`
- **Problem**: `app.last_file_states` is a `Vec<(path, content)>` stack. No git stash/commit. Multi-step rollback impossible. Revert to pre-edit only (not pre-session).
- **Fix**:
  1. Before agent runs, optionally auto-commit (if git repo + uncommitted changes)
  2. Store commit hash as undo baseline
  3. Ctrl+Z reverts file to pre-edit state (keep current behavior as quick undo)
  4. Add Ctrl+Shift+Z to revert to git baseline (session start state)
- **Accept**: Ctrl+Z reverts last edit. Ctrl+Shift+Z reverts all edits since session start (if git available).

---

## MEDIUM (25 items) — Impacts quality/reliability

### M1. No parallel tool execution retry for batch
- **File**: `agent/tool_exec.rs:21-57`
- **Fix**: When parallel execution (H2) is done, retry logic must handle individual tool failures within the batch, not stop the entire batch.
- **Linked**: H2

### M2. No streaming diff preview
- **File**: `tui.rs:146-173`, `agent/output.rs`
- **Fix**: When `ToolCallStart` with name="edit", show a live diff panel. Parse `old_string`/`new_string` from args. Render with `similar` crate (already a dep).
- **Accept**: User sees diff as LLM generates edit arguments.

### M3. No tool result caching
- **File**: `agent/tool_exec.rs`
- **Fix**: Add `ToolResultCache` (HashMap with content-hash key). Cache `read`, `glob`, `grep` results. Invalidate on `write`/`edit`/`delete`.
- **Accept**: Repeated reads of same file content return cached result. Write invalidates cache.

### M4. Cost tracking uses wrong token estimation
- **File**: `agent/mod.rs:39-42`, `runtime.rs:112-130`
- **Problem**: `(max_cost * 1_000_000.0) as u64` assumes all tokens cost $0.000001. No per-model pricing.
- **Fix**:
  1. Add `ModelPricing` struct with `input_per_1k` and `output_per_1k` fields
  2. Built-in pricing table for common models (gpt-4o, gpt-4o-mini, claude-sonnet, etc.)
  3. Calculate real dollar cost in `add_usage()`
  4. Compare against `max_cost_per_session` in dollars
- **Accept**: Budget limit works in dollars, not arbitrary token counts.

### M5. No test-run auto-fix loop
- **File**: `tools/verify.rs`, `agent/engine.rs`
- **Fix**: After verify returns FAIL, inject a system message: "The verification failed. Please fix the errors and re-run verify." The LLM already does this via ReAct, but we can make it more explicit.
- **Accept**: After failed verify, next round includes fix instruction.

### M6. No file watching
- **File**: New module or in `runtime.rs`
- **Fix**: Add optional `notify` crate dependency. Watch workspace for file changes. Emit `FileChanged` event (variant exists but unused). Low priority — skip for now.
- **Accept**: External file changes detected during session.

### M7. Model routing not connected
- **File**: `router.rs:31-59`, `agent/engine.rs`
- **Problem**: `classify_complexity()` and `recommended_model()` exist but `recommended_model()` is never called. Engine always uses configured single model.
- **Fix**: In `react_loop_inner`, before first `provider.stream()`, check `classify_complexity`. If model differs from current, log a suggestion (don't auto-switch — would need multi-provider setup).
- **Accept**: Complexity classification logged. Model suggestion visible in verbose mode.

### M8. No proper retry with exponential backoff (provider)
- **File**: `agent/engine.rs:156-162`
- **Problem**: Linear delay `5 * provider_errors` seconds. No jitter.
- **Fix**: `delay = min(base * 2^attempts + random_jitter, max_delay)`. Reset on success.
- **Accept**: Retry delays increase exponentially with jitter.

### M9. Dead code: `#![allow(dead_code)]` at crate level
- **File**: `main.rs:2`
- **Fix**: Remove the crate-level `#![allow(dead_code)]`. Fix individual dead code items one by one.
- **Linked**: M10

### M10. Dead code: `FileChanged` event variant unused
- **File**: `agent/event.rs:9-10`
- **Fix**: Either use it (M6 file watching) or remove it. If removing, also remove handler in `tui.rs:185-187`.
- **Linked**: M9

### M11. Dead code: `chat()` method on LlmProvider
- **File**: `provider/mod.rs:61`, `provider/openai.rs`, `provider/anthropic.rs:344-409`
- **Problem**: `chat()` only used by `generate_plan()` in engine but `generate_plan()` now uses `stream()`. Dead code.
- **Fix**: Remove `chat()` from trait and all implementations. Remove unused import/code in `openai.rs` and `anthropic.rs`.
- **Accept**: `LlmProvider` trait only has `stream()`.

### M12. Anthropic thinking not streamed
- **File**: `provider/anthropic.rs:293-303`
- **Problem**: `thinking` block type mapped from `reasoning` content in build_messages, but streaming `thinking` deltas not mapped to `StreamEventKind::Reasoning`.
- **Fix**: In streaming handler, detect `thinking` block type in `ContentBlockStart`. Emit `StreamEventKind::Reasoning` for `thinking_delta` events.
- **Accept**: Anthropic extended thinking shown as reasoning in TUI.

### M13. Tokenizer always uses cl100k_base
- **File**: `tokenizer.rs`
- **Fix**: Add `estimate_tokens_for_model(model_name, messages)` that selects tokenizer based on provider. cl100k for OpenAI, rough char/4 for Anthropic, char/4 for Ollama.
- **Accept**: Context estimates roughly correct for non-OpenAI providers.

### M14. Grep results hard-capped at 50 lines
- **File**: `tools/filesystem.rs:326`
- **Fix**: Change to configurable limit (default 200). Add `"truncated": true` indicator to output when hit.
- **Accept**: LLM knows when results are truncated. Default limit higher.

### M15. Verify tool has no timeout
- **File**: `tools/verify.rs:48-111`
- **Fix**: Wrap each `Command::output()` in `tokio::time::timeout()` with 300s default. Add `timeout_secs` parameter.
- **Accept**: Verify doesn't hang forever on large workspaces.

### M16. No config validation
- **File**: `config.rs:171-186`
- **Fix**: After `toml::from_str`, validate: provider is one of [openai, anthropic, ollama], max_rounds > 0, tool_timeout_secs > 0. Print warnings for suspicious values.
- **Accept**: Invalid config caught early with helpful error messages.

### M17. MCP no initialized notification
- **File**: `mcp.rs:46-49`
- **Problem**: Sends `initialize` but never sends `notifications/initialized` per MCP spec.
- **Fix**: After receiving initialize response, send `notifications/initialized` with empty params.
- **Accept**: MCP spec compliant. Servers that require initialized notification work correctly.

### M18. MCP single-threaded tool calls
- **File**: `mcp.rs:156-162`
- **Problem**: `call_tool(&mut self)` takes `&mut self` on `McpManager`, locking the entire connections map. Only one MCP call at a time.
- **Fix**: Use `HashMap<String, Arc<Mutex<McpConnection>>>` in `McpManager`. `call_tool` locks only the specific connection, not the map.
- **Accept**: Parallel MCP calls to different servers don't block each other.

### M19. Batch edit doesn't restore on failure
- **File**: `tools/batch_edit.rs:64-88`
- **Problem**: Writes `.batchtmp` files. On error, some files renamed, some not — inconsistent state. Never restores backups.
- **Fix**: On any failure, attempt to restore all written files from `.batchtmp` backups. Clean up `.batchtmp` files in finally block.
- **Accept**: Failed batch edit leaves filesystem in consistent state.

### M20. No TUI input history
- **File**: `tui/input.rs`
- **Fix**: Add `history: Vec<String>` and `history_index: usize` to input state. Up/Down arrows recall previous prompts (only in Idle mode with empty input).
- **Accept**: Up arrow shows previous prompt. Down arrow goes forward.

### M21. Debug panel is dead feature
- **File**: `tui.rs:398-418`, `tui/ui.rs`
- **Fix**: Either implement debug panel rendering in `ui.rs` (show debug log entries) or remove `show_debug` flag and all related keybindings (Ctrl+D, Ctrl+L, scroll in debug mode).
- **Accept**: Debug panel either works or is removed entirely.

### M22. Ctrl+C behavior inconsistent
- **File**: `tui.rs:313-332, 422-423`
- **Fix**: In Idle mode, Ctrl+C should do nothing (or show "press Ctrl+D to quit"). First Ctrl+C in Waiting cancels. Second Ctrl+C in Waiting force-quits.
- **Accept**: Consistent Ctrl+C behavior across modes.

### M23. Cross-session memory no relevance scoring
- **File**: `memory.rs`
- **Fix**: Add recency decay: `score = count * (1.0 / (1 + days_since_last_use))`. Add per-project isolation: store project hash in memory key.
- **Accept**: Memory entries decay over time. Different projects don't pollute each other.

### M24. No session expiry/cleanup
- **File**: `session.rs`
- **Fix**: Add `Session::cleanup(sessions_dir, max_age_days)` that deletes old sessions. Call on startup.
- **Accept**: Sessions older than 30 days auto-cleaned.

### M25. Grep regex not pre-compiled
- **File**: `tools/web.rs:68-69`
- **Fix**: Use `once_cell::sync::Lazy` or `std::sync::LazyLock` for regex compilation.
- **Accept**: Regex compiled once, not per call.

---

## LOW (9 items) — Minor/cosmetic

### L1. Context cache stale on branch change
- **File**: `prompt.rs:11`
- **Fix**: Add git branch hash to cache key. Invalidate on branch change.
- **Accept**: Branch switch refreshes context.

### L2. `server_count()` always returns 0
- **File**: `mcp.rs:65-67`
- **Fix**: Either implement properly or remove the method.
- **Accept**: No dead code.

### L3. Unused variable `_stop_reason` in anthropic.rs
- **File**: `provider/anthropic.rs:376`
- **Fix**: Remove the unused variable.
- **Accept**: No compiler warnings.

### L4. Batch edit double `remove_file` on tmp
- **File**: `tools/batch_edit.rs:85,88`
- **Fix**: Remove the duplicate `remove_file` call.
- **Accept**: Clean error path.

### L5. Mutex poisoning in tools/filesystem.rs
- **File**: `tools/filesystem.rs:324`
- **Fix**: Use `.lock().unwrap_or_else(|e| e.into_inner())` to recover from poisoned mutex.
- **Accept**: Tool doesn't panic on previous thread panic.

### L6. Mutex poisoning in runtime.rs
- **File**: `runtime.rs:48`
- **Fix**: Use `.lock().unwrap_or_else(|e| e.into_inner())` in `with_runtime()`.
- **Accept**: Runtime doesn't permanently break on panic.

### L7. `RuntimeInner.last_api_call` redundant with `rate_limit_wait`
- **File**: `runtime.rs:19`
- **Fix**: Already properly used. Just note that it's correct.
- **Accept**: No action needed.

### L8. No `.gitignore` for `~/.i-rs-code/`
- **File**: `config.rs` (init)
- **Fix**: Create `~/.i-rs-code/.gitignore` on first init with `*` (ignore everything).
- **Accept**: Config dir not accidentally committed.

### L9. Zero-test modules need basic tests
- **Files**: 25 modules with no tests
- **Fix**: Add at least one basic test per module. Priority: `tools/web.rs`, `tools/verify.rs`, `tools/git.rs`, `prompt.rs`, `diff.rs`.
- **Accept**: Every module has at least 1 test.

---

## Execution Order

### Phase 1: Critical (must do first)
| ID | Est. Effort | Dependencies |
|----|-------------|-------------|
| C1 | 30 min | None |
| C2 | 45 min | None |

### Phase 2: High (core functionality)
| ID | Est. Effort | Dependencies |
|----|-------------|-------------|
| H1 | 30 min | None |
| H2 | 45 min | None |
| H3 | 30 min | None |
| H4 | 20 min | None |
| H5 | 45 min | None |
| H6 | 30 min | None |
| H7 | 15 min | None |
| H8 | 30 min | None |

### Phase 3: Medium (quality improvements)
| ID | Est. Effort | Dependencies |
|----|-------------|-------------|
| M1 | (with H2) | H2 |
| M3 | 60 min | None |
| M4 | 45 min | None |
| M5 | 20 min | None |
| M7 | 15 min | None |
| M8 | 15 min | None |
| M9 | 5 min | M10, M11 |
| M10 | 10 min | M6 or remove |
| M11 | 30 min | None |
| M12 | 20 min | None |
| M13 | 20 min | None |
| M14 | 15 min | None |
| M15 | 15 min | None |
| M16 | 20 min | None |
| M17 | 10 min | None |
| M18 | 30 min | None |
| M19 | 20 min | None |
| M20 | 30 min | None |
| M21 | 20 min | None |
| M22 | 10 min | None |
| M23 | 20 min | None |
| M24 | 15 min | None |
| M25 | 5 min | None |
| M2 | 90 min | None |

### Phase 4: Low (cleanup)
| ID | Est. Effort | Dependencies |
|----|-------------|-------------|
| L1 | 10 min | None |
| L2 | 5 min | None |
| L3 | 5 min | None |
| L4 | 5 min | None |
| L5 | 10 min | None |
| L6 | 5 min | None |
| L8 | 5 min | None |
| L9 | 120 min | None |
| M6 | 60 min | M10 |

---

**Total estimated effort**: ~20 hours

**Verification after each phase**: `cargo check -p i-rs-code && cargo test -p i-rs-code`
