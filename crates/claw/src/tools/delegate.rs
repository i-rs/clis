use crate::error::ClawError;
use crate::llm::LlmEvent;
use crate::providers::create_provider_for;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

const MAX_DELEGATE_ROUNDS: u32 = 10;

pub struct DelegateTool;

#[async_trait::async_trait]
impl ClawTool for DelegateTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "将任务委托给指定的子智能体处理。子智能体拥有独立的模型配置和工具访问能力，\
         适用于需要专业知识或特定能力的复杂任务。\
         使用前需在 config.toml 中配置子智能体（如 [sub_agents.xxx]）。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "目标子智能体 ID，在 config.toml 的 [sub_agents] 中配置（如 'analyst'、'coder'）"
                },
                "task": {
                    "type": "string",
                    "description": "需要委托给子智能体处理的任务描述，应清晰完整"
                },
                "context": {
                    "type": "string",
                    "description": "可选的附加上下文信息（如父会话中的相关对话摘要），帮助子智能体理解任务背景"
                }
            },
            "required": ["agent_id", "task"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let agent_id = args
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: agent_id".to_string()))?;

        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: task".to_string()))?;

        let task_context = args
            .get("context")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let agent_config = ctx.config.agent_config(agent_id);

        let mut system_prompt = String::new();
        if let Some(sp) = &agent_config.system_prompt {
            system_prompt.push_str(sp);
            system_prompt.push_str("\n\n");
        }
        system_prompt.push_str(
            "你是一个专门处理委托任务的智能体。请基于用户提供的任务和上下文，\
             用中文简洁、专业地完成任务。你可以使用可用的工具来完成工作。",
        );

        let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];

        if let Some(c) = task_context {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!("背景信息：\n{}", c)
            }));
        }

        messages.push(serde_json::json!({"role": "user", "content": task}));

        tracing::info!(
            agent_id = %agent_id,
            model = %agent_config.model,
            "委托任务给子智能体（含工具支持）"
        );

        let provider = create_provider_for(
            &ctx.http_client,
            &agent_config.provider,
            &agent_config.api_key,
            &agent_config.base_url,
            &agent_config.model,
        );

        let (mcp, skills, tool_frequency) = match &ctx.delegate_runtime {
            Some(rt) => (rt.mcp_registry.clone(), rt.skills.clone(), rt.tool_frequency.clone()),
            None => (
                crate::mcp::McpRegistry::new(&[]),
                vec![],
                std::collections::HashMap::new(),
            ),
        };

        let mut sub_config = ctx.config.clone();
        sub_config.max_react_rounds = MAX_DELEGATE_ROUNDS;
        if !agent_config.enabled_tools.is_empty() {
            sub_config.enabled_tools = agent_config.enabled_tools.clone();
        }
        if !ctx.config.allow_recursive_delegation {
            sub_config.exclude_delegate_tool = true;
        }

        let timeout_secs = if ctx.config.delegate_timeout_secs > 0 {
            ctx.config.delegate_timeout_secs
        } else {
            ctx.config.cli_timeout_secs.max(30) * 3
        };
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            crate::core::engine::chat_loop(
                provider,
                sub_config,
                messages,
                tx,
                mcp,
                skills,
                tool_frequency,
                reqwest::Client::new(),
            )
            .await;
        });

        let mut text = String::new();
        let mut last_error = String::new();
        let mut tool_summary = Vec::new();
        let mut total_input_tokens: u32 = 0;
        let mut total_output_tokens: u32 = 0;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            async {
                while let Some(event) = rx.recv().await {
                    match event {
                        LlmEvent::Token(t) => text.push_str(&t),
                        LlmEvent::ToolExecuted { name, result, .. } => {
                            let short = if result.len() > 200 {
                                let s: String = result.chars().take(197).collect();
                                format!("{}...", s)
                            } else {
                                result.clone()
                            };
                            tool_summary.push(format!("[{}] {}", name, short));
                        }
                        LlmEvent::Error(e) => last_error = e,
                        LlmEvent::Done(_, usage, _) => {
                            if let Some(u) = usage {
                                total_input_tokens += u.prompt_tokens;
                                total_output_tokens += u.completion_tokens;
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            },
        )
        .await;

        if result.is_err() {
            return Err(ClawError::Execution(format!(
                "子智能体调用超时 ({}s)",
                timeout_secs
            )));
        }

        if text.is_empty() && !last_error.is_empty() {
            Err(ClawError::Execution(last_error))
        } else if text.is_empty() {
            Err(ClawError::Execution("子智能体未返回任何内容".to_string()))
        } else {
            let mut output = text;
            if !tool_summary.is_empty() {
                output.push_str("\n\n--- 子智能体工具调用 ---\n");
                output.push_str(&tool_summary.join("\n"));
            }
            if total_input_tokens > 0 || total_output_tokens > 0 {
                output.push_str(&format!(
                    "\n\n--- 子智能体用量 ---\n输入: {} tokens, 输出: {} tokens",
                    total_input_tokens, total_output_tokens
                ));
            }
            Ok(output)
        }
    }
}
