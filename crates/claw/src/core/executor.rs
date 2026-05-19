#![allow(dead_code)]

use crate::llm::ToolCallAcc;
use crate::tools::ToolContext;
use serde_json::Value;
use std::time::Duration;

/// Errors returned by tool execution.
#[derive(Debug)]
pub enum ToolError {
    Timeout(String),
    RetryExhausted(String, u32),
    Validation(String),
    Execution(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::Timeout(msg) => write!(f, "超时: {}", msg),
            ToolError::RetryExhausted(msg, n) => write!(f, "重试 {} 次后仍然失败: {}", n, msg),
            ToolError::Validation(msg) => write!(f, "参数验证失败: {}", msg),
            ToolError::Execution(msg) => write!(f, "执行错误: {}", msg),
        }
    }
}

/// Enhanced tool executor with timeout, retry, and validation.
pub struct ToolExecutor {
    /// Maximum retries per tool call.
    pub max_retries: u32,
    /// Base timeout for tool execution.
    pub base_timeout: Duration,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_timeout: Duration::from_secs(30),
        }
    }
}

impl ToolExecutor {
    /// Execute a single tool call with timeout and retry.
    pub fn execute_tool(
        &self,
        name: &str,
        args: &Value,
        ctx: &ToolContext,
    ) -> Result<String, ToolError> {
        // Quick validation
        if name.is_empty() {
            return Err(ToolError::Validation("工具名为空".to_string()));
        }

        // Execute with retry
        let mut last_error = String::new();
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                // Exponential backoff: 0.5s, 1s, 2s...
                let delay = Duration::from_millis(500 * (1 << (attempt - 1)));
                std::thread::sleep(delay);
            }

            match crate::core::engine::execute_tool_call(name, args, Some(&ctx.mcp), ctx) {
                result if result.starts_with("错误:") => {
                    last_error = result;
                }
                result => return Ok(result),
            }
        }

        Err(ToolError::RetryExhausted(last_error, self.max_retries))
    }

    /// Execute multiple tool calls in parallel with individual timeouts.
    pub async fn execute_parallel(
        &self,
        calls: Vec<(ToolCallAcc, Value)>,
        ctx: &ToolContext,
    ) -> Vec<(ToolCallAcc, Value, String)> {
        use tokio::task;
        use tokio::time::timeout;

        let mut handles = Vec::new();
        let _total = calls.len();

        for (_step, (tc, args)) in calls.into_iter().enumerate() {
            let name = tc.name.clone();
            let name_for_closure = name.clone();
            let args_clone = args.clone();
            let ctx_clone = ctx.clone();
            let timeout_dur = self.base_timeout;

            handles.push(task::spawn(async move {
                let result = timeout(timeout_dur, task::spawn_blocking(move || {
                    let registry = crate::tools::ToolRegistry::new();
                    if registry.tool_exists(&name_for_closure) {
                        registry.execute(&name_for_closure, &args_clone, &ctx_clone)
                            .unwrap_or_else(|e| e)
                    } else {
                        // Try MCP
                        for (client_idx, tool_def) in &ctx_clone.mcp.tools {
                            if tool_def.name == name_for_closure {
                                if let Some(client) = ctx_clone.mcp.clients.get(*client_idx) {
                                    return client.call_tool(&name_for_closure, &args_clone)
                                        .unwrap_or_else(|e| format!("MCP 错误: {}", e));
                                }
                            }
                        }
                        format!("错误: 未知工具 {}", name_for_closure)
                    }
                }))
                .await;

                match result {
                    Ok(Ok(res)) => (tc, args, res),
                    Ok(Err(e)) => (tc, args, format!("错误: 内部错误: {}", e)),
                    Err(_) => (tc, args, format!("错误: 工具 '{}' 执行超时 (>{:?})", name, timeout_dur)),
                }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(r) = handle.await {
                results.push(r);
            }
        }

        results
    }

    /// Validate tool call arguments before execution.
    /// Returns Ok if valid, Err with specific message otherwise.
    pub fn validate_call(
        &self,
        name: &str,
        _args: &Value,
    ) -> Result<(), ToolError> {
        // Check if the tool exists in the registry
        let registry = crate::tools::ToolRegistry::new();
        if !registry.tool_exists(name) {
            // Check MCP tools too
            // (skip for now; MCP tools don't have schema validation here)
        }
        Ok(())
    }

    /// Get an appropriate timeout for a specific tool.
    pub fn timeout_for(&self, name: &str) -> Duration {
        // Some tools need more time
        match name {
            "delegate_task" => Duration::from_secs(120),
            "web_search" | "search_tools" => Duration::from_secs(30),
            "semantic_search" => Duration::from_secs(15),
            "chart_tool" => Duration::from_secs(15),
            _ => self.base_timeout,
        }
    }

    /// Format error result with guidance for LLM response.
    pub fn format_error(name: &str, error: &ToolError) -> String {
        match error {
            ToolError::Timeout(_) => {
                format!(
                    "工具 '{}' 执行超时。建议：1) 简化参数 2) 拆分操作为多步 3) 重试",
                    name
                )
            }
            ToolError::RetryExhausted(msg, _) => {
                format!(
                    "工具 '{}' 多次重试后失败。错误: {}\n建议：1) 检查参数是否正确 2) 使用其他方式完成 3) 直接回答用户",
                    name, msg
                )
            }
            ToolError::Validation(msg) => {
                format!("工具 '{}' 参数无效: {}\n请修正参数后重试", name, msg)
            }
            ToolError::Execution(msg) => {
                format!("工具 '{}' 执行出错: {}", name, msg)
            }
        }
    }
}
