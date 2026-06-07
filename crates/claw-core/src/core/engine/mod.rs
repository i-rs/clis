pub(crate) mod builder;
mod execution;

pub(crate) use builder::{MessageBuildParams, build_messages, smart_compress};
pub(crate) use execution::execute_tool_call;

use crate::config::Config;
use crate::core::context::ContextManager;
use crate::llm::{LlmEvent, StreamResult};
use crate::mcp::McpRegistry;
use crate::providers::LlmProvider;
use crate::skill_store::SkillDefinition;
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

// ── Pipeline stages for chat_loop ──

/// One-time initialization: build tool schemas, inject context advisory,
/// create the shared executor, and prepare retry counters.
struct ChatLoopInit {
    tool_schemas: Vec<Value>,
    executor: crate::core::executor::ToolCallExecutor,
    ctx_mgr: ContextManager,
    max_retries: u32,
    max_rounds: u32,
    tool_frequency: HashMap<String, usize>,
    plan_then_execute: bool,
    checkpoint_store: std::sync::Arc<std::sync::Mutex<crate::core::checkpoint::CheckpointStore>>,
}

#[allow(clippy::too_many_arguments)]
fn prepare_loop(
    provider: &dyn LlmProvider,
    config: &Config,
    msgs: &mut [Value],
    mcp: &McpRegistry,
    skills: &[SkillDefinition],
    tool_frequency: HashMap<String, usize>,
    http_client: reqwest::Client,
    delegate_runtime: Option<std::sync::Arc<crate::tools::DelegateRuntime>>,
    layered_memory: Option<
        std::sync::Arc<std::sync::Mutex<crate::core::layered_memory::LayeredMemory>>,
    >,
    checkpoint_store: std::sync::Arc<std::sync::Mutex<crate::core::checkpoint::CheckpointStore>>,
) -> ChatLoopInit {
    let enabled = if config.enabled_tools.is_empty() {
        None
    } else {
        Some(&config.enabled_tools)
    };
    let i_rs_tool_names: Vec<&str> = config.i_rs_tools.iter().map(|s| s.as_str()).collect();
    let mut reg = crate::tools::ToolRegistry::new()
        .with_skills(skills)
        .with_mcp(mcp);
    if config.exclude_delegate_tool {
        reg = reg.exclude_tool("delegate_task");
    }
    let tool_registry = Arc::new(reg);
    let tool_schemas = tool_registry.enabled_schemas(&i_rs_tool_names, enabled);

    let ctx_mgr = ContextManager::for_model(provider.model());
    let advisory = ctx_mgr.context_advisory(msgs);
    if !advisory.is_empty()
        && msgs
            .first()
            .and_then(|m| m.get("role").and_then(|r| r.as_str()))
            == Some("system")
        && let Some(system_msg) = msgs.first_mut()
        && let Some(content) = system_msg.get("content").and_then(|c| c.as_str())
    {
        system_msg["content"] = Value::String(format!("{}\n{}", content, advisory));
    }

    let tool_ctx = crate::tools::ToolContext {
        config: config.clone(),
        http_client: http_client.clone(),
        delegate_runtime,
    };
    let executor = crate::core::executor::ToolCallExecutor::new(tool_registry, tool_ctx)
        .with_timeout(config.cli_timeout_secs)
        .with_truncation(4096, 500)
        .with_guardrails(
            crate::tools::guardrails::GuardrailManager::new().with_tool(Box::new(
                crate::tools::guardrails::DangerousToolGuardrail::new(vec![
                    "delete".to_string(),
                    "shell".to_string(),
                ]),
            )),
        )
        .with_hitl_policy(
            crate::core::hitl::HitlPolicy::new()
                .with_auto_approve_high_risk(config.hitl.auto_approve_high_risk)
                .auto_approve("i_rs")
                .auto_approve("search")
                .auto_approve("rag")
                .auto_approve("web_search")
                .auto_approve("chart")
                .auto_approve("skill")
                .auto_approve("progress")
                .auto_approve("chain")
                .auto_approve("orchestrate")
                .auto_approve("update_user_memory")
                .require_confirm("file_ops")
                .deny("delete")
                .with_risk_threshold(crate::core::hitl::RiskLevel::High),
        )
        .with_callbacks(std::sync::Arc::new(
            crate::core::callbacks::CallbackChain::new()
                .with(Box::new(crate::core::callbacks::LoggingCallback::new()))
                .with(Box::new(crate::core::callbacks::AuditLogCallback::new())),
        ));
    let executor = if let Some(ref lm) = layered_memory {
        executor.with_layered_memory(std::sync::Arc::clone(lm))
    } else {
        executor
    };

    ChatLoopInit {
        tool_schemas,
        executor,
        ctx_mgr,
        max_retries: config.max_tool_retries,
        max_rounds: config.max_react_rounds,
        tool_frequency,
        plan_then_execute: config.execution_mode == crate::config::ExecutionMode::PlanThenExecute,
        checkpoint_store,
    }
}

/// Stage 1: Stream — send messages to LLM, return StreamResult.
async fn stream_to_llm(
    provider: &dyn LlmProvider,
    msgs: &[Value],
    tool_schemas: &[Value],
    tx: &mpsc::UnboundedSender<LlmEvent>,
    trace_id: &str,
) -> anyhow::Result<StreamResult> {
    provider.stream_chat(msgs, tool_schemas, tx, trace_id).await
}

/// Stage 2: Execute — dispatch tool calls through the executor, trace results.
async fn dispatch_tools(
    executor: &mut crate::core::executor::ToolCallExecutor,
    calls: Vec<(crate::llm::ToolCallAcc, Value)>,
    content: &str,
    tx: &mpsc::UnboundedSender<LlmEvent>,
    msgs: &mut Vec<Value>,
    reasoning_content: &str,
) -> Vec<crate::core::executor::ToolCallResult> {
    // Build assistant tool_call message. Persist the prose the LLM emitted
    // alongside the tool calls (e.g. "好的，先看看 water 工具") instead of
    // always writing `content: null` — otherwise this text would be lost
    // from the session and never rendered on reload.
    let tool_calls_array: Vec<Value> = calls
        .iter()
        .map(|(tc, _)| {
            serde_json::json!({
                "id": tc.id, "type": "function",
                "function": { "name": tc.name, "arguments": tc.arguments }
            })
        })
        .collect();
    let mut assistant_msg = serde_json::json!({
        "role": "assistant", "content": content, "tool_calls": tool_calls_array,
    });
    if !reasoning_content.is_empty() {
        assistant_msg["reasoning_content"] = Value::String(reasoning_content.to_string());
    }
    msgs.push(assistant_msg);

    let total = calls.len();
    let _ = tx.send(LlmEvent::Status(format!("⚡ 并行执行 {} 个工具...", total)));

    executor.execute(calls, tx).await
}

/// Stage 3: Inject — push tool results into messages, decide whether to retry.
/// Returns the backoff duration if a sleep is needed before continuing.
fn inject_results(
    results: &[crate::core::executor::ToolCallResult],
    msgs: &mut Vec<Value>,
    retry_counts: &mut HashMap<String, (u32, u32)>,
    max_retries: u32,
) -> Option<Duration> {
    let mut should_retry = false;
    for r in results {
        if r.category.is_retryable_or_fatal() {
            let entry = retry_counts.entry(r.call.name.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += 1;
            if entry.0 <= max_retries && r.category.is_retryable() {
                should_retry = true;
            }
        } else if let Some(entry) = retry_counts.get_mut(&r.call.name) {
            entry.0 = 0;
            entry.1 = 0;
        }
    }

    if should_retry {
        for r in results {
            msgs.push(serde_json::json!({
                "role": "tool", "tool_call_id": r.call.id,
                "content": utils::compact_tool_result(&r.call.name, &r.result, 500),
            }));
        }
        // Exponential backoff based on consecutive failures
        let max_consecutive = results
            .iter()
            .filter_map(|r| retry_counts.get(&r.call.name))
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(1);
        let backoff_secs = 2u64.saturating_pow(max_consecutive.min(4));
        msgs.push(serde_json::json!({
            "role": "system",
            "content": "部分工具调用返回错误，请修正参数后重试。",
        }));
        if backoff_secs > 1 {
            Some(Duration::from_secs(backoff_secs))
        } else {
            None
        }
    } else {
        for r in results {
            let trimmed = utils::compact_tool_result(&r.call.name, &r.result, 500);
            msgs.push(serde_json::json!({
                "role": "tool", "tool_call_id": r.call.id, "content": trimmed,
            }));
            if r.category.is_retryable_or_fatal() && !r.category.is_retryable() {
                msgs.push(serde_json::json!({
                    "role": "system",
                    "content": format!(
                        "🔧 工具 '{}' 连续 {} 次调用失败，进入反思阶段。\n\n\
                         请按以下步骤分析失败原因并制定替代方案：\n\
                         1. **分析错误**：上次调用的参数是什么？错误信息暗示了什么？\n\
                         2. **参数修正**：如果参数有误，修正后重试\n\
                         3. **替代方案**：如果此工具确实无法完成任务：\n\
                            - 是否有其他工具可以替代？\n\
                            - 是否可以拆分为更简单的步骤？\n\
                            - 是否需要向用户确认需求？\n\
                         4. **最终兜底**：如果无法完成，请坦诚告知用户并说明原因\n\n\
                         重要：不要无意义地重复相同的调用。",
                        r.call.name, max_retries
                    ),
                }));
            }
        }
        None
    }
}

/// Trace all tool call results with structured logging.
fn trace_tool_results(
    results: &[crate::core::executor::ToolCallResult],
    trace_id: &str,
    start_time: std::time::Instant,
) {
    for r in results {
        tracing::info!(
            trace_id = %trace_id,
            tool = %r.call.name,
            args = %r.call.arguments,
            category = ?r.category,
            elapsed_ms = %start_time.elapsed().as_millis(),
            result_len = r.result.len(),
            "工具调用"
        );
    }
}

/// Stage 4: Handle provider error — retry with backoff or give up.
/// Returns true to continue retrying, false to break the loop.
async fn handle_provider_error(
    err: anyhow::Error,
    consecutive_errors: &mut u32,
    max_retries: u32,
    tx: &mpsc::UnboundedSender<LlmEvent>,
) -> bool {
    let err_msg = format!("{}", err);
    let is_transient = err_msg.starts_with("API 限流")
        || err_msg.starts_with("API 服务器错误")
        || err_msg.starts_with("API 请求失败");

    if is_transient && *consecutive_errors < max_retries {
        *consecutive_errors += 1;
        let wait_secs = 2u64.saturating_pow((*consecutive_errors).min(5));
        tracing::warn!(
            "Provider 瞬态错误 ({}/{}), 等待 {}s 后重试: {}",
            *consecutive_errors,
            max_retries,
            wait_secs,
            err_msg
        );
        let _ = tx.send(LlmEvent::Status(format!(
            "⚠️ 网络波动，{}s 后重试 ({}/{})…",
            wait_secs, *consecutive_errors, max_retries
        )));
        tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        true
    } else {
        let _ = tx.send(LlmEvent::Error(err_msg));
        false
    }
}

// ── Orchestrator ──

/// Main chat loop: stream, handle tool calls, continue until done.
///
/// Pipeline: prepare → [stream → dispatch → inject → trace → compress] × N
#[allow(clippy::too_many_arguments, unused_variables, unused_assignments)]
#[tracing::instrument(skip(provider, config, messages, tx, mcp, skills, delegate_runtime))]
pub async fn chat_loop(
    provider: Box<dyn LlmProvider>,
    config: Config,
    messages: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
    mcp: McpRegistry,
    skills: Vec<SkillDefinition>,
    tool_frequency: HashMap<String, usize>,
    http_client: reqwest::Client,
    delegate_runtime: Option<std::sync::Arc<crate::tools::DelegateRuntime>>,
    layered_memory: Option<
        std::sync::Arc<std::sync::Mutex<crate::core::layered_memory::LayeredMemory>>,
    >,
    checkpoint_store: std::sync::Arc<std::sync::Mutex<crate::core::checkpoint::CheckpointStore>>,
) {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let mut msgs = messages;
    let mut init = prepare_loop(
        provider.as_ref(),
        &config,
        &mut msgs,
        &mcp,
        &skills,
        tool_frequency.clone(),
        http_client,
        delegate_runtime,
        layered_memory,
        checkpoint_store,
    );
    let mut retry_counts: HashMap<String, (u32, u32)> = HashMap::new();
    let mut round_count = 0u32;
    let mut consecutive_provider_errors: u32 = 0;
    let mut plan_steps: Vec<crate::app::PlanStep> = Vec::new();
    let mut structured_plan: Option<crate::core::planning::StructuredPlan> = None;
    const MAX_PROVIDER_RETRIES: u32 = 2;
    const HARD_MAX_ROUNDS: u32 = 50;

    loop {
        round_count += 1;
        let effective_max = init.max_rounds.min(HARD_MAX_ROUNDS);
        if round_count > effective_max {
            let _ = tx.send(LlmEvent::Error(format!(
                "已达最大执行轮数限制 ({}), 已停止循环。",
                effective_max
            )));
            break;
        }
        let _ = tx.send(LlmEvent::NewRound(crate::llm::RoundData {
            assistant_text: String::new(),
            tool_calls: Vec::new(),
        }));
        let _ = tx.send(LlmEvent::Status("🤔 思考中…".to_string()));

        let round_start = std::time::Instant::now();
        match stream_to_llm(provider.as_ref(), &msgs, &init.tool_schemas, &tx, &trace_id).await {
            Ok(StreamResult::Text(usage, text, reasoning)) => {
                #[allow(unused_assignments)]
                {
                    consecutive_provider_errors = 0;
                }
                if init.plan_then_execute && round_count == 1 {
                    if let Some(sp) =
                        crate::core::planning::StructuredPlan::parse_from_llm_output(&text)
                    {
                        if matches!(sp.status, crate::core::planning::PlanStatus::Completed) {
                            let _ = tx.send(LlmEvent::Status("📋 计划已完成".to_string()));
                        }
                        structured_plan = Some(sp);
                    }
                    plan_steps = parse_plan_steps(&text);
                    if !plan_steps.is_empty() {
                        let _ = tx.send(LlmEvent::PlanProgress(plan_steps.clone()));
                    }
                }
                if !text.is_empty() || !reasoning.is_empty() {
                    let mut msg = serde_json::json!({ "role": "assistant", "content": text });
                    if !reasoning.is_empty() {
                        msg["reasoning_content"] = Value::String(reasoning);
                    }
                    msgs.push(msg);
                }
                if init.plan_then_execute && !plan_steps.is_empty() {
                    let pending = plan_steps.iter().filter(|s| !s.done).count();
                    if pending > 0 {
                        if let Some(step) = plan_steps.iter_mut().find(|s| !s.done) {
                            step.done = true;
                        }
                        let _ = tx.send(LlmEvent::PlanProgress(plan_steps.clone()));
                    }
                }
                let _ = tx.send(LlmEvent::Done(Arc::new(msgs), usage, trace_id.clone()));
                break;
            }
            Ok(StreamResult::ToolCalls(calls, content, reasoning_content)) => {
                consecutive_provider_errors = 0;
                if calls.is_empty() {
                    tracing::warn!("LLM returned empty tool_calls, treating as done");
                    let _ = tx.send(LlmEvent::Done(Arc::new(msgs), None, trace_id.clone()));
                    break;
                }
                if init.plan_then_execute && !plan_steps.is_empty() {
                    if let Some(step) = plan_steps.iter_mut().find(|s| !s.done) {
                        step.done = true;
                    }
                    let _ = tx.send(LlmEvent::PlanProgress(plan_steps.clone()));
                }
                let results = dispatch_tools(
                    &mut init.executor,
                    calls,
                    &content,
                    &tx,
                    &mut msgs,
                    &reasoning_content,
                )
                .await;

                if let Some(backoff) =
                    inject_results(&results, &mut msgs, &mut retry_counts, init.max_retries)
                {
                    let _ = tx.send(LlmEvent::Status(format!(
                        "⏳ 等待 {}s 后重试失败的工具...",
                        backoff.as_secs()
                    )));
                    tokio::time::sleep(backoff).await;
                }

                trace_tool_results(&results, &trace_id, round_start);

                if let Ok(mut store) = init.checkpoint_store.lock() {
                    let tool_results: std::collections::HashMap<String, String> = results
                        .iter()
                        .map(|r| (r.call.name.clone(), r.result.clone()))
                        .collect();
                    store.save(
                        crate::core::checkpoint::Checkpoint::new(
                            &trace_id,
                            round_count,
                            msgs.clone(),
                        )
                        .with_tool_results(tool_results),
                    );
                }

                init.ctx_mgr.compress(&mut msgs, &init.tool_frequency);
            }
            Err(e) => {
                if !handle_provider_error(
                    e,
                    &mut consecutive_provider_errors,
                    MAX_PROVIDER_RETRIES,
                    &tx,
                )
                .await
                {
                    break;
                }
            }
        }
    }
}

fn parse_plan_steps(text: &str) -> Vec<crate::app::PlanStep> {
    let mut steps = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("📋") || trimmed.contains("执行计划") {
            continue;
        }
        if let Some(rest) = trimmed
            .strip_prefix(|c: char| c.is_ascii_digit())
            .and_then(|s| s.strip_prefix('.'))
            .or_else(|| trimmed.strip_prefix("- "))
        {
            let desc = rest.trim();
            if !desc.is_empty() && desc.len() > 3 {
                steps.push(crate::app::PlanStep {
                    description: desc.to_string(),
                    done: false,
                });
            }
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::builder::build_system_prompt;
    use super::*;
    use crate::providers::ProviderKind;
    use serde_json::json;

    fn tz_test() -> chrono::FixedOffset {
        chrono::FixedOffset::east_opt(8 * 3600).unwrap()
    }

    // ── smart_compress tests ──

    #[test]
    fn test_smart_compress_empty_noop() {
        let mut msgs = vec![];
        smart_compress(&mut msgs, &HashMap::new(), 5, 5, 6);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_smart_compress_below_threshold_noop() {
        let msgs = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "hi"}),
            json!({"role": "assistant", "content": "hello"}),
        ];
        let expected = msgs.clone();
        let mut actual = msgs;
        smart_compress(&mut actual, &HashMap::new(), 5, 5, 6);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_smart_compress_keeps_system_message() {
        let mut msgs: Vec<Value> = (0..20)
            .map(|i| json!({"role": "user", "content": format!("msg {}", i)}))
            .collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        smart_compress(&mut msgs, &HashMap::new(), 5, 5, 6);
        assert_eq!(msgs[0]["role"], "system");
    }

    #[test]
    fn test_smart_compress_keeps_recent_messages() {
        let mut msgs: Vec<Value> = (0..20)
            .map(|i| json!({"role": "user", "content": format!("msg {}", i)}))
            .collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        let before_len = msgs.len();
        smart_compress(&mut msgs, &HashMap::new(), 5, 5, 6);
        assert!(msgs.len() < before_len);
        assert_eq!(msgs[msgs.len() - 1]["content"], "msg 19");
        assert_eq!(msgs[msgs.len() - 5]["content"], "msg 15");
    }

    #[test]
    fn test_smart_compress_keeps_top_teach_pairs() {
        let mut msgs: Vec<Value> = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "filler 1"}),
            json!({"role": "assistant", "content": "filler 1 response"}),
            json!({"role": "user", "content": "filler 2"}),
            json!({"role": "assistant", "content": "filler 2 response"}),
        ];
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{
                "function": {
                    "name": "i_rs",
                    "arguments": r#"{"command":"skill","tool":"weight"}"#
                }
            }]
        }));
        msgs.push(json!({"role": "tool", "content": "weight skill doc"}));
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{
                "function": {
                    "name": "i_rs",
                    "arguments": r#"{"command":"skill","tool":"mood"}"#
                }
            }]
        }));
        msgs.push(json!({"role": "tool", "content": "mood skill doc"}));
        for i in 0..10 {
            msgs.push(json!({"role": "user", "content": format!("recent {}", i)}));
            msgs.push(json!({"role": "assistant", "content": format!("response {}", i)}));
        }
        msgs.push(json!({"role": "user", "content": "final query"}));

        let mut freq = HashMap::new();
        freq.insert("weight".to_string(), 5);
        freq.insert("mood".to_string(), 1);

        smart_compress(&mut msgs, &freq, 1, 5, 6);

        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(
            content_str.contains("weight skill doc"),
            "high-frequency teach pair should be kept"
        );
    }

    #[test]
    fn test_smart_compress_preserves_tool_call_pairs() {
        let mut msgs: Vec<Value> = vec![json!({"role": "system", "content": "sys"})];
        for i in 0..15 {
            msgs.push(json!({"role": "user", "content": format!("old msg {}", i)}));
            msgs.push(json!({"role": "assistant", "content": format!("old resp {}", i)}));
        }
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{"function": {"name": "some_tool", "arguments": "{}"}}]
        }));
        msgs.push(json!({"role": "tool", "content": "result"}));
        msgs.push(json!({"role": "user", "content": "recent"}));
        msgs.push(json!({"role": "assistant", "content": "response"}));

        smart_compress(&mut msgs, &HashMap::new(), 5, 5, 6);

        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(content_str.contains("result"), "tool result should be kept");
        assert!(
            content_str.contains("some_tool"),
            "tool_call should be kept"
        );
    }

    // ── build_messages tests ──

    #[test]
    fn test_build_messages_first_turn() {
        use crate::app::Message;
        let params = MessageBuildParams {
            app_messages: &[Message::User {
                text: "hello".to_string(),
            }],
            user_text: "hello",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: Some("custom system prompt"),
            plan_then_execute: false,
            max_conversation_turns: 8,
            tz_offset: tz_test(),
            identity: "",
            routing_hint: "",
            model: "test",
        };
        let result = build_messages(params);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[0]["content"], "custom system prompt");
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[1]["content"], "hello");
    }

    #[test]
    fn test_build_messages_second_turn() {
        use crate::app::Message;
        let saved = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "prev"}),
            json!({"role": "assistant", "content": "response"}),
        ];
        let params = MessageBuildParams {
            app_messages: &[
                Message::User {
                    text: "prev".to_string(),
                },
                Message::Assistant {
                    text: "response".to_string(),
                    reasoning: String::new(),
                    token_usage: None,
                },
                Message::User {
                    text: "new".to_string(),
                },
            ],
            user_text: "new",
            saved_api_messages: &Some(saved),
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: None,
            plan_then_execute: false,
            max_conversation_turns: 8,
            tz_offset: tz_test(),
            identity: "",
            routing_hint: "",
            model: "test",
        };
        let result = build_messages(params);
        assert!(result.len() >= 3);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[result.len() - 1]["role"], "user");
        assert_eq!(result[result.len() - 1]["content"], "new");
    }

    #[test]
    fn test_build_messages_with_reminder() {
        use crate::app::Message;
        let params = MessageBuildParams {
            app_messages: &[Message::User {
                text: "remind".to_string(),
            }],
            user_text: "remind",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: Some("吃药"),
            system_prompt_override: Some("sys"),
            plan_then_execute: false,
            max_conversation_turns: 8,
            tz_offset: tz_test(),
            identity: "",
            routing_hint: "",
            model: "test",
        };
        let result = build_messages(params);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[1]["role"], "system");
        assert!(result[1]["content"].as_str().unwrap().contains("吃药"));
        assert_eq!(result[2]["role"], "user");
    }

    #[test]
    fn test_build_messages_second_turn_stale_reminder_removed() {
        use crate::app::Message;
        let saved = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "system", "content": "注意：用户有以下即将到期或已到期的提醒事项：\n- 吃药"}),
            json!({"role": "user", "content": "done"}),
            json!({"role": "assistant", "content": "ok"}),
        ];
        let params = MessageBuildParams {
            app_messages: &[
                Message::User {
                    text: "done".to_string(),
                },
                Message::Assistant {
                    text: "ok".to_string(),
                    reasoning: String::new(),
                    token_usage: None,
                },
                Message::User {
                    text: "new".to_string(),
                },
            ],
            user_text: "new",
            saved_api_messages: &Some(saved),
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: Some("新提醒"),
            system_prompt_override: None,
            plan_then_execute: false,
            max_conversation_turns: 8,
            tz_offset: tz_test(),
            identity: "",
            routing_hint: "",
            model: "test",
        };
        let result = build_messages(params);
        let system_msgs: Vec<_> = result.iter().filter(|m| m["role"] == "system").collect();
        assert_eq!(system_msgs.len(), 2);
        let has_old_reminder = system_msgs
            .iter()
            .any(|m| m["content"].as_str().unwrap_or("").contains("吃药"));
        assert!(!has_old_reminder, "old reminder should be removed");
        let has_new_reminder = system_msgs
            .iter()
            .any(|m| m["content"].as_str().unwrap_or("").contains("新提醒"));
        assert!(has_new_reminder, "new reminder should be present");
    }

    #[test]
    fn test_build_messages_max_turns() {
        use crate::app::Message;
        let app_msgs: Vec<Message> = (0..20)
            .flat_map(|i| {
                vec![
                    Message::User {
                        text: format!("q{}", i),
                    },
                    Message::Assistant {
                        text: format!("a{}", i),
                        reasoning: String::new(),
                        token_usage: None,
                    },
                ]
            })
            .collect();
        let params = MessageBuildParams {
            app_messages: &app_msgs,
            user_text: "final",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: Some("sys"),
            plan_then_execute: false,
            max_conversation_turns: 2,
            tz_offset: tz_test(),
            identity: "",
            routing_hint: "",
            model: "test",
        };
        let result = build_messages(params);
        assert_eq!(result.len(), 4);
        assert_eq!(result[1]["content"], "q19");
        assert_eq!(result[2]["content"], "a19");
        assert_eq!(result[3]["content"], "final");
    }

    // ── chat_loop 测试 ──

    struct AlwaysToolCall;

    #[async_trait::async_trait]
    impl LlmProvider for AlwaysToolCall {
        fn kind(&self) -> ProviderKind {
            ProviderKind::OpenAI
        }
        fn model(&self) -> &str {
            "mock"
        }
        async fn stream_chat(
            &self,
            _msgs: &[Value],
            _schemas: &[Value],
            _tx: &mpsc::UnboundedSender<LlmEvent>,
            _trace_id: &str,
        ) -> anyhow::Result<StreamResult> {
            Ok(StreamResult::ToolCalls(
                vec![(
                    crate::llm::ToolCallAcc {
                        id: "call_1".to_string(),
                        name: "nonexistent_tool".to_string(),
                        arguments: "{}".to_string(),
                    },
                    serde_json::json!({}),
                )],
                String::new(),
                String::new(),
            ))
        }
    }

    #[tokio::test]
    async fn test_chat_loop_single_turn_text() {
        let provider: Box<dyn LlmProvider> =
            Box::new(crate::test_helpers::MockProvider::new(vec![
                LlmEvent::Token("hello".to_string()),
            ]));
        let config = crate::test_helpers::test_config();
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "hi"})];

        chat_loop(
            provider,
            config,
            messages,
            tx,
            mcp,
            vec![],
            HashMap::new(),
            reqwest::Client::new(),
            None,
            None,
            std::sync::Arc::new(std::sync::Mutex::new(
                crate::core::checkpoint::CheckpointStore::new(20),
            )),
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, LlmEvent::Token(t) if t == "hello")),
            "应收到 Token 事件"
        );
    }

    #[tokio::test]
    async fn test_chat_loop_provider_error() {
        let provider: Box<dyn LlmProvider> = Box::new(
            crate::test_helpers::MockProvider::new(vec![])
                .with_result(Err(anyhow::anyhow!("模拟错误"))),
        );
        let config = crate::test_helpers::test_config();
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "hi"})];

        chat_loop(
            provider,
            config,
            messages,
            tx,
            mcp,
            vec![],
            HashMap::new(),
            reqwest::Client::new(),
            None,
            None,
            std::sync::Arc::new(std::sync::Mutex::new(
                crate::core::checkpoint::CheckpointStore::new(20),
            )),
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, LlmEvent::Error(msg) if msg.contains("模拟错误"))),
            "应收到 Error 事件"
        );
    }

    #[tokio::test]
    async fn test_chat_loop_max_rounds_exceeded() {
        let mut config = crate::test_helpers::test_config();
        config.max_react_rounds = 2;
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "do work"})];

        chat_loop(
            Box::new(AlwaysToolCall),
            config,
            messages,
            tx,
            mcp,
            vec![],
            HashMap::new(),
            reqwest::Client::new(),
            None,
            None,
            std::sync::Arc::new(std::sync::Mutex::new(
                crate::core::checkpoint::CheckpointStore::new(20),
            )),
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Error(_))),
            "超出最大轮次后应收到 Error 事件"
        );
    }

    #[test]
    fn test_plan_then_execute_prompt() {
        let prompt = build_system_prompt("", "", "", "", "", true, tz_test(), "", "");
        assert!(
            prompt.contains("Plan-then-Execute"),
            "plan_then_execute=true 时系统提示词应包含 Plan-then-Execute 模式说明"
        );
        assert!(prompt.contains("执行计划"), "应包含'执行计划'关键词");
    }

    #[test]
    fn test_react_prompt_default() {
        let prompt = build_system_prompt("", "", "", "", "", false, tz_test(), "", "");
        assert!(
            prompt.contains("无需预先规划整个流程"),
            "plan_then_execute=false 时系统提示词应包含 ReAct 模式说明"
        );
    }

    #[test]
    fn test_build_system_prompt_date_injection() {
        let prompt = build_system_prompt("", "", "", "", "", false, tz_test(), "", "");
        let today = crate::utils::now_in_tz(tz_test())
            .format("%Y-%m-%d")
            .to_string();
        assert!(prompt.contains(&today), "应注入当前日期");
        assert!(!prompt.contains("{current_date}"), "占位符应被替换");
    }

    #[test]
    fn test_build_system_prompt_tool_index_injection() {
        let prompt = build_system_prompt("★工具索引★", "", "", "", "", false, tz_test(), "", "");
        assert!(prompt.contains("★工具索引★"), "应注入工具索引");
        assert!(
            !prompt.contains("{{TOOL_INDEX}}"),
            "TOOL_INDEX 占位符应被替换"
        );
    }

    #[test]
    fn test_build_system_prompt_all_sections() {
        let prompt = build_system_prompt(
            "TOOLS",
            "HOT_TOOLS",
            "SKILLS",
            "MEMORY",
            "PROFILE",
            false,
            tz_test(),
            "",
            "",
        );
        assert!(prompt.contains("TOOLS"), "应有工具索引");
        assert!(prompt.contains("HOT_TOOLS"), "应有热门工具");
        assert!(prompt.contains("SKILLS"), "应有技能");
        assert!(prompt.contains("MEMORY"), "应有记忆");
        assert!(prompt.contains("PROFILE"), "应有用户画像");
        assert!(!prompt.contains("{{TOOL_INDEX}}"));
        assert!(!prompt.contains("{{HOT_TOOLS}}"));
        assert!(!prompt.contains("{{SKILLS}}"));
        assert!(!prompt.contains("{{USER_MEMORY}}"));
        assert!(!prompt.contains("{{USER_PROFILE}}"));
        assert!(!prompt.contains("{{PLAN_MODE}}"));
    }
}
