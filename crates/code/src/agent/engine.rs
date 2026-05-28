use crate::provider::*;
use crate::router::{ExecutionMode, build_plan_prompt, classify_complexity};
use crate::tools::ToolRegistry;
use super::event::AgentEvent;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;

const MAX_PROVIDER_RETRIES: u32 = 2;
const MAX_TOOL_RETRIES: u32 = 2;

pub trait LoopHooks: Send + Sync {
    fn on_round_start(&self, _round: u32, _messages: &[LlmMessage]) {}
    fn on_tool_result(&self, _name: &str, _result: &str) {}
    fn on_done(&self, _rounds: u32, _usage: &Option<Usage>) {}
}

const MODEL_CONTEXT_LIMIT: usize = 128_000;

fn estimate_tokens(messages: &[LlmMessage], _tool_defs: &[Value]) -> usize {
    let mut total = 0usize;
    for msg in messages {
        match msg {
            LlmMessage::System(s) | LlmMessage::User(s) | LlmMessage::Assistant(s) => total += s.len() / 4,
            LlmMessage::AssistantWithReasoning { content, reasoning, .. } => {
                total += content.len() / 4 + reasoning.len() / 4;
            }
            LlmMessage::Tool { content, .. } => total += content.len() / 4,
            LlmMessage::ToolCall { args, .. } => total += args.to_string().len() / 4,
        }
    }
    total
}

fn exceeds_budget(messages: &[LlmMessage], tool_defs: &[Value]) -> bool {
    estimate_tokens(messages, tool_defs) > MODEL_CONTEXT_LIMIT * 8 / 10
}

pub enum PlanResult {
    Plan(String),
    Direct(String),
}

pub async fn determine_execution_mode(
    task: &str,
    provider: &dyn LlmProvider,
) -> anyhow::Result<(ExecutionMode, Option<String>)> {
    let complexity = classify_complexity(task);
    if complexity.execution_mode() == ExecutionMode::ReAct {
        return Ok((ExecutionMode::ReAct, None));
    }
    let plan = generate_plan(provider, task).await?;
    Ok((ExecutionMode::PlanThenExecute, Some(plan)))
}

pub async fn generate_plan(
    provider: &dyn LlmProvider,
    task: &str,
) -> anyhow::Result<String> {
    let prompt = build_plan_prompt(task);
    let messages = vec![LlmMessage::User(prompt)];
    let tool_defs = vec![];
    let mut rx = provider.stream(&messages, &tool_defs).await;
    let mut plan = String::new();
    while let Some(event) = rx.recv().await {
        match event.kind {
            StreamEventKind::Token(t) => plan.push_str(&t),
            StreamEventKind::Done { .. } => break,
            StreamEventKind::Error(e) => anyhow::bail!("plan generation: {}", e),
            _ => {}
        }
    }
    Ok(plan)
}

fn is_transient_error(e: &str) -> bool {
    crate::provider::error::ProviderError::from_message(e).is_retryable()
}

struct ToolExecResult {
    tool_messages: Vec<(String, String, String)>,
}

async fn execute_tools(
    pending_tool_calls: &[ToolCall],
    tools: &ToolRegistry,
    retry_counts: &mut HashMap<String, u32>,
    tool_timeout_secs: u64,
) -> ToolExecResult {
    let handles: Vec<_> = pending_tool_calls.iter().map(|tc| {
        let tool = tools.get(&tc.name);
        let tc = ToolCall { id: tc.id.clone(), name: tc.name.clone(), args: tc.args.clone() };
        tokio::spawn(async move {
            let result = if let Some(tool) = tool {
                match tc.args.as_object() {
                    Some(obj) => tool.call(obj).await,
                    None => tool.call(&serde_json::Map::new()).await,
                }
            } else {
                Err(anyhow::anyhow!("Unknown tool: {}", tc.name))
            };
            (tc, result)
        })
    }).collect();

    let tc_list: Vec<ToolCall> = pending_tool_calls.to_vec();
    let mut tool_messages = Vec::new();

    for (i, handle) in handles.into_iter().enumerate() {
        let tc = &tc_list[i];
        let result_str = match tokio::time::timeout(std::time::Duration::from_secs(tool_timeout_secs), handle).await {
            Ok(Ok(inner)) => match &inner.1 {
                Ok(s) => s.clone(),
                Err(e) => format!("Error: {}", e),
            },
            Ok(Err(join_err)) => format!("Error: tool task panicked: {}", join_err),
            Err(_) => format!("Error: tool execution timed out ({}s)", tool_timeout_secs),
        };

        if result_str.starts_with("Error:") {
            let count = retry_counts.entry(tc.id.clone()).or_insert(0);
            *count += 1;
        } else {
            retry_counts.remove(&tc.id);
        }
        tool_messages.push((tc.name.clone(), tc.id.clone(), result_str));
    }
    ToolExecResult { tool_messages }
}

fn build_over_limit_message(retry_counts: &HashMap<String, u32>) -> Option<String> {
    let over_limit: Vec<_> = retry_counts.iter()
        .filter(|(_, c)| **c > MAX_TOOL_RETRIES)
        .map(|(id, c)| (id.clone(), *c))
        .collect();
    if over_limit.is_empty() { return None; }
    let details: Vec<String> = over_limit.iter()
        .map(|(id, c)| format!("tool_call_id='{}' ({}次)", id, c))
        .collect();
    Some(format!(
        "工具连续 {} 次调用失败。请反思：\n1. 参数是否正确？\n2. 是否需要换一种方式？\n3. 是否不需要这个工具？\n失败的调用: {}",
        MAX_TOOL_RETRIES, details.join("; ")
    ))
}

enum OutputMode<'a> {
    Stdout { json_output: bool },
    Channel { event_tx: &'a mpsc::Sender<AgentEvent> },
}

impl OutputMode<'_> {
    async fn emit_token(&self, text: &str) -> anyhow::Result<()> {
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

    async fn emit_reasoning(&self, text: &str) -> anyhow::Result<()> {
        if let Self::Channel { event_tx } = self
            && event_tx.send(AgentEvent::Reasoning(text.into())).await.is_err() {
                anyhow::bail!("channel closed");
        }
        Ok(())
    }

    async fn emit_tool_call_start(&self, id: &str, name: &str, args: Value) -> anyhow::Result<()> {
        if let Self::Channel { event_tx } = self
            && event_tx.send(AgentEvent::ToolCallStart {
                id: id.into(), name: name.into(), args,
            }).await.is_err() {
                anyhow::bail!("channel closed");
        }
        Ok(())
    }

    async fn emit_retry(&self, wait: u64, attempt: u32) -> anyhow::Result<()> {
        match self {
            Self::Stdout { json_output } if *json_output => {
                let ev = serde_json::json!({"event": "retry", "wait": wait, "attempt": attempt});
                println!("{}", serde_json::to_string(&ev)?);
            }
            Self::Channel { event_tx } => {
                event_tx.send(AgentEvent::Status(
                    format!("网络波动，{}s 后重试 ({}/{})...", wait, attempt, MAX_PROVIDER_RETRIES)
                )).await.ok();
            }
            _ => {}
        }
        Ok(())
    }

    async fn emit_error(&self, error: &str) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::Error(error.into())).await.ok();
        }
    }

    async fn emit_tool_call_end(&self, id: &str, name: &str, result: &str) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::ToolCallEnd {
                id: id.into(), name: name.into(), result: result.into(),
            }).await.ok();
        }
    }

    fn emit_tool_created(&self, tool: &Value) {
        if let Self::Stdout { json_output } = self
            && *json_output {
                let ev = serde_json::json!({"event": "tool_created", "tool": tool});
                println!("{}", serde_json::to_string(&ev).unwrap_or_default());
        }
    }

    fn emit_tool_result(&self, call_id: &str, name: &str, result: &str) {
        if let Self::Stdout { json_output } = self
            && *json_output {
                let ev = serde_json::json!({
                    "event": "tool_result", "tool_call_id": call_id, "name": name, "result": result,
                });
                println!("{}", serde_json::to_string(&ev).unwrap_or_default());
        }
    }

    fn emit_request(&self, val: &Value) {
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

    async fn emit_done(&self, usage: Option<crate::provider::Usage>, messages: &[LlmMessage], pct: f64) {
        if let Self::Channel { event_tx } = self {
            event_tx.send(AgentEvent::Done {
                usage, messages: messages.to_vec(), context_pct: pct,
            }).await.ok();
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn react_loop_inner(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    output: OutputMode<'_>,
    max_rounds: u32,
    tool_timeout_secs: u64,
    mut cancel_rx: Option<tokio::sync::oneshot::Receiver<()>>,
    hooks: Option<&dyn LoopHooks>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();
    let mut provider_errors: u32 = 0;
    let mut retry_counts: HashMap<String, u32> = HashMap::new();

    for _round in 0..max_rounds {
        if let Some(ref mut rx) = cancel_rx
            && rx.try_recv().is_ok()
        {
            anyhow::bail!("cancelled");
        }
        if let Some(h) = hooks { h.on_round_start(_round, &messages); }
        if exceeds_budget(&messages, tool_defs) {
            messages = ctx.compress(&messages);
        }
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut pending_tool_calls = Vec::new();
        let mut round_usage: Option<crate::provider::Usage> = None;
        let mut had_error = false;

        while let Some(event) = rx.recv().await {
            match event.kind {
                StreamEventKind::Token(t) => {
                    content.push_str(&t);
                    output.emit_token(&t).await?;
                }
                StreamEventKind::Reasoning(r) => {
                    reasoning.push_str(&r);
                    output.emit_reasoning(&r).await?;
                }
                StreamEventKind::ToolCall { id, name, args } => {
                    pending_tool_calls.push(ToolCall { id: id.clone(), name: name.clone(), args: args.clone() });
                    output.emit_tool_call_start(&id, &name, args).await?;
                }
                StreamEventKind::Done { usage, .. } => {
                    round_usage = usage;
                    break;
                }
                StreamEventKind::Error(e) => {
                    had_error = true;
                    if is_transient_error(&e) && provider_errors < MAX_PROVIDER_RETRIES {
                        provider_errors += 1;
                        let wait = 3 * provider_errors as u64;
                        output.emit_retry(wait, provider_errors).await?;
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                        continue;
                    }
                    output.emit_error(&e).await;
                    anyhow::bail!("{}", e);
                }
            }
        }

        if had_error { continue; }
        provider_errors = 0;

        if let Some(u) = round_usage {
            total_usage.input_tokens = total_usage.input_tokens.saturating_add(u.input_tokens);
            total_usage.output_tokens = total_usage.output_tokens.saturating_add(u.output_tokens);
        }

        if !content.is_empty() || !reasoning.is_empty() || !pending_tool_calls.is_empty() {
            final_text = content.clone();
            if !pending_tool_calls.is_empty() {
                messages.push(LlmMessage::AssistantWithReasoning {
                    content, reasoning,
                    tool_calls: pending_tool_calls.clone(),
                });
            } else if reasoning.is_empty() {
                messages.push(LlmMessage::Assistant(content));
            } else {
                messages.push(LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls: Vec::new() });
            }
        }

        if pending_tool_calls.is_empty() { break; }

        let ToolExecResult { tool_messages } = execute_tools(&pending_tool_calls, tools, &mut retry_counts, tool_timeout_secs).await;

        for (name, call_id, result_str) in &tool_messages {
            output.emit_tool_call_end(name, call_id, result_str).await;

            if let Ok(val) = serde_json::from_str::<Value>(result_str)
                && val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                    output.emit_request(&val);
                    output.emit_done(Some(total_usage.clone()), &messages, 0.0).await;
                    return Ok((val.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(), messages));
                }

            if let Ok(val) = serde_json::from_str::<Value>(result_str)
                && val.get("requires_registration").and_then(|v| v.as_bool()).unwrap_or(false)
                && let Some(tool_info) = val.get("tool") {
                    output.emit_tool_created(tool_info);
                }

            output.emit_tool_result(call_id, name, result_str);
            if let Some(h) = hooks { h.on_tool_result(name, result_str); }
            messages.push(LlmMessage::Tool {
                name: name.clone(), content: result_str.clone(), call_id: call_id.clone(),
            });
        }

        if let Some(sys_msg) = build_over_limit_message(&retry_counts) {
            messages.push(LlmMessage::System(sys_msg));
        }
    }

    let _estimated = super::context::ContextManager::estimate_tokens(&messages) as f64 / 128_000.0;
    if let Some(h) = hooks { h.on_done(max_rounds, &Some(total_usage.clone())); }
    output.emit_done(Some(total_usage), &messages, _estimated).await;
    Ok((final_text, messages))
}

pub async fn react_loop(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    json_output: bool,
    max_rounds: u32,
    tool_timeout_secs: u64,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Stdout { json_output };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None, None).await
}

pub async fn react_loop_streaming(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    event_tx: mpsc::Sender<AgentEvent>,
    max_rounds: u32,
    tool_timeout_secs: u64,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Channel { event_tx: &event_tx };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use crate::tools::ToolRegistry;

    fn make_tool_def(name: &str) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": name,
                "description": "test tool",
                "parameters": {"type": "object", "properties": {}}
            }
        })
    }

    #[tokio::test]
    async fn test_determine_execution_mode_simple_is_react() {
        let provider = MockLlmProvider::with_response("");
        let (mode, plan) = determine_execution_mode("read file", &provider).await.unwrap();
        assert_eq!(mode, ExecutionMode::ReAct);
        assert!(plan.is_none());
    }

    #[tokio::test]
    async fn test_determine_execution_mode_heavy_is_plan() {
        let provider = MockLlmProvider::with_response("1. Read the file\n2. Analyze");
        let (mode, plan) = determine_execution_mode("重构这个模块", &provider).await.unwrap();
        assert_eq!(mode, ExecutionMode::PlanThenExecute);
        assert!(plan.is_some());
        assert!(plan.unwrap().contains("Read the file"));
    }

    #[tokio::test]
    async fn test_generate_plan_returns_plan_text() {
        let provider = MockLlmProvider::with_response("Step 1: Do this\nStep 2: Do that");
        let plan = generate_plan(&provider, "implement feature").await.unwrap();
        assert!(plan.contains("Step 1"));
    }

    #[tokio::test]
    async fn test_react_loop_simple_response() {
        let provider = MockLlmProvider::with_response("Hello, world!");
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Say hello".into())];

        let (text, _msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 5, 30
        ).await.expect("react_loop should succeed");

        assert!(text.contains("Hello"), "Expected 'Hello' in response, got: {}", text);
    }

    #[tokio::test]
    async fn test_react_loop_tool_call_then_response() {
        let provider = MockLlmProvider::with_text_and_tool(
            "Let me check...",
            "read", "call-1", r#"{"file_path": "test.txt"}"#,
        );
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Read test.txt".into())];

        let (text, msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 5, 30
        ).await.expect("react_loop should succeed");

        assert!(text.contains("Let me check"), "Response should contain initial text");
        let has_tool_result = msgs.iter().any(|m| matches!(m, LlmMessage::Tool { .. }));
        assert!(has_tool_result, "Tool result should be in message history");
    }

    #[tokio::test]
    async fn test_react_loop_tool_error_triggers_over_limit() {
        let failing_tool = MockTool::with_error("read", "File not found");
        let mut registry = ToolRegistry::new_empty();
        registry.register(std::sync::Arc::new(failing_tool));
        let echo_tool = MockTool::new("bash", "done");
        registry.register(std::sync::Arc::new(echo_tool));

        let provider = MockLlmProvider::with_tool_only("read", "call-1", r#"{"file_path": "missing.txt"}"#);
        let tool_defs = vec![make_tool_def("read"), make_tool_def("bash")];
        let messages = vec![LlmMessage::User("Test".into())];

        let (_text, msgs) = react_loop(
            &provider, &registry, messages, &tool_defs, false, 3, 5
        ).await.expect("react_loop should not bail on tool errors");

        let has_over_limit = msgs.iter().any(|m| matches!(m, LlmMessage::System(s) if s.contains("工具连续")));
        assert!(has_over_limit, "Over-limit reflection prompt should be injected");
    }

    #[tokio::test]
    async fn test_react_loop_max_rounds_limits_loop() {
        let provider = MockLlmProvider::with_tool_only("read", "call-1", r#"{"file_path": "test.txt"}"#);
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Loop test".into())];

        let (_text, _msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 1, 30
        ).await.expect("react_loop with max_rounds=1 should not infinite loop");
    }

    #[tokio::test]
    async fn test_react_loop_streaming_emits_events() {
        let provider = MockLlmProvider::with_response("Hello streaming");
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Test".into())];
        let (tx, mut rx) = mpsc::channel(16);

        let text = {
            let (t, _) = react_loop_streaming(
                &provider, &tools, messages, &tool_defs, tx, 5, 30
            ).await.expect("streaming should succeed");
            t
        };

        assert!(text.contains("Hello"));
        // Should have received Done event
        let events: Vec<AgentEvent> = {
            let mut evs = Vec::new();
            while let Ok(e) = rx.try_recv() { evs.push(e); }
            evs
        };
        assert!(events.iter().any(|e| matches!(e, AgentEvent::Done { .. })));
    }
}
