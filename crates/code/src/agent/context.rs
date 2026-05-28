use crate::memory::CrossSessionMemory;
use crate::provider::LlmMessage;

pub struct ContextManager {
    max_tokens: usize,
    project_dir: Option<std::path::PathBuf>,
    memory: Option<CrossSessionMemory>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { max_tokens: 128_000, project_dir: None, memory: None }
    }

    pub fn with_project_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.project_dir = Some(dir);
        self
    }

    pub fn with_memory(mut self, memory: CrossSessionMemory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn estimate_tokens(messages: &[LlmMessage]) -> usize {
        let total_chars: usize = messages.iter().map(|m| match m {
            LlmMessage::System(s) => s.len(),
            LlmMessage::User(s) => s.len(),
            LlmMessage::Assistant(s) => s.len(),
            LlmMessage::AssistantWithReasoning { content, reasoning, .. } => content.len() + reasoning.len(),
            LlmMessage::Tool { content, .. } => content.len(),
            LlmMessage::ToolCall { args, .. } => args.to_string().len(),
        }).sum();
        total_chars / 4
    }

    pub fn should_compress(&self, messages: &[LlmMessage]) -> bool {
        Self::estimate_tokens(messages) > self.max_tokens * 80 / 100
    }

    pub fn inject_memory(messages: &[LlmMessage], memory: &CrossSessionMemory) -> Vec<LlmMessage> {
        let mem_text = memory.format_for_prompt();
        if mem_text.is_empty() {
            return messages.to_vec();
        }
        let note = format!("[Memory from previous sessions]\n{}", mem_text);
        let mut result = messages.to_vec();
        result.insert(0, LlmMessage::System(note));
        result
    }

    fn compress_paths(&self, text: &str) -> String {
        if let Some(ref proj_dir) = self.project_dir {
            let proj_str = proj_dir.to_string_lossy();
            text.replace(proj_str.as_ref(), ".")
        } else {
            text.to_string()
        }
    }

    pub fn compress(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        let processed = self.apply_path_compression(messages);

        if Self::estimate_tokens(&processed) <= self.max_tokens * 80 / 100 {
            return processed;
        }
        self.compress_core(&processed)
    }

    fn apply_path_compression(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        messages.iter().map(|msg| match msg {
            LlmMessage::System(s) => LlmMessage::System(self.compress_paths(s)),
            LlmMessage::User(s) => LlmMessage::User(self.compress_paths(s)),
            LlmMessage::Assistant(s) => LlmMessage::Assistant(self.compress_paths(s)),
            LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls } => {
                LlmMessage::AssistantWithReasoning {
                    content: self.compress_paths(content),
                    reasoning: self.compress_paths(reasoning),
                    tool_calls: tool_calls.clone(),
                }
            }
            LlmMessage::Tool { name, content, call_id } => {
                LlmMessage::Tool {
                    name: name.clone(),
                    content: self.compress_paths(content),
                    call_id: call_id.clone(),
                }
            }
            LlmMessage::ToolCall { id, name, args } => {
                LlmMessage::ToolCall { id: id.clone(), name: name.clone(), args: args.clone() }
            }
        }).collect()
    }

    fn compress_core(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        let msg_count = messages.len();
        let tool_call_count = messages.iter().filter(|m| matches!(m, LlmMessage::ToolCall { .. })).count();
        let keep_recent = tool_call_count.saturating_sub(5);

        let mut early_pairs: Vec<(LlmMessage, LlmMessage)> = Vec::new();
        let mut late_pairs: Vec<(LlmMessage, LlmMessage)> = Vec::new();
        let mut i = 0;
        while i < msg_count {
            if matches!(messages[i], LlmMessage::ToolCall { .. })
                && let Some(result) = messages.get(i + 1)
                && matches!(result, LlmMessage::Tool { .. })
            {
                let pair = (messages[i].clone(), result.clone());
                if early_pairs.len() + late_pairs.len() < keep_recent {
                    late_pairs.push(pair);
                } else {
                    early_pairs.push(pair);
                }
                i += 2;
                continue;
            }
            i += 1;
        }

        let mut compressed: Vec<LlmMessage> = Vec::new();
        i = 0;
        while i < msg_count {
            match &messages[i] {
                LlmMessage::System(_) => {
                    compressed.push(messages[i].clone());
                    i += 1;
                }
                LlmMessage::User(_) | LlmMessage::Assistant(_) | LlmMessage::AssistantWithReasoning { .. } => {
                    if compressed.iter().filter(|m| matches!(m, LlmMessage::User(_))).count() < 2 {
                        compressed.push(messages[i].clone());
                    }
                    i += 1;
                }
                LlmMessage::ToolCall { .. } => {
                    if let Some(LlmMessage::Tool { .. }) = messages.get(i + 1) {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                LlmMessage::Tool { .. } => {
                    i += 1;
                }
            }
        }

        let total_calls = early_pairs.len() + late_pairs.len();
        if total_calls > 0 {
            let summary = self.summarize_pairs(&early_pairs);
            compressed.push(LlmMessage::Assistant(
                format!("[Previous conversation: {} tool calls, earlier ones summarized]\n{}",
                    total_calls, summary)
            ));

            for (call, result) in late_pairs {
                let (name, call_id, content) = match &result {
                    LlmMessage::Tool { name, call_id, content } => (name.clone(), call_id.clone(), content.clone()),
                    _ => continue,
                };
                let content = if content.len() > 200 {
                    format!("{}...", content.chars().take(200).collect::<String>())
                } else { content };
                compressed.push(call);
                compressed.push(LlmMessage::Tool { name, content, call_id });
            }
        }

        compressed
    }

    fn summarize_pairs(&self, pairs: &[(LlmMessage, LlmMessage)]) -> String {
        let names: Vec<&str> = pairs.iter().filter_map(|(_, result)| {
            match result {
                LlmMessage::Tool { name, .. } => Some(name.as_str()),
                _ => None,
            }
        }).collect();

        if names.is_empty() {
            return String::new();
        }

        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for n in names {
            *counts.entry(n).or_insert(0) += 1;
        }
        let mut items: Vec<_> = counts.into_iter().collect();
        items.sort_by_key(|b| std::cmp::Reverse(b.1));
        let tool_summary: Vec<String> = items.iter()
            .map(|(name, count)| format!("{} ×{}", name, count))
            .collect();
        format!("Tools used: {}", tool_summary.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_compression() {
        let cm = ContextManager::new().with_project_dir("/home/user/project".into());
        let text = "/home/user/project/src/main.rs";
        assert_eq!(cm.compress_paths(text), "./src/main.rs");
    }

    #[test]
    fn test_no_path_compression_without_project_dir() {
        let cm = ContextManager::new();
        let text = "/home/user/project/src/main.rs";
        assert_eq!(cm.compress_paths(text), text);
    }

    #[test]
    fn test_inject_memory() {
        let dir = std::env::temp_dir().join("i-rs-code-test-ctx-mem");
        let _ = std::fs::create_dir_all(&dir);
        let mut mem = CrossSessionMemory::new(&dir);
        mem.add_preference("use tabs");
        let msgs = vec![LlmMessage::User("hello".into())];
        let result = ContextManager::inject_memory(&msgs, &mem);
        assert_eq!(result.len(), 2);
        assert!(matches!(&result[0], LlmMessage::System(s) if s.contains("use tabs")));
    }

    #[test]
    fn test_estimate_tokens() {
        let msgs = vec![
            LlmMessage::System("you are a helpful assistant".into()),
            LlmMessage::User("hello world".into()),
        ];
        assert!(ContextManager::estimate_tokens(&msgs) > 0);
    }

    #[test]
    fn test_summarize_pairs() {
        let cm = ContextManager::new();
        let pairs = vec![
            (LlmMessage::ToolCall { id: "1".into(), name: "bash".into(), args: serde_json::json!({"cmd": "ls"}) },
             LlmMessage::Tool { name: "bash".into(), content: "ok".into(), call_id: "1".into() }),
            (LlmMessage::ToolCall { id: "2".into(), name: "bash".into(), args: serde_json::json!({"cmd": "pwd"}) },
             LlmMessage::Tool { name: "bash".into(), content: "/".into(), call_id: "2".into() }),
        ];
        let summary = cm.summarize_pairs(&pairs);
        assert!(summary.contains("bash ×2"));
    }
}
