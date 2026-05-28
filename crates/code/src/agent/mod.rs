pub mod engine;
pub mod context;
pub mod event;
pub mod session_trait;

use crate::config::Config;
use crate::provider::{LlmProvider, LlmMessage};
use crate::tools::ToolRegistry;
use tokio::sync::mpsc;

pub struct Agent {
    pub config: Config,
    pub provider: Box<dyn LlmProvider>,
    pub tools: ToolRegistry,
    pub messages: Vec<LlmMessage>,
    pub json_output: bool,
}

impl Agent {
    pub fn new(
        config: Config,
        provider: Box<dyn LlmProvider>,
        tools: ToolRegistry,
        json_output: bool,
    ) -> Self {
        Self {
            config,
            provider,
            tools,
            messages: Vec::new(),
            json_output,
        }
    }

    pub fn add_system_prompt(&mut self, prompt: &str) {
        self.messages.push(LlmMessage::System(prompt.to_string()));
    }

    pub async fn run_once(&mut self, prompt: &str) -> anyhow::Result<()> {
        let system_text = crate::prompt::SYSTEM;

        let tool_defs = self.tools.schemas();

        let msgs = build_messages(&self.messages, system_text, prompt);

        let (final_text, new_messages) = engine::react_loop(
            &*self.provider,
            &self.tools,
            msgs,
            &tool_defs,
            self.json_output,
            self.config.max_rounds,
            self.config.tool_timeout_secs,
        ).await?;

        self.messages = new_messages;

        if self.json_output && !final_text.is_empty() {
            let event = serde_json::json!({"event": "done", "content": final_text});
            println!("{}", serde_json::to_string(&event)?);
        } else if !final_text.is_empty() {
            println!("{}", final_text);
        }

        Ok(())
    }

    pub async fn run_once_streaming(
        &mut self,
        prompt: &str,
        event_tx: mpsc::Sender<event::AgentEvent>,
        history: Vec<LlmMessage>,
    ) -> anyhow::Result<String> {
        let system_text = crate::prompt::SYSTEM;

        let tool_defs = self.tools.schemas();
        let msgs = build_messages(&history, system_text, prompt);

        let (final_text, new_messages) = engine::react_loop_streaming(
            &*self.provider,
            &self.tools,
            msgs,
            &tool_defs,
            event_tx,
            self.config.max_rounds,
            self.config.tool_timeout_secs,
        ).await?;

        self.messages = new_messages;
        Ok(final_text)
    }
}

fn build_messages(
    history: &[LlmMessage],
    system_text: &str,
    prompt: &str,
) -> Vec<LlmMessage> {
    let mut msgs = Vec::new();
    let context = crate::prompt::build_context();
    let system = if context.is_empty() {
        system_text.to_string()
    } else {
        format!("{}\n\n## Current Context\n{}", system_text, context)
    };
    msgs.push(LlmMessage::System(system));
    for msg in history {
        match msg {
            LlmMessage::User(c) => msgs.push(LlmMessage::User(c.clone())),
            LlmMessage::Assistant(c) => msgs.push(LlmMessage::Assistant(c.clone())),
            LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls } => {
                msgs.push(LlmMessage::AssistantWithReasoning {
                    content: content.clone(),
                    reasoning: reasoning.clone(),
                    tool_calls: tool_calls.clone(),
                });
            }
            LlmMessage::ToolCall { id, name, args } => {
                msgs.push(LlmMessage::ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    args: args.clone(),
                });
            }
            LlmMessage::Tool {
                name,
                content,
                call_id,
            } => {
                msgs.push(LlmMessage::Tool {
                    name: name.clone(),
                    content: content.clone(),
                    call_id: call_id.clone(),
                });
            }
            _ => {}
        }
    }
    msgs.push(LlmMessage::User(prompt.to_string()));
    msgs
}

use session_trait::{ChatInput, ChatOutput, ChatSession, StreamingChatSession};
use std::pin::Pin;
use std::future::Future;

impl ChatSession for Agent {
    fn run(
        &mut self,
        input: ChatInput,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>> {
        Box::pin(async move {
            let system_text = crate::prompt::SYSTEM;
            let tool_defs = self.tools.schemas();
            let msgs = build_messages(&input.history, system_text, &input.prompt);
            let (final_text, new_messages) = engine::react_loop(
                &*self.provider, &self.tools, msgs, &tool_defs,
                self.json_output, self.config.max_rounds, self.config.tool_timeout_secs,
            ).await?;
            self.messages = new_messages.clone();
            Ok(ChatOutput { text: final_text, messages: new_messages, usage: None })
        })
    }
}

impl StreamingChatSession for Agent {
    fn run_streaming(
        &mut self,
        input: ChatInput,
        event_tx: mpsc::Sender<event::AgentEvent>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>> {
        Box::pin(async move {
            let system_text = crate::prompt::SYSTEM;
            let tool_defs = self.tools.schemas();
            let msgs = build_messages(&input.history, system_text, &input.prompt);
            let (final_text, new_messages) = engine::react_loop_streaming(
                &*self.provider, &self.tools, msgs, &tool_defs,
                event_tx, self.config.max_rounds, self.config.tool_timeout_secs,
            ).await?;
            self.messages = new_messages.clone();
            Ok(ChatOutput { text: final_text, messages: new_messages, usage: None })
        })
    }
}
