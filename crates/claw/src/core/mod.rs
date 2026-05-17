pub mod engine;

use crate::app::Message;
use crate::config::Config;
use crate::llm::LlmEvent;
use crate::mcp::McpRegistry;
use crate::memory::CrossSessionMemory;
use crate::session::SessionManager;
use crate::skill_store::SkillStore;
use crate::tool_cache::ToolDocCache;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[allow(dead_code)]

/// Central storage for per-agent runtime data.
/// Each agent gets its own memory, tool cache, skill store, and MCP registry.
pub struct AgentRuntimeStore {
    memories: HashMap<String, CrossSessionMemory>,
    tool_caches: HashMap<String, ToolDocCache>,
    skill_stores: HashMap<String, SkillStore>,
    mcp_registries: HashMap<String, McpRegistry>,
}

impl AgentRuntimeStore {
    pub fn new(config: &Config, claw_dir: &PathBuf) -> Self {
        let agent_ids = config.agent_ids();
        let mut store = Self {
            memories: HashMap::new(),
            tool_caches: HashMap::new(),
            skill_stores: HashMap::new(),
            mcp_registries: HashMap::new(),
        };
        for id in &agent_ids {
            store.memories.insert(id.clone(), CrossSessionMemory::for_agent(claw_dir, id));
            store.tool_caches.insert(id.clone(), ToolDocCache::for_agent(claw_dir, id));
            store.skill_stores.insert(id.clone(), SkillStore::for_agent(claw_dir, id));
            let resolved = config.agent_config(id);
            store.mcp_registries.insert(id.clone(), McpRegistry::for_agent(&resolved, &config.mcp_servers));
        }
        store
    }

    pub fn memory_for(&self, agent_id: &str) -> &CrossSessionMemory {
        self.memories.get(agent_id).unwrap_or_else(|| &self.memories["default"])
    }

    pub fn memory_for_mut(&mut self, agent_id: &str) -> &mut CrossSessionMemory {
        if self.memories.contains_key(agent_id) {
            self.memories.get_mut(agent_id).expect("just checked")
        } else {
            self.memories.get_mut("default").expect("default agent must exist")
        }
    }

    pub fn tool_cache_for(&self, agent_id: &str) -> &ToolDocCache {
        self.tool_caches.get(agent_id).unwrap_or_else(|| &self.tool_caches["default"])
    }

    pub fn skill_store_for(&self, agent_id: &str) -> &SkillStore {
        self.skill_stores.get(agent_id).unwrap_or_else(|| &self.skill_stores["default"])
    }

    pub fn mcp_registry_for(&self, agent_id: &str) -> &McpRegistry {
        self.mcp_registries.get(agent_id).unwrap_or_else(|| &self.mcp_registries["default"])
    }

    /// Initialize runtime data for a new agent.
    /// Called after adding an agent to config.
    pub fn add_agent(&mut self, config: &Config, claw_dir: &PathBuf, agent_id: &str) {
        let resolved = config.agent_config(agent_id);
        self.memories.insert(agent_id.to_string(), CrossSessionMemory::for_agent(claw_dir, agent_id));
        self.tool_caches.insert(agent_id.to_string(), ToolDocCache::for_agent(claw_dir, agent_id));
        self.skill_stores.insert(agent_id.to_string(), SkillStore::for_agent(claw_dir, agent_id));
        self.mcp_registries.insert(agent_id.to_string(), McpRegistry::for_agent(&resolved, &config.mcp_servers));
    }

    /// Remove runtime data for an agent.
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.memories.remove(agent_id);
        self.tool_caches.remove(agent_id);
        self.skill_stores.remove(agent_id);
        self.mcp_registries.remove(agent_id);
    }
}

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
    /// Per-agent runtime data (memory, tool cache, skills, MCP).
    pub agent_store: AgentRuntimeStore,
    claw_dir: PathBuf,
}

impl AppCore {
    /// Create a new AppCore from configuration.
    /// Initializes session manager and per-agent runtime data.
    pub fn new(config: Config) -> Self {
        let claw_dir = dirs::home_dir()
            .expect("cannot get home directory")
            .join(".i-rs-claw")
            .join("claw");

        // Migrate legacy data to agents/default/ if needed
        Self::migrate_legacy_data(&claw_dir);

        let session_mgr = SessionManager::new(claw_dir.clone());
        let agent_store = AgentRuntimeStore::new(&config, &claw_dir);

        Self {
            config,
            session_mgr,
            agent_store,
            claw_dir,
        }
    }

    /// Migrate legacy data files (memory.json, skills/, etc.) to agents/default/
    /// on first run after upgrade.
    fn migrate_legacy_data(claw_dir: &PathBuf) {
        let default_dir = claw_dir.join("agents").join("default");
        let default_memory = default_dir.join("memory.json");
        let legacy_memory = claw_dir.join("memory.json");

        // Only migrate if legacy memory.json exists AND default doesn't
        if legacy_memory.exists() && !default_memory.exists() {
            let _ = std::fs::create_dir_all(&default_dir);
            let _ = std::fs::copy(&legacy_memory, &default_memory);

            // Migrate hot_docs_cache.json
            let legacy_cache = claw_dir.join("hot_docs_cache.json");
            let default_cache = default_dir.join("hot_docs_cache.json");
            if legacy_cache.exists() && !default_cache.exists() {
                let _ = std::fs::copy(&legacy_cache, &default_cache);
            }

            // Migrate skills directory
            let legacy_skills = claw_dir.join("skills");
            let default_skills = default_dir.join("skills");
            if legacy_skills.exists() && !default_skills.exists() {
                let _ = std::fs::create_dir_all(&default_skills);
                if let Ok(entries) = std::fs::read_dir(&legacy_skills) {
                    for entry in entries.flatten() {
                        let _ = std::fs::copy(entry.path(), default_skills.join(entry.file_name()));
                    }
                }
            }
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

        let memory = self.agent_store.memory_for(agent_id);

        engine::build_messages(
            app_messages,
            user_text,
            saved_api_messages,
            memory.tool_frequency(),
            &tool_index,
            &self.agent_store.tool_cache_for(agent_id).format_hot_tools(
                &memory.tool_frequency().keys().cloned().collect::<Vec<_>>(),
            ),
            &self.agent_store.skill_store_for(agent_id).format_skills(),
            &memory.format_user_memory(),
            &memory.format_user_profile(),
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

        // Clone the agent's MCP registry (cheap: Arc inside)
        let mcp = self.agent_store.mcp_registry_for(agent_id).clone();

        rt.spawn(async move {
            engine::chat_loop(provider, agent_config, messages, llm_tx, mcp).await;
        });
    }

    /// Compress API messages after a conversation turn completes.
    pub fn compress_api_messages(&self, msgs: &mut Vec<Value>, agent_id: &str) {
        let memory = self.agent_store.memory_for(agent_id);
        engine::compress_api_messages(msgs, memory.tool_frequency());
    }

    /// Get the base directory for claw data.
    #[allow(dead_code)]
    pub fn claw_dir(&self) -> &PathBuf {
        &self.claw_dir
    }
}
