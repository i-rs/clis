use crate::config::Config;
use crate::mcp::McpRegistry;
use crate::memory::CrossSessionMemory;
use crate::skill_store::SkillStore;
use crate::storage::ClawStorage;
use crate::tool_cache::ToolDocCache;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Runtime data for a single agent.
pub struct AgentRuntime {
    pub memory: CrossSessionMemory,
    pub tool_cache: ToolDocCache,
    pub skill_store: SkillStore,
    pub mcp_registry: McpRegistry,
    pub layered_memory: crate::core::layered_memory::LayeredMemory,
}

impl AgentRuntime {
    fn new(
        #[cfg_attr(test, allow(unused_variables))] config: &Config,
        storage: &Arc<ClawStorage>,
        agent_id: &str,
    ) -> Self {
        #[cfg(not(test))]
        let resolved = config.agent_config(agent_id);
        Self {
            memory: CrossSessionMemory::for_agent_with_storage(storage, agent_id),
            tool_cache: ToolDocCache::for_agent_with_storage(storage, agent_id),
            skill_store: SkillStore::for_agent_with_storage(storage, agent_id),
            #[cfg(test)]
            mcp_registry: crate::mcp::McpRegistry::empty_for_test(),
            #[cfg(not(test))]
            mcp_registry: McpRegistry::for_agent(&resolved, &config.mcp_servers),
            layered_memory: crate::core::layered_memory::LayeredMemory::new(),
        }
    }
}

/// Central storage for per-agent runtime data.
/// Each (user, agent) pair gets its own memory, tool cache, skill store, and MCP registry.
pub struct AgentRuntimeStore {
    pub(crate) runtimes: HashMap<(String, String), AgentRuntime>,
}

fn runtime_key(user_id: &str, agent_id: &str) -> (String, String) {
    (user_id.to_string(), agent_id.to_string())
}

impl AgentRuntimeStore {
    /// Legacy constructor (backward-compatible, uses file backend internally).
    #[allow(dead_code)]
    pub fn new(config: &Config, claw_dir: &std::path::Path) -> Self {
        let storage = Arc::new(ClawStorage::file(claw_dir.to_path_buf()));
        Self::new_with_storage(config, &storage)
    }

    /// Create with a shared storage backend.
    /// Initializes runtimes for the "default" user and all configured agents.
    pub fn new_with_storage(config: &Config, storage: &Arc<ClawStorage>) -> Self {
        let agent_ids = config.all_agent_ids();
        let mut runtimes = HashMap::new();
        for id in &agent_ids {
            runtimes.insert(runtime_key("default", id), AgentRuntime::new(config, storage, id));
        }
        let mut store = Self { runtimes };
        store.prefetch_hot_tools();
        store
    }

    fn prefetch_hot_tools(&mut self) {
        for rt in self.runtimes.values_mut() {
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

    fn get_or_init(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut AgentRuntime, crate::error::ClawError> {
        let key = runtime_key(user_id, agent_id);
        if !self.runtimes.contains_key(&key) {
            let src_key = if self.runtimes.contains_key(&runtime_key("default", agent_id)) {
                runtime_key("default", agent_id)
            } else {
                runtime_key("default", "default")
            };
            if let Some(src) = self.runtimes.get(&src_key) {
                let cloned = AgentRuntime {
                    memory: src.memory.clone(),
                    tool_cache: src.tool_cache.clone(),
                    skill_store: src.skill_store.clone(),
                    layered_memory: src.layered_memory.clone(),
                    mcp_registry: src.mcp_registry.clone(),
                };
                self.runtimes.insert(key.clone(), cloned);
            }
        }
        self.runtimes.get_mut(&key).ok_or_else(|| {
            crate::error::ClawError::NotFound(format!(
                "AgentRuntimeStore: (user='{}', agent='{}') not found and no 'default' fallback initialized",
                user_id, agent_id
            ))
        })
    }

    fn get_ref(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&AgentRuntime, crate::error::ClawError> {
        let key = runtime_key(user_id, agent_id);
        self.runtimes
            .get(&key)
            .or_else(|| self.runtimes.get(&runtime_key("default", agent_id)))
            .or_else(|| self.runtimes.get(&runtime_key("default", "default")))
            .ok_or_else(|| {
                crate::error::ClawError::NotFound(format!(
                    "AgentRuntimeStore: (user='{}', agent='{}') not found and no 'default' fallback initialized",
                    user_id, agent_id
                ))
            })
    }

    pub fn memory_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&CrossSessionMemory, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.memory)
    }

    pub fn memory_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut CrossSessionMemory, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.memory)
    }

    pub fn tool_cache_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&ToolDocCache, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.tool_cache)
    }

    pub fn tool_cache_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut ToolDocCache, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.tool_cache)
    }

    pub fn skill_store_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&SkillStore, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.skill_store)
    }

    pub fn mcp_registry_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&McpRegistry, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.mcp_registry)
    }

    pub fn mcp_registry_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut McpRegistry, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.mcp_registry)
    }

    pub fn layered_memory_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&crate::core::layered_memory::LayeredMemory, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.layered_memory)
    }

    pub fn layered_memory_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut crate::core::layered_memory::LayeredMemory, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.layered_memory)
    }

    /// Initialize runtime data for a new agent across all existing users.
    pub fn add_agent(&mut self, config: &Config, agent_id: &str) {
        for user_id in self.user_ids() {
            let key = runtime_key(&user_id, agent_id);
            if !self.runtimes.contains_key(&key) {
                let src_key = runtime_key(&user_id, "default");
                if let Some(src) = self.runtimes.get(&src_key) {
                    self.runtimes.insert(key.clone(), AgentRuntime {
                        memory: src.memory.clone(),
                        tool_cache: src.tool_cache.clone(),
                        skill_store: src.skill_store.clone(),
                        layered_memory: src.layered_memory.clone(),
                        mcp_registry: src.mcp_registry.clone(),
                    });
                }
            }
        }
        let _ = config;
    }

    /// Remove runtime data for an agent across all users.
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.runtimes.retain(|(_, a), _| a != agent_id);
    }

    fn user_ids(&self) -> Vec<String> {
        self.runtimes.keys().map(|(u, _)| u.clone())
            .collect::<HashSet<_>>()
            .into_iter().collect()
    }
}
