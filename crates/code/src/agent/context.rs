use crate::provider::LlmMessage;

pub struct ContextManager {
    max_tokens: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { max_tokens: 128_000 }
    }

    pub fn check_usage(&self, messages: &[LlmMessage]) -> f64 {
        let total: usize = messages.iter().map(|m| {
            match m {
                LlmMessage::System(s) => s.len(),
                LlmMessage::User(s) => s.len(),
                LlmMessage::Assistant(s) => s.len(),
                LlmMessage::Tool { content, .. } => content.len(),
                LlmMessage::ToolCall { args, .. } => args.to_string().len(),
            }
        }).sum();
        total as f64 / self.max_tokens as f64
    }

    pub fn compress(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        // MVP: simple truncation - keep system + last N messages
        if self.check_usage(messages) > 0.8 {
            let mut compressed = Vec::new();
            for msg in messages {
                match msg {
                    LlmMessage::System(s) => {
                        compressed.push(LlmMessage::System(s.clone()));
                    }
                    _ => {
                        if compressed.len() > messages.len().saturating_sub(20) {
                            compressed.push(match msg {
                                LlmMessage::User(c) => LlmMessage::User(c.clone()),
                                LlmMessage::Assistant(c) => LlmMessage::Assistant(c.clone()),
                                LlmMessage::Tool { name, content, call_id } => {
                                    let truncated = if content.len() > 500 {
                                        format!("{}...[truncated {} bytes]", &content[..500], content.len() - 500)
                                    } else {
                                        content.clone()
                                    };
                                    LlmMessage::Tool { name: name.clone(), content: truncated, call_id: call_id.clone() }
                                }
                                _ => continue,
                            });
                        }
                    }
                }
            }
            compressed
        } else {
            messages.to_vec()
        }
    }
}
