use crate::error::ClawError;
use crate::llm::LlmEvent;
use crate::provider::create_provider_for;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// Built-in tool that delegates a task to a sub-agent.
///
/// The sub-agent is configured in config.toml under `[agents.{agent_id}]`.
/// The delegator passes the task (and optional context) to the sub-agent,
/// which runs a single LLM inference (no tools) and returns the result.
pub struct DelegateTool;

impl ClawTool for DelegateTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "将任务委托给指定的子智能体处理。适用于需要专业知识或特定能力的复杂任务。\
         使用前需在 config.toml 中配置子智能体（如 [agents.xxx]）。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "目标智能体 ID，在 config.toml 的 [agents] 中配置（如 'analyst'、'coder'）"
                },
                "task": {
                    "type": "string",
                    "description": "需要委托给子智能体处理的任务描述，应清晰完整"
                },
                "context": {
                    "type": "string",
                    "description": "可选的附加上下文信息，帮助子智能体理解任务背景"
                }
            },
            "required": ["agent_id", "task"],
            "additionalProperties": false
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
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

        // Look up agent config by ID
        let agent_config = ctx.config.agent_config(agent_id);

        // Build system prompt for the sub-agent
        let mut system_prompt = String::new();
        if let Some(sp) = &agent_config.system_prompt {
            system_prompt.push_str(sp);
            system_prompt.push_str("\n\n");
        }
        system_prompt.push_str(
            "你是一个专门处理委托任务的智能体。请基于用户提供的任务和上下文，\
             用中文简洁、专业地完成任务。返回你的分析结果或处理结果。",
        );

        let mut messages = vec![
            serde_json::json!({"role": "system", "content": system_prompt}),
        ];

        // Inject optional context as a system message
        if let Some(c) = task_context {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!("背景信息：\n{}", c)
            }));
        }

        messages.push(serde_json::json!({"role": "user", "content": task}));

        // Create provider for the sub-agent
        let provider = create_provider_for(
            &agent_config.provider,
            &agent_config.api_key,
            &agent_config.base_url,
            &agent_config.model,
        );

        // Run the LLM call on the existing tokio runtime
        let result: Result<String, ClawError> = tokio::runtime::Handle::current().block_on(async {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            provider
                .stream_chat(&messages, &[], &tx)
                .await
                .map_err(|e| ClawError::Execution(format!("子智能体调用失败: {}", e)))?;

            // Drop sender so rx.recv() will eventually return None
            drop(tx);

            let mut text = String::new();
            let mut last_error = String::new();

            while let Some(event) = rx.recv().await {
                match event {
                    LlmEvent::Token(t) => text.push_str(&t),
                    LlmEvent::Error(e) => last_error = e,
                    LlmEvent::Done(_, _) => break,
                    _ => {}
                }
            }

            if text.is_empty() && !last_error.is_empty() {
                Err(ClawError::Execution(last_error))
            } else if text.is_empty() {
                Err(ClawError::Execution("子智能体未返回任何内容".to_string()))
            } else {
                Ok(text)
            }
        });

        result
    }
}
