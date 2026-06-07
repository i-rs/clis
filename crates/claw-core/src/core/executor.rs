//! Tool call executor with parallel execution, timeout, and retry tracking.
//!
//! Extracts the tool execution pattern from `chat_loop` into a reusable component
//! that can be shared by both the TUI chat loop and the Dashboard SSE chat loop.
//! Hardcoded truncation values are replaced with configurable parameters.

use crate::core::callbacks::AgentCallbacks;
use crate::core::layered_memory::LayeredMemory;
use crate::error::{ErrorCategory, category_from_result};
use crate::llm::{LlmEvent, ToolCallAcc};
use crate::tools::guardrails::GuardrailManager;
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
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
    guardrails: Option<GuardrailManager>,
    callbacks: Option<Arc<dyn AgentCallbacks>>,
    layered_memory: Option<Arc<Mutex<LayeredMemory>>>,
    hitl_policy: Option<crate::core::hitl::HitlPolicy>,
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
            guardrails: None,
            callbacks: None,
            layered_memory: None,
            hitl_policy: None,
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

    pub fn with_guardrails(mut self, mgr: GuardrailManager) -> Self {
        self.guardrails = Some(mgr);
        self
    }

    pub fn with_callbacks(mut self, cb: Arc<dyn AgentCallbacks>) -> Self {
        self.callbacks = Some(cb);
        self
    }

    pub fn with_layered_memory(mut self, mem: Arc<Mutex<LayeredMemory>>) -> Self {
        self.layered_memory = Some(mem);
        self
    }

    pub fn with_hitl_policy(mut self, policy: crate::core::hitl::HitlPolicy) -> Self {
        self.hitl_policy = Some(policy);
        self
    }

    fn cache_key(name: &str, args: &Value) -> String {
        match serde_json::to_string(args) {
            Ok(serialized) => format!("{}:{}", name, serialized),
            Err(_) => format!("{}:__err__:{}", name, uuid::Uuid::new_v4()),
        }
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

        let mut filtered_calls: Vec<(usize, ToolCallAcc, Value)> = Vec::with_capacity(total);
        let mut blocked_results: Vec<ToolCallResult> = Vec::new();
        for (step, (tc, args)) in calls.into_iter().enumerate() {
            if let Some(ref guardrails) = self.guardrails {
                let gr = guardrails.check_tool_call(&tc.name, &args).await;
                if !gr.allowed {
                    let reason = gr.reason.unwrap_or_default();
                    tracing::warn!(tool = %tc.name, reason = %reason, "工具调用被护栏拦截");
                    blocked_results.push(ToolCallResult {
                        call: tc,
                        args,
                        result: format!("护栏拦截: {}", reason),
                        context_result: format!("护栏拦截: {}", reason),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec![reason],
                        },
                        category: ErrorCategory::Validation,
                    });
                    continue;
                }
            }
            if let Some(ref hitl) = self.hitl_policy {
                let req = hitl.check(&tc.name, &args);
                if hitl.should_deny(&req) {
                    tracing::warn!(tool = %tc.name, "工具调用被 HITL 策略拒绝");
                    let _ = tx.send(LlmEvent::Status(format!(
                        "🚫 工具 {} 被安全策略拦截 (风险: {:?})",
                        tc.name, req.risk_level
                    )));
                    blocked_results.push(ToolCallResult {
                        call: tc,
                        args,
                        result: "操作被安全策略拒绝: 此工具被配置为禁止执行".to_string(),
                        context_result: "操作被安全策略拒绝".to_string(),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec!["HITL 策略拒绝".to_string()],
                        },
                        category: ErrorCategory::Validation,
                    });
                    continue;
                }
                if !hitl.should_auto_approve(&req) && !hitl.should_deny(&req) {
                    if hitl.should_auto_approve_high_risk() {
                        // Explicit opt-in via --auto-approve or [hitl] config.
                        tracing::info!(
                            tool = %tc.name,
                            risk = ?req.risk_level,
                            "高危操作自动批准 (auto_approve_high_risk=true)"
                        );
                        let _ = tx.send(LlmEvent::Status(format!(
                            "⚠️ 高危操作 {} (风险: {:?}) — 自动批准 (opt-in)",
                            tc.name, req.risk_level
                        )));
                    } else {
                        // Default: block high-risk operations that weren't
                        // auto-approved (low risk) or explicitly denied.
                        tracing::warn!(
                            tool = %tc.name,
                            risk = ?req.risk_level,
                            "高危操作被 HITL 拦截 (默认策略；如需自动批准请使用 --auto-approve)"
                        );
                        let _ = tx.send(LlmEvent::Status(format!(
                            "🚫 高危操作 {} 被策略拦截 (风险: {:?}) — 需要 --auto-approve 或 [hitl] 配置",
                            tc.name, req.risk_level
                        )));
                        blocked_results.push(ToolCallResult {
                            call: tc,
                            args,
                            result: "操作被安全策略拒绝: 高危操作需要 --auto-approve 或用户确认".to_string(),
                            context_result: "操作被安全策略拒绝".to_string(),
                            validation: ToolResultValidation {
                                valid: false,
                                issues: vec!["HITL: 高危操作未启用 auto_approve_high_risk".to_string()],
                            },
                            category: ErrorCategory::Validation,
                        });
                        continue;
                    }
                }
            }
            filtered_calls.push((step, tc, args));
        }

        for (step, tc, args) in filtered_calls.into_iter() {
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
                let category = category_from_result(&result);

                let _ = tx.send(LlmEvent::ToolExecuted {
                    name: tc.name.clone(),
                    args: args_str,
                    result: display_result,
                    step,
                    total_steps: total,
                    category,
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
                    all_results.push(ToolCallResult {
                        call: ToolCallAcc {
                            id: String::new(),
                            name: String::new(),
                            arguments: String::new(),
                        },
                        args: Value::Null,
                        result: format!("工具任务崩溃: {}", e),
                        context_result: format!("工具任务崩溃: {}", e),
                        validation: ToolResultValidation {
                            valid: false,
                            issues: vec![format!("工具任务崩溃: {}", e)],
                        },
                        category: ErrorCategory::Execution,
                    });
                }
            }
        }
        for r in &all_results {
            if let Some(ref mem) = self.layered_memory
                && let Ok(mut mem_guard) = mem.lock()
            {
                mem_guard.record_tool_result(&r.call.name, &r.result);
            }
        }
        blocked_results.extend(all_results);
        blocked_results
    }
}
