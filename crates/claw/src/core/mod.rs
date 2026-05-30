pub mod context;
pub mod engine;
pub mod executor;

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
use tokio::sync::mpsc;

/// Runtime data for a single agent.
pub struct AgentRuntime {
    pub memory: CrossSessionMemory,
    pub tool_cache: ToolDocCache,
    pub skill_store: SkillStore,
    pub mcp_registry: McpRegistry,
}

impl AgentRuntime {
    fn new(
        #[cfg_attr(test, allow(unused_variables))] config: &Config,
        claw_dir: &std::path::Path,
        agent_id: &str,
    ) -> Self {
        #[cfg(not(test))]
        let resolved = config.agent_config(agent_id);
        Self {
            memory: CrossSessionMemory::for_agent(claw_dir, agent_id),
            tool_cache: ToolDocCache::for_agent(claw_dir, agent_id),
            skill_store: SkillStore::for_agent(claw_dir, agent_id),
            #[cfg(test)]
            mcp_registry: crate::mcp::McpRegistry::empty_for_test(),
            #[cfg(not(test))]
            mcp_registry: McpRegistry::for_agent(&resolved, &config.mcp_servers),
        }
    }

    #[allow(dead_code)]
    fn refresh_mcp(&mut self, config: &Config, agent_id: &str) {
        let resolved = config.agent_config(agent_id);
        self.mcp_registry = McpRegistry::for_agent(&resolved, &config.mcp_servers);
    }
}

/// Central storage for per-agent runtime data.
/// Each agent gets its own memory, tool cache, skill store, and MCP registry.
pub struct AgentRuntimeStore {
    runtimes: HashMap<String, AgentRuntime>,
}

impl AgentRuntimeStore {
    pub fn new(config: &Config, claw_dir: &std::path::Path) -> Self {
        let agent_ids = config.all_agent_ids();
        let mut runtimes = HashMap::new();
        for id in &agent_ids {
            runtimes.insert(id.clone(), AgentRuntime::new(config, claw_dir, id));
        }
        let mut store = Self { runtimes };
        store.prefetch_hot_tools();
        store
    }

    fn prefetch_hot_tools(&mut self) {
        for (_id, rt) in &mut self.runtimes {
            let mut tools: Vec<(String, usize)> = rt
                .memory
                .tool_frequency()
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            tools.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            let hot: Vec<String> = tools.into_iter().take(5).map(|(t, _)| t).collect();
            if !hot.is_empty() {
                rt.tool_cache.prefetch(&hot);
            }
        }
    }

    fn get(&self, agent_id: &str) -> &AgentRuntime {
        self.runtimes.get(agent_id).unwrap_or_else(|| {
            self.runtimes.get("default").expect("AgentRuntimeStore: 'default' agent not found, this is a bug")
        })
    }

    fn get_mut(&mut self, agent_id: &str) -> &mut AgentRuntime {
        if self.runtimes.contains_key(agent_id) {
            self.runtimes.get_mut(agent_id).expect("bug: agent just checked not found")
        } else {
            self.runtimes.get_mut("default").expect("AgentRuntimeStore: 'default' agent not found, this is a bug")
        }
    }

    pub fn memory_for(&self, agent_id: &str) -> &CrossSessionMemory {
        &self.get(agent_id).memory
    }

    pub fn memory_for_mut(&mut self, agent_id: &str) -> &mut CrossSessionMemory {
        &mut self.get_mut(agent_id).memory
    }

    pub fn tool_cache_for(&self, agent_id: &str) -> &ToolDocCache {
        &self.get(agent_id).tool_cache
    }

    pub fn tool_cache_for_mut(&mut self, agent_id: &str) -> &mut ToolDocCache {
        &mut self.get_mut(agent_id).tool_cache
    }

    pub fn skill_store_for(&self, agent_id: &str) -> &SkillStore {
        &self.get(agent_id).skill_store
    }

    pub fn mcp_registry_for(&self, agent_id: &str) -> &McpRegistry {
        &self.get(agent_id).mcp_registry
    }

    pub fn mcp_registry_for_mut(&mut self, agent_id: &str) -> &mut McpRegistry {
        &mut self.get_mut(agent_id).mcp_registry
    }

    /// Refresh MCP registries for all agents (e.g. after plugin discovery).
    #[allow(dead_code)]
    pub fn refresh_mcp_registries(&mut self, config: &Config) {
        let agent_ids: Vec<String> = self.runtimes.keys().cloned().collect();
        for id in agent_ids {
            if let Some(runtime) = self.runtimes.get_mut(&id) {
                runtime.refresh_mcp(config, &id);
            }
        }
    }

    /// Initialize runtime data for a new agent.
    #[allow(dead_code)]
    pub fn add_agent(&mut self, config: &Config, claw_dir: &std::path::Path, agent_id: &str) {
        self.runtimes.insert(agent_id.to_string(), AgentRuntime::new(config, claw_dir, agent_id));
    }

    /// Remove runtime data for an agent.
    #[allow(dead_code)]
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.runtimes.remove(agent_id);
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
    pub agent_store: AgentRuntimeStore,
    pub stats_manager: crate::stats::StatsManager,
    pub http_client: reqwest::Client,
}

impl AppCore {
    /// Create a new AppCore from configuration.
    /// Initializes session manager, per-agent runtime data, and i-rs tool discovery.
    pub fn new(mut config: Config) -> anyhow::Result<Self> {
        let claw_dir = crate::utils::claw_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;

        config.discover_i_rs_tools(&claw_dir);

        let session_mgr = SessionManager::new(claw_dir.clone());
        let agent_store = AgentRuntimeStore::new(&config, &claw_dir);
        let stats_manager = crate::stats::StatsManager::new(&claw_dir, &config.stats);

        Ok(Self {
            config,
            session_mgr,
            agent_store,
            stats_manager,
            http_client: crate::providers::shared_client(),
        })
    }

    /// Flush all in-memory state to disk before exit.
    /// Call this after the TUI main loop ends, before terminal restore.
    pub fn shutdown(&mut self) {
        tracing::info!("AppCore shutting down, flushing state to disk...");
        for (agent_id, rt) in &mut self.agent_store.runtimes {
            rt.memory.flush();
            tracing::debug!("Flushed memory for agent '{}'", agent_id);
        }
        self.stats_manager.flush();
        tracing::info!("AppCore shutdown complete");
    }

    /// Resolve the config for a given agent ID.
    /// Falls back to default config if agent doesn't exist.
    #[allow(dead_code)]
    pub fn agent_config(&self, id: &str) -> crate::config::ResolvedAgentConfig {
        self.config.agent_config(id)
    }

    /// Build the API message list for an LLM chat call.
    /// Wraps engine::build_messages with AppCore's state.
    #[allow(dead_code)]
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
    #[tracing::instrument(skip(self, app_messages, saved_api_messages, reminder_text))]
    pub fn build_messages_for(
        &self,
        app_messages: &[Message],
        user_text: &str,
        saved_api_messages: &Option<Vec<Value>>,
        reminder_text: Option<&str>,
        agent_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let tool_index = self.build_irs_tool_index(&resolved);

        let memory = self.agent_store.memory_for(agent_id);

        engine::build_messages(engine::MessageBuildParams {
            app_messages,
            user_text,
            saved_api_messages,
            tool_frequency: memory.tool_frequency(),
            tool_index: &tool_index,
            hot_tools: &self.agent_store.tool_cache_for(agent_id).format_hot_tools(
                &memory.tool_frequency().keys().cloned().collect::<Vec<_>>(),
            ),
            skills: &self.agent_store.skill_store_for(agent_id).format_skills(),
            user_memory: &memory.format_user_memory(),
            user_profile: &memory.format_user_profile(),
            reminder_text,
            system_prompt_override: resolved.system_prompt.as_deref(),
            plan_then_execute: self.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute,
            max_conversation_turns: self.config.max_conversation_turns,
        })
    }

    /// Spawn the LLM chat loop in a background task.
    /// The `llm_tx` sender receives LlmEvent updates (tokens, tool calls, errors, done).
    #[allow(dead_code)]
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
        let provider = crate::providers::create_provider_for(
            &self.http_client,
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

        // Load executable skills as callable tools
        let skills = self.agent_store.skill_store_for(agent_id).executable_skills();

        rt.spawn(async move {
            engine::chat_loop(provider, agent_config, messages, llm_tx, mcp, skills).await;
        });
    }

    /// Build the tool index string for system prompt from discovered i-rs tools.
    pub fn build_irs_tool_index(&self, resolved: &crate::config::ResolvedAgentConfig) -> String {
        if self.config.i_rs_tool_index.is_empty() {
            return String::new();
        }

        let enabled = &resolved.enabled_tools;
        let mut result = String::from("## i-rs 工具索引\n\n");
        for name in &self.config.i_rs_tools {
            if !enabled.is_empty() && !enabled.contains(name) {
                continue;
            }
            if let Some(desc) = self.config.i_rs_tool_index.get(name) {
                if !desc.is_empty() {
                    result.push_str(&format!("- {}: {}\n", name, desc));
                } else {
                    result.push_str(&format!("- {}\n", name));
                }
            }
        }
        result
    }

    /// Compress API messages after a conversation turn completes.
    /// Uses ContextManager for adaptive token-aware compression.
    pub fn compress_api_messages(&self, msgs: &mut Vec<Value>, agent_id: &str) {
        let memory = self.agent_store.memory_for(agent_id);
        // Use ContextManager for adaptive compression based on token budget
        let resolved = self.config.agent_config(agent_id);
        let ctx_mgr = context::ContextManager::for_model(&resolved.model);
        ctx_mgr.compress(msgs, memory.tool_frequency());
    }

    /// Get the base directory for claw data.
    #[allow(dead_code)]
    pub fn claw_dir(&self) -> anyhow::Result<std::path::PathBuf> {
        crate::utils::claw_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))
    }
}

pub fn record_tool_memory(
    agent_store: &mut AgentRuntimeStore,
    i_rs_tool_index: &HashMap<String, String>,
    agent_id: &str,
    name: &str,
    args: &str,
    result: &str,
) {
    if name == "update_user_memory" {
        if let Ok(parsed) = serde_json::from_str::<Value>(args) {
            if let Some(user_name) = parsed
                .get("user_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                agent_store.memory_for_mut(agent_id).set_user_name(user_name);
            }
            if let Some(info) = parsed.get("user_info").and_then(|v| v.as_array()) {
                for item in info {
                    if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                        agent_store.memory_for_mut(agent_id).add_user_info(s);
                    }
                }
            }
            if let Some(prefs) = parsed.get("preferences").and_then(|v| v.as_array()) {
                for item in prefs {
                    if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                        agent_store.memory_for_mut(agent_id).add_preference(s);
                    }
                }
            }
        }
    }

    if name == "i_rs" {
        if let Ok(parsed) = serde_json::from_str::<Value>(args) {
            if let Some(tool) = parsed.get("tool").and_then(|t| t.as_str()) {
                if i_rs_tool_index.contains_key(tool) {
                    agent_store.memory_for_mut(agent_id).record_tool_use(tool);
                }
                let cmd = parsed.get("command").and_then(|c| c.as_str());
                if cmd == Some("skill")
                    && parsed
                        .get("args")
                        .and_then(|a| a.as_array())
                        .map(|arr| arr.iter().any(|v| v.as_str() == Some("teach")))
                        .unwrap_or(false)
                {
                    let cache = agent_store.tool_cache_for_mut(agent_id);
                    cache.hot_docs.insert(tool.to_string(), result.to_string());
                    cache.save_hot_docs();
                }
            }
        }
    } else if i_rs_tool_index.contains_key(name) || name.starts_with("skill_") {
        agent_store.memory_for_mut(agent_id).record_tool_use(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_appcore_new() {
        let (_config, _core) = crate::test_helpers::test_core();
        // AppCore::new 成功创建 = 测试通过
    }

    #[test]
    fn test_agent_runtime_store_init() {
        let config = crate::test_helpers::test_config();
        let dir = tempfile::tempdir().unwrap();
        let store = AgentRuntimeStore::new(&config, dir.path());

        // 默认应包含 "default" agent
        let default_memory = store.memory_for("default");
        let default_skills = store.skill_store_for("default");
        let _ = default_memory;
        let _ = default_skills;
    }

    #[test]
    fn test_build_messages_for_default() {
        let (_config, core) = crate::test_helpers::test_core();
        let msgs = core.build_messages(
            &[],
            "hello",
            &None,
            None,
        );
        assert!(msgs.len() >= 2, "至少应有 system + user 消息");
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs.last().unwrap()["role"], "user");
        assert_eq!(msgs.last().unwrap()["content"], "hello");
    }

    #[test]
    fn test_build_messages_for_agent() {
        let (_config, core) = crate::test_helpers::test_core();
        let msgs = core.build_messages_for(
            &[],
            "test",
            &None,
            None,
            "default",
        );
        assert!(msgs.len() >= 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs.last().unwrap()["role"], "user");
    }

    #[test]
    fn test_spawn_chat_for() {
        // 在独立线程创建 tokio runtime 避免嵌套
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (_config, core) = crate::test_helpers::test_core();
            let (tx, _rx) = mpsc::unbounded_channel();
            let messages = vec![json!({"role": "user", "content": "hi"})];
            // spawn_chat 不应 panic
            core.spawn_chat(&rt, tx, messages);
            std::thread::sleep(std::time::Duration::from_millis(50));
        })
        .join()
        .expect("spawn_chat_for 不应 panic");
    }

}
