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
        crate::tokenizer::estimate_message_tokens(messages)
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

    fn extract_paths(content: &str) -> Vec<String> {
        let mut paths = Vec::new();
        for word in content.split(|c: char| c.is_whitespace() || c == ':' || c == '(' || c == ')' || c == '{' || c == '}') {
            if (word.starts_with('/') && word.contains('.')) || (word.starts_with("./") && word.len() > 2) {
                paths.push(word.to_string());
            }
        }
        paths.dedup();
        paths.truncate(30);
        paths
    }

    fn extract_error_lines(content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        for line in content.lines() {
            let lower = line.to_lowercase();
            if lower.contains("error:") || lower.contains("error[") || lower.contains("failed") || lower.contains("panic!") {
                errors.push(line.to_string());
            }
        }
        errors.truncate(20);
        errors
    }

    fn smart_truncate(content: &str, head: usize, tail: usize) -> String {
        if content.len() <= head + tail {
            return content.to_string();
        }
        let head_str = &content[..head];
        let tail_start = content.len().saturating_sub(tail);
        let tail_str = &content[tail_start..];
        format!("{}...\n[...]\n{}", head_str, tail_str)
    }

    fn summarize_pairs(&self, pairs: &[(LlmMessage, LlmMessage)]) -> String {
        let mut details: Vec<String> = Vec::new();
        for (_call, result) in pairs {
            let (name, _call_id, content) = match &result {
                LlmMessage::Tool { name, call_id, content } => (name.clone(), call_id.clone(), content),
                    _ => continue,
            };
            details.push(format!("{}: {} chars", name, content.len()));
            let paths = Self::extract_paths(content);
            if !paths.is_empty() {
                details.push(format!("  files: {}", paths.join(", ")));
            }
            let errors = Self::extract_error_lines(content);
            if !errors.is_empty() {
                details.push(format!("  errors: {}", errors.join("; ")));
            }
        }
        if details.is_empty() {
            return String::new();
        }
        format!("Earlier tool results ({} pairs):\n{}", pairs.len(), details.join("\n"))
    }

    fn compress_core(&self, messages: &[LlmMessage]) -> Vec<LlmMessage> {
        let msg_count = messages.len();
        let tool_call_count = messages.iter().filter(|m| matches!(m, LlmMessage::ToolCall { .. })).count();
        let keep_recent = tool_call_count.saturating_sub(8);

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
        compressed.push(LlmMessage::System(
            "[Context compressed due to length limit]".into()
        ));

        for msg in messages {
            match msg {
                LlmMessage::System(_) => {
                    compressed.push(msg.clone());
                }
                LlmMessage::User(_) => {
                    compressed.push(msg.clone());
                }
                LlmMessage::Assistant(_) | LlmMessage::AssistantWithReasoning { .. } => {
                    compressed.push(msg.clone());
                }
                LlmMessage::ToolCall { .. } | LlmMessage::Tool { .. } => {}
            }
        }

        let total_calls = early_pairs.len() + late_pairs.len();
        if total_calls > 0 {
            let late_summary = self.summarize_pairs(&late_pairs);
            compressed.push(LlmMessage::Assistant(
                format!("\n## Recent tool calls (last {} of {})\n{}", late_pairs.len(), total_calls, late_summary)
            ));

            if !early_pairs.is_empty() {
                let early_summary = self.summarize_pairs(&early_pairs);
                compressed.push(LlmMessage::Assistant(
                    format!("\n## Earlier tool calls ({})\n{}", early_pairs.len(), early_summary)
                ));
            }
        }

        compressed
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
    fn test_summarize_pairs_includes_details() {
        let cm = ContextManager::new();
        let pairs = vec![
            (LlmMessage::ToolCall { id: "1".into(), name: "read".into(), args: serde_json::json!({"file_path": "src/main.rs"}) },
              LlmMessage::Tool { name: "read".into(), content: "fn main() { println!(\"hello\"); }\n// error: unused variable".into(), call_id: "1".into() }),
            (LlmMessage::ToolCall { id: "2".into(), name: "bash".into(), args: serde_json::json!({"command": "cargo check"}) },
              LlmMessage::Tool { name: "bash".into(), content: "   Compiling i-rs v0.1.0\n    Finished dev [unoptimized + debuginfo]\n     Running `cargo test`\nerror[E0001]: cannot find function `foo` in this scope".into(), call_id: "2".into() }),
        ];
        let summary = cm.summarize_pairs(&pairs);
        assert!(summary.contains("read:"), "summary should include read content length");
        assert!(summary.contains("error: unused variable"), "summary should include error lines");
        assert!(summary.contains("error[E0001]"), "summary should include rustc-style errors");
    }

    #[test]
    fn test_smart_truncate_short() {
        let s = "short string";
        assert_eq!(ContextManager::smart_truncate(s, 100, 100), s);
    }

    #[test]
    fn test_smart_truncate_long() {
        let s = "x".repeat(500);
        let result = ContextManager::smart_truncate(&s, 100, 50);
        assert!(result.contains("..."));
        assert!(result.len() < s.len());
    }

    #[test]
    fn test_extract_paths() {
        let content = "Reading /home/user/project/src/main.rs and writing to ./lib/utils.rs";
        let paths = ContextManager::extract_paths(content);
        assert!(paths.contains(&"/home/user/project/src/main.rs".to_string()));
        assert!(paths.contains(&"./lib/utils.rs".to_string()));
    }

    #[test]
    fn test_extract_error_lines() {
        let content = "Compiling...\nerror[E0001]: undefined variable\nwarning: unused import\nerror[E0002]: type mismatch";
        let errors = ContextManager::extract_error_lines(content);
        assert!(errors.iter().any(|e| e.contains("E0001")));
        assert!(errors.iter().any(|e| e.contains("E0002")));
    }

    #[test]
    fn test_compress_preserves_all_users() {
        let msgs = vec![
            LlmMessage::User("step 1: fix bug".into()),
            LlmMessage::Assistant("ok, fixing now".into()),
            LlmMessage::User("step 2: run tests".into()),
            LlmMessage::Assistant("tests pass".into()),
        ];
        let cm = ContextManager::new();
        let compressed = cm.compress(&msgs);
        let users: Vec<_> = compressed.iter().filter(|m| matches!(m, LlmMessage::User(_))).collect();
        assert_eq!(users.len(), 2, "all user messages should be preserved");
    }
}
