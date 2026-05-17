pub mod engine;

use crate::app::Message;
use crate::config::Config;
use crate::llm::LlmEvent;
use crate::memory::CrossSessionMemory;
use crate::session::SessionManager;
use crate::skill_store::SkillStore;
use crate::tool_cache::ToolDocCache;
use serde_json::Value;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[allow(dead_code)]

/// The shared core of the claw assistant, wrapping all persistent state.
///
/// AppCore is the single source of truth for:
/// - Configuration (provider, model, tools, MCP)
/// - Session management (conversation persistence)
/// - Cross-session memory (tool frequency, user preferences)
/// - Tool documentation cache (hot tool skill teach docs)
/// - Skill store (user-defined skills)
///
/// Both the TUI and the Gateway/Dashboard backends use AppCore.
pub struct AppCore {
    pub config: Config,
    pub session_mgr: SessionManager,
    pub cross_memory: CrossSessionMemory,
    pub tool_cache: ToolDocCache,
    pub skill_store: SkillStore,
    claw_dir: PathBuf,
}

impl AppCore {
    /// Create a new AppCore from configuration.
    /// Initializes session manager, cross-session memory, tool cache, and skill store.
    pub fn new(config: Config) -> Self {
        let claw_dir = dirs::home_dir()
            .expect("cannot get home directory")
            .join(".i-rs-claw")
            .join("claw");

        let tool_cache = ToolDocCache::new(claw_dir.clone());
        let skill_store = SkillStore::new(claw_dir.clone());
        let session_mgr = SessionManager::new(claw_dir.clone());
        let cross_memory = CrossSessionMemory::new(claw_dir.clone());

        Self {
            config,
            session_mgr,
            cross_memory,
            tool_cache,
            skill_store,
            claw_dir,
        }
    }

    /// Resolve the config for a given agent ID.
    /// Falls back to default config if agent doesn't exist.
    pub fn agent_config(&self, id: &str) -> crate::config::ResolvedAgentConfig {
        self.config.agent_config(id)
    }

    /// Build the API message list for an LLM chat call.
    /// Wraps engine::build_messages with AppCore's state.
    pub fn build_messages(
        &self,
        app_messages: &[Message],
        user_text: &str,
        saved_api_messages: &Option<Vec<Value>>,
        reminder_text: Option<&str>,
    ) -> Vec<Value> {
        self.build_messages_for(app_messages, user_text, saved_api_messages, reminder_text, "default")
    }

    /// Build the API message list for a specific agent.
    pub fn build_messages_for(
        &self,
        app_messages: &[Message],
        user_text: &str,
        saved_api_messages: &Option<Vec<Value>>,
        reminder_text: Option<&str>,
        agent_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let enabled = if resolved.enabled_tools.is_empty() {
            None
        } else {
            Some(&resolved.enabled_tools)
        };
        let tool_index = crate::tools::format_index(enabled);

        engine::build_messages(
            app_messages,
            user_text,
            saved_api_messages,
            self.cross_memory.tool_frequency(),
            &tool_index,
            &self.cross_memory.format_hot_tools(&self.tool_cache),
            &self.skill_store.format_skills(),
            &self.cross_memory.format_user_memory(),
            &self.cross_memory.format_user_profile(),
            reminder_text,
            resolved.system_prompt.as_deref(),
        )
    }

    /// Spawn the LLM chat loop in a background task.
    /// The `llm_tx` sender receives LlmEvent updates (tokens, tool calls, errors, done).
    pub fn spawn_chat(
        &self,
        rt: &tokio::runtime::Runtime,
        llm_tx: mpsc::UnboundedSender<LlmEvent>,
        messages: Vec<Value>,
    ) {
        self.spawn_chat_for(rt, llm_tx, messages, "default")
    }

    /// Spawn the LLM chat loop for a specific agent.
    pub fn spawn_chat_for(
        &self,
        rt: &tokio::runtime::Runtime,
        llm_tx: mpsc::UnboundedSender<LlmEvent>,
        messages: Vec<Value>,
        agent_id: &str,
    ) {
        let resolved = self.config.agent_config(agent_id);

        // Create provider for this agent config
        let provider = crate::provider::create_provider_for(
            &resolved.provider,
            &resolved.api_key,
            &resolved.base_url,
            &resolved.model,
        );

        // Clone config with agent-specific tool overrides
        let mut agent_config = self.config.clone();
        agent_config.enabled_tools = resolved.enabled_tools;

        rt.spawn(async move {
            engine::chat_loop(provider, agent_config, messages, llm_tx).await;
        });
    }

    /// Compress API messages after a conversation turn completes.
    pub fn compress_api_messages(&self, msgs: &mut Vec<Value>) {
        engine::compress_api_messages(msgs, self.cross_memory.tool_frequency());
    }

    /// Get the base directory for claw data.
    #[allow(dead_code)]
    pub fn claw_dir(&self) -> &PathBuf {
        &self.claw_dir
    }
}
