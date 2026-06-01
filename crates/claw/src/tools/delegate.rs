use crate::error::ClawError;
use crate::llm::LlmEvent;
use crate::providers::create_provider_for;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

const MAX_DELEGATE_ROUNDS: u32 = 10;
const MAX_RECENT_CONTEXT_TURNS: usize = 4;

pub struct DelegateTool;

#[async_trait::async_trait]
impl ClawTool for DelegateTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "将任务委托给指定的子智能体处理。子智能体拥有独立的模型配置和工具访问能力，\
         适用于需要专业知识或特定能力的复杂任务。\
         使用前需在 config.toml 中配置子智能体（如 [sub_agents.xxx]）。\
         agent_id 可设为 \"auto\" 自动选择最合适的子智能体。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "目标子智能体 ID（如 'analyst'、'coder'），或 \"auto\" 自动选择"
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

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let raw_agent_id = args
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: agent_id".to_string()))?;

        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: task".to_string()))?;

        let extra_context = args
            .get("context")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let agent_id = if raw_agent_id == "auto" {
            resolve_auto_agent(task, &ctx.config)?
        } else {
            validate_agent_exists(raw_agent_id, &ctx.config)?
        };

        let agent_config = ctx.config.agent_config(&agent_id);

        let mut system_prompt = String::new();
        if let Some(sp) = &agent_config.system_prompt {
            system_prompt.push_str(sp);
            system_prompt.push_str("\n\n");
        }

        system_prompt.push_str("你是一个专门处理委托任务的智能体。请基于用户提供的任务和上下文，\
             用中文简洁、专业地完成任务。你可以使用可用的工具来完成工作。");

        if let Some(rt) = &ctx.delegate_runtime {
            if !rt.user_identity.is_empty() {
                system_prompt.push_str("\n\n");
                system_prompt.push_str(&rt.user_identity);
            }
            if !rt.user_profile.is_empty() {
                system_prompt.push_str("\n\n## 用户画像\n");
                system_prompt.push_str(&rt.user_profile);
            }
            if !rt.user_memory.is_empty() {
                system_prompt.push_str("\n\n## 用户偏好记忆\n");
                system_prompt.push_str(&rt.user_memory);
            }
        }

        let mut messages =
            vec![serde_json::json!({"role": "system", "content": system_prompt})];

        if let Some(rt) = &ctx.delegate_runtime {
            let recent: Vec<Value> = rt
                .recent_messages
                .iter()
                .rev()
                .filter(|m| {
                    m.get("role")
                        .and_then(|r| r.as_str())
                        .map(|r| r == "user" || r == "assistant")
                        .unwrap_or(false)
                })
                .take(MAX_RECENT_CONTEXT_TURNS * 2)
                .cloned()
                .collect();
            if !recent.is_empty() {
                messages.push(serde_json::json!({
                    "role": "system",
                    "content": format!("以下是父会话的最近对话记录，供你参考上下文：\n{}",
                        recent.into_iter().rev().map(|m| {
                            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("unknown");
                            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            format!("[{}] {}", role, content)
                        }).collect::<Vec<_>>().join("\n"))
                }));
            }
        }

        if let Some(c) = extra_context {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!("附加背景信息：\n{}", c)
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
            Some(rt) => (
                rt.mcp_registry.clone(),
                rt.skills.clone(),
                rt.tool_frequency.clone(),
            ),
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
        let parent_tx = ctx.delegate_runtime.as_ref().map(|rt| rt.parent_tx.clone());
        let http_client = ctx.http_client.clone();

        tokio::spawn(async move {
            crate::core::engine::chat_loop(
                provider,
                sub_config,
                messages,
                tx,
                mcp,
                skills,
                tool_frequency,
                http_client,
                None,
            )
            .await;
        });

        let mut text = String::new();
        let mut errors: Vec<(String, crate::error::ErrorCategory)> = Vec::new();
        let mut tool_summary = Vec::new();
        let mut total_input_tokens: u32 = 0;
        let mut total_output_tokens: u32 = 0;
        let mut final_usage: Option<crate::llm::TokenUsage> = None;
        let mut final_model = String::new();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            async {
                while let Some(event) = rx.recv().await {
                    match &event {
                        LlmEvent::Token(t) => text.push_str(t),
                        LlmEvent::ToolExecuted {
                            name,
                            result,
                            step,
                            total_steps,
                            ..
                        } => {
                            let short = if result.len() > 200 {
                                let s: String = result.chars().take(197).collect();
                                format!("{}...", s)
                            } else {
                                result.clone()
                            };
                            tool_summary.push(format!("[{}] {}", name, short));

                            if let Some(ref ptx) = parent_tx {
                                let _ = ptx.send(LlmEvent::Status(format!(
                                    "子智能体 [{}/{}] {}",
                                    step + 1,
                                    total_steps,
                                    name
                                )));
                            }
                        }
                        LlmEvent::Error(e) => {
                            let cat = crate::error::category_from_result(e);
                            errors.push((e.clone(), cat));
                        }
                        LlmEvent::Done(_, usage, trace_id) => {
                            if let Some(u) = usage {
                                total_input_tokens += u.prompt_tokens;
                                total_output_tokens += u.completion_tokens;
                                final_usage = Some(*u);
                            }
                            final_model = trace_id.clone();
                            break;
                        }
                        _ => {}
                    }
                }
            },
        )
        .await;

        if let Some(rt) = &ctx.delegate_runtime {
            if let Some(usage) = final_usage {
                let record = rt.stats_manager.create_record(
                    &format!("delegate:{}", agent_id),
                    &final_model,
                    &agent_config.provider,
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    !tool_summary.is_empty(),
                    tool_summary.len() as u32,
                    0,
                    result.is_ok() && !text.is_empty(),
                    0,
                    &uuid::Uuid::new_v4().to_string(),
                );
                rt.stats_manager.record(record);
            }
        }

        if result.is_err() {
            return Err(ClawError::Timeout(format!(
                "子智能体调用超时 ({}s)",
                timeout_secs
            )));
        }

        if text.is_empty() {
            if !errors.is_empty() {
                let (msg, cat) = &errors[errors.len() - 1];
                return Err(match cat {
                    crate::error::ErrorCategory::Timeout => ClawError::Timeout(msg.clone()),
                    crate::error::ErrorCategory::Network => ClawError::Network(msg.clone()),
                    crate::error::ErrorCategory::Validation => {
                        ClawError::Validation(msg.clone())
                    }
                    _ => ClawError::Execution(msg.clone()),
                });
            }
            return Err(ClawError::Execution("子智能体未返回任何内容".to_string()));
        }

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

fn validate_agent_exists(agent_id: &str, config: &crate::config::Config) -> Result<String, ClawError> {
    if config.agents.contains_key(agent_id) || config.sub_agents.contains_key(agent_id) {
        return Ok(agent_id.to_string());
    }
    let available: Vec<&str> = config
        .agents
        .keys()
        .chain(config.sub_agents.keys())
        .map(|s| s.as_str())
        .collect();
    Err(ClawError::NotFound(format!(
        "子智能体 '{}' 不存在。可用的智能体: {}",
        agent_id,
        available.join(", ")
    )))
}

fn resolve_auto_agent(task: &str, config: &crate::config::Config) -> Result<String, ClawError> {
    if config.sub_agents.is_empty() && config.agents.len() <= 1 {
        return Err(ClawError::NotFound(
            "没有可用的子智能体进行自动路由".to_string(),
        ));
    }

    let sub_agents: Vec<crate::config::ResolvedAgentConfig> = config
        .sub_agents
        .keys()
        .map(|id| config.agent_config(id))
        .collect();

    for agent in &sub_agents {
        if !agent.capabilities.is_empty() {
            for cap in &agent.capabilities {
                if task.contains(cap.as_str()) {
                    tracing::info!(
                        agent_id = %agent.agent_id,
                        capability = %cap,
                        "自动路由匹配子智能体"
                    );
                    return Ok(agent.agent_id.clone());
                }
            }
        }
    }

    let agents: Vec<crate::config::ResolvedAgentConfig> = config
        .agents
        .keys()
        .filter(|id| id.as_str() != "default")
        .map(|id| config.agent_config(id))
        .collect();

    for agent in &agents {
        if !agent.capabilities.is_empty() {
            for cap in &agent.capabilities {
                if task.contains(cap.as_str()) {
                    return Ok(agent.agent_id.clone());
                }
            }
        }
    }

    if let Some(first) = sub_agents.first() {
        return Ok(first.agent_id.clone());
    }

    Err(ClawError::NotFound(
        "自动路由未能匹配到合适的子智能体".to_string(),
    ))
}
