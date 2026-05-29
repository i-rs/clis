use std::sync::OnceLock;

static TOKENIZER: OnceLock<tiktoken_rs::CoreBPE> = OnceLock::new();

fn get_tokenizer() -> &'static tiktoken_rs::CoreBPE {
    TOKENIZER.get_or_init(|| {
        tiktoken_rs::cl100k_base().unwrap_or_else(|_| {
            tiktoken_rs::p50k_base().expect("fallback tokenizer failed")
        })
    })
}

pub fn count_tokens(text: &str) -> usize {
    let bpe = get_tokenizer();
    bpe.encode_with_special_tokens(text).len()
}

pub fn count_tokens_for_model(model: &str, text: &str) -> usize {
    if model.contains("claude") || model.contains("anthropic") {
        (text.len() as f64 / 4.0).ceil() as usize
    } else if model.contains("llama") || model.contains("ollama") {
        (text.len() as f64 / 4.0).ceil() as usize
    } else {
        count_tokens(text)
    }
}

pub fn estimate_message_tokens(messages: &[crate::provider::LlmMessage]) -> usize {
    let mut total = 0usize;
    for msg in messages {
        match msg {
            crate::provider::LlmMessage::System(c) => {
                total += 4 + count_tokens(c);
            }
            crate::provider::LlmMessage::User(c) => {
                total += 4 + count_tokens(c);
            }
            crate::provider::LlmMessage::Assistant(c) => {
                total += 4 + count_tokens(c);
            }
            crate::provider::LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls } => {
                total += 4 + count_tokens(content) + count_tokens(reasoning);
                for tc in tool_calls {
                    total += 8 + count_tokens(&tc.name) + count_tokens(&tc.args.to_string());
                }
            }
            crate::provider::LlmMessage::ToolCall { name, args, .. } => {
                total += 6 + count_tokens(name) + count_tokens(&args.to_string());
            }
            crate::provider::LlmMessage::Tool { name, content, .. } => {
                total += 4 + count_tokens(name) + count_tokens(content);
            }
        }
    }
    total
}
