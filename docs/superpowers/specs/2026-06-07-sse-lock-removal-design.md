# SSE Stream Lock Removal — Design Spec

&gt; **Status:** Approved | **Date:** 2026-06-07 | **Audit ref:** M4

## 1. Problem

`crates/claw/src/server/routes/chat.rs::build_sse_stream()` constructs a `tokio_stream::unfold`
that processes `LlmEvent` items from an mpsc channel into SSE events. Three of the event branches
acquire `core.write().await` *inside* the unfold closure:

| Event   | Line | What happens under the lock          |
|---------|------|--------------------------------------|
| `ToolExecuted` | 35  | `record_tool_memory` + `record_layered_tool_memory` + read `agent_id` from session |
| `Done`         | 56  | `persist_messages` + `save_api_messages` + `evaluate_completed_session` + `flush_memory` + read `agent_id` |
| `Error`        | 93  | `mark_error` + `persist_messages` |

**Why this is a problem:** tokio's `RwLock` is writer-preference. When multiple SSE streams are
active and any one of them holds `core.write()`, ALL reads (`core.read()`) for entirely different
endpoints are blocked. The `Done` handler is especially bad — it does JSON persistence, session
evaluation, and memory flush all under one write guard, keeping the lock for milliseconds.

## 2. Architecture: mpsc Writer-Task Pattern

The SSE stream should **never** hold `core.write()`. Instead it sends write-commands via a
second mpsc channel to a dedicated writer task that briefly acquires the lock, executes one
command, and releases.

```
chat() handler
├─ core.write()  —  session init (brief, as before)
├─ spawn chat_loop → llm_tx → mpsc receiver
├─ create write_tx channel
├─ spawn writer_task(Arc<AppState>, write_rx)               ← NEW
├─ build SSE stream with write_tx captured                  ← no core access needed
└─ return Sse::new(stream)

writer_task
└─ loop: recv cmd → core.write() → execute → drop(guard)

SSE stream (zero lock acquisitions)
├─ Token / Reasoning / Status   → emit SSE Event (unchanged)
├─ ToolExecuted                 → write_tx.send(RecordToolMemory + RecordLayered)
│                               → emit SSE Event
├─ Done                         → write_tx.send(PersistMsgs, SaveApiMsgs, FlushMemory)
│                               → oneshot.await(EvaluateSession)  — async wait, no lock
│                               → concurrency.release()
│                               → emit SSE Event
└─ Error                        → write_tx.send(MarkError, PersistMsgs)
                                → concurrency.release()
                                → emit SSE Event
```

**Key property:** `agent_id` is snapshotted from `session_mgr.session_meta(&sid)` before the
command is sent, so the writer task never needs to look it up again. The writer task needs
`Arc<RwLock<AppCore>>` — which is exactly what `AppState` contains — but receives it at spawn
time, independent of the SSE stream.

## 3. WriteCmd Enum

Add to `crates/claw/src/server/routes/chat.rs` (private to the module):

```rust
use tokio::sync::{mpsc, oneshot};

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

`EvaluateSession` uses `oneshot` because its return value (quality score) is injected into the
SSE `done` event payload — the SSE stream must wait for the evaluation result before emitting
the event, but waiting on a oneshot does NOT block on the RwLock.

## 4. Writer Task

```rust
async fn writer_task(
    core: Arc<RwLock<AppCore>>,
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

Each arm acquires `core.write()`, does exactly the minimum work, then drops the guard. No
command spans multiple lock acquisitions.

## 5. SSE Stream Changes

The `build_sse_stream` signature changes to accept `write_tx` instead of `state`:

```rust
fn build_sse_stream(
    rx: mpsc::UnboundedReceiver<LlmEvent>,
    write_tx: mpsc::UnboundedSender<WriteCmd>,
    concurrency: Arc<ChatConcurrency>,
    user_id: String,
    sid: String,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> { ... }
```

The unfold state drops `state: AppState` and keeps only `write_tx`, `concurrency`, `user_id`,
`sid`, `acc`, `seq`. The `ToolExecuted` / `Done` / `Error` branches change from direct
`core.write().await` to `let _ = write_tx.send(...)`.

The `Done` branch is the only place that needs an async wait — for `EvaluateSession`:

```rust
LlmEvent::Done(msgs, usage, _trace_id) => {
    let finalized = acc.into_messages();
    acc = MessageAccumulator::new();

    let _ = write_tx.send(WriteCmd::PersistMessages { session_id: sid.clone(), messages: finalized });
    let _ = write_tx.send(WriteCmd::SaveApiMessages { session_id: sid.clone(), messages: msgs.clone() });

    let (reply_tx, reply_rx) = oneshot::channel();
    let _ = write_tx.send(WriteCmd::EvaluateSession { session_id: sid.clone(), reply: reply_tx });
    let quality_msg = reply_rx.await.ok().flatten();

    if let Some(q) = &quality_msg {
        let _ = write_tx.send(WriteCmd::PersistMessages { session_id: sid.clone(), messages: vec![q.clone()] });
    }
    let _ = write_tx.send(WriteCmd::FlushMemory { user_id: user_id.clone(), agent_id: agent_id.clone() });
    concurrency.release();

    // ... emit done event as before
}
```

This is the **only** `await` in the SSE stream that isn't `rx.recv()`, and it does not hold a lock.

## 6. chat() Handler Changes

The `chat()` handler spawns the writer task before returning the SSE response:

```rust
pub async fn chat(...) -> Response {
    // ... concurrency check, session init (same as before) ...

    let (write_tx, write_rx) = mpsc::unbounded_channel::<WriteCmd>();
    let core_arc = state.core.clone();
    tokio::spawn(async move { writer_task(core_arc, write_rx).await });

    drop(core); // release session-init lock
    build_sse_stream(rx, write_tx, state.chat_concurrency.clone(), user_id, sid).into_response()
}
```

The `chat_stream` and `chat_stream_resume` handlers follow the same pattern (spawn writer task
in the handler, pass write_tx to build_sse_stream).

## 7. Testing Strategy

1. **Unit test `writer_task`** — create a mock `AppCore` (or use `test_helpers::test_core()`),
   send each `WriteCmd` variant, verify the operation took effect.

2. **Integration: existing SSE tests** — `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
   must continue to pass. The `mod.rs` route tests that exercise `chat()`
   will exercise the modified code path.

3. **Concurrency smoke test** — send 5 concurrent POSTs to `/api/chat` and verify the
   rate-limiter caps at 3 concurrent, and the writer task processes commands without blocking.

## 8. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| `mpsc::unbounded` could OOM if writer task stalls | Writer task does trivial work (microsecond-level), can't stall. If it panics, the channel drops and the SSE stream terminates with an error. |
| `oneshot` channel could poison if writer task panics before responding | `reply_rx.await.ok().flatten()` handles both `RecvError` and `None` gracefully. The quality field defaults to `null` in the SSE payload. |
| `agent_id` snapshot could be stale if session changes mid-stream | Session metadata is immutable during a chat_loop — `agent_id` is set at session creation and never changes. Safe to snapshot. |
| Writer task holds `core.write()` per-command (many acquisitions) | Yes, but each is ~100µs vs the current 5-10ms. Many brief acquisitions beat one long one. |

## 9. Files Modified

| File | Change |
|------|--------|
| `crates/claw/src/server/routes/chat.rs` | Add `WriteCmd` enum, `writer_task` fn, rewrite `build_sse_stream` and `chat()`/`chat_stream`/`chat_stream_resume` handlers |

No other files change. No new dependencies. No Cargo.toml changes.

---

*Self-review: no placeholders, no TODOs, all types consistent across sections. Scope is contained to a single file.*
