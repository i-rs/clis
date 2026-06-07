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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::shared_client;
    use crate::tools::guardrails::{ToolCallGuardrail, GuardrailResult};
    use crate::tools::{ToolContext, ToolRegistry};
    use serde_json::json;

    fn test_ctx() -> ToolContext {
        ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: shared_client(),
            delegate_runtime: None,
        }
    }

    fn make_call(name: &str, expression: &str) -> (ToolCallAcc, Value) {
        let tc = ToolCallAcc {
            id: "call-1".into(),
            name: name.into(),
            arguments: expression.into(),
        };
        let args = json!({ "expression": expression });
        (tc, args)
    }

    // ── validate_tool_result ──────────────────────────────────────────────

    #[test]
    fn test_validate_empty_result() {
        let (v, cat) = validate_tool_result("test", "");
        assert!(!v.valid);
        assert_eq!(cat, ErrorCategory::EmptyResult);
        assert!(v.issues.iter().any(|i| i.contains("空结果")));
    }

    #[test]
    fn test_validate_normal_text() {
        let (v, cat) = validate_tool_result("test", "hello world");
        assert!(v.valid);
        assert_eq!(cat, ErrorCategory::Unknown);
    }

    #[test]
    fn test_validate_json_error_field() {
        let (v, _) = validate_tool_result("test", r#"{"error": "not found"}"#);
        assert!(!v.valid);
        assert!(v.issues.iter().any(|i| i.contains("错误字段")),
            "expected error field issue, got: {:?}", v.issues);
    }

    #[test]
    fn test_validate_json_err_field() {
        let (v, _) = validate_tool_result("test", r#"{"err": "timeout"}"#);
        assert!(!v.valid);
    }

    #[test]
    fn test_validate_json_success_false() {
        let (v, _) = validate_tool_result("test", r#"{"success": false}"#);
        assert!(!v.valid);
    }

    #[test]
    fn test_validate_json_success_true() {
        let (v, _) = validate_tool_result("test", r#"{"success": true, "data": "ok"}"#);
        assert!(v.valid);
    }

    #[test]
    fn test_validate_empty_array() {
        let (v, _) = validate_tool_result("test", "[]");
        assert!(!v.valid);
    }

    #[test]
    fn test_validate_empty_object() {
        let (v, _) = validate_tool_result("test", "{}");
        assert!(!v.valid);
    }

    #[test]
    fn test_validate_valid_json_array() {
        let (v, _) = validate_tool_result("test", r#"[{"id": 1}]"#);
        assert!(v.valid);
    }

    #[test]
    fn test_validate_plain_text_not_json() {
        // Plain text that doesn't start with { or [ is not validated as JSON
        let (v, _) = validate_tool_result("test", "not json");
        assert!(v.valid, "plain text without JSON prefix skips JSON validation");
    }

    #[test]
    fn test_validate_error_message() {
        let (v, cat) = validate_tool_result("test", "错误: 网络连接失败");
        assert!(!v.valid);
        assert!(cat.is_retryable_or_fatal());
    }

    // ── Cache ─────────────────────────────────────────────────────────────

    #[test]
    fn test_cache_key_deterministic() {
        let args = json!({ "a": 1, "b": "x" });
        let k1 = ToolCallExecutor::cache_key("tool", &args);
        let k2 = ToolCallExecutor::cache_key("tool", &args);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_args() {
        let k1 = ToolCallExecutor::cache_key("tool", &json!({"a": 1}));
        let k2 = ToolCallExecutor::cache_key("tool", &json!({"a": 2}));
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_miss() {
        let mut exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx());
        assert!(exec.get_cached("nonexistent").is_none());
    }

    #[test]
    fn test_cache_hit() {
        let mut exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx());
        exec.put_cache("k:v".into(), "result".into());
        let cached = exec.get_cached("k:v");
        assert_eq!(cached, Some("result".into()));
    }

    #[test]
    fn test_cache_eviction() {
        let reg = Arc::new(ToolRegistry::new());
        let ctx = test_ctx();
        // Create executor with small cache to trigger eviction
        let mut exec = ToolCallExecutor::new(reg, ctx);
        exec.cache_max_size = 2;
        exec.put_cache("k1".into(), "v1".into());
        exec.put_cache("k2".into(), "v2".into());
        exec.put_cache("k3".into(), "v3".into()); // should evict k1
        assert!(exec.get_cached("k1").is_none());
        assert_eq!(exec.get_cached("k2"), Some("v2".into()));
        assert_eq!(exec.get_cached("k3"), Some("v3".into()));
    }

    #[test]
    fn test_cache_ttl() {
        let mut exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx());
        exec.cache_ttl_secs = 0; // 0s TTL → immediate expiry
        exec.put_cache("k".into(), "v".into());
        // get_cached checks elapsed >= ttl so 0 TTL → expires immediately
        assert!(exec.get_cached("k").is_none());
    }

    // ── execute ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_execute_calculator() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx());
        let (tx, _rx) = mpsc::unbounded_channel();
        let results = exec.execute(vec![make_call("calculator", "2 + 3")], &tx).await;

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert!(r.validation.valid, "calculator result should be valid, got: {}", r.result);
        assert!(r.result.contains("5") || r.result == "5",
            "expected 5, got: {}", r.result);
        assert!(!r.category.is_error());
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx());
        let (tx, _rx) = mpsc::unbounded_channel();
        let results = exec.execute(vec![make_call("不存在", "anything")], &tx).await;

        assert_eq!(results.len(), 1);
        assert!(!results[0].validation.valid);
        assert!(results[0].result.contains("未知"));
    }

    #[tokio::test]
    async fn test_execute_multiple_tools_parallel() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx());
        let (tx, _rx) = mpsc::unbounded_channel();

        let calls = vec![
            make_call("calculator", "1 + 1"),
            make_call("calculator", "2 * 3"),
            make_call("calculator", "10 - 4"),
        ];
        let results = exec.execute(calls, &tx).await;

        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.validation.valid, "unexpected invalid: {}", r.result);
        }
    }

    #[tokio::test]
    async fn test_execute_mixed_valid_invalid() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx());
        let (tx, _rx) = mpsc::unbounded_channel();

        let calls = vec![
            make_call("calculator", "1 + 1"),
            make_call("不存在", "nope"),
        ];
        let results = exec.execute(calls, &tx).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].validation.valid);
        assert!(!results[1].validation.valid);
    }

    #[tokio::test]
    async fn test_execute_cache_hit() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx());
        // Pre-populate cache
        let key = ToolCallExecutor::cache_key("calculator", &json!({"expression": "1 + 1"}));
        exec.put_cache(key, "42".into());

        let (tx, _rx) = mpsc::unbounded_channel();
        let results = exec.execute(vec![make_call("calculator", "1 + 1")], &tx).await;

        // Should get cached "42" instead of computing "2"
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result, "42");
    }

    #[tokio::test]
    async fn test_execute_timeout() {
        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx())
            .with_timeout(1); // 1-second timeout

        let (tx, _rx) = mpsc::unbounded_channel();

        // Calculator should complete within 1s, so this verifies
        // timeout doesn't break normal tools.
        let results = exec.execute(vec![make_call("calculator", "1 + 1")], &tx).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].validation.valid);
    }

    #[tokio::test]
    async fn test_execute_guardrail_blocks() {
        struct BlockAllGuardrail;

        #[async_trait::async_trait]
        impl ToolCallGuardrail for BlockAllGuardrail {
            fn name(&self) -> &str {
                "block_all"
            }
            async fn check(&self, _name: &str, _args: &Value) -> GuardrailResult {
                GuardrailResult {
                    allowed: false,
                    reason: Some("测试拦截".into()),
                }
            }
        }

        let reg = Arc::new(ToolRegistry::new());
        let mut exec = ToolCallExecutor::new(reg, test_ctx())
            .with_guardrails(GuardrailManager::new().with_tool(Box::new(BlockAllGuardrail)));

        let (tx, _rx) = mpsc::unbounded_channel();
        let results = exec.execute(vec![make_call("calculator", "1 + 1")], &tx).await;

        assert_eq!(results.len(), 1);
        assert!(!results[0].validation.valid);
        assert!(results[0].result.contains("护栏拦截"));
        assert_eq!(results[0].category, ErrorCategory::Validation);
    }

    // ── builder methods ───────────────────────────────────────────────────

    #[test]
    fn test_builder_with_timeout() {
        let exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx())
            .with_timeout(60);
        assert_eq!(exec.cli_timeout_secs, 60);
    }

    #[test]
    fn test_builder_with_truncation() {
        let exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx())
            .with_truncation(100, 50);
        assert_eq!(exec.truncate_display, 100);
        assert_eq!(exec.truncate_context, 50);
    }

    #[test]
    fn test_builder_defaults() {
        let exec = ToolCallExecutor::new(Arc::new(ToolRegistry::new()), test_ctx());
        assert_eq!(exec.cli_timeout_secs, 30);
        assert_eq!(exec.truncate_display, 4096);
        assert_eq!(exec.truncate_context, 500);
        assert!(exec.guardrails.is_none());
        assert!(exec.layered_memory.is_none());
        assert!(exec.hitl_policy.is_none());
    }
}
