//! Tool call executor with parallel execution, timeout, and retry tracking.
//!
//! Extracts the tool execution pattern from `chat_loop` into a reusable component
//! that can be shared by both the TUI chat loop and the Dashboard SSE chat loop.
//! Hardcoded truncation values are replaced with configurable parameters.

use crate::llm::{LlmEvent, ToolCallAcc};
use crate::mcp::McpRegistry;
use crate::skill_store::SkillDefinition;
use crate::utils;
use serde_json::Value;
use tokio::sync::mpsc;

/// Result of a single tool call execution.
pub struct ToolCallResult {
    pub call: ToolCallAcc,
    #[allow(dead_code)]
    pub args: Value,
    pub result: String,
    #[allow(dead_code)]
    pub context_result: String,
}

/// Executes tool calls in parallel with configurable timeout and truncation.
///
/// Encapsulates:
/// - Parallel spawn with timeout per tool call
/// - Truncation sizes for display vs LLM context
pub struct ToolCallExecutor {
    mcp: McpRegistry,
    tool_ctx: crate::tools::ToolContext,
    skills: Vec<SkillDefinition>,
    cli_timeout_secs: u64,
    truncate_display: usize,
    truncate_context: usize,
}

impl ToolCallExecutor {
    pub fn new(
        tool_ctx: crate::tools::ToolContext,
        mcp: McpRegistry,
        skills: Vec<SkillDefinition>,
    ) -> Self {
        Self {
            mcp,
            tool_ctx,
            skills,
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
            let args_for_blocking = args.clone();
            let mcp_for_exec = self.mcp.clone();
            let ctx_for_spawn = self.tool_ctx.clone();
            let skills_for_spawn = self.skills.clone();
            let timeout_dur = std::time::Duration::from_secs(self.cli_timeout_secs.max(10));
            let trunc_display = self.truncate_display;
            let trunc_context = self.truncate_context;

            handles.push(tokio::spawn(async move {
                let result = match tokio::time::timeout(timeout_dur, async {
                    crate::core::engine::execute_tool_call(
                        &tc_name,
                        &args_for_blocking,
                        &skills_for_spawn,
                        Some(&mcp_for_exec),
                        &ctx_for_spawn,
                    ).await
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

                (tc, args, context_result)
            }));
        }

        // Collect all results in order
        let mut all_results: Vec<ToolCallResult> = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok((call, args, context_result)) = handle.await {
                all_results.push(ToolCallResult { call, args, result: context_result.clone(), context_result });
            }
        }
        all_results
    }
}
