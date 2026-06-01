//! Tool call executor with parallel execution, timeout, and retry tracking.
//!
//! Extracts the tool execution pattern from `chat_loop` into a reusable component
//! that can be shared by both the TUI chat loop and the Dashboard SSE chat loop.
//! Hardcoded truncation values are replaced with configurable parameters.

use crate::error::{ErrorCategory, category_from_result};
use crate::llm::{LlmEvent, ToolCallAcc};
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
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
    pub category: ErrorCategory,
}

#[derive(Debug, Clone)]
pub struct ToolResultValidation {
    pub valid: bool,
    pub issues: Vec<String>,
}

fn validate_tool_result(name: &str, result: &str) -> (ToolResultValidation, ErrorCategory) {
    let mut issues = Vec::new();

    if result.is_empty() {
        issues.push("工具返回空结果".to_string());
        return (
            ToolResultValidation {
                valid: false,
                issues,
            },
            ErrorCategory::EmptyResult,
        );
    }

    let base_category = category_from_result(result);
    if base_category.is_retryable_or_fatal() {
        issues.push(format!("工具执行失败: {}", result));
        return (
            ToolResultValidation {
                valid: false,
                issues,
            },
            base_category,
        );
    }

    if result.trim_start().starts_with('{') || result.trim_start().starts_with('[') {
        match serde_json::from_str::<Value>(result) {
            Ok(json) => {
                if let Some(obj) = json.as_object() {
                    if let Some(error) = obj
                        .get("error")
                        .or_else(|| obj.get("err"))
                        .and_then(|v| v.as_str())
                    {
                        issues.push(format!("JSON 响应包含错误字段: '{}'", error));
                    }
                    if let Some(success) = obj.get("success").and_then(|v| v.as_bool())
                        && !success
                    {
                        issues.push("JSON 响应的 success 字段为 false".to_string());
                    }
                }
                if json.as_array().is_some_and(|a| a.is_empty()) {
                    issues.push("JSON 响应为空数组".to_string());
                }
                if json.as_object().is_some_and(|o| o.is_empty()) {
                    issues.push("JSON 响应为空对象".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("工具输出不是合法 JSON: {}", e));
            }
        }
    }

    let category = if issues.is_empty() {
        base_category
    } else {
        ErrorCategory::BadOutput
    };
    let _ = name;
    (
        ToolResultValidation {
            valid: issues.is_empty(),
            issues,
        },
        category,
    )
}

pub struct ToolCallExecutor {
    tool_registry: Arc<crate::tools::ToolRegistry>,
    tool_ctx: crate::tools::ToolContext,
    cli_timeout_secs: u64,
    truncate_display: usize,
    truncate_context: usize,
    result_cache: HashMap<String, (String, std::time::Instant)>,
    cache_max_size: usize,
    cache_ttl_secs: u64,
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
            result_cache: HashMap::new(),
            cache_max_size: 50,
            cache_ttl_secs: 300,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.cli_timeout_secs = secs;
        self
    }

    pub fn with_truncation(mut self, display: usize, context: usize) -> Self {
        self.truncate_display = display;
        self.truncate_context = context;
        self
    }

    fn cache_key(name: &str, args: &Value) -> String {
        let args_hash = serde_json::to_string(args).unwrap_or_default();
        format!("{}:{}", name, args_hash)
    }

    fn get_cached(&mut self, key: &str) -> Option<String> {
        if let Some((result, inserted)) = self.result_cache.get(key) {
            if inserted.elapsed().as_secs() < self.cache_ttl_secs {
                return Some(result.clone());
            }
            self.result_cache.remove(key);
        }
        None
    }

    fn put_cache(&mut self, key: String, result: String) {
        if self.result_cache.len() >= self.cache_max_size {
            let oldest_key = self
                .result_cache
                .iter()
                .min_by_key(|(_, (_, t))| *t)
                .map(|(k, _)| k.clone());
            if let Some(old) = oldest_key {
                self.result_cache.remove(&old);
            }
        }
        self.result_cache
            .insert(key, (result, std::time::Instant::now()));
    }

    pub async fn execute(
        &mut self,
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

            let cache_key = Self::cache_key(&tc_name, &args);
            let cache_entry = self.get_cached(&cache_key);

            handles.push(tokio::spawn(async move {
                let result = if let Some(cached) = cache_entry {
                    tracing::debug!(tool = %tc_name, "工具结果缓存命中");
                    cached
                } else {
                    match tokio::time::timeout(timeout_dur, async {
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
                    }
                };

                let display_result = utils::smart_truncate(&result, trunc_display);
                let context_result = utils::compact_tool_result(&tc_name, &result, trunc_context);

                let _ = tx.send(LlmEvent::ToolExecuted {
                    name: tc.name.clone(),
                    args: args_str,
                    result: display_result,
                    step,
                    total_steps: total,
                });

                let (validation, category) = validate_tool_result(&tc.name, &result);
                if !validation.valid {
                    let _ = tx.send(LlmEvent::Evaluation {
                        tool: tc.name.clone(),
                        valid: false,
                        issues: validation.issues.clone(),
                    });
                }

                (tc, args, context_result, validation, category)
            }));
        }

        let mut all_results: Vec<ToolCallResult> = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok((call, args, context_result, validation, category)) => {
                    if !category.is_error() {
                        let key = Self::cache_key(&call.name, &args);
                        self.put_cache(key, context_result.clone());
                    }
                    all_results.push(ToolCallResult {
                        call,
                        args,
                        result: context_result.clone(),
                        context_result,
                        validation,
                        category,
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
