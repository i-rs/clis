pub mod loader;
pub mod verifier;
pub mod executor;
pub mod storage_verify;

use std::collections::HashMap;
use i_rs_claw_core::storage::ClawStorage;

/// Information about a tool call made by the agent.
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub tool: String,
    pub command: Option<String>,
    pub args: HashMap<String, String>,
}

impl ToolCallInfo {
    pub fn new(tool: &str) -> Self {
        Self {
            tool: tool.to_string(),
            command: None,
            args: HashMap::new(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn with_arg(mut self, key: &str, val: &str) -> Self {
        self.args.insert(key.to_string(), val.to_string());
        self
    }
}

/// Metadata for a conversation script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptMeta {
    pub tool: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub storage_backends: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// One step in a conversation script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptStep {
    pub step: u32,
    pub title: String,
    pub user_message: String,
    #[serde(default)]
    pub expected_tool: Option<String>,
    #[serde(default)]
    pub expected_command: Option<String>,
    #[serde(default)]
    pub expected_args: Option<HashMap<String, String>>,
    #[serde(default)]
    pub expected_flags: Option<HashMap<String, String>>,
    #[serde(default)]
    pub check_reply: Option<ReplyCheck>,
    #[serde(default)]
    pub verify_storage: Option<StorageCheck>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplyCheck {
    #[serde(default)]
    pub contains: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageCheck {
    #[serde(default)]
    pub file_check: Option<String>,
    #[serde(default)]
    pub expected_state: Option<serde_json::Value>,
    #[serde(default)]
    pub no_new_records: Option<bool>,
}

/// A complete conversation script loaded from disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Script {
    pub meta: ScriptMeta,
    pub steps: Vec<ScriptStep>,
}

/// Result of executing a single conversation step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepResult {
    pub step: u32,
    pub title: String,
    pub passed: bool,
    #[serde(default)]
    pub tool_mismatches: Vec<String>,
    #[serde(default)]
    pub missing_keywords: Vec<String>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    #[serde(default)]
    pub storage_results: Vec<storage_verify::StorageCheckResult>,
    pub error: Option<String>,
}

/// Result of executing an entire conversation script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptResult {
    pub meta: ScriptMeta,
    pub passed: bool,
    pub total_steps: usize,
    pub passed_steps: usize,
    pub steps: Vec<StepResult>,
    pub total_prompt_tokens: u32,
    pub total_completion_tokens: u32,
}

/// Coordinates script loading, execution, and verification.
pub struct ScriptRunner {
    script: Script,
}

impl ScriptRunner {
    pub fn new(script: Script) -> Self {
        Self { script }
    }

    /// Execute the script against a session backend and verify results.
    pub async fn run(
        &self,
        session: &mut dyn executor::SessionBackend,
    ) -> ScriptResult {
        self.run_with_storage(session, None).await
    }

    /// Execute against a session backend with optional storage verification.
    pub async fn run_with_storage(
        &self,
        session: &mut dyn executor::SessionBackend,
        storage: Option<&std::sync::Arc<ClawStorage>>,
    ) -> ScriptResult {
        let mut steps = Vec::new();
        let mut total_prompt = 0;
        let mut total_completion = 0;
        let mut all_passed = true;

        for step_def in &self.script.steps {
            let (result, step_pass) = self.execute_step(session, step_def, storage).await;
            total_prompt += result.prompt_tokens;
            total_completion += result.completion_tokens;
            if !step_pass {
                all_passed = false;
            }
            steps.push(result);
        }

        let passed = steps.iter().filter(|s| s.passed).count();

        ScriptResult {
            meta: self.script.meta.clone(),
            passed: all_passed,
            total_steps: steps.len(),
            passed_steps: passed,
            steps,
            total_prompt_tokens: total_prompt,
            total_completion_tokens: total_completion,
        }
    }

    async fn execute_step(
        &self,
        session: &mut dyn executor::SessionBackend,
        step: &ScriptStep,
        storage: Option<&std::sync::Arc<ClawStorage>>,
    ) -> (StepResult, bool) {
        let output = match session.send_message(&step.user_message).await {
            Ok(o) => o,
            Err(e) => {
                return (StepResult {
                    step: step.step,
                    title: step.title.clone(),
                    passed: false,
                    tool_mismatches: vec![],
                    missing_keywords: vec![],
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    storage_results: vec![],
                    error: Some(format!("执行错误: {}", e)),
                }, false);
            }
        };

        let mut all_pass = true;

        let tool_call = output.tool_calls.first();
        let tool_mismatches = match tool_call {
            Some(tc) => verifier::check_tool_call(step, tc),
            None if step.expected_tool.is_some() => {
                all_pass = false;
                vec!["未产生任何工具调用".into()]
            }
            None => vec![],
        };
        if !tool_mismatches.is_empty() {
            all_pass = false;
        }

        let missing_keywords = verifier::check_reply(step, &output.reply);
        if !missing_keywords.is_empty() {
            all_pass = false;
        }

        let storage_results = if let Some(storage) = storage {
            storage_verify::verify_step(storage, step, &output.session_id).await
        } else {
            vec![]
        };
        let storage_pass = storage_results.iter().all(|r| r.passed);
        if !storage_pass {
            all_pass = false;
        }

        (
            StepResult {
                step: step.step,
                title: step.title.clone(),
                passed: all_pass,
                tool_mismatches,
                missing_keywords,
                storage_results,
                prompt_tokens: output.prompt_tokens,
                completion_tokens: output.completion_tokens,
                error: None,
            },
            all_pass,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversations::runner::executor::{MockSession, StepOutput};

    fn make_mock_session() -> MockSession {
        MockSession::new(vec![
            StepOutput {
                reply: "已记录 blog_url".into(),
                tool_calls: vec![ToolCallInfo::new("i-rs-kv")
                    .with_command("add")
                    .with_arg("KEY", "blog_url")
                    .with_arg("VALUE", "https://example.com")],
                prompt_tokens: 100,
                completion_tokens: 30,
                session_id: String::new(),
            },
            StepOutput {
                reply: "已删除 blog_url".into(),
                tool_calls: vec![ToolCallInfo::new("i-rs-kv")
                    .with_command("delete")
                    .with_arg("KEY", "blog_url")],
                prompt_tokens: 50,
                completion_tokens: 20,
                session_id: String::new(),
            },
        ])
    }

    #[tokio::test]
    async fn test_script_runner_all_pass() {
        let script = Script {
            meta: ScriptMeta {
                tool: "i-rs-kv".into(),
                name: "test".into(),
                description: "test".into(),
                required_capabilities: vec![],
                storage_backends: vec![],
                tags: vec![],
            },
            steps: vec![
                ScriptStep {
                    step: 1,
                    title: "记录".into(),
                    user_message: "存一下 blog_url".into(),
                    expected_tool: Some("i-rs-kv".into()),
                    expected_command: Some("add".into()),
                    expected_args: Some(HashMap::from([
                        ("KEY".into(), "blog_url".into()),
                    ])),
                    expected_flags: None,
                    check_reply: Some(ReplyCheck {
                        contains: vec!["已记录".into()],
                    }),
                    verify_storage: None,
                },
                ScriptStep {
                    step: 2,
                    title: "删除".into(),
                    user_message: "删掉 blog_url".into(),
                    expected_tool: Some("i-rs-kv".into()),
                    expected_command: Some("delete".into()),
                    expected_args: Some(HashMap::from([
                        ("KEY".into(), "blog_url".into()),
                    ])),
                    expected_flags: None,
                    check_reply: Some(ReplyCheck {
                        contains: vec!["已删除".into()],
                    }),
                    verify_storage: None,
                },
            ],
        };

        let runner = ScriptRunner::new(script);
        let mut session = make_mock_session();
        let result = runner.run(&mut session).await;

        assert!(result.passed, "all steps should pass");
        assert_eq!(result.total_steps, 2);
        assert_eq!(result.passed_steps, 2);
        assert_eq!(result.total_prompt_tokens, 150);
        assert_eq!(result.total_completion_tokens, 50);
    }

    #[tokio::test]
    async fn test_script_runner_tool_mismatch() {
        let script = Script {
            meta: ScriptMeta {
                tool: "i-rs-kv".into(),
                name: "test".into(),
                description: "test".into(),
                required_capabilities: vec![],
                storage_backends: vec![],
                tags: vec![],
            },
            steps: vec![ScriptStep {
                step: 1,
                title: "记录".into(),
                user_message: "存一下".into(),
                expected_tool: Some("i-rs-weight".into()), // wrong tool
                expected_command: Some("add".into()),
                expected_args: None,
                expected_flags: None,
                check_reply: None,
                verify_storage: None,
            }],
        };

        let runner = ScriptRunner::new(script);
        let mut session = make_mock_session();
        let result = runner.run(&mut session).await;

        assert!(!result.passed);
        assert_eq!(result.passed_steps, 0);
        assert!(!result.steps[0].tool_mismatches.is_empty());
    }
}

