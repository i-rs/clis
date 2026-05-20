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
use std::path::Path;
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
        Self { runtimes }
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

    pub fn skill_store_for(&self, agent_id: &str) -> &SkillStore {
        &self.get(agent_id).skill_store
    }

    pub fn mcp_registry_for(&self, agent_id: &str) -> &McpRegistry {
        &self.get(agent_id).mcp_registry
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
    /// Per-agent runtime data (memory, tool cache, skills, MCP).
    pub agent_store: AgentRuntimeStore,
    /// Token usage statistics manager.
    pub stats_manager: crate::stats::StatsManager,
}

impl AppCore {
    /// Create a new AppCore from configuration.
    /// Initializes session manager and per-agent runtime data.
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let claw_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .join(".i-rs-claw")
            .join("claw");

        // Migrate legacy data to agents/default/ if needed
        Self::migrate_legacy_data(&claw_dir);

        let session_mgr = SessionManager::new(claw_dir.clone());
        let agent_store = AgentRuntimeStore::new(&config, &claw_dir);
        let stats_manager = crate::stats::StatsManager::new(&claw_dir, &config.stats);

        Ok(Self {
            config,
            session_mgr,
            agent_store,
            stats_manager,
        })
    }

    /// Migrate legacy data files (memory.json, skills/, etc.) to agents/default/
    /// on first run after upgrade.
    fn migrate_legacy_data(claw_dir: &Path) {
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
        let enabled = if resolved.enabled_tools.is_empty() {
            None
        } else {
            Some(&resolved.enabled_tools)
        };
        let tool_index = crate::tools::format_index(enabled);

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
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        Ok(home.join(".i-rs-claw").join("claw"))
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
        // 不 panic 即通过
        drop(default_memory);
        drop(default_skills);
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

    #[test]
    fn test_migrate_legacy_data() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("memory.json"), r#"{"key":"legacy"}"#).unwrap();

        AppCore::migrate_legacy_data(dir.path());

        let dest = dir.path().join("agents").join("default").join("memory.json");
        assert!(dest.exists(), "旧版 memory.json 应被迁移到 agents/default/");
        let content = std::fs::read_to_string(&dest).unwrap();
        assert!(content.contains("legacy"), "迁移后内容应一致");
    }

    #[test]
    fn test_migrate_legacy_data_noop_when_default_exists() {
        let dir = tempfile::tempdir().unwrap();
        let default_dir = dir.path().join("agents").join("default");
        std::fs::create_dir_all(&default_dir).unwrap();
        std::fs::write(default_dir.join("memory.json"), "new").unwrap();
        std::fs::write(dir.path().join("memory.json"), "legacy").unwrap();

        AppCore::migrate_legacy_data(dir.path());

        let content = std::fs::read_to_string(default_dir.join("memory.json")).unwrap();
        assert_eq!(content, "new", "已存在的 default 数据不应被覆盖");
    }

    #[test]
    fn test_migrate_legacy_data_noop_when_no_legacy() {
        let dir = tempfile::tempdir().unwrap();

        AppCore::migrate_legacy_data(dir.path());

        // 没有旧文件时不应创建目录
        assert!(
            !dir.path().join("agents").exists()
                || dir.path().join("agents").read_dir().unwrap().next().is_none(),
            "无旧数据时不应创建 agents 目录"
        );
    }
}
