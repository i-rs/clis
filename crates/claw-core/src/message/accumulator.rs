//! Streaming accumulator that converts a sequence of `LlmEvent`s into a
//! finalized `Vec<Message>`. Used by Dashboard and Gateway to batch-persist
//! messages at stream end rather than dual-writing (per-event append + final
//! save_all overwrite — the historical silent-drop bug).
//!
//! Invariants:
//! - Each `ToolExecuted` finalizes any pending assistant text/reasoning
//!   first (preserves prose emitted before tool calls), then emits a
//!   `Message::ToolCall` with full `step` / `total_steps`.
//! - `Evaluation` and `ImageGenerated` also flush pending assistant first.
//! - `LlmEvent::Done` flushes any pending assistant and stores `token_usage`
//!   on the trailing Assistant message (mirrors the TUI backfill).
//! - `LlmEvent::Error` flushes pending assistant and emits a `Message::Error`.
//! - Status / HttpLog / UsageRecord / PlanProgress are not persisted (they're
//!   transient UI / stats events).

use crate::app::Message;
use crate::llm::{LlmEvent, TokenUsage};

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
            LlmEvent::NewRound(_) => {
                self.flush_pending_assistant(None);
                true
            }
            LlmEvent::ToolExecuted {
                name,
                args,
                result,
                step,
                total_steps,
                category: _,
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
            LlmEvent::Evaluation {
                tool,
                valid,
                issues,
            } => {
                self.flush_pending_assistant(None);
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
                self.flush_pending_assistant(None);
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
            return;
        }
        // If no pending text but usage is provided, backfill the last
        // Assistant message. Walk backwards because the tail may be
        // ToolCall / Evaluation records flushed by earlier events.
        if let Some(u) = usage {
            for msg in self.messages.iter_mut().rev() {
                if let Message::Assistant { token_usage, .. } = msg {
                    if token_usage.is_none() {
                        *token_usage = Some(u);
                    }
                    break;
                }
            }
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

    /// Build a `Vec<Message>` from a historical `LlmEvent::Done` payload
    /// (API-format message list). Used by Dashboard `send_message`
    /// (non-streaming path) to convert API responses for persistence.
    pub fn from_api_messages(api_msgs: &[serde_json::Value]) -> Vec<Message> {
        let mut acc = Self::new();
        for m in api_msgs {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
            match role {
                "user" => {
                    let text = m
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();
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
    use std::sync::Arc;

    #[test]
    fn accum_token_then_done_produces_assistant() {
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Token("hello ".into()));
        acc.apply(&LlmEvent::Token("world".into()));
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
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
            category: crate::error::ErrorCategory::Unknown,
        });
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::ToolCall {
                step, total_steps, ..
            } => {
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
            category: crate::error::ErrorCategory::Unknown,
        });
        acc.apply(&LlmEvent::Done(Arc::new(Vec::new()), None, String::new()));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 2);
        match &msgs[0] {
            Message::Assistant { text, .. } => assert_eq!(text, "好的，我来查"),
            _ => panic!(),
        }
        assert!(matches!(&msgs[1], Message::ToolCall { .. }));
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
        assert!(matches!(&msgs[0], Message::Evaluation { .. }));
        assert!(matches!(&msgs[1], Message::Image { .. }));
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
    fn accum_usage_backfills_after_tool_call() {
        // When Done arrives after tool calls (no pending text), usage should
        // backfill the last assistant message if there was one.
        let mut acc = MessageAccumulator::new();
        acc.apply(&LlmEvent::Token("let me check".into()));
        acc.apply(&LlmEvent::ToolExecuted {
            name: "weight".into(),
            args: "{}".into(),
            result: "ok".into(),
            step: 0,
            total_steps: 1,
            category: crate::error::ErrorCategory::Unknown,
        });
        // Done arrives — no pending text, but usage should backfill the
        // assistant prose emitted before the tool call.
        acc.apply(&LlmEvent::Done(
            Arc::new(Vec::new()),
            Some(TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 10,
                total_tokens: 30,
                estimated_cost_usd: None,
            }),
            String::new(),
        ));
        let msgs = acc.into_messages();
        assert_eq!(msgs.len(), 2);
        match &msgs[0] {
            Message::Assistant {
                token_usage, text, ..
            } => {
                assert_eq!(text, "let me check");
                assert!(token_usage.is_some());
                assert_eq!(token_usage.unwrap().total_tokens, 30);
            }
            _ => panic!("expected Assistant, got something else"),
        }
        assert!(matches!(&msgs[1], Message::ToolCall { .. }));
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
        assert_eq!(msgs.len(), 4); // user + assistant prose + 2 tool calls
        assert!(matches!(&msgs[0], Message::User { .. }));
        assert!(matches!(&msgs[1], Message::Assistant { .. }));
        match &msgs[2] {
            Message::ToolCall {
                name, total_steps, ..
            } => {
                assert_eq!(name, "weight");
                assert_eq!(*total_steps, 2);
            }
            _ => panic!(),
        }
        match &msgs[3] {
            Message::ToolCall { name, .. } => assert_eq!(name, "mood"),
            _ => panic!(),
        }
    }

    /// Golden test: full event stream → accumulator output is deterministic.
    #[test]
    fn accum_golden_full_roundtrip() {
        let events = vec![
            LlmEvent::Token("I'll ".into()),
            LlmEvent::Token("search".into()),
            LlmEvent::ToolExecuted {
                name: "weight".into(),
                args: r#"{"limit": 7}"#.into(),
                result: "ok".into(),
                step: 0,
                total_steps: 1,
                category: crate::error::ErrorCategory::Unknown,
            },
            LlmEvent::Token("done".into()),
            LlmEvent::Evaluation {
                tool: "weight".into(),
                valid: true,
                issues: vec![],
            },
            LlmEvent::Done(
                Arc::new(Vec::new()),
                Some(TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    total_tokens: 150,
                    estimated_cost_usd: None,
                }),
                String::new(),
            ),
        ];

        let mut acc = MessageAccumulator::new();
        for event in &events {
            acc.apply(event);
        }
        let msgs = acc.into_messages();

        // Expected: Assistant("I'll search"), ToolCall, Assistant("done"), Evaluation
        assert_eq!(msgs.len(), 4);
        // 1. Assistant (prose before first tool call)
        assert!(matches!(&msgs[0], Message::Assistant { text, .. } if text == "I'll search"));
        // 2. ToolCall with step/total_steps
        assert!(
            matches!(&msgs[1], Message::ToolCall { name, args, step, total_steps, .. }
            if name == "weight" && args == r#"{"limit": 7}"# && *step == 0 && *total_steps == 1)
        );
        // 3. Assistant (prose after tool call, usage backfilled by Done)
        match &msgs[2] {
            Message::Assistant {
                text, token_usage, ..
            } => {
                assert_eq!(text, "done");
                assert!(token_usage.is_some());
                assert_eq!(token_usage.unwrap().total_tokens, 150);
            }
            other => panic!("expected Assistant, got {:?}", other),
        }
        // 4. Evaluation
        assert!(matches!(&msgs[3], Message::Evaluation { tool, valid, .. }
            if tool == "weight" && *valid));
    }
}
