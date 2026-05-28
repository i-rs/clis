use crate::provider::LlmMessage;

pub struct ContextManager {
    max_tokens: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { max_tokens: 128_000 }
    }

    fn estimate_tokens(messages: &[LlmMessage]) -> usize {
        let total_chars: usize = messages.iter().map(|m| match m {
            LlmMessage::System(s) => s.len(),
            LlmMessage::User(s) => s.len(),
            LlmMessage::Assistant(s) => s.len(),
            LlmMessage::AssistantWithReasoning { content, reasoning, .. } => content.len() + reasoning.len(),
            LlmMessage::Tool { content, .. } => content.len(),
            LlmMessage::ToolCall { args, .. } => args.to_string().len(),
        }).sum();
        total_chars.div_ceil(4)
    }

    pub fn should_compress(&self, messages: &[LlmMessage]) -> bool {
        Self::estimate_tokens(messages) > self.max_tokens * 80 / 100
    }

    pub fn compress(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        if !self.should_compress(messages) {
            return messages.to_vec();
        }

        let mut compressed: Vec<LlmMessage> = Vec::new();
        let mut early_tool_pairs: Vec<(LlmMessage, LlmMessage)> = Vec::new();
        let mut late_tool_pairs: Vec<(LlmMessage, LlmMessage)> = Vec::new();

        let msg_count = messages.len();
        let tool_call_count = messages.iter().filter(|m| matches!(m, LlmMessage::ToolCall { .. })).count();
        let keep_recent = tool_call_count.saturating_sub(5);

        let mut i = 0;
        while i < msg_count {
            if matches!(messages[i], LlmMessage::ToolCall { .. })
                && let Some(call) = messages.get(i)
                && let Some(result) = messages.get(i + 1)
                && matches!(result, LlmMessage::Tool { .. })
            {
                if early_tool_pairs.len() + late_tool_pairs.len() < keep_recent {
                    late_tool_pairs.push((call.clone(), result.clone()));
                } else {
                    early_tool_pairs.push((call.clone(), result.clone()));
                }
                i += 2;
                continue;
            }
            i += 1;
        }

        let total_calls = early_tool_pairs.len() + late_tool_pairs.len();

        for msg in messages.iter() {
            match msg {
                LlmMessage::System(s) => {
                    compressed.push(LlmMessage::System(s.clone()));
                }
                _ => {
                    if compressed.len() < 3
                        && matches!(msg, LlmMessage::User(_) | LlmMessage::Assistant(_)) {
                            compressed.push(msg.clone());
                        }
                }
            }
        }

        if total_calls > 0 {
            let truncated: Vec<LlmMessage> = early_tool_pairs.into_iter().flat_map(|(call, result)| {
                let (name, call_id, content) = match &result {
                    LlmMessage::Tool { name, call_id, content } => (name.clone(), call_id.clone(), content.clone()),
                    _ => (String::new(), String::new(), String::new()),
                };
                let summary = if content.len() > 200 {
                    let trimmed: String = content.chars().take(200).collect();
                    format!("{}...", trimmed)
                } else {
                    content
                };
                vec![
                    call,
                    LlmMessage::Tool { name, content: format!("[compressed] {}", summary), call_id },
                ]
            }).collect();

            let recent: Vec<LlmMessage> = late_tool_pairs.into_iter()
                .flat_map(|(call, result)| vec![call, result])
                .collect();

            if !truncated.is_empty() {
                compressed.push(LlmMessage::Assistant(
                    format!("[Previous conversation: {} tool calls, earlier ones compressed]", total_calls)
                ));
                compressed.extend(truncated);
            }
            compressed.extend(recent);
        }

        compressed
    }
}
