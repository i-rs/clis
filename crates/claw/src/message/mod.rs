//! Message storage envelope + accumulator module.
//!
//! Splits the on-disk `StoredRecord` (envelope + payload) from the in-memory
//! `Message` enum (re-exported from `crate::app`). All persistence goes
//! through `MessageLog::append_batch` which wraps each `Message` in a
//! `StoredRecord` before writing.

#![allow(dead_code)]

pub mod accumulator;

pub use accumulator::MessageAccumulator;

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
        let msg = Message::User {
            text: "hello".into(),
        };
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
            Message::ToolCall {
                name,
                step,
                total_steps,
                ..
            } => {
                assert_eq!(name, "weight");
                assert_eq!(step, 2);
                assert_eq!(total_steps, 5);
            }
            other => panic!("expected ToolCall, got {:?}", other),
        }
    }

    #[test]
    fn stored_record_payload_is_object_with_type() {
        let msg = Message::Assistant {
            text: "hi".into(),
            reasoning: String::new(),
            token_usage: None,
        };
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
