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

        let system_prompt = build_sub_agent_prompt(&agent_config, ctx);

        let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];

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
            execution_mode = ?agent_config.execution_mode,
            "委托任务给子智能体"
        );

        let provider = create_provider_for(
            &ctx.http_client,
            agent_config.provider,
            &agent_config.api_key,
            &agent_config.base_url,
            &agent_config.model,
        );

        let (mcp, skills, tool_frequency) = match &ctx.delegate_runtime {
            Some(rt) => {
                let mcp = if agent_config.mcp_servers.is_empty() {
                    rt.mcp_registry.clone()
                } else {
                    crate::mcp::McpRegistry::for_agent(&agent_config, &ctx.config.mcp_servers)
                };
                (mcp, rt.skills.clone(), rt.tool_frequency.clone())
            }
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
        if !agent_config.allowed_dirs.is_empty() {
            sub_config.allowed_dirs = agent_config.allowed_dirs.clone();
        }
        if !ctx.config.allow_recursive_delegation {
            sub_config.exclude_delegate_tool = true;
        }
        sub_config.execution_mode = agent_config.execution_mode;

        let timeout_secs = if ctx.config.delegate_timeout_secs > 0 {
            ctx.config.delegate_timeout_secs
        } else {
            ctx.config.cli_timeout_secs.max(30) * 3
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let parent_tx = ctx.delegate_runtime.as_ref().map(|rt| rt.parent_tx.clone());
        let http_client = ctx.http_client.clone();

        let handle = tokio::spawn(async move {
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
                None,
                std::sync::Arc::new(std::sync::Mutex::new(
                    crate::core::checkpoint::CheckpointStore::new(20),
                )),
            )
            .await;
        });

        let mut text = String::new();
        let mut errors: Vec<(String, crate::error::ErrorCategory)> = Vec::new();
        let mut tool_summary = Vec::new();
        let mut total_input_tokens: u32 = 0;
        let mut total_output_tokens: u32 = 0;
        let mut final_model = String::new();
        let mut usage_recorded = false;

        let result = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async {
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
                    LlmEvent::UsageRecord(record) => {
                        total_input_tokens += record.prompt_tokens;
                        total_output_tokens += record.completion_tokens;
                        final_model = record.model.clone();
                        usage_recorded = true;
                    }
                    LlmEvent::Done(_, usage, _trace_id) => {
                        if !usage_recorded && let Some(u) = usage {
                            total_input_tokens += u.prompt_tokens;
                            total_output_tokens += u.completion_tokens;
                        }
                        break;
                    }
                    _ => {}
                }
            }
        })
        .await;

        // Cancel the background task on timeout (#5)
        if result.is_err() {
            handle.abort();
            return Err(ClawError::Timeout(format!(
                "子智能体调用超时 ({}s)",
                timeout_secs
            )));
        }

        // Fallback if no UsageRecord fired (shouldn't happen, but safety)
        if final_model.is_empty() {
            final_model = agent_config.model.clone();
        }

        // Persist usage (#6)
        if let Some(rt) = &ctx.delegate_runtime {
            rt.stats_manager.record(rt.stats_manager.create_record(
                &format!("delegate:{}", agent_id),
                &final_model,
                agent_config.provider.as_str(),
                total_input_tokens,
                total_output_tokens,
                !tool_summary.is_empty(),
                tool_summary.len() as u32,
                0,
                !text.is_empty(),
                0,
                &uuid::Uuid::new_v4().to_string(),
            ));
        }

        if text.is_empty() {
            if !errors.is_empty() {
                let (msg, cat) = &errors[errors.len() - 1];
                return Err(match cat {
                    crate::error::ErrorCategory::Timeout => ClawError::Timeout(msg.clone()),
                    crate::error::ErrorCategory::Network => ClawError::Network(msg.clone()),
                    crate::error::ErrorCategory::Validation => ClawError::Validation(msg.clone()),
                    _ => ClawError::Execution(msg.clone()),
                });
            }
            return Err(ClawError::Execution("子智能体未返回任何内容".to_string()));
        }

        // Structured return (#9): separate text, tools, usage into metadata
        let mut output = serde_json::json!({
            "text": text,
        });
        if !tool_summary.is_empty() {
            output["tools"] = serde_json::json!(tool_summary);
        }
        if total_input_tokens > 0 || total_output_tokens > 0 {
            output["usage"] = serde_json::json!({
                "input_tokens": total_input_tokens,
                "output_tokens": total_output_tokens,
            });
        }
        Ok(serde_json::to_string(&output).unwrap_or(text))
    }
}

fn build_sub_agent_prompt(
    agent_config: &crate::config::ResolvedAgentConfig,
    ctx: &ToolContext,
) -> String {
    let mut prompt = String::new();

    if let Some(sp) = &agent_config.system_prompt {
        prompt.push_str(sp);
        prompt.push_str("\n\n");
    }

    // Timezone-aware date/time injection (#2)
    if let Some(rt) = &ctx.delegate_runtime {
        let now = crate::utils::now_in_tz(rt.tz_offset);
        prompt.push_str(&format!(
            "当前时间: {} ({}), {} (UTC{})\n\n",
            now.format("%Y-%m-%d"),
            now.format("%A"),
            now.format("%H:%M"),
            crate::utils::tz_label(rt.tz_offset),
        ));
    }

    prompt.push_str(
        "你是一个专门处理委托任务的智能体。请基于用户提供的任务和上下文，\
         用中文简洁、专业地完成任务。你可以使用可用的工具来完成工作。",
    );

    // Tool index injection
    if let Some(rt) = &ctx.delegate_runtime
        && !rt.irs_tool_index.is_empty()
    {
        let mut tool_index_section = String::from("\n\n## 可用工具\n");
        let enabled = &agent_config.enabled_tools;
        let mut has_tools = false;
        for (name, desc) in &rt.irs_tool_index {
            if !enabled.is_empty() && !enabled.contains(name) {
                continue;
            }
            if !desc.is_empty() {
                tool_index_section.push_str(&format!("\n- {}: {}", name, desc));
            } else {
                tool_index_section.push_str(&format!("\n- {}", name));
            }
            has_tools = true;
        }
        if has_tools {
            prompt.push_str(&tool_index_section);
        }
    }

    // Plan mode injection (#2)
    if let Some(rt) = &ctx.delegate_runtime
        && rt.plan_then_execute
    {
        prompt.push_str(
            "\n\n## 执行模式：先计划再执行\n\
             请先输出一个明确的多步骤计划，逐步执行，每步完成后告知结果。",
        );
    }

    if let Some(rt) = &ctx.delegate_runtime {
        if !rt.user_identity.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&rt.user_identity);
        }
        if !rt.user_profile.is_empty() {
            prompt.push_str("\n\n## 用户画像\n");
            prompt.push_str(&rt.user_profile);
        }
        if !rt.user_memory.is_empty() {
            prompt.push_str("\n\n## 用户偏好记忆\n");
            prompt.push_str(&rt.user_memory);
        }
    }

    prompt
}

fn validate_agent_exists(
    agent_id: &str,
    config: &crate::config::Config,
) -> Result<String, ClawError> {
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

/// Expanded keyword matching with synonyms (#7)
#[allow(dead_code)]
fn match_capability(task: &str, capabilities: &[String]) -> bool {
    let task_lower = task.to_lowercase();
    for cap in capabilities {
        if task_lower.contains(cap.to_lowercase().as_str()) {
            return true;
        }
        let expanded = expand_keywords(cap);
        for kw in &expanded {
            if task_lower.contains(kw) {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
fn expand_keywords(cap: &str) -> Vec<String> {
    match cap.to_lowercase().as_str() {
        "数据分析" => vec![
            "数据".to_string(),
            "分析".to_string(),
            "趋势".to_string(),
            "统计".to_string(),
            "对比".to_string(),
            "图表".to_string(),
        ],
        "代码生成" => vec![
            "代码".to_string(),
            "写".to_string(),
            "编程".to_string(),
            "函数".to_string(),
            "实现".to_string(),
        ],
        "数据可视化" => vec![
            "图表".to_string(),
            "可视化".to_string(),
            "图".to_string(),
            "曲线".to_string(),
            "饼图".to_string(),
        ],
        _ => vec![],
    }
}

fn resolve_auto_agent(task: &str, config: &crate::config::Config) -> Result<String, ClawError> {
    if config.sub_agents.is_empty() && config.agents.len() <= 1 {
        return Err(ClawError::NotFound(
            "没有可用的子智能体进行自动路由".to_string(),
        ));
    }

    let agents: Vec<crate::config::ResolvedAgentConfig> = config
        .agents
        .keys()
        .map(|id| config.agent_config(id))
        .collect();
    let sub_agents: Vec<crate::config::ResolvedAgentConfig> = config
        .sub_agents
        .keys()
        .map(|id| config.agent_config(id))
        .collect();

    let router = crate::router::TaskRouter::new(agents, sub_agents);
    if let Some((agent_id, _is_sub)) = router.select_agent(task, "default", "default") {
        tracing::info!(
            agent_id = %agent_id,
            "自动路由匹配智能体 (via TaskRouter)"
        );
        return Ok(agent_id.to_string());
    }

    if let Some(first) = config.sub_agents.keys().next() {
        return Ok(first.clone());
    }

    Err(ClawError::NotFound(
        "自动路由未能匹配到合适的子智能体".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_config_with_sub_agent() -> crate::config::Config {
        let mut config = crate::test_helpers::test_config();
        config.sub_agents.insert(
            "analyst".to_string(),
            crate::config::AgentConfig {
                model: Some("gpt-4o".to_string()),
                capabilities: vec!["数据分析".to_string()],
                ..Default::default()
            },
        );
        config.sub_agents.insert(
            "coder".to_string(),
            crate::config::AgentConfig {
                model: Some("claude-3-opus".to_string()),
                capabilities: vec!["代码生成".to_string()],
                ..Default::default()
            },
        );
        config
    }

    #[test]
    fn test_validate_agent_exists_ok() {
        let config = test_config_with_sub_agent();
        let result = validate_agent_exists("analyst", &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "analyst");
    }

    #[test]
    fn test_validate_agent_exists_not_found() {
        let config = test_config_with_sub_agent();
        let result = validate_agent_exists("nonexistent", &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ClawError::NotFound(msg) => {
                assert!(msg.contains("nonexistent"));
                assert!(msg.contains("analyst") || msg.contains("coder"));
            }
            _ => panic!("应为 NotFound 错误"),
        }
    }

    #[test]
    fn test_resolve_auto_agent_by_capability() {
        let config = test_config_with_sub_agent();
        let result = resolve_auto_agent("帮我做数据分析", &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "analyst");
    }

    #[test]
    fn test_resolve_auto_agent_by_expanded_keyword() {
        let config = test_config_with_sub_agent();
        let result = resolve_auto_agent("帮我做代码生成", &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "coder");
    }

    #[test]
    fn test_resolve_auto_agent_fallback_to_first() {
        let config = test_config_with_sub_agent();
        // No capability keywords match — falls back to first sub_agent
        let result = resolve_auto_agent("记录体重 70kg", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_auto_agent_no_sub_agents() {
        let config = crate::test_helpers::test_config();
        let result = resolve_auto_agent("随便", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_match_capability_exact() {
        assert!(match_capability("数据分析报告", &["数据分析".to_string()]));
    }

    #[test]
    fn test_match_capability_expanded() {
        assert!(match_capability("看下趋势", &["数据分析".to_string()]));
    }

    #[test]
    fn test_match_capability_no_match() {
        assert!(!match_capability("记录体重", &["数据分析".to_string()]));
    }

    #[test]
    fn test_expand_keywords_data_analysis() {
        let expanded = expand_keywords("数据分析");
        assert!(expanded.contains(&"趋势".to_string()));
        assert!(expanded.contains(&"统计".to_string()));
    }

    #[test]
    fn test_expand_keywords_unknown() {
        let expanded = expand_keywords("翻译");
        assert!(expanded.is_empty());
    }

    #[test]
    fn test_build_sub_agent_prompt_uses_system_prompt() {
        let config = crate::config::ResolvedAgentConfig {
            agent_id: "test".to_string(),
            provider: crate::providers::ProviderKind::OpenAI,
            api_key: String::new(),
            base_url: String::new(),
            model: "gpt-4o-mini".to_string(),
            enabled_tools: Default::default(),
            system_prompt: Some("你是专业财务分析师".to_string()),
            mcp_servers: vec![],
            allowed_dirs: vec![],
            capabilities: vec![],
            execution_mode: crate::config::ExecutionMode::React,
        };
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
        };
        let prompt = build_sub_agent_prompt(&config, &ctx);
        assert!(prompt.contains("财务分析师"));
        assert!(prompt.contains("委托任务"));
    }

    #[test]
    fn test_build_sub_agent_prompt_with_user_identity() {
        use std::sync::Arc;
        let config = crate::config::ResolvedAgentConfig {
            agent_id: "test".to_string(),
            provider: crate::providers::ProviderKind::OpenAI,
            api_key: String::new(),
            base_url: String::new(),
            model: "gpt-4o-mini".to_string(),
            enabled_tools: Default::default(),
            system_prompt: None,
            mcp_servers: vec![],
            allowed_dirs: vec![],
            capabilities: vec![],
            execution_mode: crate::config::ExecutionMode::React,
        };
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: Some(Arc::new(crate::tools::DelegateRuntime {
                irs_tool_index: std::collections::HashMap::new(),
                mcp_registry: crate::mcp::McpRegistry::new(&[]),
                skills: vec![],
                tool_frequency: std::collections::HashMap::new(),
                parent_tx: tokio::sync::mpsc::unbounded_channel().0,
                stats_manager: std::sync::Arc::new(crate::stats::StatsManager::with_storage(
                    std::sync::Arc::new(crate::storage::ClawStorage::file(
                        std::env::temp_dir().join("claw-test-delegate"),
                    )),
                    &Default::default(),
                    chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
                )),
                user_identity: "用户称呼你为小助手".to_string(),
                user_memory: String::new(),
                user_profile: String::new(),
                recent_messages: vec![],
                tz_offset: chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
                plan_then_execute: false,
            })),
        };
        let prompt = build_sub_agent_prompt(&config, &ctx);
        assert!(prompt.contains("小助手"));
        assert!(prompt.contains("当前时间"));
    }

    #[test]
    fn test_build_sub_agent_prompt_with_tool_index() {
        let config = crate::config::ResolvedAgentConfig {
            agent_id: "analyst".to_string(),
            provider: crate::providers::ProviderKind::OpenAI,
            api_key: String::new(),
            base_url: String::new(),
            model: "gpt-4o".to_string(),
            enabled_tools: std::collections::HashSet::from_iter([
                "i-rs-weight".into(),
                "i-rs-mood".into(),
            ]),
            system_prompt: Some("你是数据分析师".to_string()),
            mcp_servers: vec![],
            allowed_dirs: vec![],
            capabilities: vec![],
            execution_mode: crate::config::ExecutionMode::PlanThenExecute,
        };
        let mut tool_index = std::collections::HashMap::new();
        tool_index.insert("i-rs-weight".into(), "体重记录管理".into());
        tool_index.insert("i-rs-mood".into(), "情绪记录管理".into());
        tool_index.insert("i-rs-run".into(), "跑步记录管理".into());
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: Some(Arc::new(crate::tools::DelegateRuntime {
                irs_tool_index: tool_index,
                mcp_registry: crate::mcp::McpRegistry::new(&[]),
                skills: vec![],
                tool_frequency: std::collections::HashMap::new(),
                parent_tx: tokio::sync::mpsc::unbounded_channel().0,
                stats_manager: std::sync::Arc::new(crate::stats::StatsManager::with_storage(
                    std::sync::Arc::new(crate::storage::ClawStorage::file(
                        std::env::temp_dir().join("claw-test-delegate-tool-index"),
                    )),
                    &Default::default(),
                    chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
                )),
                user_identity: String::new(),
                user_memory: String::new(),
                user_profile: String::new(),
                recent_messages: vec![],
                tz_offset: chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
                plan_then_execute: true,
            })),
        };
        let prompt = build_sub_agent_prompt(&config, &ctx);
        // System prompt from agent config
        assert!(prompt.contains("你是数据分析师"));
        // Tool index — only enabled tools should appear
        assert!(prompt.contains("i-rs-weight: 体重记录管理"));
        assert!(prompt.contains("i-rs-mood: 情绪记录管理"));
        // Disabled tool should not appear
        assert!(!prompt.contains("i-rs-run"));
        // Plan mode injection
        assert!(prompt.contains("先计划再执行"));
    }
}
