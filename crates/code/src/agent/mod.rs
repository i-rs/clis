pub mod engine;
pub mod context;

use crate::config::Config;
use crate::provider::{LlmProvider, LlmMessage};
use crate::tools::ToolRegistry;

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
        let system_text = "You are i-rs-code, a code editor AI agent. You can read/write files, execute commands, create i-rs CLI tools, and more. Always use the available tools to help the user. After making changes, verify with cargo check or equivalent commands.";

        let tool_defs = self.tools.schemas();

        let mut msgs = Vec::new();
        msgs.push(LlmMessage::System(system_text.to_string()));
        for msg in &self.messages {
            match msg {
                LlmMessage::User(c) => msgs.push(LlmMessage::User(c.clone())),
                LlmMessage::Assistant(c) => msgs.push(LlmMessage::Assistant(c.clone())),
                LlmMessage::ToolCall { id, name, args } => {
                    msgs.push(LlmMessage::ToolCall { id: id.clone(), name: name.clone(), args: args.clone() });
                }
                LlmMessage::Tool { name, content, call_id } => {
                    msgs.push(LlmMessage::Tool { name: name.clone(), content: content.clone(), call_id: call_id.clone() });
                }
                _ => {}
            }
        }
        msgs.push(LlmMessage::User(prompt.to_string()));

        let (final_text, new_messages) = engine::react_loop(
            &*self.provider,
            &self.tools,
            msgs,
            &tool_defs,
            self.json_output,
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
}
