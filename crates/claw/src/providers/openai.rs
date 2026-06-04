use crate::llm::{LlmEvent, StreamResult};
use crate::providers::sse::openai_stream_chat_impl;
use crate::providers::{LlmProvider, ProviderKind};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

// =============================================
// OpenAI Provider
// =============================================

pub struct OpenaiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenaiProvider {
    pub fn new(client: reqwest::Client, api_key: String, base_url: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            client,
            api_key,
            base_url,
            model,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenaiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAI
    }

    fn model(&self) -> &str {
        &self.model
    }

    #[tracing::instrument(skip(self, messages, tool_schemas, tx))]
    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
        trace_id: &str,
    ) -> anyhow::Result<StreamResult> {
        let url = format!("{}/chat/completions", self.base_url);
        openai_stream_chat_impl(
            &self.client,
            &url,
            Some(&self.api_key),
            &self.model,
            "openai",
            messages,
            tool_schemas,
            tx,
            trace_id,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers;

    #[test]
    fn test_parse_empty_sse() {
        let (events, usage) = test_helpers::parse_openai_sse("");
        assert!(events.is_empty(), "空流不应产生事件");
        assert!(usage.is_none(), "空流不应有 usage");
    }

    #[test]
    fn test_parse_text_token() {
        let sse = r#"data: {"choices":[{"index":0,"delta":{"content":"hello"}}]}

"#;
        let (events, _) = test_helpers::parse_openai_sse(sse);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], LlmEvent::Token(t) if t == "hello"),
            "预期 Token(hello)，得到 {:?}",
            events[0]
        );
    }

    #[test]
    fn test_parse_multiple_tokens() {
        let sse = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello \"}}]}\n\ndata: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"World\"}}]}\n\ndata: [DONE]\n\n";
        let (events, _) = test_helpers::parse_openai_sse(sse);
        assert_eq!(events.len(), 2);
        assert!(
            matches!(&events[0], LlmEvent::Token(t) if t == "Hello "),
            "预期 Token(Hello )，得到 {:?}",
            events[0]
        );
        assert!(
            matches!(&events[1], LlmEvent::Token(t) if t == "World"),
            "预期 Token(World)，得到 {:?}",
            events[1]
        );
    }

    #[test]
    fn test_parse_tool_calls() {
        let sse = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"get_weather","arguments":""}}]}}]}

data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"city\":\"Beijing\"}"}}]}}]}

data: [DONE]

"#;
        let (events, _) = test_helpers::parse_openai_sse(sse);
        assert!(
            events.is_empty(),
            "纯粹的 tool_calls delta 不应产生认知事件，得到 {:?}",
            events
        );
    }

    #[test]
    fn test_parse_tool_call_streaming() {
        let chunk1 = r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"search","arguments":""}}]}}]}"#;
        let chunk2 = r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"query\":\""}}]}}]}"#;
        let chunk3 = r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"Rust\"}"}}]}}]}"#;
        let sse = format!(
            "data: {}\n\ndata: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            chunk1, chunk2, chunk3
        );
        let (events, _) = test_helpers::parse_openai_sse(&sse);
        assert!(events.is_empty(), "tool_calls 流式 delta 不应产生认知事件");
    }

    #[test]
    fn test_parse_reasoning_content() {
        let sse = r#"data: {"choices":[{"index":0,"delta":{"reasoning_content":"Let me think..."}}]}

data: {"choices":[{"index":0,"delta":{"content":"answer"}}]}

"#;
        let (events, _) = test_helpers::parse_openai_sse(sse);
        assert_eq!(events.len(), 2);
        assert!(
            matches!(&events[0], LlmEvent::Reasoning(r) if r == "Let me think..."),
            "预期 Reasoning，得到 {:?}",
            events[0]
        );
        assert!(
            matches!(&events[1], LlmEvent::Token(t) if t == "answer"),
            "预期 Token(answer)，得到 {:?}",
            events[1]
        );
    }

    #[test]
    fn test_parse_usage() {
        let sse = r#"data: {"choices":[{"index":0,"delta":{"content":"hello"}}]}

data: {"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}

data: [DONE]

"#;
        let (events, usage) = test_helpers::parse_openai_sse(sse);
        assert_eq!(events.len(), 1, "usage chunk 不产生 Token 事件");
        let u = usage.expect("应解析出 usage");
        assert_eq!(u.prompt_tokens, 10);
        assert_eq!(u.completion_tokens, 5);
        assert_eq!(u.total_tokens, 15);
    }

    #[test]
    fn test_parse_done_signal() {
        let sse = "data: [DONE]\n\n";
        let (events, usage) = test_helpers::parse_openai_sse(sse);
        assert!(events.is_empty(), "[DONE] 不产生事件");
        assert!(usage.is_none());
    }

    #[test]
    fn test_parse_mixed_token_and_done() {
        let sse = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"}}]}\n\ndata: [DONE]\n\n";
        let (events, _) = test_helpers::parse_openai_sse(sse);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], LlmEvent::Token(t) if t == "hi"),
            "预期 Token(hi)，得到 {:?}",
            events[0]
        );
    }
}
