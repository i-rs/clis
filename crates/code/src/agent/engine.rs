use crate::agent::tool_cache::ToolResultCache;
use crate::memory::CrossSessionMemory;
use crate::provider::*;
use crate::router::{ExecutionMode, build_plan_prompt, classify_complexity};
use crate::tools::ToolRegistry;
use super::event::AgentEvent;
use super::output::OutputMode;
use super::tool_exec::{execute_tools, build_over_limit_message, ToolExecResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

const MAX_PROVIDER_RETRIES: u32 = 3;

pub trait LoopHooks: Send + Sync {
    fn on_round_start(&self, _round: u32, _messages: &[LlmMessage]) {}
    fn on_tool_result(&self, _name: &str, _result: &str) {}
    fn on_done(&self, _rounds: u32, _usage: &Option<Usage>) {}
    fn on_round_complete(&self, _round: u32, _messages: &[LlmMessage]) {}
}

const MODEL_CONTEXT_LIMIT: usize = 128_000;

fn estimate_tokens(messages: &[LlmMessage], _tool_defs: &[Value]) -> usize {
    super::context::ContextManager::estimate_tokens(messages)
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
    memory: &mut Option<CrossSessionMemory>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();
    let mut provider_errors: u32 = 0;
    let mut retry_counts: HashMap<String, u32> = HashMap::new();
    let tool_cache = Arc::new(Mutex::new(ToolResultCache::new()));

    // Plan-then-execute: generate plan for complex tasks
    if let Some(first_user_msg) = messages.iter().find_map(|m| match m {
        LlmMessage::User(text) => Some(text.clone()),
        _ => None,
    }) {
        let complexity = crate::router::classify_complexity(&first_user_msg);
        if complexity.execution_mode() == crate::router::ExecutionMode::PlanThenExecute {
            match generate_plan(provider, &first_user_msg).await {
                Ok(plan) if !plan.is_empty() => {
                    let steps: Vec<String> = plan.lines()
                        .filter(|l| l.trim().starts_with(|c: char| c.is_ascii_digit()))
                        .map(|l| l.trim().to_string())
                        .collect();
                    if !steps.is_empty() {
                        let plan_header = format!(
                            "## Execution Plan\n{}\n\nFollow this plan step by step. Mark steps complete as you finish them.",
                            steps.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
                        );
                        messages.insert(0, LlmMessage::System(plan_header));
                        output.emit_plan(steps).await;
                    }
                }
                _ => {}
            }
        }
    }

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
        crate::runtime::rate_limit_wait().await;
        if _round == 0 && crate::runtime::is_verbose() {
            let first_msg = messages.iter().find_map(|m| match m {
                LlmMessage::User(t) => Some(t.as_str()),
                _ => None,
            });
            if let Some(text) = first_msg {
                let complexity = crate::router::classify_complexity(text);
                if let Some(model) = complexity.recommended_model() {
                    eprintln!("[router] complexity={:?} recommended={}", complexity, model);
                }
            }
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
                        let base_delay = 2u64;
                        let delay = base_delay.saturating_mul(1 << provider_errors);
                        let jitter = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() % 1000;
                        let wait = delay + jitter as u64;
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
            crate::runtime::add_usage_with_cost(provider.name(), u.input_tokens, u.output_tokens);
            if crate::runtime::exceeds_token_budget() {
                let used = crate::runtime::total_usage_tokens();
                let budget = crate::runtime::session_token_budget();
                output.emit_tool_result("", "budget", &format!(
                    "Token budget exceeded: {} / {}. Stopping.",
                    used, budget
                ));
                break;
            }
            if crate::runtime::exceeds_cost_budget() {
                let cost = crate::runtime::total_cost_cents();
                output.emit_tool_result("", "budget", &format!(
                    "Cost budget exceeded: ${:.2}. Stopping.",
                    cost as f64 / 100.0
                ));
                break;
            }
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

        let ToolExecResult { tool_messages } = execute_tools(&pending_tool_calls, tools, &mut retry_counts, tool_timeout_secs, memory, Some(&tool_cache)).await;

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

        // M5: After verify tool returns FAIL, inject fix instruction
        for (name, _call_id, result_str) in &tool_messages {
            if name == "verify" && result_str.contains("FAIL") {
                messages.push(LlmMessage::System(
                    "The verification above failed. Please fix the reported errors, then run verify again to confirm. Do not ask for permission — just fix the issues.".into()
                ));
                break;
            }
        }

        if let Some(h) = hooks { h.on_round_complete(_round, &messages); }
    }

    let _estimated = super::context::ContextManager::estimate_tokens(&messages) as f64 / 128_000.0;
    if let Some(h) = hooks { h.on_done(max_rounds, &Some(total_usage.clone())); }
    output.emit_done(Some(total_usage), &messages, _estimated).await;
    Ok((final_text, messages))
}

#[allow(clippy::too_many_arguments)]
pub async fn react_loop(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    json_output: bool,
    max_rounds: u32,
    tool_timeout_secs: u64,
    memory: &mut Option<CrossSessionMemory>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Stdout { json_output };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None, None, memory).await
}

#[allow(clippy::too_many_arguments)]
pub async fn react_loop_streaming(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    event_tx: mpsc::Sender<AgentEvent>,
    max_rounds: u32,
    tool_timeout_secs: u64,
    memory: &mut Option<CrossSessionMemory>,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let output = OutputMode::Channel { event_tx: &event_tx };
    react_loop_inner(provider, tools, messages, tool_defs, output, max_rounds, tool_timeout_secs, None, None, memory).await
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
            &provider, &tools, messages, &tool_defs, false, 5, 30, &mut None
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
            &provider, &tools, messages, &tool_defs, false, 5, 30, &mut None
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
            &provider, &registry, messages, &tool_defs, false, 3, 5, &mut None
        ).await.expect("react_loop should not bail on tool errors");

        let has_over_limit = msgs.iter().any(|m| matches!(m, LlmMessage::System(s) if s.contains("consecutive times")));
        assert!(has_over_limit, "Over-limit reflection prompt should be injected");
    }

    #[tokio::test]
    async fn test_react_loop_max_rounds_limits_loop() {
        let provider = MockLlmProvider::with_tool_only("read", "call-1", r#"{"file_path": "test.txt"}"#);
        let tools = mock_tool_registry();
        let tool_defs = vec![make_tool_def("read")];
        let messages = vec![LlmMessage::User("Loop test".into())];

        let (_text, _msgs) = react_loop(
            &provider, &tools, messages, &tool_defs, false, 1, 30, &mut None
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
                &provider, &tools, messages, &tool_defs, tx, 5, 30, &mut None
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
