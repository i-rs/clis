# Read-Path Migration & Dead Code Removal

**Date:** 2026-06-06
**Status:** Draft
**Depends on:** `2026-06-05-message-log-refactor.md` (Tasks 1-10 complete)

## Problem

The append-only `MessageLog` refactor migrated all **write** paths, but the
**read** paths and legacy infrastructure remain:

| Issue | Impact |
|-------|--------|
| `session::load_messages` reads from dead `messages` table | New sessions have **no LLM context** when resuming |
| `build_messages_from_jsonl` takes `&[Value]` | Duplicates `build_messages_for` logic, untyped |
| `ConvStore::search` calls `MessageRepo::search` | Search reads stale `messages` table |
| `MessageRepo` trait + impls + DDL still exist | ~300 LOC dead code, confusing |
| `api_msgs_to_jsonl` / `save_chat_result` retained | Dead since Task 10 |
| `append_message` (legacy JSONL API) still called by tests | Tests block removal |

## Design

### Phase 1 — Migrate Read Paths (no breaking changes)

#### 1.1 `export_markdown` / `export_json` → `Vec<Message>`

**File:** `session.rs:296-354`

Both functions iterate records by inspecting JSON keys (`record.get("type")`).
Switch to `load_app_messages` + match on `Message` enum variants.

```rust
pub fn export_markdown(&self, id: &str) -> Option<String> {
    let messages = self.load_app_messages(id, 1000);
    let meta = self.find_index(id).map(|i| &self.sessions[i])?;
    let mut md = format!("...");
    for msg in &messages {
        match msg {
            Message::User { text } => md.push_str(&format!("**用户:** {}\n\n", text)),
            Message::Assistant { text, .. } => md.push_str(&format!("**Claw:** {}\n\n", text)),
            Message::ToolCall { name, .. } => md.push_str(&format!("*[工具调用: {}]*\n\n", name)),
            Message::Error { text } => md.push_str(&format!("**错误:** {}\n\n", text)),
            Message::Evaluation { tool, valid, issues, .. } if !valid => { ... }
            _ => {}
        }
    }
    Some(md)
}
```

Same for `export_json`: serialize `Vec<Message>` via `serde_json::to_value`.

**Also delete:** `load_messages` (the `Vec<Value>` version) — no remaining callers.

#### 1.2 `memory::analyze_sessions` → `Vec<Message>`

**File:** `memory.rs:165-182`

```rust
for meta in sessions {
    let messages = session_mgr.load_app_messages(&meta.id, 1000);
    for msg in &messages {
        if let Message::ToolCall { name, .. } = msg {
            *self.tool_frequency.entry(name.clone()).or_insert(0) += 1;
        }
    }
}
```

#### 1.3 Dashboard: `build_messages_from_jsonl` → `build_messages_from_log`

**File:** `core/mod.rs:766-844` + `dashboard/routes.rs:303-305, 371-374`

Replace `build_messages_from_jsonl(&[Value])` with `build_messages_from_log(&[Message])`.
Logic is identical but type-safe. Callers switch from `load_messages` to
`load_app_messages`.

```rust
// core/mod.rs
pub fn build_messages_from_log(&self, messages: &[Message], agent_id: &str) -> Vec<Value> {
    // same system-prompt construction as build_messages_from_jsonl
    let mut msgs = vec![json!({ "role": "system", "content": system_prompt })];
    let mut tool_call_counter = 0;
    for msg in messages {
        match msg {
            Message::User { text } => msgs.push(json!({"role":"user","content":text})),
            Message::Assistant { text, .. } if !text.is_empty() => {
                msgs.push(json!({"role":"assistant","content":text}))
            }
            Message::ToolCall { name, args, result, .. } => {
                tool_call_counter += 1;
                let call_id = format!("call_{}_{}", name, tool_call_counter);
                msgs.push(json!({"role":"assistant","content":null,"tool_calls":[{...}]}));
                msgs.push(json!({"role":"tool","tool_call_id":call_id,"content":result}));
            }
            _ => {}
        }
    }
    msgs
}
```

Dashboard callers:
```rust
let messages = core.session_mgr.load_app_messages(&sid, 50);
let msgs = core.build_messages_from_log(&messages, &agent_id);
core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &messages);
```

Note: `spawn_chat_for_async`'s `recent_messages` parameter type changes from
`&[Value]` to `&[Message]`. Its body passes them to `build_delegate_runtime`
which currently takes `Vec<Value>` — need to check/adjust.

#### 1.4 `ConvStore::search` → `MessageLog::search`

**File:** `convstore.rs:30-40`

One-line change: `storage.messages.search` → `storage.message_log.search`.

Both traits define `search` with the same signature (`&str, usize -> Vec<SearchResult>`).

#### 1.5 `delete_session` → `MessageLog::delete_session`

**File:** `session.rs:202-203`

```rust
let _ = storage.message_log.delete_session(&sid).await;
```

### Phase 2 — Remove Dead Code

After Phase 1 has zero callers of `MessageRepo` / `load_messages(Value)` /
`api_msgs_to_jsonl` / `save_chat_result` / `save_all_messages` /
`append_message`:

| What | Where |
|------|-------|
| `MessageRepo` trait | `storage/mod.rs:101-118` |
| `MessageRepo` impl (file) | `storage/file.rs:153-210` |
| `MessageRepo` impl (SQL) | `storage/sql/mod.rs:96-115` |
| `messages` field in `ClawStorage` | `storage/mod.rs:254` |
| `messages` from `into_storage()` constructors | `storage/file.rs`, `storage/sql/*.rs` |
| `messages` table DDL | `storage/sql/{sqlite,mysql,postgres}.rs` |
| `FileMessageStore` messages JSONL logic | `storage/file.rs` |
| `api_msgs_to_jsonl` | `core/mod.rs:889-992` |
| `save_chat_result` | `core/mod.rs:997-1007` |
| `build_messages_from_jsonl` | `core/mod.rs:766-844` |
| `append_message` (legacy) | `session.rs:366-401` |
| `save_all_messages` | `session.rs:450-461` |
| `load_messages` (Vec<Value>) | `session.rs:403-408` |
| Tests for `api_msgs_to_jsonl` | `core/mod.rs:1194-1280` |

### Phase 3 — Test Migration

Rewrite tests currently using `save_all_messages` / `load_messages` /
`append_message`:

| Test | Current | Migrated |
|------|---------|----------|
| `test_save_load_messages` | `save_all_messages` + `load_messages` | `append_batch` + `load_app_messages` |
| `test_export_markdown` | `save_all_messages` | `append_batch` with `Message::User/Assistant` |
| `test_export_json` | `save_all_messages` | `append_batch` |
| `test_sqlite_create_does_not_wipe_messages` | `append_message` + `load_messages` | `append_one` + `load_app_messages` |
| `test_sqlite_switch_session_preserves_messages` | same | same |
| `test_message_search` (`file.rs:1150`) | `MessageRepo::search` | `MessageLog::search` |
| `test_api_msgs_to_jsonl_*` (3 tests) | `api_msgs_to_jsonl` | **Delete** (replaced by accumulator golden test) |

## Execution Order

```
Phase 1 (read-path migration, no deletions):
  1.1 export_markdown / export_json
  1.2 memory::analyze_sessions
  1.3 Dashboard build_messages_from_log + spawn_chat_for_async
  1.4 ConvStore::search
  1.5 delete_session → message_log
  → cargo check + cargo test

Phase 2 (dead code removal):
  2.1 Remove session.rs: load_messages, save_all_messages, append_message
  2.2 Remove core/mod.rs: build_messages_from_jsonl, api_msgs_to_jsonl, save_chat_result
  2.3 Remove storage: MessageRepo trait + impls + messages field + DDL
  2.4 Remove FileMessageStore messages logic
  → cargo check + cargo test

Phase 3 (test cleanup):
  3.1 Migrate session.rs tests
  3.2 Migrate file.rs search test
  3.3 Delete dead api_msgs_to_jsonl tests
  → cargo test -p i-rs-claw -- --test-threads=1
```

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| `spawn_chat_for_async` `recent_messages` type change | Trace all delegate_rt callers; it builds `DelegateRuntime` which stores `Vec<Value>` — may need conversion |
| `export_json` output format changes (Message vs raw JSON) | Acceptable — the new format is a superset (includes `type` tag + all fields) |
| SQL `messages` table DDL removal blocks new DB creation | No migration needed — `message_log` table is the live one; old `messages` table becomes orphaned (harmless) |
| `MessageRepo::count` used anywhere? | Checked: no production callers |

## Verification

```bash
cargo check --workspace
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw -- --test-threads=1
# Confirm: 0 warnings, all tests pass
# Confirm: no remaining references to MessageRepo, api_msgs_to_jsonl, save_all_messages
rg 'MessageRepo|api_msgs_to_jsonl|save_all_messages|build_messages_from_jsonl' crates/claw/src/
# Should return zero matches
```
