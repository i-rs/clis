use serde_json::Value;

#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

impl GuardrailResult {
    pub fn allow() -> Self {
        Self { allowed: true, reason: None }
    }
    pub fn deny(reason: &str) -> Self {
        Self { allowed: false, reason: Some(reason.to_string()) }
    }
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait InputGuardrail: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self, user_input: &str) -> GuardrailResult;
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait OutputGuardrail: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self, output: &str) -> GuardrailResult;
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait ToolCallGuardrail: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self, tool_name: &str, args: &Value) -> GuardrailResult;
}

pub struct PromptInjectionGuardrail;

#[async_trait::async_trait]
impl InputGuardrail for PromptInjectionGuardrail {
    fn name(&self) -> &str {
        "prompt_injection"
    }
    async fn check(&self, user_input: &str) -> GuardrailResult {
        let lower = user_input.to_lowercase();
        let patterns = [
            "ignore all previous instructions",
            "ignore your instructions",
            "disregard your",
            "you are now",
            "new instructions:",
            "system prompt:",
            "jailbreak",
            " simulated ",
            "pretend you are",
            "act as if",
        ];
        for pattern in &patterns {
            if lower.contains(pattern) {
                return GuardrailResult::deny(&format!(
                    "检测到潜在的提示注入攻击模式: '{}'",
                    pattern.trim()
                ));
            }
        }
        GuardrailResult::allow()
    }
}

pub struct PiiDetectionGuardrail;

#[allow(dead_code)]
fn has_sequence_of_digits(text: &str, count: usize) -> bool {
    let mut digit_run = 0;
    for c in text.chars() {
        if c.is_ascii_digit() {
            digit_run += 1;
            if digit_run >= count {
                return true;
            }
        } else {
            digit_run = 0;
        }
    }
    false
}

#[allow(dead_code)]
fn extract_digit_sequences(text: &str) -> Vec<String> {
    let mut sequences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() || c == 'X' || c == 'x' {
            current.push(c);
        } else {
            if current.len() >= 10 {
                sequences.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= 10 {
        sequences.push(current);
    }
    sequences
}

#[async_trait::async_trait]
impl OutputGuardrail for PiiDetectionGuardrail {
    fn name(&self) -> &str {
        "pii_detection"
    }
    async fn check(&self, output: &str) -> GuardrailResult {
        for seq in extract_digit_sequences(output) {
            if seq.len() == 18 {
                return GuardrailResult::deny("输出包含可能的身份证号");
            }
            if seq.len() == 11 && seq.starts_with('1') {
                return GuardrailResult::deny("输出包含可能的手机号");
            }
        }
        GuardrailResult::allow()
    }
}

#[allow(dead_code)]
pub struct DangerousToolGuardrail {
    dangerous_tools: Vec<String>,
}

impl DangerousToolGuardrail {
    pub fn new(dangerous_tools: Vec<String>) -> Self {
        Self { dangerous_tools }
    }
}

#[async_trait::async_trait]
impl ToolCallGuardrail for DangerousToolGuardrail {
    fn name(&self) -> &str {
        "dangerous_tool"
    }
    async fn check(&self, tool_name: &str, _args: &Value) -> GuardrailResult {
        if self.dangerous_tools.contains(&tool_name.to_string()) {
            return GuardrailResult::deny(&format!(
                "工具 '{}' 被标记为危险操作，需要用户确认", tool_name
            ));
        }
        GuardrailResult::allow()
    }
}

pub struct GuardrailManager {
    input_guardrails: Vec<Box<dyn InputGuardrail>>,
    output_guardrails: Vec<Box<dyn OutputGuardrail>>,
    tool_guardrails: Vec<Box<dyn ToolCallGuardrail>>,
}

impl GuardrailManager {
    pub fn new() -> Self {
        Self {
            input_guardrails: vec![Box::new(PromptInjectionGuardrail)],
            output_guardrails: vec![Box::new(PiiDetectionGuardrail)],
            tool_guardrails: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_input(mut self, guardrail: Box<dyn InputGuardrail>) -> Self {
        self.input_guardrails.push(guardrail);
        self
    }

    #[allow(dead_code)]
    pub fn with_output(mut self, guardrail: Box<dyn OutputGuardrail>) -> Self {
        self.output_guardrails.push(guardrail);
        self
    }

    pub fn with_tool(mut self, guardrail: Box<dyn ToolCallGuardrail>) -> Self {
        self.tool_guardrails.push(guardrail);
        self
    }

    #[allow(dead_code)]
    pub async fn check_input(&self, user_input: &str) -> GuardrailResult {
        for g in &self.input_guardrails {
            let result = g.check(user_input).await;
            if !result.allowed {
                return result;
            }
        }
        GuardrailResult::allow()
    }

    #[allow(dead_code)]
    pub async fn check_output(&self, output: &str) -> GuardrailResult {
        for g in &self.output_guardrails {
            let result = g.check(output).await;
            if !result.allowed {
                return result;
            }
        }
        GuardrailResult::allow()
    }

    pub async fn check_tool_call(&self, tool_name: &str, args: &Value) -> GuardrailResult {
        for g in &self.tool_guardrails {
            let result = g.check(tool_name, args).await;
            if !result.allowed {
                return result;
            }
        }
        GuardrailResult::allow()
    }
}

impl Default for GuardrailManager {
    fn default() -> Self {
        Self::new()
    }
}
