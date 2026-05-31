//! Tool call executor with parallel execution, timeout, and retry tracking.
//!
//! Extracts the tool execution pattern from `chat_loop` into a reusable component
//! that can be shared by both the TUI chat loop and the Dashboard SSE chat loop.
//! Hardcoded truncation values are replaced with configurable parameters.

use crate::llm::{LlmEvent, ToolCallAcc};
use crate::utils;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ToolCallResult {
    pub call: ToolCallAcc,
    #[allow(dead_code)]
    pub args: Value,
    pub result: String,
    #[allow(dead_code)]
    pub context_result: String,
    #[allow(dead_code)]
    pub validation: ToolResultValidation,
}

/// Basic validation of tool execution results.
#[derive(Debug, Clone)]
pub struct ToolResultValidation {
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Run basic checks on a tool result to detect common issues.
fn validate_tool_result(_name: &str, result: &str) -> ToolResultValidation {
    let mut issues = Vec::new();

    if result.is_empty() {
        issues.push("工具返回空结果".to_string());
        return ToolResultValidation {
            valid: false,
            issues,
        };
    }

    if result.starts_with("错误:")
        || result.starts_with("执行错误:")
        || result.starts_with("MCP 错误:")
    {
        issues.push(format!("工具执行失败: {}", result));
        return ToolResultValidation {
            valid: false,
            issues,
        };
    }

    // If the result looks like JSON, verify it's well-formed
    if result.trim_start().starts_with('{') || result.trim_start().starts_with('[') {
        match serde_json::from_str::<Value>(result) {
            Ok(json) => {
                // Check for common error patterns in JSON responses
                if let Some(obj) = json.as_object() {
                    if let Some(error) = obj
                        .get("error")
                        .or_else(|| obj.get("err"))
                        .and_then(|v| v.as_str())
                    {
                        issues.push(format!("JSON 响应包含错误字段: '{}'", error));
                    }
                    if let Some(success) = obj.get("success").and_then(|v| v.as_bool()) {
                        if !success {
                            issues.push("JSON 响应的 success 字段为 false".to_string());
                        }
                    }
                }
                // Empty array/object with no helpful content
                if json.as_array().map_or(false, |a| a.is_empty()) {
                    issues.push("JSON 响应为空数组".to_string());
                }
                if json.as_object().map_or(false, |o| o.is_empty()) {
                    issues.push("JSON 响应为空对象".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("工具输出不是合法 JSON: {}", e));
            }
        }
    }

    ToolResultValidation {
        valid: issues.is_empty(),
        issues,
    }
}

pub struct ToolCallExecutor {
    tool_registry: Arc<crate::tools::ToolRegistry>,
    tool_ctx: crate::tools::ToolContext,
    cli_timeout_secs: u64,
    truncate_display: usize,
    truncate_context: usize,
}

impl ToolCallExecutor {
    pub fn new(
        tool_registry: Arc<crate::tools::ToolRegistry>,
        tool_ctx: crate::tools::ToolContext,
    ) -> Self {
        Self {
            tool_registry,
            tool_ctx,
            cli_timeout_secs: 30,
            truncate_display: 4096,
            truncate_context: 500,
        }
    }

    /// Set the CLI tool timeout in seconds.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.cli_timeout_secs = secs;
        self
    }

    /// Set truncation sizes: `display` for ToolExecuted events, `context` for LLM injection.
    pub fn with_truncation(mut self, display: usize, context: usize) -> Self {
        self.truncate_display = display;
        self.truncate_context = context;
        self
    }

    /// Execute all tool calls in parallel with timeout.
    ///
    /// Each call emits a [`LlmEvent::ToolExecuted`] upon completion.
    /// Returns the results in the same order as input.
    pub async fn execute(
        &self,
        calls: Vec<(ToolCallAcc, Value)>,
        tx: &mpsc::UnboundedSender<LlmEvent>,
    ) -> Vec<ToolCallResult> {
        let total = calls.len();
        let mut handles = Vec::with_capacity(total);

        for (step, (tc, args)) in calls.into_iter().enumerate() {
            let tx = tx.clone();
            let tc_name = tc.name.clone();
            let args_str = serde_json::to_string(&args).unwrap_or_default();
            let ctx_for_spawn = self.tool_ctx.clone();
            let registry_for_spawn = Arc::clone(&self.tool_registry);
            let timeout_dur = std::time::Duration::from_secs(self.cli_timeout_secs.max(10));
            let trunc_display = self.truncate_display;
            let trunc_context = self.truncate_context;

            handles.push(tokio::spawn(async move {
                let result = match tokio::time::timeout(timeout_dur, async {
                    crate::core::engine::execute_tool_call(
                        &tc_name,
                        &args,
                        &registry_for_spawn,
                        &ctx_for_spawn,
                    )
                    .await
                })
                .await
                {
                    Ok(r) => r,
                    Err(_) => format!("错误: 工具执行超时 (>{:?})", timeout_dur),
                };

                let display_result = utils::smart_truncate(&result, trunc_display);
                let context_result = utils::smart_truncate(&result, trunc_context);

                let _ = tx.send(LlmEvent::ToolExecuted {
                    name: tc.name.clone(),
                    args: args_str,
                    result: display_result,
                    step,
                    total_steps: total,
                });

                // Basic result validation
                let validation = validate_tool_result(&tc.name, &result);
                if !validation.valid {
                    let _ = tx.send(LlmEvent::Evaluation {
                        tool: tc.name.clone(),
                        valid: false,
                        issues: validation.issues.clone(),
                    });
                }

                (tc, args, context_result, validation)
            }));
        }

        // Collect all results in order
        let mut all_results: Vec<ToolCallResult> = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok((call, args, context_result, validation)) => {
                    all_results.push(ToolCallResult {
                        call,
                        args,
                        result: context_result.clone(),
                        context_result,
                        validation,
                    });
                }
                Err(e) => {
                    tracing::error!("Tool task panicked: {}", e);
                }
            }
        }
        all_results
    }
}
