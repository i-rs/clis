pub(crate) mod builder;
mod execution;

pub(crate) use builder::{
    build_messages, smart_compress, MessageBuildParams,
};
pub(crate) use execution::execute_tool_call;

use crate::config::Config;
use crate::llm::{LlmEvent, StreamResult};
use crate::mcp::McpRegistry;
use crate::providers::LlmProvider;
use crate::skill_store::SkillDefinition;
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Main chat loop: stream, handle tool calls, continue until done
#[tracing::instrument(skip(provider, config, messages, tx, mcp, skills))]
pub async fn chat_loop(
    provider: Box<dyn LlmProvider>,
    config: Config,
    messages: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
    mcp: McpRegistry,
    skills: Vec<SkillDefinition>,
) {
    let enabled = if config.enabled_tools.is_empty() {
        None
    } else {
        Some(&config.enabled_tools)
    };
    let mut tool_schemas = crate::tools::ToolRegistry::with_skills(&skills).enabled_schemas(enabled);
    // Append MCP tool schemas if available
    for (client_idx, tool_def) in &mcp.tools {
        if let Some(_client) = mcp.clients.get(*client_idx) {
            let schema = crate::tools::mcp_tools::mcp_schema_to_openai(tool_def);
            tool_schemas.push(schema);
        }
    }
    let mut msgs = messages;
    let tool_ctx = crate::tools::ToolContext {
        config: config.clone(),
        mcp: mcp.clone(),
        http_client: crate::providers::shared_client(),
    };
    let mut retry_counts: HashMap<String, u32> = HashMap::new();
    let max_retries = config.max_tool_retries;
    let max_rounds = config.max_react_rounds;
    let mut round_count = 0u32;
    let mut consecutive_provider_errors = 0u32;
    const MAX_PROVIDER_RETRIES: u32 = 2;

    // Create shared ToolCallExecutor with configurable parameters
    let executor = crate::core::executor::ToolCallExecutor::new(tool_ctx, mcp.clone(), skills.clone())
        .with_timeout(config.cli_timeout_secs)
        .with_truncation(4096, 500);

    loop {
        round_count += 1;
        if round_count > max_rounds {
            let _ = tx.send(LlmEvent::Error(format!("已达最大执行轮数限制 ({}), 已停止循环。", max_rounds)));
            break;
        }
        let _ = tx.send(LlmEvent::NewRound);
        let _ = tx.send(LlmEvent::Status("🤔 思考中…".to_string()));

        match provider.stream_chat(&msgs, &tool_schemas, &tx).await {
            Ok(StreamResult::Text(usage, text, reasoning)) => {
                if !text.is_empty() || !reasoning.is_empty() {
                    let mut msg = serde_json::json!({
                        "role": "assistant",
                        "content": text,
                    });
                    if !reasoning.is_empty() {
                        msg["reasoning_content"] = serde_json::Value::String(reasoning);
                    }
                    msgs.push(msg);
                }
                let _ = tx.send(LlmEvent::Done(Arc::new(msgs), usage));
                break;
            }
            Ok(StreamResult::ToolCalls(calls, reasoning_content)) => {
                consecutive_provider_errors = 0;
                let tool_calls_array: Vec<Value> = calls
                    .iter()
                    .map(|(tc, _)| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments,
                            }
                        })
                    })
                    .collect();

                let mut assistant_msg = serde_json::json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": tool_calls_array,
                });
                // DeepSeek requires reasoning_content to be echoed back
                if !reasoning_content.is_empty() {
                    assistant_msg["reasoning_content"] = Value::String(reasoning_content);
                }
                msgs.push(assistant_msg);

                // Parallel execute all tool calls via ToolCallExecutor
                let total = calls.len();
                let _ = tx.send(LlmEvent::Status(format!("⚡ 并行执行 {} 个工具...", total)));

                let all_results = executor.execute(calls, &tx).await;

                // Check for errors and track retry counts
                let mut should_retry = false;
                for result in &all_results {
                    if result.result.starts_with("错误:") {
                        let count = retry_counts.entry(result.call.id.clone()).or_insert(0);
                        *count += 1;
                        if *count <= max_retries {
                            should_retry = true;
                        }
                    }
                }

                if should_retry {
                    // Push all results so LLM sees what succeeded/failed
                    for result in &all_results {
                        msgs.push(serde_json::json!({ "role": "tool", "tool_call_id": result.call.id, "content": utils::smart_truncate(&result.result, 500) }));
                    }
                    // Add retry guidance
                    msgs.push(serde_json::json!({
                        "role": "system",
                        "content": "部分工具调用返回错误，请修正参数后重试。".to_string(),
                    }));
                } else {
                    // Add reflection for max-retries-exceeded errors, then push all results
                    for result in &all_results {
                        if result.result.starts_with("错误:") {
                            msgs.push(serde_json::json!({
                                "role": "system",
                                "content": format!(
                                    "工具 '{}' 连续 {} 次调用失败。请反思：\n\
                                     1. 参数是否正确？\n\
                                     2. 是否需要换一种方式完成用户请求？\n\
                                     3. 是否不需要这个工具，用其他方式回答用户？\n\
                                     错误信息：{}",
                                    result.call.name, max_retries, result.result
                                ),
                            }));
                        }
                        let trimmed = utils::smart_truncate(&result.result, 500);
                        msgs.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": result.call.id,
                            "content": trimmed,
                        }));
                    }
                }
                // Continue loop: send tool results back to LLM
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                let is_transient = err_msg.starts_with("API 限流")
                    || err_msg.starts_with("API 服务器错误")
                    || err_msg.starts_with("API 请求失败");

                if is_transient && consecutive_provider_errors < MAX_PROVIDER_RETRIES {
                    consecutive_provider_errors += 1;
                    let wait_secs = 3 * consecutive_provider_errors;
                    tracing::warn!(
                        "Provider 瞬态错误 ({}/{}), 等待 {}s 后重试: {}",
                        consecutive_provider_errors, MAX_PROVIDER_RETRIES, wait_secs, err_msg
                    );
                    let _ = tx.send(LlmEvent::Status(format!(
                        "⚠️ 网络波动，{}s 后重试 ({}/{})…", wait_secs, consecutive_provider_errors, MAX_PROVIDER_RETRIES
                    )));
                    tokio::time::sleep(std::time::Duration::from_secs(wait_secs as u64)).await;
                    continue;
                }
                let _ = tx.send(LlmEvent::Error(err_msg));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builder::build_system_prompt;
    use super::*;
    use crate::providers::ProviderKind;
    use serde_json::json;

    // ── smart_compress tests ──

    #[test]
    fn test_smart_compress_empty_noop() {
        let mut msgs = vec![];
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
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
        smart_compress(&mut actual, &HashMap::new(), 5, 5);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_smart_compress_keeps_system_message() {
        let mut msgs: Vec<Value> = (0..20).map(|i| {
            json!({"role": "user", "content": format!("msg {}", i)})
        }).collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
        assert_eq!(msgs[0]["role"], "system");
    }

    #[test]
    fn test_smart_compress_keeps_recent_messages() {
        let mut msgs: Vec<Value> = (0..20).map(|i| {
            json!({"role": "user", "content": format!("msg {}", i)})
        }).collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        let before_len = msgs.len();
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
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

        smart_compress(&mut msgs, &freq, 1, 5);

        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(content_str.contains("weight skill doc"), "high-frequency teach pair should be kept");
    }

    #[test]
    fn test_smart_compress_preserves_tool_call_pairs() {
        let mut msgs: Vec<Value> = vec![
            json!({"role": "system", "content": "sys"}),
        ];
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

        smart_compress(&mut msgs, &HashMap::new(), 5, 5);

        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(content_str.contains("result"), "tool result should be kept");
        assert!(content_str.contains("some_tool"), "tool_call should be kept");
    }

    // ── build_messages tests ──

    #[test]
    fn test_build_messages_first_turn() {
        use crate::app::Message;
        let params = MessageBuildParams {
            app_messages: &[Message::User { text: "hello".to_string() }],
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
        };
        let result = build_messages(params);
        assert_eq!(result.len(), 3);
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
            app_messages: &[Message::User { text: "prev".to_string() }, Message::Assistant { text: "response".to_string(), reasoning: String::new() }, Message::User { text: "new".to_string() }],
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
            app_messages: &[Message::User { text: "remind".to_string() }],
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
        };
        let result = build_messages(params);
        assert_eq!(result.len(), 4);
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
            app_messages: &[Message::User { text: "done".to_string() }, Message::Assistant { text: "ok".to_string(), reasoning: String::new() }, Message::User { text: "new".to_string() }],
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
        };
        let result = build_messages(params);
        let system_msgs: Vec<_> = result.iter().filter(|m| m["role"] == "system").collect();
        assert_eq!(system_msgs.len(), 2);
        let has_old_reminder = system_msgs.iter().any(|m|
            m["content"].as_str().unwrap_or("").contains("吃药"));
        assert!(!has_old_reminder, "old reminder should be removed");
        let has_new_reminder = system_msgs.iter().any(|m|
            m["content"].as_str().unwrap_or("").contains("新提醒"));
        assert!(has_new_reminder, "new reminder should be present");
    }

    #[test]
    fn test_build_messages_max_turns() {
        use crate::app::Message;
        let app_msgs: Vec<Message> = (0..20).flat_map(|i| vec![
            Message::User { text: format!("q{}", i) },
            Message::Assistant { text: format!("a{}", i), reasoning: String::new() },
        ]).collect();
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
        ) -> anyhow::Result<StreamResult> {
            Ok(StreamResult::ToolCalls(Vec::new(), String::new()))
        }
    }

    #[tokio::test]
    async fn test_chat_loop_single_turn_text() {
        let provider: Box<dyn LlmProvider> = Box::new(
            crate::test_helpers::MockProvider::new(vec![LlmEvent::Token("hello".to_string())]),
        );
        let config = crate::test_helpers::test_config();
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "hi"})];

        chat_loop(provider, config, messages, tx, mcp, vec![]).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Token(t) if t == "hello")),
            "应收到 Token 事件"
        );
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Done(..))),
            "应收到 Done 事件"
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

        chat_loop(provider, config, messages, tx, mcp, vec![]).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Error(msg) if msg.contains("模拟错误"))),
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

        chat_loop(Box::new(AlwaysToolCall), config, messages, tx, mcp, vec![]).await;

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
        let prompt = build_system_prompt("", "", "", "", "", true);
        assert!(
            prompt.contains("Plan-then-Execute"),
            "plan_then_execute=true 时系统提示词应包含 Plan-then-Execute 模式说明"
        );
        assert!(
            prompt.contains("执行计划"),
            "应包含'执行计划'关键词"
        );
    }

    #[test]
    fn test_react_prompt_default() {
        let prompt = build_system_prompt("", "", "", "", "", false);
        assert!(
            prompt.contains("无需预先规划整个流程"),
            "plan_then_execute=false 时系统提示词应包含 ReAct 模式说明"
        );
    }

    #[test]
    fn test_build_system_prompt_date_injection() {
        let prompt = build_system_prompt("", "", "", "", "", false);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(prompt.contains(&today), "应注入当前日期");
        assert!(!prompt.contains("{current_date}"), "占位符应被替换");
    }

    #[test]
    fn test_build_system_prompt_tool_index_injection() {
        let prompt = build_system_prompt("★工具索引★", "", "", "", "", false);
        assert!(prompt.contains("★工具索引★"), "应注入工具索引");
        assert!(!prompt.contains("{{TOOL_INDEX}}"), "TOOL_INDEX 占位符应被替换");
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
