use crate::app::{self, App};
use crate::core;
use crate::llm::{LlmEvent, TokenUsage};
use serde_json::Value;
use std::sync::Arc;

use super::Action;

pub struct LlmEventHandler<'a> {
    pub app: &'a mut App,
    pub app_core: &'a mut core::AppCore,
}

impl<'a> LlmEventHandler<'a> {
    pub fn new(app: &'a mut App, app_core: &'a mut core::AppCore) -> Self {
        Self { app, app_core }
    }

    pub fn handle(&mut self, event: LlmEvent) -> Action {
        match event {
            LlmEvent::NewRound => self.handle_new_round(),
            LlmEvent::Token(text) => self.handle_token(&text),
            LlmEvent::Reasoning(text) => self.handle_reasoning(&text),
            LlmEvent::Status(text) => self.handle_status(&text),
            LlmEvent::ToolExecuted {
                name,
                args,
                result,
                step,
                total_steps,
            } => {
                self.handle_tool_executed(&name, &args, &result, step, total_steps);
            }
            LlmEvent::Error(text) => self.handle_error(&text),
            LlmEvent::HttpLog(data) => {
                self.handle_http_log(&data);
            }
            LlmEvent::UsageRecord(mut record) => {
                if record.estimated_cost_usd == 0.0 {
                    record.estimated_cost_usd = self.app_core.stats_manager.estimate_cost(
                        &record.model,
                        record.prompt_tokens,
                        record.completion_tokens,
                    );
                }
                if record.agent_id == "default" {
                    record.agent_id = self.app.current_agent.clone();
                }
                self.app_core.stats_manager.record(record);
                self.app.today_stats = self.app_core.stats_manager.today_summary();
                self.app.mark_overlay_dirty();
            }
            LlmEvent::Done(msgs, usage, _trace_id) => {
                let msgs = Arc::try_unwrap(msgs).unwrap_or_else(|arc| (*arc).clone());
                return self.handle_done(msgs, usage);
            }
            LlmEvent::Evaluation {
                tool,
                valid,
                issues,
            } => {
                self.handle_evaluation(&tool, valid, &issues);
            }
            LlmEvent::PlanProgress(steps) => {
                self.app.plan_steps = steps;
                self.app.mark_overlay_dirty();
            }
            LlmEvent::ImageGenerated { path, alt_text, format: _, width, height } => {
                self.app.messages.push(app::Message::Image {
                    path,
                    alt_text,
                    width,
                    height,
                    format: "png".to_string(),
                });
                self.app.message_timestamps.push(chrono::Local::now().naive_local());
                self.app.mark_dirty();
            }
        }
        Action::Continue
    }

    fn handle_new_round(&mut self) {
        self.app.start_assistant_message();
    }

    fn handle_token(&mut self, text: &str) {
        self.app.append_assistant_text(text);
        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute
            && (text.contains('\n') || self.app.plan_steps.is_empty())
        {
            let should_detect = self.app.messages.last().is_some_and(|m| {
                if let super::AppMessage::Assistant { text: t, .. } = m {
                    !t.is_empty()
                } else {
                    false
                }
            });
            if should_detect {
                let plan_text = match self.app.messages.last() {
                    Some(super::AppMessage::Assistant { text: t, .. }) => t.clone(),
                    _ => String::new(),
                };
                if !plan_text.is_empty() {
                    self.app.detect_plan(&plan_text);
                    if let Some(sid) = self.app_core.session_mgr.current_id() {
                        let sid = sid.to_string();
                        self.app_core.session_mgr.save_plan_steps(&sid, &self.app.plan_steps);
                    }
                }
            }
        }
    }

    fn handle_reasoning(&mut self, text: &str) {
        self.app.current_reasoning.push_str(text);
    }

    fn handle_status(&mut self, text: &str) {
        self.app.set_status(text);
        if (text.starts_with("⚡") || text.contains("并行执行"))
            && let Some(sid) = self.app_core.session_mgr.current_id()
        {
            let sid = sid.to_string();
            self.app_core.session_mgr.mark_waiting_for_tool(&sid);
        }
    }

    fn handle_tool_executed(
        &mut self,
        name: &str,
        args: &str,
        result: &str,
        step: usize,
        total_steps: usize,
    ) {
        self.app
            .add_tool_call(name, args, result, step, total_steps);

        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
            self.app.mark_next_plan_step_done();
            if let Some(sid) = self.app_core.session_mgr.current_id() {
                self.app_core
                    .session_mgr
                    .save_plan_steps(sid, &self.app.plan_steps);
            }
        }

        let agent_id = &self.app.current_agent;
        crate::core::record_tool_memory(
            &mut self.app_core.agent_store,
            &self.app_core.config.i_rs_tool_index,
            agent_id,
            name,
            args,
            result,
        );
        if !result.starts_with("错误") && !result.starts_with("护栏拦截") {
            crate::core::record_layered_tool_memory(
                &mut self.app_core.agent_store,
                agent_id,
                name,
                result,
            );
        }
    }

    fn handle_error(&mut self, text: &str) {
        self.app.add_error(text);
        if let Some(sid) = self.app_core.session_mgr.current_id() {
            let sid = sid.to_string();
            self.app_core.session_mgr.mark_error(&sid, text);
        }
    }

    fn handle_evaluation(&mut self, tool: &str, valid: bool, issues: &[String]) {
        self.app.messages.push(app::Message::Evaluation {
            tool: tool.to_string(),
            valid,
            issues: issues.to_vec(),
        });
        self.app.message_timestamps.push(chrono::Local::now().naive_local());
        self.app.mark_dirty();
        if !valid {
            tracing::info!(tool, issues = ?issues, "工具结果验证告警");
        }
    }

    fn handle_http_log(&mut self, data: &crate::llm::HttpLogData) {
        let msg_count = serde_json::from_str::<serde_json::Value>(&data.request_body)
            .ok()
            .and_then(|v| v["messages"].as_array().map(|a| a.len()))
            .unwrap_or(0);
        self.app.add_http_log(app::HttpLog {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            status: data.status,
            duration_ms: data.duration_ms,
            model: data.model.clone(),
            prompt_tokens: data.prompt_tokens,
            completion_tokens: data.completion_tokens,
            error: data.error.clone(),
            request_body: data.request_body.clone(),
            msg_count,
        });
    }

    fn handle_done(&mut self, mut msgs: Vec<Value>, usage: Option<TokenUsage>) -> Action {
        self.app_core
            .compress_api_messages(&mut msgs, &self.app.current_agent);

        if let Some(sid) = self.app_core.session_mgr.current_id() {
            let sid = sid.to_string();
            self.app_core.session_mgr.mark_active(&sid);
        }

        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute
            && let Some(sid) = self.app_core.session_mgr.current_id()
        {
            self.app_core.session_mgr.save_plan_steps(sid, &[]);
        }

        let session_id = match self.app_core.session_mgr.current_id() {
            Some(id) => id.to_string(),
            None => {
                tracing::warn!("未找到当前会话，跳过持久化");
                self.app.finish_processing(Some(msgs));
                self.app.token_usage = usage;
                return Action::Continue;
            }
        };

        crate::tui::clipboard::save_session_messages(
            &self.app_core.session_mgr,
            &session_id,
            &self.app.messages,
            Some(&msgs),
        );

        self.app.finish_processing(Some(msgs));
        self.app.token_usage = usage;

        let needs_rename = self
            .app_core
            .session_mgr
            .current_session()
            .map(|s| s.title == "新对话" || s.title.is_empty())
            .unwrap_or(false);
        if needs_rename
            && let Some(first_user) = self.app.messages.iter().find_map(|m| {
                if let super::AppMessage::User { text } = m {
                    Some(text.clone())
                } else {
                    None
                }
            })
        {
            self.app_core
                .session_mgr
                .rename_session(&session_id, &first_user);
        }

        self.app_core
            .agent_store
            .memory_for_mut(&self.app.current_agent)
            .flush();

        if let Some(quality) = self.app_core.evaluate_completed_session(&session_id) {
            self.app.messages.push(quality);
            self.app.message_timestamps.push(chrono::Local::now().naive_local());
            self.app.mark_dirty();
        }

        Action::Continue
    }
}
