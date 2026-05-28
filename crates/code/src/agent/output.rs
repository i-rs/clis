use crate::agent::event::AgentEvent;
use crate::provider::{LlmMessage, Usage};
use serde_json::Value;
use tokio::sync::mpsc;

const MAX_PROVIDER_RETRIES: u32 = 3;

pub(crate) enum OutputMode<'a> {
    Stdout { json_output: bool },
    Channel { event_tx: &'a mpsc::Sender<AgentEvent> },
}

impl OutputMode<'_> {
    pub(crate) async fn emit_token(&self, text: &str) -> anyhow::Result<()> {
        match self {
            Self::Stdout { json_output } => {
                if *json_output {
                    let ev = serde_json::json!({"event": "token", "content": text});
                    println!("{}", serde_json::to_string(&ev)?);
                } else {
                    print!("{}", text);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            }
            Self::Channel { event_tx } => {
                if event_tx.send(AgentEvent::Token(text.into())).await.is_err() {
                    anyhow::bail!("channel closed");
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn emit_reasoning(&self, text: &str) -> anyhow::Result<()> {
        if let Self::Channel { event_tx } = self
            && event_tx.send(AgentEvent::Reasoning(text.into())).await.is_err() {
                anyhow::bail!("channel closed");
        }
        Ok(())
    }

    pub(crate) async fn emit_tool_call_start(&self, id: &str, name: &str, args: Value) -> anyhow::Result<()> {
        if let Self::Channel { event_tx } = self
            && event_tx.send(AgentEvent::ToolCallStart {
                id: id.into(), name: name.into(), args,
            }).await.is_err() {
                anyhow::bail!("channel closed");
        }
        Ok(())
    }

    pub(crate) async fn emit_retry(&self, wait: u64, attempt: u32) -> anyhow::Result<()> {
        match self {
            Self::Stdout { json_output } if *json_output => {
                let ev = serde_json::json!({"event": "retry", "wait": wait, "attempt": attempt});
                println!("{}", serde_json::to_string(&ev)?);
            }
            Self::Channel { event_tx } => {
                event_tx.send(AgentEvent::Status(
                    format!("Network unstable, retrying in {}s ({}/{})...", wait, attempt, MAX_PROVIDER_RETRIES)
                )).await.ok();
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) async fn emit_error(&self, error: &str) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::Error(error.into())).await.ok();
        }
    }

    pub(crate) async fn emit_tool_call_end(&self, id: &str, name: &str, result: &str) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::ToolCallEnd {
                id: id.into(), name: name.into(), result: result.into(),
            }).await.ok();
        }
    }

    pub(crate) fn emit_tool_created(&self, tool: &Value) {
        if let Self::Stdout { json_output } = self
            && *json_output {
                let ev = serde_json::json!({"event": "tool_created", "tool": tool});
                println!("{}", serde_json::to_string(&ev).unwrap_or_default());
        }
    }

    pub(crate) fn emit_tool_result(&self, call_id: &str, name: &str, result: &str) {
        if let Self::Stdout { json_output } = self
            && *json_output {
                let ev = serde_json::json!({
                    "event": "tool_result", "tool_call_id": call_id, "name": name, "result": result,
                });
                println!("{}", serde_json::to_string(&ev).unwrap_or_default());
        }
    }

    pub(crate) fn emit_request(&self, val: &Value) {
        if let Self::Stdout { json_output } = self
            && *json_output {
                let ev = serde_json::json!({
                    "event": "request",
                    "type": val.get("request_type"),
                    "content": val.get("content"),
                });
                println!("{}", serde_json::to_string(&ev).unwrap_or_default());
        }
    }

    pub(crate) async fn emit_done(&self, usage: Option<Usage>, messages: &[LlmMessage], pct: f64) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::Done {
                usage, messages: messages.to_vec(), context_pct: pct,
            }).await.ok();
        }
    }

    pub(crate) async fn emit_plan(&self, steps: Vec<String>) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::Plan { steps }).await.ok();
        }
    }
}
