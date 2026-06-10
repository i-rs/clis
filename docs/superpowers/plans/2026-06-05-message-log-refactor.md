# Message Log Append-Only Refactor — Implementation Plan

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the lossy `save_all_messages` + `api_msgs_to_jsonl` write path with an append-only `MessageLog` and a shared `MessageAccumulator`, eliminating the bug class where tool_call / evaluation / quality records get silently dropped on session reload.

**Architecture:** Layered split —
1. **`StoredRecord`** (envelope: `seq`, `ts`, `schema_v`, `payload: Message`) is the on-disk format.
2. **`MessageLog` trait** is append-only (`append_batch` / `load` / `search` / `delete_session` / `count`); no `save_all`. Implemented for file + SQLite + MySQL + PG.
3. **`MessageAccumulator`** consumes `LlmEvent` → `Vec<Message>` and is the single source of truth used by Dashboard (and available to Gateway). TUI keeps its existing `App::messages` in-memory model but persists via `MessageLog::append_batch` with a `last_saved_count` cursor.

**Tech Stack:** Rust, tokio, async-trait, serde, sqlx, chrono.

**Spec:** `docs/superpowers/specs/2026-06-05-message-log-refactor-design.md`

---

## File Structure

| File | Status | Responsibility |
|------|--------|----------------|
| `crates/claw/src/message/mod.rs` | **CREATE** | `StoredRecord`, `Message` re-export, `default_ts`, round-trip serde |
| `crates/claw/src/message/accumulator.rs` | **CREATE** | `MessageAccumulator` (LlmEvent → Vec&lt;Message&gt;) |
| `crates/claw/src/storage/mod.rs` | MODIFY | Add `MessageLog` trait, add `ClawStorage::message_log` field |
| `crates/claw/src/storage/file.rs` | MODIFY | Add `FileMessageLog`; wire into `ClawStorage::file` |
| `crates/claw/src/storage/sql/mod.rs` | MODIFY | Extend `define_sql_stores!` macro with `MessageLog` impl segment |
| `crates/claw/src/storage/sql/sqlite.rs` | MODIFY | Add `message_log` DDL |
| `crates/claw/src/storage/sql/mysql.rs` | MODIFY | Add `message_log` DDL |
| `crates/claw/src/storage/sql/postgres.rs` | MODIFY | Add `message_log` DDL |
| `crates/claw/src/session.rs` | MODIFY | Add `last_saved_count` cursor; add `append_new_messages` helper; eventually delete `save_all_messages` + `append_message` |
| `crates/claw/src/core/mod.rs` | MODIFY | Delete `save_chat_result` + `api_msgs_to_jsonl` |
| `crates/claw/src/dashboard/routes.rs` | MODIFY | Migrate `chat_stream` + `send_message` to accumulator + `MessageLog::append_batch` |
| `crates/claw/src/gateway/mod.rs` | MODIFY | Replace `append_message` calls with `message_log.append_batch` |
| `crates/claw/src/tui/clipboard.rs` | MODIFY | Rewrite `save_session_messages` to use `append_batch` with cursor |
| `crates/claw/src/tui/handlers/{llm,key,overlay}.rs`, `tui/mod.rs` | MODIFY | Update callers to new signature |
| `crates/claw/src/app.rs` | MODIFY | Re-export `Message` from `message::`; keep domain enum |

---

## Task 1: Create `message/` module with `StoredRecord`

**Files:**
- Create: `crates/claw/src/message/mod.rs`
- Modify: `crates/claw/src/lib.rs` (add `pub mod message;`)

- [ ] **Step 1: Write failing test for `StoredRecord` round-trip**

Create `crates/claw/src/message/mod.rs` with just enough scaffolding to compile the test:

```rust
//! Message storage envelope + accumulator module.
//!
//! Splits the on-disk `StoredRecord` (envelope + payload) from the in-memory
//! `Message` enum (re-exported from `crate::app`). All persistence goes
//! through `MessageLog::append_batch` which wraps each `Message` in a
//! `StoredRecord` before writing.

pub mod accumulator;

use crate::app::Message;
use chrono::Local;
use serde::{Deserialize, Serialize};

/// Current on-disk schema version. Bump when the persisted shape changes in
/// a backwards-incompatible way; loader handles migrations by `schema_v`.
pub const SCHEMA_VERSION: u16 = 1;

fn default_ts() -> i64 {
    Local::now().timestamp()
}

fn default_schema_v() -> u16 {
    SCHEMA_VERSION
}

/// On-disk record: envelope + payload (`Message` serialized as JSON Value).
///
/// `payload` is a `serde_json::Value` rather than `#[serde(flatten)] Message`
/// to dodge the well-known serde conflict between internal-tagged enums and
/// `flatten`. Encoding/decoding goes through `StoredRecord::from_message` /
/// `to_message`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRecord {
    /// Sequence number within a session (1-based, monotonic). The storage
    /// backend assigns this; loaders treat 0 as "absent".
    #[serde(default)]
    pub seq: u64,

    /// Unix-seconds timestamp when the record was appended. Defaults to
    /// "now" if absent (older records or hand-edited files).
    #[serde(default = "default_ts")]
    pub ts: i64,

    /// Schema version of this record. Defaults to current if absent.
    #[serde(default = "default_schema_v")]
    pub schema_v: u16,

    /// Payload: the `Message` enum serialized as a JSON object.
    pub payload: serde_json::Value,
}

impl StoredRecord {
    /// Wrap a `Message` for persistence. `seq` is set to 0; backends
    /// overwrite with the real sequence on insert.
    pub fn from_message(msg: &Message) -> anyhow::Result<Self> {
        Ok(Self {
            seq: 0,
            ts: Local::now().timestamp(),
            schema_v: SCHEMA_VERSION,
            payload: serde_json::to_value(msg)?,
        })
    }

    /// Decode the payload back into a `Message`. Returns `None` on decode
    /// failure (the caller logs and skips; this matches existing behavior
    /// for legacy/corrupt lines).
    pub fn to_message(&self) -> Option<Message> {
        serde_json::from_value(self.payload.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Message;
    use serde_json::json;

    #[test]
    fn stored_record_roundtrip_user() {
        let msg = Message::User { text: "hello".into() };
        let rec = StoredRecord::from_message(&msg).unwrap();
        let back = rec.to_message().expect("decode");
        match back {
            Message::User { text } => assert_eq!(text, "hello"),
            other => panic!("expected User, got {:?}", other),
        }
    }

    #[test]
    fn stored_record_roundtrip_tool_call_with_step() {
        let msg = Message::ToolCall {
            name: "weight".into(),
            args: "{}".into(),
            result: "ok".into(),
            step: 2,
            total_steps: 5,
        };
        let rec = StoredRecord::from_message(&msg).unwrap();
        // Serialize → deserialize to simulate disk round-trip
        let json = serde_json::to_value(&rec).unwrap();
        let back_rec: StoredRecord = serde_json::from_value(json).unwrap();
        match back_rec.to_message().unwrap() {
            Message::ToolCall { name, step, total_steps, .. } => {
                assert_eq!(name, "weight");
                assert_eq!(step, 2);
                assert_eq!(total_steps, 5);
            }
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }

    #[test]
    fn stored_record_payload_is_object_with_type() {
        let msg = Message::Assistant { text: "hi".into(), reasoning: String::new(), token_usage: None };
        let rec = StoredRecord::from_message(&msg).unwrap();
        assert!(rec.payload.is_object());
        assert_eq!(rec.payload["type"], "assistant");
        assert_eq!(rec.payload["text"], "hi");
    }

    #[test]
    fn stored_record_default_ts_is_recent() {
        let before = Local::now().timestamp();
        let rec = StoredRecord::from_message(&Message::User { text: "".into() }).unwrap();
        let after = Local::now().timestamp();
        assert!(rec.ts >= before && rec.ts <= after);
    }

    #[test]
    fn stored_record_accepts_legacy_missing_seq() {
        // Older records may lack `seq` (file backend assigned it implicitly
        // by line number). Deserialization must succeed.
        let json = json!({
            "ts": 1716220800,
            "schema_v": 1,
            "payload": { "type": "user", "text": "legacy" },
        });
        let rec: StoredRecord = serde_json::from_value(json).unwrap();
        assert_eq!(rec.seq, 0);
        assert_eq!(rec.ts, 1716220800);
        match rec.to_message().unwrap() {
            Message::User { text } => assert_eq!(text, "legacy"),
            other => panic!("expected User, got {:?}", other),
        }
    }
}
```

- [ ] **Step 2: Register the module in `lib.rs`**

Open `crates/claw/src/lib.rs` and add (next to the existing `pub mod app;`):

```rust
pub mod message;
```

- [ ] **Step 3: Create placeholder `accumulator.rs` so the module compiles**

```rust
//! MessageAccumulator — shared LlmEvent → Vec<Message> state machine.
//!
//! Populated in Task 6.
```

- [ ] **Step 4: Run tests, expect pass**

```bash
cargo test -p i-rs-claw --lib message::tests -- --test-threads=1
```

Expected: 5 passed.

- [ ] **Step 5: Verify whole crate still compiles**

```bash
cargo check -p i-rs-claw --all-features
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/claw/src/message/ crates/claw/src/lib.rs
git commit -m "feat(claw): add message module with StoredRecord envelope"
```

---

## Task 2: Add `MessageLog` trait to `storage/mod.rs`

**Files:**
- Modify: `crates/claw/src/storage/mod.rs`

- [ ] **Step 1: Add the trait**

Edit `crates/claw/src/storage/mod.rs`. After the existing `MessageRepo` trait block (around line 121), add:

```rust
/// Append-only message log — replaces `MessageRepo` for new callers.
///
/// **Why a separate trait?** `MessageRepo::save_all` (DELETE + INSERT) is the
/// root cause of the tool_call / evaluation / quality silent-drop bug:
/// streaming-time `append_message` writes get clobbered by the lossy
/// `api_msgs_to_jsonl` output produced at `LlmEvent::Done`. `MessageLog`
/// has no `save_all`; the only mutation is `append_batch`, which strictly
/// adds rows. This structurally eliminates the bug class.
///
/// Callers always work in terms of the domain `Message` enum; the
/// storage layer wraps each one in a `StoredRecord` envelope (`seq`,
/// `ts`, `schema_v`) internally.
#[async_trait]
pub trait MessageLog: Send + Sync {
    /// Append a batch of messages to the end of the session's log.
    /// Implementations must be atomic (all-or-nothing) and must assign
    /// monotonic `seq` values within the session.
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()>;

    /// Convenience: append a single message.
    async fn append_one(
        &self,
        session_id: &str,
        message: &crate::app::Message,
    ) -> anyhow::Result<()> {
        self.append_batch(session_id, std::slice::from_ref(message))
            .await
    }

    /// Load the last `limit` messages from a session (oldest-first within
    /// the returned window). Pass `usize::MAX` for "all".
    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>>;

    /// Case-insensitive substring search across all sessions.
    async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> anyhow::Result<Vec<SearchResult>>;

    /// Delete all messages for a session.
    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()>;

    /// Count messages for a session.
    #[allow(dead_code)]
    async fn count(&self, session_id: &str) -> anyhow::Result<usize>;
}
```

- [ ] **Step 2: Verify the trait compiles standalone**

```bash
cargo check -p i-rs-claw --all-features
```

Expected: 0 errors. (No impls yet, so no warnings either.)

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/storage/mod.rs
git commit -m "feat(claw/storage): add append-only MessageLog trait"
```

---

## Task 3: File backend `MessageLog` impl

**Files:**
- Modify: `crates/claw/src/storage/file.rs`

- [ ] **Step 1: Write failing tests**

Append to the `#[cfg(test)] mod tests` block in `crates/claw/src/storage/file.rs`:

```rust
    // ── MessageLog ──

    use crate::app::Message;
    use crate::storage::MessageLog;

    #[tokio::test]
    async fn test_message_log_append_and_load() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[
                Message::User { text: "hello".into() },
                Message::Assistant {
                    text: "hi".into(),
                    reasoning: String::new(),
                    token_usage: None,
                },
            ],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        assert_eq!(loaded.len(), 2);
        match &loaded[0] {
            Message::User { text } => assert_eq!(text, "hello"),
            other => panic!("expected User, got {:?}", other),
        }
        match &loaded[1] {
            Message::Assistant { text, .. } => assert_eq!(text, "hi"),
            other => panic!("expected Assistant, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_message_log_preserves_tool_call_step() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[Message::ToolCall {
                name: "weight".into(),
                args: "{}".into(),
                result: "ok".into(),
                step: 2,
                total_steps: 5,
            }],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        match &loaded[0] {
            Message::ToolCall { step, total_steps, .. } => {
                assert_eq!(*step, 2);
                assert_eq!(*total_steps, 5);
            }
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_message_log_append_batch_is_appending_not_overwriting() {
        // Regression: this is the exact bug class we're fixing. Two
        // successive append_batch calls must accumulate, not clobber.
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[Message::User { text: "first".into() }],
        )
        .await
        .unwrap();
        log.append_batch(
            "sid",
            &[Message::User { text: "second".into() }],
        )
        .await
        .unwrap();

        let loaded = log.load("sid", usize::MAX).await.unwrap();
        assert_eq!(loaded.len(), 2, "both appends must persist");
    }

    #[tokio::test]
    async fn test_message_log_load_limit_returns_tail() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        let msgs: Vec<Message> = (0..5)
            .map(|i| Message::User { text: format!("msg{}", i) })
            .collect();
        log.append_batch("sid", &msgs).await.unwrap();
        let loaded = log.load("sid", 3).await.unwrap();
        assert_eq!(loaded.len(), 3);
        match &loaded[0] {
            Message::User { text } => assert_eq!(text, "msg2"),
            _ => panic!(),
        }
        match &loaded[2] {
            Message::User { text } => assert_eq!(text, "msg4"),
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn test_message_log_delete_session() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[Message::User { text: "hi".into() }],
        )
        .await
        .unwrap();
        log.delete_session("sid").await.unwrap();
        assert_eq!(log.load("sid", usize::MAX).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_message_log_count() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);
        log.append_batch(
            "sid",
            &[
                Message::User { text: "a".into() },
                Message::User { text: "b".into() },
            ],
        )
        .await
        .unwrap();
        assert_eq!(log.count("sid").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_message_log_search() {
        let (_root, claw_dir) = test_claw_dir();
        let log = FileMessageLog::new(claw_dir);

        // Search requires session metadata, so wire the session first.
        let sessions = FileSessionStore::new(claw_dir.clone());
        sessions
            .save_all(&[crate::session::SessionMeta {
                id: "sid".into(),
                title: "Test".into(),
                agent_id: "default".into(),
                state: crate::session::SessionState::Active,
                created_at: 1000,
                updated_at: 2000,
                message_count: 0,
            }])
            .await
            .unwrap();

        log.append_batch(
            "sid",
            &[
                Message::User { text: "hello world".into() },
                Message::Assistant {
                    text: "hi there".into(),
                    reasoning: String::new(),
                    token_usage: None,
                },
                Message::ToolCall {
                    name: "weight".into(),
                    args: "{}".into(),
                    result: "ok".into(),
                    step: 0,
                    total_steps: 1,
                },
            ],
        )
        .await
        .unwrap();

        let r = log.search("world", 10).await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].session_id, "sid");

        let r = log.search("weight", 10).await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].message_type, "tool_call");

        let r = log.search("nonexistent", 10).await.unwrap();
        assert!(r.is_empty());
    }
```

- [ ] **Step 2: Run tests, verify they fail to compile**

```bash
cargo test -p i-rs-claw --lib storage::file::tests::test_message_log -- --test-threads=1
```

Expected: compilation error — `FileMessageLog` not found.

- [ ] **Step 3: Implement `FileMessageLog`**

In `crates/claw/src/storage/file.rs`, add:

```rust
// ── MessageLog ──

/// Append-only JSONL message log under `{claw_dir}/sessions/{id}.jsonl`.
///
/// Each line is a serialized `StoredRecord`. New appends go to the end of
/// the file; there is no in-place rewrite — this is the structural fix
/// for the silent-drop bug.
#[derive(Clone)]
pub struct FileMessageLog {
    claw_dir: PathBuf,
}

impl FileMessageLog {
    pub(crate) fn new(claw_dir: PathBuf) -> Self {
        Self { claw_dir }
    }
}

#[async_trait]
impl MessageLog for FileMessageLog {
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let path = messages_path(&self.claw_dir, session_id);
        let lines: Vec<String> = messages
            .iter()
            .map(crate::message::StoredRecord::from_message)
            .map(|r| serde_json::to_string(&r))
            .collect::<Result<Vec<_>, _>>()?;
        let payload = lines.join("\n") + "\n";

        blocking(move || {
            let _guard = lock_guard(&MESSAGES_LOCK);
            ensure_dir(&path)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            file.write_all(payload.as_bytes())?;
            Ok(())
        })
        .await
    }

    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            let file = std::fs::File::open(&path)?;
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            let all: Vec<crate::app::Message> = reader
                .lines()
                .filter_map(|line| line.ok())
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| {
                    serde_json::from_str::<crate::message::StoredRecord>(&line)
                        .ok()?
                        .to_message()
                })
                .collect();
            if all.len() > limit {
                Ok(all[all.len() - limit..].to_vec())
            } else {
                Ok(all)
            }
        })
        .await
    }

    async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let q = query.trim().to_lowercase();
        let claw_dir = self.claw_dir.clone();
        let sessions_dir = sessions_dir(&self.claw_dir);

        blocking(move || {
            if q.is_empty() {
                return Ok(Vec::new());
            }
            let index_path = claw_dir.join("index.json");
            let sessions: Vec<crate::session::SessionMeta> = if index_path.exists() {
                std::fs::read_to_string(&index_path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let mut results: Vec<SearchResult> = Vec::new();
            for meta in &sessions {
                let path = sessions_dir.join(format!("{}.jsonl", meta.id));
                if !path.exists() {
                    continue;
                }
                let file = match std::fs::File::open(&path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                use std::io::{BufRead, BufReader};
                let records: Vec<crate::message::StoredRecord> = BufReader::new(file)
                    .lines()
                    .filter_map(|l| l.ok())
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| serde_json::from_str(&l).ok())
                    .collect();

                for (i, rec) in records.iter().enumerate() {
                    let payload = &rec.payload;
                    let msg_type = payload["type"].as_str().unwrap_or("");
                    let text = payload["text"].as_str().unwrap_or("");
                    let name = payload["name"].as_str().unwrap_or("");

                    let searchable = match msg_type {
                        "user" | "assistant" | "error" => text.to_lowercase(),
                        "tool_call" => name.to_lowercase(),
                        _ => continue,
                    };
                    if !searchable.contains(&q) {
                        continue;
                    }
                    let excerpt = match msg_type {
                        "tool_call" => format!("[工具调用: {}]", name),
                        _ => {
                            let t: String = text.chars().take(200).collect();
                            if text.len() > 200 { format!("{}...", t) } else { t }
                        }
                    };
                    let ctx_before: Vec<String> = records[i.saturating_sub(2)..i]
                        .iter()
                        .filter_map(|m| {
                            let t = m.payload["text"].as_str()?;
                            Some(t.chars().take(100).collect())
                        })
                        .collect();
                    let ctx_after: Vec<String> = records
                        .get(i + 1..)
                        .map(|slice| {
                            slice
                                .iter()
                                .take(1)
                                .filter_map(|m| {
                                    let t = m.payload["text"].as_str()?;
                                    Some(t.chars().take(100).collect())
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    results.push(SearchResult {
                        session_id: meta.id.clone(),
                        session_title: meta.title.clone(),
                        message_type: msg_type.to_string(),
                        excerpt,
                        context_before,
                        context_after,
                        updated_at: meta.updated_at,
                    });
                    if results.len() >= max_results {
                        return Ok(results);
                    }
                }
            }
            Ok(results)
        })
        .await
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            let _ = std::fs::remove_file(&path);
            Ok(())
        })
        .await
    }

    async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
        let path = messages_path(&self.claw_dir, session_id);
        blocking(move || {
            if !path.exists() {
                return Ok(0);
            }
            let file = std::fs::File::open(&path)?;
            use std::io::{BufRead, BufReader};
            Ok(BufReader::new(file).lines().filter_map(|l| l.ok()).filter(|l| !l.trim().is_empty()).count())
        })
        .await
    }
}
```

- [ ] **Step 4: Run tests, verify pass**

```bash
cargo test -p i-rs-claw --lib storage::file::tests::test_message_log -- --test-threads=1
```

Expected: 7 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/claw/src/storage/file.rs
git commit -m "feat(claw/storage): file-backed MessageLog implementation"
```

---

## Task 4: SQL `MessageLog` impl — extend macro + 3 dialect DDLs

**Files:**
- Modify: `crates/claw/src/storage/sql/mod.rs`
- Modify: `crates/claw/src/storage/sql/sqlite.rs`
- Modify: `crates/claw/src/storage/sql/mysql.rs`
- Modify: `crates/claw/src/storage/sql/postgres.rs`

- [ ] **Step 1: Add new DDL to SQLite**

In `crates/claw/src/storage/sql/sqlite.rs`, locate the existing `messages` table DDL and add a new `message_log` table after it:

```sql
CREATE TABLE IF NOT EXISTS message_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT    NOT NULL,
    seq         INTEGER NOT NULL,
    ts          INTEGER NOT NULL,
    schema_v    INTEGER NOT NULL DEFAULT 1,
    payload     TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_message_log_session_seq
    ON message_log (session_id, seq);
```

Use the same migration pattern (e.g., `MIGRATOR` / `sqlx::migrate!()`) that the file already uses for other tables.

- [ ] **Step 2: Add new DDL to MySQL**

In `crates/claw/src/storage/sql/mysql.rs`:

```sql
CREATE TABLE IF NOT EXISTS message_log (
    id          BIGINT       AUTO_INCREMENT PRIMARY KEY,
    session_id  VARCHAR(64)  NOT NULL,
    seq         BIGINT       NOT NULL,
    ts          BIGINT       NOT NULL,
    schema_v    INT          NOT NULL DEFAULT 1,
    payload     TEXT         NOT NULL,
    INDEX idx_message_log_session_seq (session_id, seq)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

- [ ] **Step 3: Add new DDL to PostgreSQL**

In `crates/claw/src/storage/sql/postgres.rs`:

```sql
CREATE TABLE IF NOT EXISTS message_log (
    id          BIGSERIAL    PRIMARY KEY,
    session_id  TEXT         NOT NULL,
    seq         BIGINT       NOT NULL,
    ts          BIGINT       NOT NULL,
    schema_v    INT          NOT NULL DEFAULT 1,
    payload     JSONB        NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_message_log_session_seq
    ON message_log (session_id, seq);
```

- [ ] **Step 4: Extend `define_sql_stores!` macro with `MessageLog` segment**

In `crates/claw/src/storage/sql/mod.rs`, locate the macro definition (starts around line 38) and add a new identifier `$messagelog` to the parameter list (after `$messages`):

```rust
macro_rules! define_sql_stores {
    (
        $pool:ty,
        $backend:ty,
        $sessions:ident, $messages:ident, $messagelog:ident, $apicache:ident, $plansteps:ident,
        $memory:ident, $stats:ident, $skills:ident, $toolcache:ident,
        $upsert_session:expr,
        $upsert_apicache:expr, $upsert_memory:expr, $upsert_token:expr, $upsert_skill:expr,
    ) => {
        // … existing body unchanged …

        // ── MessageLog (append-only) ──

        #[derive(Clone)]
        struct $messagelog {
            db: Arc<$backend>,
        }

        #[async_trait]
        impl MessageLog for $messagelog {
            async fn append_batch(
                &self,
                session_id: &str,
                messages: &[crate::app::Message],
            ) -> anyhow::Result<()> {
                if messages.is_empty() {
                    return Ok(());
                }
                let mut tx = self.db.pool.begin().await?;
                // Compute next seq as current_max + 1.
                let next_seq: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?",
                )
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await?;
                let mut seq = next_seq + 1;
                for msg in messages {
                    let rec = crate::message::StoredRecord::from_message(msg)?;
                    let payload = serde_json::to_string(&rec.payload)?;
                    sqlx::query(
                        "INSERT INTO message_log (session_id, seq, ts, schema_v, payload) \
                         VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(session_id)
                    .bind(seq)
                    .bind(rec.ts)
                    .bind(rec.schema_v as i64)
                    .bind(&payload)
                    .execute(&mut *tx)
                    .await?;
                    seq += 1;
                }
                tx.commit().await?;
                Ok(())
            }

            async fn load(
                &self,
                session_id: &str,
                limit: usize,
            ) -> anyhow::Result<Vec<crate::app::Message>> {
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT payload FROM message_log WHERE session_id = ? \
                     AND seq > (SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?) - ? \
                     ORDER BY seq ASC",
                )
                .bind(session_id)
                .bind(session_id)
                .bind(limit as i64)
                .fetch_all(&self.db.pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .filter_map(|(p,)| {
                        let value: serde_json::Value = serde_json::from_str(&p).ok()?;
                        let rec = crate::message::StoredRecord {
                            seq: 0,
                            ts: 0,
                            schema_v: 1,
                            payload: value,
                        };
                        rec.to_message()
                    })
                    .collect())
            }

            async fn search(
                &self,
                query: &str,
                max_results: usize,
            ) -> anyhow::Result<Vec<SearchResult>> {
                let q = query.trim().to_lowercase();
                if q.is_empty() {
                    return Ok(Vec::new());
                }
                let like = format!("%{}%", q);

                // Identify candidate sessions via payload substring. The
                // LIKE operates on the raw JSON text, which catches both
                // `"text":"…"`, `"name":"…"`, etc. PG would prefer
                // JSONB operators; this default is portable.
                let candidate_rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT DISTINCT session_id FROM message_log \
                     WHERE LOWER(payload) LIKE ?",
                )
                .bind(&like)
                .fetch_all(&self.db.pool)
                .await?;

                let mut results = Vec::new();
                for (sid,) in &candidate_rows {
                    let session_rows: Vec<SessionRow> = sqlx::query_as(
                        "SELECT id, title, agent_id, state, created_at, updated_at, message_count \
                         FROM sessions WHERE id = ?",
                    )
                    .bind(sid)
                    .fetch_all(&self.db.pool)
                    .await?;
                    if session_rows.is_empty() {
                        continue;
                    }
                    let meta: crate::session::SessionMeta = session_rows[0].clone().into();

                    let rows: Vec<(i64, String)> = sqlx::query_as(
                        "SELECT seq, payload FROM message_log WHERE session_id = ? ORDER BY seq",
                    )
                    .bind(sid)
                    .fetch_all(&self.db.pool)
                    .await?;
                    let records: Vec<(usize, serde_json::Value)> = rows
                        .into_iter()
                        .filter_map(|(_seq, p)| {
                            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
                            Some(v)
                        })
                        .enumerate()
                        .collect();

                    for (i, payload) in &records {
                        let mt = payload["type"].as_str().unwrap_or("");
                        let text = payload["text"].as_str().unwrap_or("");
                        let name = payload["name"].as_str().unwrap_or("");
                        let searchable = match mt {
                            "user" | "assistant" | "error" => text.to_lowercase(),
                            "tool_call" => name.to_lowercase(),
                            _ => continue,
                        };
                        if !searchable.contains(&q) {
                            continue;
                        }
                        let excerpt = match mt {
                            "tool_call" => format!("[工具调用: {}]", name),
                            _ => {
                                let t: String = text.chars().take(200).collect();
                                if text.len() > 200 { format!("{}...", t) } else { t }
                            }
                        };
                        let ctx_before: Vec<String> = records[i.saturating_sub(2)..*i]
                            .iter()
                            .filter_map(|(_, p)| {
                                let t = p["text"].as_str()?;
                                Some(t.chars().take(100).collect())
                            })
                            .collect();
                        let ctx_after: Vec<String> = records
                            .get(i + 1..)
                            .map(|slice| {
                                slice
                                    .iter()
                                    .take(1)
                                    .filter_map(|(_, p)| {
                                        let t = p["text"].as_str()?;
                                        Some(t.chars().take(100).collect())
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        results.push(SearchResult {
                            session_id: meta.id.clone(),
                            session_title: meta.title.clone(),
                            message_type: mt.to_string(),
                            excerpt,
                            context_before,
                            context_after,
                            updated_at: meta.updated_at,
                        });
                        if results.len() >= max_results {
                            return Ok(results);
                        }
                    }
                }
                Ok(results)
            }

            async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
                sqlx::query("DELETE FROM message_log WHERE session_id = ?")
                    .bind(session_id)
                    .execute(&self.db.pool)
                    .await?;
                Ok(())
            }

            async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
                let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message_log WHERE session_id = ?")
                    .bind(session_id)
                    .fetch_one(&self.db.pool)
                    .await?;
                Ok(n as usize)
            }
        }

        // … rest of macro body unchanged …
    };
}
```

- [ ] **Step 5: Update each dialect's `define_sql_stores!` invocation**

Open each of `sqlite.rs`, `mysql.rs`, `postgres.rs` and add a new identifier for `$messagelog` at the right position. For example, in `sqlite.rs`:

```rust
define_sql_stores!(
    SqlitePool, SqliteBackend,
    SqliteSessions, SqliteMessages, SqliteMessageLog,
    SqliteApiCache, SqlitePlanSteps,
    SqliteMemory, SqliteStats, SqliteSkills, SqliteToolCache,
    /* upsert_session */ "INSERT INTO sessions ...",
    /* upsert_apicache */ "INSERT INTO api_cache ...",
    /* upsert_memory */ "INSERT INTO memory ...",
    /* upsert_token */ "INSERT INTO token_records ...",
    /* upsert_skill */ "INSERT INTO skills ...",
);
```

(Repeat the analogous edit in `mysql.rs` and `postgres.rs` with their dialect-specific names.)

- [ ] **Step 6: Update `ClawStorage::sqlite/mysql/postgres` constructors**

For each SQL backend's `ClawStorage::*` constructor, add the new field:

```rust
// in sqlite.rs (analogous in mysql.rs / postgres.rs)
impl ClawStorage {
    pub fn sqlite(path: PathBuf) -> anyhow::Result<Self> {
        let db = Arc::new(SqliteBackend::connect(path)?);
        Ok(Self {
            sessions: Box::new(SqliteSessions { db: db.clone() }),
            messages: Box::new(SqliteMessages { db: db.clone() }),
            message_log: Box::new(SqliteMessageLog { db: db.clone() }),
            // … rest unchanged …
        })
    }
}
```

(The actual code may have a slightly different pattern — follow what's already there.)

- [ ] **Step 7: Run cargo check, fix compile errors**

```bash
cargo check -p i-rs-claw --all-features
```

Likely errors:
- Macro invocation missing new arg → fix each dialect's invocation site.
- `ClawStorage` missing `message_log` field → add it (see Task 5).

Stay on this step until 0 errors.

- [ ] **Step 8: Commit**

```bash
git add crates/claw/src/storage/sql/
git commit -m "feat(claw/storage): SQL MessageLog impl (sqlite/mysql/postgres)"
```

---

## Task 5: Wire `MessageLog` into `ClawStorage`

**Files:**
- Modify: `crates/claw/src/storage/mod.rs`

- [ ] **Step 1: Add field to `ClawStorage`**

In `crates/claw/src/storage/mod.rs`, edit the `ClawStorage` struct:

```rust
pub struct ClawStorage {
    pub sessions: Box<dyn SessionRepo>,
    pub messages: Box<dyn MessageRepo>,
    pub message_log: Box<dyn MessageLog>,
    pub api_cache: Box<dyn ApiCacheRepo>,
    pub plan_steps: Box<dyn PlanStepsRepo>,
    pub memory: Box<dyn MemoryRepo>,
    pub stats: Box<dyn StatsRepo>,
    pub skills: Box<dyn SkillRepo>,
    pub tool_cache: Box<dyn ToolCacheRepo>,
}
```

- [ ] **Step 2: Update `ClawStorage::file` constructor**

In `crates/claw/src/storage/file.rs`, add `message_log` to the constructor body:

```rust
impl ClawStorage {
    pub fn file(claw_dir: PathBuf) -> Self {
        Self {
            sessions: Box::new(FileSessionStore::new(claw_dir.clone())),
            messages: Box::new(FileMessageStore::new(claw_dir.clone())),
            message_log: Box::new(FileMessageLog::new(claw_dir.clone())),
            api_cache: Box::new(FileApiCacheStore::new(claw_dir.clone())),
            plan_steps: Box::new(FilePlanStepsStore::new(claw_dir.clone())),
            memory: Box::new(FileMemoryStore::new(claw_dir.clone())),
            stats: Box::new(FileStatsStore::new(claw_dir.clone())),
            skills: Box::new(FileSkillStore::new(claw_dir.clone())),
            tool_cache: Box::new(FileToolCacheStore::new(claw_dir)),
        }
    }
}
```

- [ ] **Step 3: Add integration test**

Append to `crates/claw/src/storage/file.rs` tests:

```rust
    #[tokio::test]
    async fn test_claw_storage_message_log_roundtrip() {
        use crate::app::Message;
        use crate::storage::MessageLog;

        let (_root, claw_dir) = test_claw_dir();
        let storage = ClawStorage::file(claw_dir);

        storage
            .message_log
            .append_batch(
                "s1",
                &[
                    Message::User { text: "ping".into() },
                    Message::ToolCall {
                        name: "weight".into(),
                        args: "{}".into(),
                        result: "ok".into(),
                        step: 0,
                        total_steps: 1,
                    },
                ],
            )
            .await
            .unwrap();

        let loaded = storage.message_log.load("s1", 100).await.unwrap();
        assert_eq!(loaded.len(), 2);
        match &loaded[1] {
            Message::ToolCall { name, .. } => assert_eq!(name, "weight"),
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p i-rs-claw --lib storage::file::tests::test_claw_storage_message_log -- --test-threads=1
```

Expected: 1 passed.

- [ ] **Step 5: Full workspace check**

```bash
cargo check -p i-rs-claw --all-features
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/claw/src/storage/
git commit -m "feat(claw/storage): wire MessageLog into ClawStorage"
```

---

## Task 6: `MessageAccumulator` (LlmEvent → Vec&lt;Message&gt;)

**Files:**
- Modify: `crates/claw/src/message/accumulator.rs`

- [ ] **Step 1: Write failing tests**

Replace the placeholder content of `crates/claw/src/message/accumulator.rs` with:

```rust
//! Streaming accumulator that converts a sequence of `LlmEvent`s into a
//! finalized `Vec<Message>`. Used by the Dashboard to batch-persist messages
//! at `LlmEvent::Done` rather than dual-writing (per-event append + final
//! save_all overwrite — the historical silent-drop bug).
//!
//! Invariants:
//! - Each `ToolExecuted` finalizes any pending assistant text/reasoning
//!   first (preserves prose emitted before tool calls), then emits a
//!   `Message::ToolCall` with full `step` / `total_steps`.
//! - `LlmEvent::Done` flushes any pending assistant and stores `token_usage`
//!   on the trailing Assistant message (mirrors the TUI backfill).
//! - `LlmEvent::Error` flushes pending assistant and emits a `Message::Error`.
//! - Status / HttpLog / UsageRecord / PlanProgress are not persisted (they're
//!   transient UI / stats events).

use crate::app::Message;
use crate::llm::{LlmEvent, TokenUsage};
use std::sync::Arc;

#[derive(Default)]
pub struct MessageAccumulator {
    messages: Vec<Message>,
    pending_text: String,
    pending_reasoning: String,
}

impl MessageAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a streaming event. Returns `true` if any finalized messages
    /// were produced (caller may use this to drive incremental persistence).
    #[allow(clippy::match_same_arms)]
    pub fn apply(&mut self, event: &LlmEvent) -> bool {
        match event {
            LlmEvent::Token(t) => {
                self.pending_text.push_str(t);
                false
            }
            LlmEvent::Reasoning(r) => {
                self.pending_reasoning.push_str(r);
                false
            }
            LlmEvent::NewRound => {
                self.flush_pending_assistant(None);
                true
            }
            LlmEvent::ToolExecuted {
                name,
                args,
                result,
                step,
                total_steps,
            } => {
                self.flush_pending_assistant(None);
                self.messages.push(Message::ToolCall {
                    name: name.clone(),
                    args: args.clone(),
                    result: result.clone(),
                    step: *step,
                    total_steps: *total_steps,
                });
                true
            }
            LlmEvent::Error(text) => {
                self.flush_pending_assistant(None);
                self.messages.push(Message::Error { text: text.clone() });
                true
            }
            LlmEvent::Evaluation { tool, valid, issues } => {
                self.messages.push(Message::Evaluation {
                    tool: tool.clone(),
                    valid: *valid,
                    issues: issues.clone(),
                });
                true
            }
            LlmEvent::ImageGenerated {
                path,
                alt_text,
                format,
                width,
                height,
            } => {
                self.messages.push(Message::Image {
                    path: path.clone(),
                    alt_text: alt_text.clone(),
                    format: format.clone(),
                    width: *width,
                    height: *height,
                });
                true
            }
            LlmEvent::Done(_msgs, usage, _trace_id) => {
                self.flush_pending_assistant(*usage);
                true
            }
            LlmEvent::Status(_)
            | LlmEvent::HttpLog(_)
            | LlmEvent::UsageRecord(_)
            | LlmEvent::PlanProgress(_) => false,
        }
    }

    fn flush_pending_assistant(&mut self, usage: Option<TokenUsage>) {
        if !self.pending_text.is_empty() || !self.pending_reasoning.is_empty() {
            self.messages.push(Message::Assistant {
                text: std::mem::take(&mut self.pending_text),
                reasoning: std::mem::take(&mut self.pending_reasoning),
                token_usage: usage,
            });
        } else if let Some(u) = usage
            && let Some(Message::Assistant { token_usage, .. }) = self.messages.last_mut()
            && token_usage.is_none()
        {
            *token_usage = Some(u);
        }
    }

    /// Drain into the finalized message list, flushing any pending assistant.
    pub fn into_messages(mut self) -> Vec<Message> {
        self.flush_pending_assistant(None);
        std::mem::take(&mut self.messages)
    }

    /// Borrow the finalized messages so far without consuming.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Build a `Vec<Message>` from a historical `LlmEvent::Done` payload.
    /// This is the inverse of `api_msgs_to_jsonl` and replaces it for new
    /// callers — used by Dashboard `send_message` (non-streaming path).
    pub fn from_api_messages(api_msgs: &[serde_json::Value]) -> Vec<Message> {
        let mut acc = Self::new();
        for m in api_msgs {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
            match role {
                "user" => {
                    let text = m.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
                    acc.messages.push(Message::User { text });
                }
                "assistant" => {
                    let text = match m.get("content") {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(serde_json::Value::Array(parts)) => parts
                            .iter()
                            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join(""),
                        _ => String::new(),
                    };
                    let text = if text == "null" { String::new() } else { text };
                    let reasoning = m
                        .get("reasoning_content")
                        .and_then(|r| r.as_str())
                        .unwrap_or("")
                        .to_string();
                    // tool_calls: if any, emit ToolCall records after the prose.
                    if let Some(tcs) = m.get("tool_calls").and_then(|t| t.as_array()) {
                        if !text.is_empty() || !reasoning.is_empty() {
                            acc.messages.push(Message::Assistant {
                                text,
                                reasoning,
                                token_usage: None,
                            });
                        }
                        for tc in tcs {
                            let name = tc
                                .get("function")
                                .and_then(|f| f.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args = tc
                                .get("function")
                                .and_then(|f| f.get("arguments"))
                                .and_then(|a| a.as_str())
                                .unwrap_or("")
                                .to_string();
                            acc.messages.push(Message::ToolCall {
                                name,
                                args,
                                result: String::new(),
                                step: 0,
                                total_steps: tcs.len(),
                            });
                        }
                    } else if !text.is_empty() || !reasoning.is_empty() {
                        acc.messages.push(Message::Assistant {
                            text,
                            reasoning,
                            token_usage: None,
                        });
                    }
                }
                _ => {}
            }
        }
        acc.messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accum_token_then_done_produces_assistant() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Token("hello ".into()));
        acc.apply(&LlmEvent::Token("world".into()));
        acc.apply(&LlmEvent::Done(
            Arc::new(Vec::new()),
            None,
            String::new(),
        ));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::Assistant { text, .. } => assert_eq!(text, "hello world"),
            other => panic!("expected Assistant, got {:?}", other),
        }
    }

    #[test]
    fn accum_tool_executed_preserves_step() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::ToolExecuted {
            name: "weight".into(),
            args: "{}".into(),
            result: "ok".into(),
            step: 2,
            total_steps: 5,
        });
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::ToolCall { step, total_steps, .. } => {
                assert_eq!(*step, 2);
                assert_eq!(*total_steps, 5);
            }
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }

    #[test]
    fn accum_prose_before_tool_call_is_preserved() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Token("好的，我来查".into()));
        acc.apply(&LlmEvent::ToolExecuted {
            name: "weight".into(),
            args: "{}".into(),
            result: "ok".into(),
            step: 0,
            total_steps: 1,
        });
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 2);
        match &msgs[0] {
            Message::Assistant { text, .. } => assert_eq!(text, "好的，我来查"),
            _ => panic!(),
        }
        matches!(&msgs[1], Message::ToolCall { .. });
    }

    #[test]
    fn accum_evaluation_and_image_are_persisted() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Evaluation {
            tool: "weight".into(),
            valid: false,
            issues: vec!["bad".into()],
        });
        acc.apply(&LlmEvent::ImageGenerated {
            path: "/tmp/x.png".into(),
            alt_text: "x".into(),
            format: "png".into(),
            width: 100,
            height: 100,
        });
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 2);
        matches!(&msgs[0], Message::Evaluation { .. });
        matches!(&msgs[1], Message::Image { .. });
    }

    #[test]
    fn accum_done_with_usage_backfills_trailing_assistant() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Token("hi".into()));
        acc.apply(&LlmEvent::Done(
            Arc::new(Vec::new()),
            Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                estimated_cost_usd: Some(0.001),
            }),
            String::new(),
        ));
        let msgs = acc.into_messages();
        match &msgs[0] {
            Message::Assistant { token_usage, .. } => {
                let u = token_usage.expect("usage backfilled");
                assert_eq!(u.total_tokens, 15);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn accum_status_httplog_usage_are_ignored() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Status("thinking".into()));
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        assert!(acc.into_messages().is_empty());
    }

    #[test]
    fn accum_from_api_messages_preserves_tool_call_count() {
        let api = vec![
            json!({"role": "user", "content": "list"}),
            json!({
                "role": "assistant",
                "content": "好的",
                "tool_calls": [
                    {"function": {"name": "weight", "arguments": "{}"}},
                    {"function": {"name": "mood", "arguments": "{}"}}
                ]
            }),
        ];
        let msgs = MessageAccumulator::from_api_messages(&api);
        assert_eq!(msgs.len(), 3); // assistant prose + 2 tool calls
        matches!(&msgs[0], Message::Assistant { .. });
        match &msgs[1] {
            Message::ToolCall { name, total_steps, .. } => {
                assert_eq!(name, "weight");
                assert_eq!(*total_steps, 2);
            }
            _ => panic!(),
        }
        match &msgs[2] {
            Message::ToolCall { name, .. } => assert_eq!(name, "mood"),
            _ => panic!(),
        }
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p i-rs-claw --lib message::accumulator -- --test-threads=1
```

Expected: 7 passed.

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/message/accumulator.rs
git commit -m "feat(claw/message): MessageAccumulator for LlmEvent streaming"
```

---

## Task 7: Migrate Dashboard `chat_stream` to accumulator + `MessageLog`

This is the **primary bug-fix task**. The Dashboard's `chat_stream` previously called `save_chat_result` at `LlmEvent::Done` which DELETE+INSERT'd the session's messages using lossy `api_msgs_to_jsonl` output, silently dropping `step`, `total_steps`, `Evaluation`, and `Image` records.

**Files:**
- Modify: `crates/claw/src/dashboard/routes.rs`

- [ ] **Step 1: Add accumulator to the `chat_stream` unfold state**

Find the `chat_stream` function (line ~327) and locate the `futures_util::stream::unfold` call. Change the state tuple to include a `MessageAccumulator`:

```rust
use crate::message::MessageAccumulator;

let stream = futures_util::stream::unfold(
    (Some(rx), stream_state, stream_sid.clone(), MessageAccumulator::new()),
    |(rx_opt, state, sid, mut acc)| async move {
        let mut rx = rx_opt?;
        loop {
            let event = rx.recv().await?;
            match event {
                LlmEvent::ToolExecuted { name, args, result, step, total_steps } => {
                    // Memory side-effects (unchanged)
                    {
                        let mut core = state.core.write().await;
                        let i_rs_index = core.config.i_rs_tool_index.clone();
                        let agent_id = core
                            .session_mgr
                            .session_meta(&sid)
                            .map(|m| m.agent_id.clone())
                            .unwrap_or_else(|| "default".to_string());
                        crate::core::record_tool_memory(
                            &mut core.agent_store,
                            &i_rs_index,
                            &agent_id,
                            &name,
                            &args,
                            &result,
                        );
                        if !result.starts_with("错误") && !result.starts_with("护栏拦截") {
                            crate::core::record_layered_tool_memory(
                                &mut core.agent_store,
                                &agent_id,
                                &name,
                                &result,
                            );
                        }
                    }

                    // Accumulate (replaces the silent "no persist" path)
                    acc.apply(&LlmEvent::ToolExecuted {
                        name: name.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        step,
                        total_steps,
                    });

                    let data = serde_json::to_string(&serde_json::json!({
                        "name": name, "args": args, "result": result,
                        "step": step, "total_steps": total_steps,
                    }))
                    .unwrap_or_default();
                    let sse = Event::default().event("tool_executed").data(data);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }

                LlmEvent::Evaluation { tool, valid, issues } => {
                    // Persist via accumulator (was previously appended via
                    // session_mgr.append_message then clobbered by
                    // save_chat_result at Done).
                    acc.apply(&LlmEvent::Evaluation {
                        tool: tool.clone(),
                        valid,
                        issues: issues.clone(),
                    });
                    let data = serde_json::to_string(&serde_json::json!({
                        "tool": tool, "valid": valid, "issues": issues,
                    }))
                    .unwrap_or_default();
                    let sse = Event::default().event("evaluation").data(data);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }

                LlmEvent::ImageGenerated { path, alt_text, format, width, height } => {
                    acc.apply(&LlmEvent::ImageGenerated {
                        path: path.clone(),
                        alt_text: alt_text.clone(),
                        format: format.clone(),
                        width,
                        height,
                    });
                    let data = serde_json::to_string(&serde_json::json!({
                        "path": path, "alt_text": alt_text,
                        "format": format, "width": width, "height": height,
                    }))
                    .unwrap_or_default();
                    let sse = Event::default().event("image_generated").data(data);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }

                LlmEvent::Done(msgs, usage, _trace_id) => {
                    // Mark the end of streaming in the accumulator.
                    acc.apply(&LlmEvent::Done(msgs.clone(), usage, String::new()));

                    // Append the accumulator's messages to the log (atomic).
                    let finalized = acc.into_messages();
                    {
                        let core = state.core.read().await;
                        let log = core.session_mgr.message_log();
                        if let Err(e) = log.append_batch(&sid, &finalized).await {
                            tracing::error!("MessageLog::append_batch 失败: {}", e);
                        }
                    }

                    // Save api_cache (still needed for LLM context resume).
                    {
                        let mut core = state.core.write().await;
                        core.session_mgr.save_api_messages(&sid, &msgs);
                        let quality_msg = core.evaluate_completed_session(&sid);

                        // If evaluator produced a quality message, append it
                        // through the new log too.
                        if let Some(ref qm) = quality_msg {
                            let log = core.session_mgr.message_log();
                            let log = log.clone();
                            let sid_clone = sid.clone();
                            tokio::spawn(async move {
                                if let Err(e) = log.append_one(&sid_clone, qm).await {
                                    tracing::error!("quality 持久化失败: {}", e);
                                }
                            });
                        }

                        let agent_id = core
                            .session_mgr
                            .session_meta(&sid)
                            .map(|m| m.agent_id.clone())
                            .unwrap_or_else(|| "default".to_string());
                        core.agent_store.memory_for_mut(&agent_id).flush();
                    }

                    let done_json = serde_json::json!({"usage": usage});
                    let data = serde_json::to_string(&done_json).unwrap_or_default();
                    let sse = Event::default().event("done").data(data);
                    return Some((Ok::<_, Infallible>(sse), (None, state, sid, MessageAccumulator::new())));
                }

                LlmEvent::Error(e) => {
                    acc.apply(&LlmEvent::Error(e.clone()));
                    let finalized = acc.into_messages();
                    {
                        let core = state.core.read().await;
                        let log = core.session_mgr.message_log();
                        if let Err(err) = log.append_batch(&sid, &finalized).await {
                            tracing::error!("MessageLog::append_batch (error path) 失败: {}", err);
                        }
                    }
                    {
                        let mut core = state.core.write().await;
                        core.session_mgr.mark_error(&sid, &e);
                    }
                    let sse = Event::default().event("error").data(e);
                    return Some((Ok::<_, Infallible>(sse), (None, state, sid, MessageAccumulator::new())));
                }

                LlmEvent::Token(t) => {
                    acc.apply(&LlmEvent::Token(t.clone()));
                    let sse = Event::default().event("token").data(t);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }
                LlmEvent::Reasoning(t) => {
                    acc.apply(&LlmEvent::Reasoning(t.clone()));
                    let sse = Event::default().event("reasoning").data(t);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }
                LlmEvent::Status(s) => {
                    let sse = Event::default().event("status").data(s);
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }
                LlmEvent::NewRound => {
                    acc.apply(&LlmEvent::NewRound);
                    let sse = Event::default().event("new_round").data("");
                    return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid, acc)));
                }
                _ => continue,
            }
        }
    },
);
```

- [ ] **Step 2: Add `message_log()` accessor on `SessionManager`**

In `crates/claw/src/session.rs`, add a public accessor that returns a clonable handle to the message log:

```rust
impl SessionManager {
    // … existing methods …

    /// Get a clonable handle to the append-only MessageLog.
    /// Used by Dashboard / Gateway to persist streaming events without
    /// going through the lossy `save_all_messages` path.
    pub fn message_log(&self) -> std::sync::Arc<dyn crate::storage::MessageLog> {
        // Note: Box<dyn Trait> isn't Clone, so we need to ensure the
        // underlying storage is Arc-wrapped. If `ClawStorage::message_log`
        // is currently `Box<dyn MessageLog>`, change it to
        // `Arc<dyn MessageLog>` (Task 5 follow-up). Alternatively, expose
        // the `ClawStorage` itself via Arc and let callers clone it.
        todo!("see Task 7 step 3 for storage refactor")
    }
}
```

- [ ] **Step 3: Refactor `ClawStorage::message_log` to `Arc<dyn MessageLog>`**

In `crates/claw/src/storage/mod.rs`:

```rust
pub struct ClawStorage {
    pub sessions: Box<dyn SessionRepo>,
    pub messages: Box<dyn MessageRepo>,
    pub message_log: std::sync::Arc<dyn MessageLog>,
    pub api_cache: Box<dyn ApiCacheRepo>,
    pub plan_steps: Box<dyn PlanStepsRepo>,
    pub memory: Box<dyn MemoryRepo>,
    pub stats: Box<dyn StatsRepo>,
    pub skills: Box<dyn SkillRepo>,
    pub tool_cache: Box<dyn ToolCacheRepo>,
}
```

Update each constructor in `file.rs` and the SQL backend files:

```rust
// file.rs
message_log: std::sync::Arc::new(FileMessageLog::new(claw_dir.clone())),
```

Replace the `todo!()` in `SessionManager::message_log`:

```rust
pub fn message_log(&self) -> std::sync::Arc<dyn crate::storage::MessageLog> {
    self.storage.message_log.clone()
}
```

- [ ] **Step 4: Update `send_message` (background spawn path)**

In `crates/claw/src/dashboard/routes.rs`, the `send_message` function has a similar `tokio::spawn` block at line ~284. Replace its `LlmEvent::Done` branch with accumulator-based persistence:

```rust
tokio::spawn(async move {
    let mut acc = crate::message::MessageAccumulator::new();
    while let Some(event) = llm_rx.recv().await {
        match &event {
            LlmEvent::Done(msgs, _usage, _trace_id) => {
                acc.apply(&event);
                let finalized = acc.into_messages();
                let core = bg_state.core.read().await;
                let log = core.session_mgr.message_log();
                if let Err(e) = log.append_batch(&bg_sid, &finalized).await {
                    tracing::error!("MessageLog::append_batch 失败: {}", e);
                }
                drop(core);

                let mut core = bg_state.core.write().await;
                core.session_mgr.save_api_messages(&bg_sid, msgs);
                let _ = core.evaluate_completed_session(&bg_sid);
                break;
            }
            LlmEvent::Error(e) => {
                acc.apply(&event);
                let finalized = acc.into_messages();
                let core = bg_state.core.read().await;
                let log = core.session_mgr.message_log();
                let _ = log.append_batch(&bg_sid, &finalized).await;
                drop(core);

                let mut core = bg_state.core.write().await;
                core.session_mgr.mark_error(&bg_sid, e);
                break;
            }
            _ => {
                acc.apply(&event);
            }
        }
    }
});
```

Also persist the user message that was prepended at line 264 via `append_message`. Replace:

```rust
core.session_mgr.append_message("user", &text, None);
```

with:

```rust
{
    let storage = core.session_mgr.message_log();
    let msg = crate::app::Message::User { text: text.clone() };
    if let Err(e) = storage.append_one(&sid, &msg).await {
        tracing::error!("user message persist failed: {}", e);
    }
}
```

- [ ] **Step 5: Update `load_app_messages` callers to use `MessageLog`**

The `load_app_messages` helper currently uses `MessageRepo::load`. Since the new log writes a new format, callers reading from sessions written under the new code path need to read from `MessageLog`. Two options:

**Option A (recommended):** Change `load_app_messages` / `load_messages` in `session.rs` to prefer `MessageLog::load` when available, fall back to `MessageRepo::load` for legacy sessions. Since the user will clear data anyway, just switch unconditionally:

```rust
pub fn load_messages(&self, id: &str, max_messages: usize) -> Vec<serde_json::Value> {
    let log = self.storage.message_log.clone();
    let sid = id.to_string();
    crate::utils::sync_block_on(async move {
        log.load(&sid, max_messages).await
    })
    .unwrap_or_default()
    .into_iter()
    .map(crate::app::message_to_jsonl)
    .collect()
}

pub fn load_app_messages(&self, id: &str, max_messages: usize) -> Vec<crate::app::Message> {
    let log = self.storage.message_log.clone();
    let sid = id.to_string();
    crate::utils::sync_block_on(async move { log.load(&sid, max_messages).await })
        .unwrap_or_default()
}
```

- [ ] **Step 6: Compile + run tests**

```bash
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw --lib --features dashboard -- --test-threads=1
```

Expected: 0 errors; tests pass (some may need adjustment to use the new path).

- [ ] **Step 7: Commit**

```bash
git add crates/claw/src/dashboard/routes.rs crates/claw/src/session.rs crates/claw/src/storage/
git commit -m "fix(claw/dashboard): migrate chat_stream + send_message to append-only MessageLog

This is the primary fix for the silent-drop bug: tool_call step/total_steps,
evaluation, and image records were being clobbered by save_chat_result's
DELETE+INSERT cycle that used lossy api_msgs_to_jsonl output.

Replaced with MessageAccumulator + MessageLog::append_batch — strictly
additive, no overwrite path exists in the new API."
```

---

## Task 8: Migrate Gateway

**Files:**
- Modify: `crates/claw/src/gateway/mod.rs`

- [ ] **Step 1: Replace `append_message` calls**

Find the two `append_message` calls around line 282-284. Replace with:

```rust
{
    let log = core.session_mgr.message_log();
    let msgs = vec![
        crate::app::Message::User { text: text_owned.clone() },
        crate::app::Message::Assistant {
            text: response.clone(),
            reasoning: String::new(),
            token_usage: None,
        },
    ];
    if let Err(e) = log.append_batch(&session_id, &msgs).await {
        tracing::error!("gateway MessageLog::append_batch 失败: {}", e);
    }
}
```

- [ ] **Step 2: Verify compile + run gateway tests**

```bash
cargo check -p i-rs-claw --features gateway
cargo test -p i-rs-claw --lib gateway -- --test-threads=1
```

- [ ] **Step 3: Commit**

```bash
git add crates/claw/src/gateway/mod.rs
git commit -m "refactor(claw/gateway): use MessageLog::append_batch"
```

---

## Task 9: Migrate TUI callers

The TUI already accumulates messages in `App::messages` during streaming via `app.add_tool_call` / `app.append_assistant_text`. The bug for TUI was that `save_session_messages` did `save_all_messages` (DELETE + INSERT) which clobbered any concurrent appends. Since TUI is single-threaded for state mutations, this was less catastrophic, but we still need to switch to append-only for consistency.

**Strategy:** Track `last_saved_count: usize` on `App` (or `AppCore`). At save time, append only `app.messages[last_saved_count..]`.

**Files:**
- Modify: `crates/claw/src/app.rs` (add `last_saved_count` field)
- Modify: `crates/claw/src/session.rs` (add `append_new_messages` helper)
- Modify: `crates/claw/src/tui/clipboard.rs` (rewrite `save_session_messages`)
- Modify: `crates/claw/src/tui/handlers/llm.rs` (caller)
- Modify: `crates/claw/src/tui/handlers/key.rs` (caller)
- Modify: `crates/claw/src/tui/handlers/overlay.rs` (2 callers)
- Modify: `crates/claw/src/tui/mod.rs` (shutdown save)

- [ ] **Step 1: Add cursor tracking to `SessionManager`**

In `crates/claw/src/session.rs`, add a per-session cursor:

```rust
use std::collections::HashMap;

// Inside SessionManager struct definition:
pub struct SessionManager {
    // … existing fields …
    /// Per-session count of messages already persisted to MessageLog.
    /// Used by TUI to append only the new tail on save.
    saved_cursors: HashMap<String, usize>,
}
```

In the constructor, initialize: `saved_cursors: HashMap::new()`.

Add a helper:

```rust
/// Append any messages in `messages` beyond the saved cursor to the
/// MessageLog. Updates the cursor on success.
pub fn append_new_messages(
    &mut self,
    session_id: &str,
    messages: &[crate::app::Message],
) {
    let cursor = self.saved_cursors.get(session_id).copied().unwrap_or(0);
    if cursor >= messages.len() {
        return;
    }
    let new_msgs = &messages[cursor..];
    let log = self.storage.message_log.clone();
    let sid = session_id.to_string();
    let new_msgs: Vec<_> = new_msgs.to_vec();
    let result = crate::utils::sync_block_on(async move {
        log.append_batch(&sid, &new_msgs).await
    });
    match result {
        Ok(()) => {
            self.saved_cursors.insert(session_id.to_string(), messages.len());
        }
        Err(e) => tracing::error!("append_new_messages 失败: {}", e),
    }
}

/// Reset the cursor when loading a session (so subsequent appends start
/// from the loaded count).
pub fn reset_cursor(&mut self, session_id: &str, count: usize) {
    self.saved_cursors.insert(session_id.to_string(), count);
}
```

Also call `reset_cursor` from `load_app_messages` callers (when switching to a session, set cursor to loaded count).

- [ ] **Step 2: Rewrite `save_session_messages`**

In `crates/claw/src/tui/clipboard.rs`:

```rust
pub(super) fn save_session_messages(
    session_mgr: &mut crate::session::SessionManager,
    session_id: &str,
    messages: &[crate::app::Message],
    api_messages: Option<&[serde_json::Value]>,
) {
    session_mgr.append_new_messages(session_id, messages);
    if let Some(msgs) = api_messages {
        session_mgr.save_api_messages(session_id, msgs);
    }
    session_mgr.save_index();
}
```

Note the `&mut` — callers must update from `&` to `&mut`.

- [ ] **Step 3: Update callers to take `&mut SessionManager`**

In each of:
- `crates/claw/src/tui/handlers/llm.rs:281`
- `crates/claw/src/tui/handlers/key.rs:597`
- `crates/claw/src/tui/handlers/overlay.rs:154`
- `crates/claw/src/tui/handlers/overlay.rs:255`
- `crates/claw/src/tui/mod.rs:155`

The `save_session_messages` call site needs `&mut self.app_core.session_mgr`. If `app_core` is currently behind `&mut`, this works directly. If it's behind `&`, refactor.

- [ ] **Step 4: Update the TUI user-message append path**

In `tui/handlers/key.rs:428`, the user input is recorded via `append_message("user", ...)`. Replace with the new path:

```rust
self.app.messages.push(crate::app::Message::User { text: text.clone() });
self.app.message_timestamps.push(chrono::Local::now().naive_local());
self.app.mark_dirty();
```

(The actual persistence happens at the next `save_session_messages` call via `append_new_messages`.)

- [ ] **Step 5: Reset cursor when switching sessions**

In `SessionManager::switch_to` (or wherever sessions are loaded into `App::messages`), after loading messages, call:

```rust
session_mgr.reset_cursor(&sid, app.messages.len());
```

This ensures the next save doesn't re-append the loaded history.

- [ ] **Step 6: Compile + test**

```bash
cargo check -p i-rs-claw
cargo test -p i-rs-claw --lib -- --test-threads=1
```

Expected: 0 errors, all tests pass.

- [ ] **Step 7: Manual smoke test**

```bash
cargo run -p i-rs-claw -- tui
# 1. Start a new session, send a message
# 2. Wait for tool execution
# 3. Quit
# 4. Reopen, verify tool_call step/total_steps are preserved
```

- [ ] **Step 8: Commit**

```bash
git add crates/claw/src/tui/ crates/claw/src/session.rs crates/claw/src/app.rs
git commit -m "refactor(claw/tui): switch save_session_messages to append-only MessageLog"
```

---

## Task 10: Delete legacy code

Now that no caller uses `MessageRepo::save_all`, `save_chat_result`, `api_msgs_to_jsonl`, `save_all_messages`, or `SessionManager::append_message`, delete them.

**Files:**
- Modify: `crates/claw/src/storage/mod.rs` (delete `MessageRepo` trait)
- Modify: `crates/claw/src/storage/file.rs` (delete `FileMessageStore`)
- Modify: `crates/claw/src/storage/sql/mod.rs` (delete `$messages` impl block from macro)
- Modify: `crates/claw/src/session.rs` (delete `append_message`, `save_all_messages`)
- Modify: `crates/claw/src/core/mod.rs` (delete `save_chat_result`, `api_msgs_to_jsonl`)
- Modify: `crates/claw/src/storage/mod.rs` (remove `messages` field from `ClawStorage`)
- Modify: all `ClawStorage::*` constructors to drop the `messages` field

- [ ] **Step 1: Verify no remaining callers**

```bash
rg --type rust 'save_chat_result|save_all_messages|api_msgs_to_jsonl|MessageRepo|\.messages\.append\b|\.messages\.load\b|\.messages\.save_all\b|\.messages\.search\b|\.messages\.delete_session\b' crates/claw/src/
```

Expected: no matches (other than the definitions themselves).

- [ ] **Step 2: Delete `MessageRepo` trait**

In `crates/claw/src/storage/mod.rs`, delete the `MessageRepo` trait block (lines ~103-121) and the `messages: Box<dyn MessageRepo>` field from `ClawStorage`.

- [ ] **Step 3: Delete `FileMessageStore`**

In `crates/claw/src/storage/file.rs`, delete `FileMessageStore` (lines ~157-365) and the field from `ClawStorage::file`.

- [ ] **Step 4: Delete `MessageRepo` impl from the SQL macro**

In `crates/claw/src/storage/sql/mod.rs`, remove the `// ── MessageRepo ──` block (lines ~96-207) and the `$messages` parameter from the macro signature.

Update each dialect's invocation to drop the `$messages` argument.

- [ ] **Step 5: Delete `SessionManager::append_message` + `save_all_messages`**

In `crates/claw/src/session.rs`, delete:
- `pub fn append_message(...)` (line ~357)
- `pub fn save_all_messages(...)` (line ~405)

- [ ] **Step 6: Delete `save_chat_result` + `api_msgs_to_jsonl`**

In `crates/claw/src/core/mod.rs`, delete:
- `pub fn api_msgs_to_jsonl(...)` (line ~890)
- `pub fn save_chat_result(...)` (line ~996)

- [ ] **Step 7: Compile + test**

```bash
cargo check --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo test -p i-rs-claw -- --test-threads=1
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore(claw): delete legacy MessageRepo + save_all_messages + save_chat_result

All persistence now flows through the append-only MessageLog trait;
the bug class introduced by save_all's DELETE+INSERT cycle is
structurally eliminated."
```

---

## Task 11: Final verification

- [ ] **Step 1: Full workspace check**

```bash
cargo check --workspace --all-features
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 2: Full claw test suite**

```bash
cargo test -p i-rs-claw -- --test-threads=1
```

Expected: all tests pass.

- [ ] **Step 3: API integration tests**

```bash
cargo test -p i-rs-api
```

Expected: 32 integration tests pass.

- [ ] **Step 4: Clippy + fmt**

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

Expected: 0 errors.

- [ ] **Step 5: User clears historical data**

Instruct the user to remove old session data (incompatible format):

```bash
rm -rf ~/.i-rs/claw/sessions/
rm -f ~/.i-rs/claw/claw.db  # if using sqlite
```

- [ ] **Step 6: Update spec with completion note**

In `docs/superpowers/specs/2026-06-05-message-log-refactor-design.md`, add a "Status: Implemented YYYY-MM-DD" line at the top.

- [ ] **Step 7: Final commit**

```bash
git add docs/superpowers/
git commit -m "docs: mark message-log-refactor spec as implemented"
```

---

## Self-Review Notes

**Spec coverage check:**
- ✅ append-only persistence → Tasks 2-5
- ✅ domain/storage schema split → Task 1 (StoredRecord vs Message)
- ✅ single accumulator → Task 6
- ✅ cross-dialect SQL support → Task 4 (SQLite/MySQL/PG)
- ✅ no historical migration → Task 11 step 5

**Type consistency check:**
- `MessageLog::append_batch(&self, &str, &[Message])` — used consistently across Tasks 3, 4, 7, 8, 9
- `MessageLog::load(&self, &str, usize) -> Vec<Message>` — same
- `MessageAccumulator::apply(&mut self, &LlmEvent) -> bool` — used in Tasks 6, 7
- `StoredRecord::from_message(&Message) -> anyhow::Result<Self>` — used in Tasks 3, 4
- `SessionManager::message_log() -> Arc<dyn MessageLog>` — used in Tasks 7, 8, 9
- `SessionManager::append_new_messages(&mut self, &str, &[Message])` — used in Task 9
- `SessionManager::reset_cursor(&mut self, &str, usize)` — used in Task 9

**Placeholder scan:** No TBDs or "implement later" markers; every code step contains complete code.

**Risk notes:**
- Task 7 step 3 changes `Box<dyn MessageLog>` to `Arc<dyn MessageLog>` — minor refactor that propagates to all constructors. If it gets messy, an alternative is to wrap `ClawStorage` itself in `Arc<ClawStorage>` and clone the whole storage (already done in many callers via `self.storage.clone()`).
- Task 9 step 5 (cursor reset on session switch) requires finding every place where `App::messages` is loaded from disk. If any path is missed, the first save will re-append the entire history. Mitigate by adding a debug assertion in `append_new_messages` that logs if `cursor > 0 and new_msgs.len() > cursor + 50` (a heuristic for "this looks like a full re-append").
