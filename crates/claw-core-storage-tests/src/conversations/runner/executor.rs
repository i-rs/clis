pub use super::ToolCallInfo;

/// Output from a single conversation step execution.
#[derive(Debug, Clone)]
pub struct StepOutput {
    pub reply: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    /// Session ID if using a real backend (empty for mocks).
    pub session_id: String,
}

/// Trait for session backends (mock or real AppCore).
#[async_trait::async_trait]
pub trait SessionBackend: Send + Sync {
    async fn send_message(&mut self, text: &str) -> anyhow::Result<StepOutput>;
}

/// Mock session backend for testing conversation scripts offline.
pub struct MockSession {
    /// Predefined responses per step index.
    responses: Vec<StepOutput>,
    step_index: usize,
}

impl MockSession {
    pub fn new(responses: Vec<StepOutput>) -> Self {
        Self {
            responses,
            step_index: 0,
        }
    }
}

#[async_trait::async_trait]
impl SessionBackend for MockSession {
    async fn send_message(&mut self, _text: &str) -> anyhow::Result<StepOutput> {
        if self.step_index >= self.responses.len() {
            anyhow::bail!("MockSession: no more responses (step {})", self.step_index);
        }
        let output = self.responses[self.step_index].clone();
        self.step_index += 1;
        Ok(output)
    }
}
