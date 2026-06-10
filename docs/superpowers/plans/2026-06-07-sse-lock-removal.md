# SSE Stream Lock Removal Implementation Plan

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove all `core.write().await` calls from the SSE event loop in `build_sse_stream()`, replacing them with an mpsc writer-task pattern that never holds the `RwLock` inside the stream.

**Architecture:** A `WriteCmd` enum captures every write operation the stream currently performs under lock. An mpsc channel delivers commands to a dedicated `writer_task` that briefly acquires `core.write()`, executes one command, and drops the guard. The SSE stream becomes lock-free — it only sends commands and emits events.

**Tech Stack:** tokio (mpsc, oneshot, RwLock, spawn), serde_json. No new deps.

**Design spec:** `docs/superpowers/specs/2026-06-07-sse-lock-removal-design.md`

**Estimated time:** 2-4 hours. Single file (`chat.rs`), all changes localized.

---

## File Structure

| File | Change |
|------|--------|
| `crates/claw/src/server/routes/chat.rs` | Add `WriteCmd` enum + `writer_task` fn. Rewrite `ToolExecuted`/`Done`/`Error` branches in `build_sse_stream`. Update `chat()`/`chat_stream`/`chat_stream_resume` to spawn writer task. |

---

## Task 1: WriteCmd enum + writer_task (infrastructure)

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs:1-17` (imports)
- Modify: `crates/claw/src/server/routes/chat.rs:141` (after `build_sse_stream`, add new types)

- [ ] **Step 1: Add `oneshot` to imports**

In `crates/claw/src/server/routes/chat.rs`, find line 15:

```rust
use tokio::sync::mpsc;
```

Replace with:

```rust
use tokio::sync::{mpsc, oneshot};
```

- [ ] **Step 2: Add `WriteCmd` enum after `build_sse_stream` function**

After the closing `}` of `build_sse_stream` (line 141), insert:

```rust
// ── Writer task infrastructure ──

enum WriteCmd {
    RecordToolMemory {
        user_id: String,
        agent_id: String,
        tool_name: String,
        tool_args: serde_json::Value,
        tool_result: String,
        i_rs_index: std::collections::HashMap<String, String>,
    },
    RecordLayeredMemory {
        user_id: String,
        agent_id: String,
        tool_name: String,
        tool_result: String,
    },
    PersistMessages {
        session_id: String,
        messages: Vec<crate::app::Message>,
    },
    SaveApiMessages {
        session_id: String,
        messages: Vec<serde_json::Value>,
    },
    EvaluateSession {
        session_id: String,
        reply: oneshot::Sender<Option<crate::app::Message>>,
    },
    FlushMemory {
        user_id: String,
        agent_id: String,
    },
    MarkError {
        session_id: String,
        error: String,
    },
}
```

- [ ] **Step 3: Add `writer_task` function**

After the `WriteCmd` enum, insert:

```rust
async fn writer_task(
    core: std::sync::Arc<tokio::sync::RwLock<i_rs_claw_core::core::AppCore>>,
    mut rx: mpsc::UnboundedReceiver<WriteCmd>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            WriteCmd::RecordToolMemory { user_id, agent_id, tool_name, tool_args, tool_result, i_rs_index } => {
                let mut c = core.write().await;
                i_rs_claw_core::core::record_tool_memory(
                    &user_id, &mut c.agent_store, &i_rs_index,
                    &agent_id, &tool_name, &tool_args, &tool_result,
                );
            }
            WriteCmd::RecordLayeredMemory { user_id, agent_id, tool_name, tool_result } => {
                let mut c = core.write().await;
                i_rs_claw_core::core::record_layered_tool_memory(
                    &user_id, &mut c.agent_store, &agent_id, &tool_name, &tool_result,
                );
            }
            WriteCmd::PersistMessages { session_id, messages } => {
                let mut c = core.write().await;
                if let Err(e) = c.session_mgr.persist_messages(&session_id, &messages) {
                    tracing::error!("persist_messages (writer) 失败: {}", e);
                }
            }
            WriteCmd::SaveApiMessages { session_id, messages } => {
                let mut c = core.write().await;
                c.session_mgr.save_api_messages(&session_id, &messages);
            }
            WriteCmd::EvaluateSession { session_id, reply } => {
                let msg = core.write().await.evaluate_completed_session(&session_id);
                let _ = reply.send(msg);
            }
            WriteCmd::FlushMemory { user_id, agent_id } => {
                let mut c = core.write().await;
                if let Ok(mem) = c.agent_store.memory_for_mut(&user_id, &agent_id) {
                    mem.flush();
                }
            }
            WriteCmd::MarkError { session_id, error } => {
                let mut c = core.write().await;
                c.session_mgr.mark_error(&session_id, &error);
            }
        }
    }
}
```

- [ ] **Step 4: Verify infrastructure compiles (unused for now)**

```bash
cargo check -p i-rs-claw --features dashboard
```
Expected: `Finished` with 0 errors (new types are defined but not wired in yet).

- [ ] **Step 5: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(sse): add WriteCmd enum + writer_task infrastructure"
```

---

## Task 2: Refactor ToolExecuted branch

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs:34-53` (ToolExecuted branch)

- [ ] **Step 1: Replace the ToolExecuted arm in build_sse_stream**

Old code (lines 34-53):
```rust
                    LlmEvent::ToolExecuted { name, args, result, step, total_steps, category } => {
                        let mut core = state.core.write().await;
                        let i_rs_index = core.config.i_rs_tool_index.clone();
                        let agent_id = core.session_mgr.session_meta(&sid)
                            .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string());
                        i_rs_claw_core::core::record_tool_memory(
                            &user_id, &mut core.agent_store, &i_rs_index, &agent_id, &name, &args, &result,
                        );
                        if !category.is_retryable_or_fatal() {
                            i_rs_claw_core::core::record_layered_tool_memory(
                                &user_id, &mut core.agent_store, &agent_id, &name, &result,
                            );
                        }
                        drop(core);
                        let data = serde_json::to_string(&serde_json::json!({
                            "name": name, "args": args, "result": result,
                            "step": step, "total_steps": total_steps,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("tool_executed").data(data).id(seq.to_string());
                        acc.apply(&LlmEvent::ToolExecuted { name: name.clone(), args: args.clone(), result: result.clone(), step, total_steps, category });
                    }
```

Replace with:
```rust
                    LlmEvent::ToolExecuted { name, args, result, step, total_steps, category } => {
                        // Snapshot agent_id (immutable during chat_loop) and i_rs_index
                        // before sending to writer — no lock needed.
                        let (agent_id, i_rs_index) = {
                            let core = state.core.read().await;
                            let aid = core.session_mgr.session_meta(&sid)
                                .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string());
                            let idx = core.config.i_rs_tool_index.clone();
                            (aid, idx)
                        };
                        let _ = write_tx.send(WriteCmd::RecordToolMemory {
                            user_id: user_id.clone(),
                            agent_id: agent_id.clone(),
                            tool_name: name.clone(),
                            tool_args: args.clone(),
                            tool_result: result.clone(),
                            i_rs_index,
                        });
                        if !category.is_retryable_or_fatal() {
                            let _ = write_tx.send(WriteCmd::RecordLayeredMemory {
                                user_id: user_id.clone(),
                                agent_id: agent_id.clone(),
                                tool_name: name.clone(),
                                tool_result: result.clone(),
                            });
                        }
                        let data = serde_json::to_string(&serde_json::json!({
                            "name": name, "args": args, "result": result,
                            "step": step, "total_steps": total_steps,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("tool_executed").data(data).id(seq.to_string());
                        acc.apply(&LlmEvent::ToolExecuted { name: name.clone(), args: args.clone(), result: result.clone(), step, total_steps, category });
                    }
```

Note: uses `core.read().await` (read lock) to snapshot immutable data — much faster and less contention than `core.write()`.

- [ ] **Step 2: Verify compiles (Done/Error branches still use state.core directly)**

```bash
cargo check -p i-rs-claw --features dashboard
```
Expected: `Finished` with 0 errors.

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(sse): move ToolExecuted writes to writer task"
```

---

## Task 3: Refactor Done branch

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs:55-87` (Done branch)

- [ ] **Step 1: Replace the Done arm in build_sse_stream**

Old code (lines 55-87):
```rust
                    LlmEvent::Done(msgs, usage, _trace_id) => {
                        let mut core = state.core.write().await;
                        let agent_id = core.session_mgr.session_meta(&sid)
                            .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string());
                        acc.apply(&LlmEvent::Done(msgs.clone(), usage, String::new()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        if let Err(e) = core.session_mgr.persist_messages(&sid, &finalized) {
                            tracing::error!("persist_messages (Done) 失败: {}", e);
                        }
                        core.session_mgr.save_api_messages(&sid, &msgs);
                        let quality_msg = core.evaluate_completed_session(&sid);
                        let quality_json = match &quality_msg {
                            Some(crate::app::Message::Quality { score, complete, issues, references_valid }) => {
                                serde_json::json!({"score": score.map(|s| s.to_string()).unwrap_or_default(), "complete": complete, "issues": issues, "references_valid": references_valid})
                            }
                            _ => serde_json::json!(null),
                        };
                        if let Some(q) = &quality_msg {
                            if let Err(e) = core.session_mgr.persist_messages(&sid, &[q.clone()]) {
                                tracing::error!("quality 持久化失败: {}", e);
                            }
                        }
                        core.agent_store.memory_for_mut(&user_id, &agent_id)
                            .expect("BUG: default agent runtime not initialized")
                            .flush();
                        drop(core);
                        state.chat_concurrency.release();
                        let done_json = serde_json::json!({"usage": usage, "quality": quality_json, "session_id": &sid});
                        let data = serde_json::to_string(&done_json).unwrap_or_default();
                        sse_event = Event::default().event("done").data(data).id(seq.to_string());
                        keep_rx = false;
                    }
```

Replace with:
```rust
                    LlmEvent::Done(msgs, usage, _trace_id) => {
                        let agent_id = {
                            let core = state.core.read().await;
                            core.session_mgr.session_meta(&sid)
                                .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string())
                        };
                        acc.apply(&LlmEvent::Done(msgs.clone(), usage, String::new()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        let _ = write_tx.send(WriteCmd::PersistMessages { session_id: sid.clone(), messages: finalized });
                        let _ = write_tx.send(WriteCmd::SaveApiMessages { session_id: sid.clone(), messages: msgs.clone() });

                        let (reply_tx, reply_rx) = oneshot::channel();
                        let _ = write_tx.send(WriteCmd::EvaluateSession { session_id: sid.clone(), reply: reply_tx });
                        let quality_msg = reply_rx.await.ok().flatten();

                        let quality_json = match &quality_msg {
                            Some(crate::app::Message::Quality { score, complete, issues, references_valid }) => {
                                serde_json::json!({"score": score.map(|s| s.to_string()).unwrap_or_default(), "complete": complete, "issues": issues, "references_valid": references_valid})
                            }
                            _ => serde_json::json!(null),
                        };
                        if let Some(q) = &quality_msg {
                            let _ = write_tx.send(WriteCmd::PersistMessages { session_id: sid.clone(), messages: vec![q.clone()] });
                        }
                        let _ = write_tx.send(WriteCmd::FlushMemory { user_id: user_id.clone(), agent_id: agent_id.clone() });
                        concurrency.release();
                        let done_json = serde_json::json!({"usage": usage, "quality": quality_json, "session_id": &sid});
                        let data = serde_json::to_string(&done_json).unwrap_or_default();
                        sse_event = Event::default().event("done").data(data).id(seq.to_string());
                        keep_rx = false;
                    }
```

- [ ] **Step 2: Verify compiles**

```bash
cargo check -p i-rs-claw --features dashboard
```
Expected: `Finished` with 0 errors.

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(sse): move Done writes to writer task + oneshot EvaluateSession"
```

---

## Task 4: Refactor Error branch

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs:88-102` (Error branch)

- [ ] **Step 1: Replace the Error arm in build_sse_stream**

Old code (lines 88-102):
```rust
                    LlmEvent::Error(e) => {
                        acc.apply(&LlmEvent::Error(e.clone()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        {
                            let mut core = state.core.write().await;
                            core.session_mgr.mark_error(&sid, &e);
                            if let Err(err) = core.session_mgr.persist_messages(&sid, &finalized) {
                                tracing::error!("persist_messages (error) 失败: {}", err);
                            }
                        }
                        state.chat_concurrency.release();
                        sse_event = Event::default().event("error").data(e).id(seq.to_string());
                        keep_rx = false;
                    }
```

Replace with:
```rust
                    LlmEvent::Error(e) => {
                        acc.apply(&LlmEvent::Error(e.clone()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        let _ = write_tx.send(WriteCmd::MarkError { session_id: sid.clone(), error: e.clone() });
                        let _ = write_tx.send(WriteCmd::PersistMessages { session_id: sid.clone(), messages: finalized });
                        concurrency.release();
                        sse_event = Event::default().event("error").data(e).id(seq.to_string());
                        keep_rx = false;
                    }
```

- [ ] **Step 2: Verify compiles (state.chat_concurrency is now accessed directly)**

```bash
cargo check -p i-rs-claw --features dashboard
```
Expected: `Finished` with 0 errors. If `state.chat_concurrency` is used, that's fine — it's an `Arc` that doesn't need a lock.

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(sse): move Error writes to writer task"
```

---

## Task 5: Update build_sse_stream signature + three handler functions

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs:18-23` (build_sse_stream sig)
- Modify: `crates/claw/src/server/routes/chat.rs:24-26` (unfold state)
- Modify: `crates/claw/src/server/routes/chat.rs:145-238` (chat handler)
- Modify: `crates/claw/src/server/routes/chat.rs:240-320` (chat_stream + chat_stream_resume handlers)

- [ ] **Step 1: Change `build_sse_stream` signature**

Old (lines 18-23):
```rust
fn build_sse_stream(
    rx: mpsc::UnboundedReceiver<LlmEvent>,
    state: AppState,
    user_id: String,
    sid: String,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = futures_util::stream::unfold(
        (Some(rx), state, user_id, sid, MessageAccumulator::new(), 0u64),
```

Replace with:
```rust
fn build_sse_stream(
    rx: mpsc::UnboundedReceiver<LlmEvent>,
    write_tx: mpsc::UnboundedSender<WriteCmd>,
    concurrency: std::sync::Arc<crate::server::rate_limit::ChatConcurrency>,
    state_for_snapshot: AppState,  // read-only snapshots only, NOT used for writes
    user_id: String,
    sid: String,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = futures_util::stream::unfold(
        (Some(rx), write_tx, concurrency, state_for_snapshot, user_id, sid, MessageAccumulator::new(), 0u64),
```

- [ ] **Step 2: Update unfold state tuple destructuring**

Old (line 26):
```rust
        |(rx_opt, state, user_id, sid, mut acc, mut seq)| async move {
```

Replace with:
```rust
        |(rx_opt, write_tx, concurrency, state, user_id, sid, mut acc, mut seq)| async move {
```

- [ ] **Step 3: Update unfold return tuple**

Old (line 135-136): `state` in the return tuple.
```rust
                return Some((Ok::<_, Infallible>(sse_event), (next_rx, state, user_id, sid, next_acc, seq)));
```

Replace with:
```rust
                return Some((Ok::<_, Infallible>(sse_event), (next_rx, write_tx, concurrency, state, user_id, sid, next_acc, seq)));
```

- [ ] **Step 4: Update `chat()` handler to spawn writer task**

In the `chat()` function, after the session init block (after line 236, before `build_sse_stream` call at line 238), add writer task spawning:

Old (lines 233-238):
```rust
        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
        drop(core);
        sid
    };

    build_sse_stream(rx, state.clone(), user_id, sid).into_response()
```

Replace with:
```rust
        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
        drop(core);
        sid
    };

    let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteCmd>();
    let core_arc = state.core.clone();
    tokio::spawn(async move { writer_task(core_arc, write_rx).await });

    build_sse_stream(rx, write_tx, state.chat_concurrency.clone(), state.clone(), user_id, sid).into_response()
```

- [ ] **Step 5: Update `chat_stream` handler**

In `chat_stream()` (line ~279-310), find the `build_sse_stream` call and apply the same pattern:

```rust
let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteCmd>();
let core_arc = state.core.clone();
tokio::spawn(async move { writer_task(core_arc, write_rx).await });

build_sse_stream(rx, write_tx, state.chat_concurrency.clone(), state.clone(), user_id, sid).into_response()
```

Also add `UserId(user_id): UserId` to the `chat_stream` parameter list if not already present. The `sid` extraction code (lines before the channel section) should already produce a `user_id` variable — if not, default to `"default".to_string()`.

- [ ] **Step 6: Update `chat_stream_resume` handler**

Same pattern as `chat_stream`. The handler is at lines ~295-328. Apply the identical writer-task spawning code.

- [ ] **Step 7: Verify compiles + all tests pass**

```bash
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```
Expected: 0 errors, all existing tests pass.

- [ ] **Step 8: Commit**

```bash
git add crates/claw/src/server/routes/chat.rs
git commit -m "refactor(sse): wire writer task into chat handlers, remove all core.write() from SSE stream"
```

---

## Task 6: Final verification

- [ ] **Step 1: Workspace check**

Run:
```bash
cargo check --workspace
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
cargo test -p i-rs-claw-core --lib -- --test-threads=1
```
Expected: 0 errors, all tests pass.

- [ ] **Step 2: Verify no remaining `core.write()` inside `build_sse_stream`**

Run:
```bash
rg -n 'core\.write\b' crates/claw/src/server/routes/chat.rs
```
Expected: No matches inside the `build_sse_stream` function body. Matches in `writer_task` and `chat()` handler are fine.

- [ ] **Step 3: Manual test suggestion (optional)**

Start the server with `cargo run -p i-rs-claw --features dashboard -- serve` and POST to `/api/chat`. Verify the SSE stream still emits `tool_executed`, `token`, `done` events correctly.

- [ ] **No commit needed if all checks pass.**

---

## Verification

```bash
# Compilation
cargo check --workspace

# All tests
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
cargo test -p i-rs-claw-core --lib -- --test-threads=1

# No stray write locks in stream
rg -n 'core\.write\b' crates/claw/src/server/routes/chat.rs
```

Expected: workspace 0 errors, all tests pass, `core.write` only in `writer_task` and `chat` handler session init.
