pub mod callbacks;
pub mod checkpoint;
pub mod context;
pub mod engine;
pub mod evals;
pub mod executor;
pub mod hitl;
pub mod layered_memory;
pub mod orchestration;
pub mod planning;
pub mod rag;
pub mod streaming;
pub mod tool_chain;

mod runtime;
pub(crate) use runtime::*;

use crate::app::Message;
use crate::config::Config;
use crate::llm::LlmEvent;
use crate::mcp::McpRegistry;
use crate::session::SessionManager;
use crate::skill_store::SkillDefinition;
use crate::storage::ClawStorage;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;

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
    pub stats_manager: std::sync::Arc<crate::stats::StatsManager>,
    #[allow(dead_code)]
    pub storage: std::sync::Arc<crate::storage::ClawStorage>,
    pub config_store: crate::storage::config_store::ConfigStore,
    pub http_client: reqwest::Client,
    pub checkpoint_store:
        std::sync::Arc<std::sync::Mutex<crate::core::checkpoint::CheckpointStore>>,
    tool_index_cache: String,
}

impl AppCore {
    /// Create a new AppCore from configuration.
    /// Initializes session manager, per-agent runtime data, and i-rs tool discovery.
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let claw_dir =
            crate::utils::claw_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        Self::with_claw_dir(config, claw_dir)
    }

    /// Create AppCore with an explicit claw data directory.
    /// Used by tests to avoid relying on HOME env var.
    pub fn with_claw_dir(mut config: Config, claw_dir: std::path::PathBuf) -> anyhow::Result<Self> {
        config.discover_i_rs_tools(&claw_dir);

        if config.behavior_analyst.enabled {
            let mut agent = crate::config::AgentConfig {
                capabilities: vec![
                    "数据分析".to_string(),
                    "行为分析".to_string(),
                    "数据可视化".to_string(),
                    "趋势总结".to_string(),
                    "健康报告".to_string(),
                ],
                ..Default::default()
            };
            let prompt_path = claw_dir.join("prompts").join("behavior_analyst.md");
            if prompt_path.exists() {
                agent.system_prompt_file = Some(prompt_path.to_string_lossy().to_string());
            }
            config
                .sub_agents
                .entry("behavior_analyst".to_string())
                .or_insert(agent);
        }

        let storage = match config.storage.backend {
            crate::storage::StorageBackend::Sqlite => {
                #[cfg(feature = "sqlite")]
                {
                    let path = config
                        .storage
                        .sqlite_path
                        .clone()
                        .map(crate::utils::expand_tilde)
                        .unwrap_or_else(|| claw_dir.join("claw.db"));
                    std::sync::Arc::new(block_on(ClawStorage::sqlite(path))?)
                }
                #[cfg(not(feature = "sqlite"))]
                anyhow::bail!(
                    "storage.backend = \"sqlite\" 但未启用 sqlite feature（需编译时添加 --features sqlite）"
                )
            }
            crate::storage::StorageBackend::Mysql => {
                #[cfg(feature = "mysql")]
                {
                    let path = config
                        .storage
                        .sql_url
                        .as_deref()
                        .unwrap_or("mysql://localhost:3306/i_rs_claw");
                    std::sync::Arc::new(block_on(ClawStorage::mysql(path))?)
                }
                #[cfg(not(feature = "mysql"))]
                anyhow::bail!(
                    "storage.backend = \"mysql\" 但未启用 mysql feature（需编译时添加 --features mysql）"
                )
            }
            crate::storage::StorageBackend::Postgres => {
                #[cfg(feature = "postgres")]
                {
                    let path = config
                        .storage
                        .sql_url
                        .as_deref()
                        .unwrap_or("postgres://localhost:5432/i_rs_claw");
                    std::sync::Arc::new(block_on(ClawStorage::postgres(path))?)
                }
                #[cfg(not(feature = "postgres"))]
                anyhow::bail!(
                    "storage.backend = \"postgres\" 但未启用 postgres feature（需编译时添加 --features postgres）"
                )
            }
            crate::storage::StorageBackend::Mongo => {
                #[cfg(feature = "mongo")]
                {
                    let url = config
                        .storage
                        .mongo_url
                        .as_deref()
                        .unwrap_or("mongodb://localhost:27017");
                    let db = config
                        .storage
                        .mongo_database
                        .as_deref()
                        .unwrap_or("i_rs_claw");
                    std::sync::Arc::new(block_on(ClawStorage::mongo(url, db))?)
                }
                #[cfg(not(feature = "mongo"))]
                anyhow::bail!(
                    "storage.backend = \"mongodb\" 但未启用 mongo feature（需编译时添加 --features mongo）"
                )
            }
            crate::storage::StorageBackend::Redis => {
                #[cfg(feature = "redis")]
                {
                    let url = config
                        .storage
                        .redis_url
                        .as_deref()
                        .unwrap_or("redis://localhost:6379/0");
                    std::sync::Arc::new(block_on(ClawStorage::redis(url))?)
                }
                #[cfg(not(feature = "redis"))]
                anyhow::bail!(
                    "storage.backend = \"redis\" 但未启用 redis feature（需编译时添加 --features redis）"
                )
            }
            crate::storage::StorageBackend::File => {
                std::sync::Arc::new(ClawStorage::file(claw_dir.clone()))
            }
        };

        let session_mgr = SessionManager::with_storage(storage.clone())?;
        let agent_store = AgentRuntimeStore::new_with_storage(&config, &storage);
        let stats_manager = std::sync::Arc::new(crate::stats::StatsManager::with_storage(
            storage.clone(),
            &config.stats,
            config.tz_offset,
        ));

        let tool_index_cache = build_full_tool_index(&config);

        let config_store = match config.storage.backend {
            crate::storage::StorageBackend::File => {
                crate::storage::config_store::ConfigStore::file(claw_dir.clone())
            }
            #[cfg(feature = "sqlite")]
            crate::storage::StorageBackend::Sqlite => {
                let path = config
                    .storage
                    .sqlite_path
                    .clone()
                    .unwrap_or_else(|| claw_dir.join("claw.db"));
                let backend = crate::utils::sync_block_on(async {
                    crate::storage::sql::sqlite::SqliteBackend::new(path).await
                })?;
                backend.into_config_store()
            }
            #[cfg(feature = "mysql")]
            crate::storage::StorageBackend::Mysql => {
                let url = config
                    .storage
                    .sql_url
                    .as_deref()
                    .unwrap_or("mysql://localhost:3306/i_rs_claw");
                let backend = crate::utils::sync_block_on(async {
                    crate::storage::sql::mysql::MySqlBackend::new(url).await
                })?;
                backend.into_config_store()
            }
            #[cfg(feature = "postgres")]
            crate::storage::StorageBackend::Postgres => {
                let url = config
                    .storage
                    .sql_url
                    .as_deref()
                    .unwrap_or("postgres://localhost:5432/i_rs_claw");
                let backend = crate::utils::sync_block_on(async {
                    crate::storage::sql::postgres::PgBackend::new(url).await
                })?;
                backend.into_config_store()
            }
            #[cfg(feature = "mongo")]
            crate::storage::StorageBackend::Mongo => {
                let url = config
                    .storage
                    .mongo_url
                    .as_deref()
                    .unwrap_or("mongodb://localhost:27017");
                let db = config
                    .storage
                    .mongo_database
                    .as_deref()
                    .unwrap_or("i_rs_claw");
                let backend = crate::utils::sync_block_on(async {
                    crate::storage::mongo::MongoBackend::new(url, db).await
                })?;
                backend.into_config_store()
            }
            #[cfg(feature = "redis")]
            crate::storage::StorageBackend::Redis => {
                let url = config
                    .storage
                    .redis_url
                    .as_deref()
                    .unwrap_or("redis://localhost:6379/0");
                let backend = crate::utils::sync_block_on(async {
                    crate::storage::redis::RedisBackend::new(url).await
                })?;
                backend.into_config_store()
            }
            // 所有 storage feature 全开时 6 个后端已被上面显式覆盖；
            // 关闭部分 feature 时仍需此分支保证穷尽。
            #[allow(unreachable_patterns)]
            _ => crate::storage::config_store::ConfigStore::default(),
        };

        Ok(Self {
            config,
            session_mgr,
            agent_store,
            stats_manager,
            storage,
            config_store,
            http_client: crate::providers::shared_client(),
            checkpoint_store: std::sync::Arc::new(std::sync::Mutex::new(
                crate::core::checkpoint::CheckpointStore::new(20),
            )),
            tool_index_cache,
        })
    }

    /// Flush all in-memory state to disk before exit.
    /// Call this after the TUI main loop ends, before terminal restore.
    pub fn shutdown(&mut self) {
        tracing::info!("AppCore shutting down, flushing state to disk...");
        for ((user_id, agent_id), rt) in &mut self.agent_store.runtimes {
            rt.memory.flush();
            tracing::debug!("Flushed memory for user '{}' agent '{}'", user_id, agent_id);
        }
        self.stats_manager.flush();
        #[allow(deprecated)]
        self.session_mgr.save_index();
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
        user_id: &str,
    ) -> Vec<Value> {
        self.build_messages_for(
            app_messages,
            user_text,
            saved_api_messages,
            reminder_text,
            "default",
            user_id,
        )
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
        user_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let tool_index = self.build_irs_tool_index(&resolved);

        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(
                agent_id,
                "agent runtime lookup failed in build_messages_for"
            );
            return Vec::new();
        };

        let identity = if let Some(nick) = memory.assistant_nickname() {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::from(
                "用户尚未给你起昵称。如果在对话中用户突然以某个名字称呼你，询问这是否是给你的新名字。",
            )
        };

        let routing_hint = {
            let agents: Vec<crate::config::ResolvedAgentConfig> = self
                .config
                .agent_ids()
                .iter()
                .map(|id| self.config.agent_config(id))
                .collect();
            let sub_agents: Vec<crate::config::ResolvedAgentConfig> = self
                .config
                .sub_agents
                .keys()
                .map(|id| self.config.agent_config(id))
                .collect();
            crate::router::TaskRouter::new(agents, sub_agents).routing_hint()
        };

        engine::build_messages(engine::MessageBuildParams {
            app_messages,
            user_text,
            saved_api_messages,
            tool_frequency: memory.tool_frequency(),
            tool_index: &tool_index,
            hot_tools: &{
                let Ok(cache) = self.agent_store.tool_cache_for(user_id, agent_id) else {
                    tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                    return Vec::new();
                };
                cache.format_hot_tools(&memory.tool_frequency().keys().cloned().collect::<Vec<_>>())
            },
            skills: &{
                match self.agent_store.skill_store_for(user_id, agent_id) {
                    Ok(s) => s.format_skills(),
                    Err(e) => {
                        tracing::error!(%e, agent_id, "skill store lookup failed");
                        return Vec::new();
                    }
                }
            },
            user_memory: &{
                let base = memory.format_user_memory();
                let layered = {
                    let Ok(lm) = self.agent_store.layered_memory_for(user_id, agent_id) else {
                        tracing::error!(agent_id, "agent runtime layered memory lookup failed");
                        return Vec::new();
                    };
                    lm.format_for_prompt()
                };
                if base.is_empty() {
                    layered
                } else if layered.is_empty() {
                    base
                } else {
                    format!("{}\n\n{}", base, layered)
                }
            },
            user_profile: &memory.format_user_profile(),
            reminder_text,
            system_prompt_override: resolved.system_prompt.as_deref(),
            plan_then_execute: self.config.execution_mode
                == crate::config::ExecutionMode::PlanThenExecute,
            max_conversation_turns: self.config.max_conversation_turns,
            tz_offset: self.config.tz_offset,
            identity: &identity,
            routing_hint: &routing_hint,
            model: &resolved.model,
        })
    }

    /// Async version of [`build_messages_for`].
    #[cfg(feature = "dashboard")]
    pub async fn build_messages_for_async(
        &self,
        app_messages: &[Message],
        user_text: &str,
        saved_api_messages: &Option<Vec<Value>>,
        reminder_text: Option<&str>,
        agent_id: &str,
        user_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let tool_index = self.build_irs_tool_index(&resolved);

        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(
                agent_id,
                "agent runtime lookup failed in build_messages_for_async"
            );
            return Vec::new();
        };

        let identity = if let Some(nick) = memory.assistant_nickname() {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::from(
                "用户尚未给你起昵称。如果在对话中用户突然以某个名字称呼你，询问这是否是给你的新名字。",
            )
        };

        let routing_hint = {
            let agents: Vec<crate::config::ResolvedAgentConfig> = self
                .config
                .agent_ids()
                .iter()
                .map(|id| self.config.agent_config(id))
                .collect();
            let sub_agents: Vec<crate::config::ResolvedAgentConfig> = self
                .config
                .sub_agents
                .keys()
                .map(|id| self.config.agent_config(id))
                .collect();
            crate::router::TaskRouter::new(agents, sub_agents).routing_hint()
        };

        let hot_tools = {
            let Ok(cache) = self.agent_store.tool_cache_for(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                return Vec::new();
            };
            cache.format_hot_tools(&memory.tool_frequency().keys().cloned().collect::<Vec<_>>())
        };
        let skills = match self.agent_store.skill_store_for(user_id, agent_id) {
            Ok(s) => s.format_skills_async().await,
            Err(e) => {
                tracing::error!(%e, agent_id, "skill store lookup failed");
                return Vec::new();
            }
        };

        engine::build_messages(engine::MessageBuildParams {
            app_messages,
            user_text,
            saved_api_messages,
            tool_frequency: memory.tool_frequency(),
            tool_index: &tool_index,
            hot_tools: &hot_tools,
            skills: &skills,
            user_memory: &{
                let base = memory.format_user_memory();
                let layered = {
                    let Ok(lm) = self.agent_store.layered_memory_for(user_id, agent_id) else {
                        tracing::error!(agent_id, "agent runtime layered memory lookup failed");
                        return Vec::new();
                    };
                    lm.format_for_prompt()
                };
                if base.is_empty() {
                    layered
                } else if layered.is_empty() {
                    base
                } else {
                    format!("{}\n\n{}", base, layered)
                }
            },
            user_profile: &memory.format_user_profile(),
            reminder_text,
            system_prompt_override: resolved.system_prompt.as_deref(),
            plan_then_execute: self.config.execution_mode
                == crate::config::ExecutionMode::PlanThenExecute,
            max_conversation_turns: self.config.max_conversation_turns,
            tz_offset: self.config.tz_offset,
            identity: &identity,
            routing_hint: &routing_hint,
            model: &resolved.model,
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
        user_id: &str,
    ) {
        self.spawn_chat_for(rt, llm_tx, messages, "default", &[], user_id)
    }

    /// Spawn the LLM chat loop for a specific agent.
    pub fn spawn_chat_for(
        &self,
        rt: &tokio::runtime::Runtime,
        llm_tx: mpsc::UnboundedSender<LlmEvent>,
        messages: Vec<Value>,
        agent_id: &str,
        recent_messages: &[Value],
        user_id: &str,
    ) {
        let (provider, agent_config, mcp, skills, tool_frequency, http_client) =
            match self.prepare_chat_loop(user_id, agent_id) {
                Ok(p) => p,
                Err(e) => {
                    let _ = llm_tx.send(LlmEvent::Error(format!(
                        "Failed to prepare chat loop: {}",
                        e
                    )));
                    return;
                }
            };
        let delegate_rt = self.build_delegate_runtime(
            user_id,
            agent_id,
            llm_tx.clone(),
            recent_messages.to_vec(),
        );
        let checkpoint_store = self.checkpoint_store.clone();
        let user_id = user_id.to_string();
        rt.spawn(async move {
            engine::chat_loop(
                provider,
                agent_config,
                messages,
                llm_tx,
                mcp,
                skills,
                tool_frequency,
                http_client,
                Some(delegate_rt),
                None,
                checkpoint_store,
                user_id,
            )
            .await;
        });
    }

    /// Shared preparation for chat loop: resolve agent config, create provider,
    /// clone per-agent state (MCP, skills, memory). Used by both sync and async spawn.
    ///
    /// Avoids constructing a full `ResolvedAgentConfig` (which clones every field)
    /// by resolving only the fields we need directly from `AgentConfig` with
    /// `as_deref()` fallbacks to top-level `Config`.
    #[allow(clippy::type_complexity)]
    fn prepare_chat_loop(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<
        (
            Box<dyn crate::providers::LlmProvider>,
            Config,
            McpRegistry,
            Vec<SkillDefinition>,
            HashMap<String, usize>,
            reqwest::Client,
        ),
        crate::error::ClawError,
    > {
        let agent = self
            .config
            .agents
            .get(agent_id)
            .or_else(|| self.config.sub_agents.get(agent_id));

        // Resolve provider config through the proper chain:
        //   agent.provider_ref → config.providers[name] → config.top-level fields
        let resolved_pc = self.config.resolve_provider_config(agent);
        // Apply agent-level overrides on top of resolved provider config
        let final_provider = agent
            .and_then(|a| a.provider)
            .unwrap_or(resolved_pc.provider);
        let final_api_key = agent
            .and_then(|a| a.api_key.as_deref())
            .unwrap_or(&resolved_pc.api_key);
        let final_base_url = agent
            .and_then(|a| a.base_url.as_deref())
            .unwrap_or(&resolved_pc.base_url);
        let final_model = agent
            .and_then(|a| a.model.as_deref())
            .unwrap_or(&resolved_pc.model);

        let provider = crate::providers::create_provider_for(
            &self.http_client,
            final_provider,
            final_api_key,
            final_base_url,
            final_model,
        );

        let mut agent_config = self.config.clone();
        if let Some(a) = agent
            && let Some(ref tools) = a.enabled_tools
        {
            agent_config.enabled_tools = tools.clone();
        }

        let mcp = self
            .agent_store
            .mcp_registry_for(user_id, agent_id)?
            .clone();
        let skills = self
            .agent_store
            .skill_store_for(user_id, agent_id)?
            .executable_skills();
        let tool_frequency = self
            .agent_store
            .memory_for(user_id, agent_id)?
            .tool_frequency()
            .clone();
        let http_client = self.http_client.clone();
        Ok((
            provider,
            agent_config,
            mcp,
            skills,
            tool_frequency,
            http_client,
        ))
    }

    /// Async version of [`prepare_chat_loop`].
    #[cfg(feature = "dashboard")]
    async fn prepare_chat_loop_async(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<
        (
            Box<dyn crate::providers::LlmProvider>,
            Config,
            McpRegistry,
            Vec<SkillDefinition>,
            HashMap<String, usize>,
            reqwest::Client,
        ),
        crate::error::ClawError,
    > {
        let agent = self
            .config
            .agents
            .get(agent_id)
            .or_else(|| self.config.sub_agents.get(agent_id));

        let resolved_pc = self.config.resolve_provider_config(agent);
        let final_provider = agent
            .and_then(|a| a.provider)
            .unwrap_or(resolved_pc.provider);
        let final_api_key = agent
            .and_then(|a| a.api_key.as_deref())
            .unwrap_or(&resolved_pc.api_key);
        let final_base_url = agent
            .and_then(|a| a.base_url.as_deref())
            .unwrap_or(&resolved_pc.base_url);
        let final_model = agent
            .and_then(|a| a.model.as_deref())
            .unwrap_or(&resolved_pc.model);

        let provider = crate::providers::create_provider_for(
            &self.http_client,
            final_provider,
            final_api_key,
            final_base_url,
            final_model,
        );

        let mut agent_config = self.config.clone();
        if let Some(a) = agent
            && let Some(ref tools) = a.enabled_tools
        {
            agent_config.enabled_tools = tools.clone();
        }

        let mcp = self
            .agent_store
            .mcp_registry_for(user_id, agent_id)?
            .clone();
        let skills = self
            .agent_store
            .skill_store_for(user_id, agent_id)?
            .executable_skills_async()
            .await;
        let tool_frequency = self
            .agent_store
            .memory_for(user_id, agent_id)?
            .tool_frequency()
            .clone();
        let http_client = self.http_client.clone();
        Ok((
            provider,
            agent_config,
            mcp,
            skills,
            tool_frequency,
            http_client,
        ))
    }

    fn build_delegate_runtime(
        &self,
        user_id: &str,
        agent_id: &str,
        parent_tx: mpsc::UnboundedSender<LlmEvent>,
        recent_messages: Vec<serde_json::Value>,
    ) -> std::sync::Arc<crate::tools::DelegateRuntime> {
        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return std::sync::Arc::new(crate::tools::DelegateRuntime {
                irs_tool_index: self.config.i_rs_tool_index.clone(),
                mcp_registry: Default::default(),
                skills: Vec::new(),
                tool_frequency: Default::default(),
                parent_tx,
                stats_manager: self.stats_manager.clone(),
                user_identity: String::new(),
                user_memory: String::new(),
                user_profile: String::new(),
                recent_messages,
                tz_offset: self.config.tz_offset,
                plan_then_execute: false,
            });
        };
        let nickname = memory.assistant_nickname().map(|s| s.to_string());
        let user_identity = if let Some(ref nick) = nickname {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::new()
        };
        std::sync::Arc::new(crate::tools::DelegateRuntime {
            irs_tool_index: self.config.i_rs_tool_index.clone(),
            mcp_registry: {
                let Ok(mcp) = self.agent_store.mcp_registry_for(user_id, agent_id) else {
                    tracing::error!(agent_id, "agent runtime MCP lookup failed");
                    return std::sync::Arc::new(crate::tools::DelegateRuntime {
                        irs_tool_index: self.config.i_rs_tool_index.clone(),
                        mcp_registry: Default::default(),
                        skills: Vec::new(),
                        tool_frequency: Default::default(),
                        parent_tx,
                        stats_manager: self.stats_manager.clone(),
                        user_identity: String::new(),
                        user_memory: String::new(),
                        user_profile: String::new(),
                        recent_messages,
                        tz_offset: self.config.tz_offset,
                        plan_then_execute: false,
                    });
                };
                mcp.clone()
            },
            skills: {
                let Ok(ss) = self.agent_store.skill_store_for(user_id, agent_id) else {
                    tracing::error!(agent_id, "agent runtime skill store lookup failed");
                    return std::sync::Arc::new(crate::tools::DelegateRuntime {
                        irs_tool_index: self.config.i_rs_tool_index.clone(),
                        mcp_registry: Default::default(),
                        skills: Vec::new(),
                        tool_frequency: Default::default(),
                        parent_tx,
                        stats_manager: self.stats_manager.clone(),
                        user_identity: String::new(),
                        user_memory: String::new(),
                        user_profile: String::new(),
                        recent_messages,
                        tz_offset: self.config.tz_offset,
                        plan_then_execute: false,
                    });
                };
                ss.executable_skills()
            },
            tool_frequency: memory.tool_frequency().clone(),
            parent_tx,
            stats_manager: self.stats_manager.clone(),
            user_identity,
            user_memory: memory.format_user_memory(),
            user_profile: memory.format_user_profile(),
            recent_messages,
            tz_offset: self.config.tz_offset,
            plan_then_execute: self.config.agent_config(agent_id).execution_mode
                == crate::config::ExecutionMode::PlanThenExecute,
        })
    }

    /// Async version of [`build_delegate_runtime`].
    #[cfg(feature = "dashboard")]
    async fn build_delegate_runtime_async(
        &self,
        user_id: &str,
        agent_id: &str,
        parent_tx: mpsc::UnboundedSender<LlmEvent>,
        recent_messages: Vec<serde_json::Value>,
    ) -> std::sync::Arc<crate::tools::DelegateRuntime> {
        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return std::sync::Arc::new(crate::tools::DelegateRuntime {
                irs_tool_index: self.config.i_rs_tool_index.clone(),
                mcp_registry: Default::default(),
                skills: Vec::new(),
                tool_frequency: Default::default(),
                parent_tx,
                stats_manager: self.stats_manager.clone(),
                user_identity: String::new(),
                user_memory: String::new(),
                user_profile: String::new(),
                recent_messages,
                tz_offset: self.config.tz_offset,
                plan_then_execute: false,
            });
        };
        let nickname = memory.assistant_nickname().map(|s| s.to_string());
        let user_identity = if let Some(ref nick) = nickname {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::new()
        };
        let mcp_registry = {
            let Ok(mcp) = self.agent_store.mcp_registry_for(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime MCP lookup failed");
                return std::sync::Arc::new(crate::tools::DelegateRuntime {
                    irs_tool_index: self.config.i_rs_tool_index.clone(),
                    mcp_registry: Default::default(),
                    skills: Vec::new(),
                    tool_frequency: Default::default(),
                    parent_tx,
                    stats_manager: self.stats_manager.clone(),
                    user_identity: String::new(),
                    user_memory: String::new(),
                    user_profile: String::new(),
                    recent_messages,
                    tz_offset: self.config.tz_offset,
                    plan_then_execute: false,
                });
            };
            mcp.clone()
        };
        let skills = {
            let Ok(ss) = self.agent_store.skill_store_for(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime skill store lookup failed");
                return std::sync::Arc::new(crate::tools::DelegateRuntime {
                    irs_tool_index: self.config.i_rs_tool_index.clone(),
                    mcp_registry: Default::default(),
                    skills: Vec::new(),
                    tool_frequency: Default::default(),
                    parent_tx,
                    stats_manager: self.stats_manager.clone(),
                    user_identity: String::new(),
                    user_memory: String::new(),
                    user_profile: String::new(),
                    recent_messages,
                    tz_offset: self.config.tz_offset,
                    plan_then_execute: false,
                });
            };
            ss.executable_skills_async().await
        };
        std::sync::Arc::new(crate::tools::DelegateRuntime {
            irs_tool_index: self.config.i_rs_tool_index.clone(),
            mcp_registry,
            skills,
            tool_frequency: memory.tool_frequency().clone(),
            parent_tx,
            stats_manager: self.stats_manager.clone(),
            user_identity,
            user_memory: memory.format_user_memory(),
            user_profile: memory.format_user_profile(),
            recent_messages,
            tz_offset: self.config.tz_offset,
            plan_then_execute: self.config.agent_config(agent_id).execution_mode
                == crate::config::ExecutionMode::PlanThenExecute,
        })
    }

    /// Build the tool index string for system prompt from discovered i-rs tools.
    pub fn build_irs_tool_index(&self, resolved: &crate::config::ResolvedAgentConfig) -> String {
        if self.config.i_rs_tool_index.is_empty() {
            return String::new();
        }

        let enabled = &resolved.enabled_tools;
        if enabled.is_empty() {
            return self.tool_index_cache.clone();
        }

        let mut result = String::from("## i-rs 工具索引\n\n");
        for name in &self.config.i_rs_tools {
            if !enabled.contains(name) {
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
        let Ok(memory) = self.agent_store.memory_for("default", agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return;
        };
        let resolved = self.config.agent_config(agent_id);
        let ctx_mgr = context::ContextManager::for_model(&resolved.model);
        ctx_mgr.compress(msgs, memory.tool_frequency());
    }

    /// Run heuristic evaluation on the completed conversation and persist it.
    /// Examines ToolCall results and the final assistant response.
    /// Returns the Quality message for TUI rendering (None if no evaluation was done).
    pub fn evaluate_completed_session(&mut self, session_id: &str) -> Option<crate::app::Message> {
        let messages = self.session_mgr.load_app_messages(session_id, 100);
        let tool_results = current_turn_tool_results(&messages);
        let last_assistant = messages.iter().rev().find_map(|m| match m {
            crate::app::Message::Assistant { text, .. } if !text.is_empty() => Some(text.as_str()),
            _ => None,
        })?;
        if last_assistant.is_empty() {
            return None;
        }

        let quality = {
            let i_rs_tools: Vec<&str> = self.config.i_rs_tools.iter().map(|s| s.as_str()).collect();
            crate::app::evaluate_response_heuristic(last_assistant, &tool_results, &i_rs_tools)
        };
        {
            let suite = crate::core::evals::builtin_eval_suite();
            let eval_tool_results: Vec<(String, String)> = messages
                .iter()
                .filter_map(|m| match m {
                    crate::app::Message::ToolCall { name, result, .. } => {
                        Some((name.clone(), result.clone()))
                    }
                    _ => None,
                })
                .collect();
            if !eval_tool_results.is_empty() {
                let eval_results = suite.evaluate(&eval_tool_results);
                let summary = suite.summary(&eval_results);
                tracing::info!(
                    suite = %suite.name,
                    passed = summary.passed,
                    total = summary.total_cases,
                    avg_score = summary.avg_score,
                    "EvalSuite 自动评估完成"
                );
            }
        }
        Some(quality)
    }

    /// Async version of [`evaluate_completed_session`].
    #[cfg(feature = "dashboard")]
    pub async fn evaluate_completed_session_async(
        &mut self,
        session_id: &str,
    ) -> Option<crate::app::Message> {
        let messages = self
            .session_mgr
            .load_app_messages_async(session_id, 100)
            .await;
        let tool_results = current_turn_tool_results(&messages);
        let last_assistant = messages.iter().rev().find_map(|m| match m {
            crate::app::Message::Assistant { text, .. } if !text.is_empty() => Some(text.as_str()),
            _ => None,
        })?;
        if last_assistant.is_empty() {
            return None;
        }

        let quality = {
            let i_rs_tools: Vec<&str> = self.config.i_rs_tools.iter().map(|s| s.as_str()).collect();
            crate::app::evaluate_response_heuristic(last_assistant, &tool_results, &i_rs_tools)
        };
        {
            let suite = crate::core::evals::builtin_eval_suite();
            let eval_tool_results: Vec<(String, String)> = messages
                .iter()
                .filter_map(|m| match m {
                    crate::app::Message::ToolCall { name, result, .. } => {
                        Some((name.clone(), result.clone()))
                    }
                    _ => None,
                })
                .collect();
            if !eval_tool_results.is_empty() {
                let eval_results = suite.evaluate(&eval_tool_results);
                let summary = suite.summary(&eval_results);
                tracing::info!(
                    suite = %suite.name,
                    passed = summary.passed,
                    total = summary.total_cases,
                    avg_score = summary.avg_score,
                    "EvalSuite 自动评估完成"
                );
            }
        }
        Some(quality)
    }

    /// Run LLM-as-Judge evaluation when enabled and conditions are met.
    /// Returns None if the judge is disabled or skipped.
    #[allow(dead_code)]
    pub async fn evaluate_with_judge(
        &self,
        session_id: &str,
        heuristic_quality: &crate::app::Message,
    ) -> Option<crate::tools::quality_judge::QualityJudgeResult> {
        if !self.config.quality_judge.enabled {
            return None;
        }

        if self.config.quality_judge.on_issues_only {
            let has_issues = match heuristic_quality {
                crate::app::Message::Quality { issues, .. } => !issues.is_empty(),
                _ => true,
            };
            if !has_issues {
                return None;
            }
        }

        let messages = self
            .session_mgr
            .load_app_messages_async(session_id, 100)
            .await;

        let user_query = messages.iter().rev().find_map(|m| match m {
            crate::app::Message::User { text } if !text.is_empty() => Some(text.clone()),
            _ => None,
        })?;

        let last_assistant = messages.iter().rev().find_map(|m| match m {
            crate::app::Message::Assistant { text, .. } if !text.is_empty() => Some(text.clone()),
            _ => None,
        })?;

        let tool_results: Vec<(String, String)> = messages
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::ToolCall { name, result, .. } => {
                    Some((name.clone(), result.clone()))
                }
                _ => None,
            })
            .collect();

        if tool_results.is_empty() {
            return None;
        }

        let resolved = self.config.agent_config("default");
        let judge_model = self
            .config
            .quality_judge
            .model
            .as_deref()
            .unwrap_or(&resolved.model);

        crate::tools::quality_judge::judge_quality(
            &self.http_client,
            &resolved.base_url,
            &resolved.api_key,
            judge_model,
            &crate::tools::quality_judge::QualityJudgeRequest {
                user_query,
                tool_results,
                final_response: last_assistant,
            },
        )
        .await
        .ok()
    }

    /// Build API messages from `Message` enum records loaded via `MessageLog`.
    /// Used by Dashboard which doesn't maintain an in-memory App message list.
    #[cfg(feature = "dashboard")]
    pub fn build_messages_from_log(
        &self,
        messages: &[crate::app::Message],
        agent_id: &str,
        user_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let tool_index = self.build_irs_tool_index(&resolved);
        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return Vec::new();
        };

        let nickname = memory.assistant_nickname().map(|s| s.to_string());
        let identity = if let Some(ref nick) = nickname {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::from(
                "用户尚未给你起昵称。如果在对话中用户突然以某个名字称呼你，询问这是否是给你的新名字。",
            )
        };

        let hot_tools = {
            let Ok(cache) = self.agent_store.tool_cache_for(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                return Vec::new();
            };
            cache.format_hot_tools(&memory.tool_frequency().keys().cloned().collect::<Vec<_>>())
        };
        let skills_fmt = match self.agent_store.skill_store_for(user_id, agent_id) {
            Ok(s) => s.format_skills(),
            Err(e) => {
                tracing::error!(%e, agent_id, "skill store lookup failed");
                return Vec::new();
            }
        };

        let system_prompt = resolved.system_prompt.clone().unwrap_or_else(|| {
            engine::builder::build_system_prompt(
                &tool_index,
                &hot_tools,
                &skills_fmt,
                &memory.format_user_memory(),
                &memory.format_user_profile(),
                self.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute,
                self.config.tz_offset,
                &identity,
                "",
            )
        });

        let mut msgs = vec![serde_json::json!({ "role": "system", "content": system_prompt })];

        let mut tool_call_counter: u32 = 0;

        for msg in messages {
            match msg {
                crate::app::Message::User { text } => {
                    msgs.push(serde_json::json!({ "role": "user", "content": text }));
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    msgs.push(serde_json::json!({ "role": "assistant", "content": text }));
                }
                crate::app::Message::ToolCall {
                    name, args, result, ..
                } => {
                    tool_call_counter += 1;
                    let call_id = format!("call_{}_{}", name, tool_call_counter);
                    msgs.push(serde_json::json!({
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args
                            }
                        }]
                    }));
                    msgs.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "content": result
                    }));
                }
                _ => {}
            }
        }

        msgs
    }

    /// Async version of [`build_messages_from_log`].
    #[cfg(feature = "dashboard")]
    pub async fn build_messages_from_log_async(
        &self,
        messages: &[crate::app::Message],
        agent_id: &str,
        user_id: &str,
    ) -> Vec<Value> {
        let resolved = self.config.agent_config(agent_id);
        let tool_index = self.build_irs_tool_index(&resolved);
        let Ok(memory) = self.agent_store.memory_for(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return Vec::new();
        };

        let nickname = memory.assistant_nickname().map(|s| s.to_string());
        let identity = if let Some(ref nick) = nickname {
            format!("用户称呼你为{}，以这个身份与用户对话。", nick)
        } else {
            String::from(
                "用户尚未给你起昵称。如果在对话中用户突然以某个名字称呼你，询问这是否是给你的新名字。",
            )
        };

        let hot_tools = {
            let Ok(cache) = self.agent_store.tool_cache_for(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                return Vec::new();
            };
            cache.format_hot_tools(&memory.tool_frequency().keys().cloned().collect::<Vec<_>>())
        };
        let skills_fmt = match self.agent_store.skill_store_for(user_id, agent_id) {
            Ok(s) => s.format_skills_async().await,
            Err(e) => {
                tracing::error!(%e, agent_id, "skill store lookup failed");
                return Vec::new();
            }
        };

        let system_prompt = resolved.system_prompt.clone().unwrap_or_else(|| {
            engine::builder::build_system_prompt(
                &tool_index,
                &hot_tools,
                &skills_fmt,
                &memory.format_user_memory(),
                &memory.format_user_profile(),
                self.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute,
                self.config.tz_offset,
                &identity,
                "",
            )
        });

        let mut msgs = vec![serde_json::json!({ "role": "system", "content": system_prompt })];

        let mut tool_call_counter: u32 = 0;

        for msg in messages {
            match msg {
                crate::app::Message::User { text } => {
                    msgs.push(serde_json::json!({ "role": "user", "content": text }));
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    msgs.push(serde_json::json!({ "role": "assistant", "content": text }));
                }
                crate::app::Message::ToolCall {
                    name, args, result, ..
                } => {
                    tool_call_counter += 1;
                    let call_id = format!("call_{}_{}", name, tool_call_counter);
                    msgs.push(serde_json::json!({
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args
                            }
                        }]
                    }));
                    msgs.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "content": result
                    }));
                }
                _ => {}
            }
        }

        msgs
    }

    /// Spawn the LLM chat loop from an async context (no Runtime reference needed).
    /// Uses `tokio::spawn` from the current tokio runtime.
    ///
    /// This is the async version — it uses async-compatible methods throughout
    /// to avoid `sync_block_on` warnings when called from within a tokio runtime.
    #[cfg(feature = "dashboard")]
    pub async fn spawn_chat_for_async(
        &self,
        llm_tx: mpsc::UnboundedSender<LlmEvent>,
        messages: Vec<Value>,
        agent_id: &str,
        recent_messages: &[Value],
        user_id: &str,
    ) {
        let (provider, agent_config, mcp, skills, tool_frequency, http_client) =
            match self.prepare_chat_loop_async(user_id, agent_id).await {
                Ok(p) => p,
                Err(e) => {
                    let _ = llm_tx.send(LlmEvent::Error(format!(
                        "Failed to prepare chat loop: {}",
                        e
                    )));
                    return;
                }
            };
        let delegate_rt = self
            .build_delegate_runtime_async(
                user_id,
                agent_id,
                llm_tx.clone(),
                recent_messages.to_vec(),
            )
            .await;
        let checkpoint_store = self.checkpoint_store.clone();
        let user_id = user_id.to_string();
        tokio::spawn(async move {
            engine::chat_loop(
                provider,
                agent_config,
                messages,
                llm_tx,
                mcp,
                skills,
                tool_frequency,
                http_client,
                Some(delegate_rt),
                None,
                checkpoint_store,
                user_id,
            )
            .await;
        });
    }

    /// Get the base directory for claw data.
    #[allow(dead_code)]
    pub fn claw_dir(&self) -> anyhow::Result<std::path::PathBuf> {
        crate::utils::claw_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))
    }
}

fn build_full_tool_index(config: &Config) -> String {
    if config.i_rs_tool_index.is_empty() {
        return String::new();
    }
    let mut result = String::from("## i-rs 工具索引\n\n");
    for name in &config.i_rs_tools {
        if let Some(desc) = config.i_rs_tool_index.get(name) {
            if !desc.is_empty() {
                result.push_str(&format!("- {}: {}\n", name, desc));
            } else {
                result.push_str(&format!("- {}\n", name));
            }
        }
    }
    result
}

/// Only collect tool results from the most recent conversation turn
/// (after the last User message), avoiding re-evaluation of stale tool
/// calls from earlier turns in the same session.
fn current_turn_tool_results(messages: &[crate::app::Message]) -> Vec<(&str, bool)> {
    let cutoff = messages
        .iter()
        .rposition(|m| matches!(m, crate::app::Message::User { .. }));
    messages
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            if cutoff.is_none_or(|c| i <= c) {
                return None;
            }
            match m {
                crate::app::Message::ToolCall { name, result, .. } => {
                    let cat = crate::error::category_from_result(result);
                    Some((name.as_str(), !cat.is_retryable_or_fatal()))
                }
                _ => None,
            }
        })
        .collect()
}

/// Central side-effect handler for tool execution results.
///
/// Called after every tool execution (TUI, Dashboard, Gateway). Handles:
/// - **User memory**: persists name/preferences/info from `update_user_memory`
/// - **Tool tracking**: records i-rs usage and general tool frequency for hot-tool analysis
///
/// All execution paths MUST call this to ensure consistent persistence.
pub fn record_tool_memory(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    i_rs_tool_index: &HashMap<String, String>,
    agent_id: &str,
    name: &str,
    args: &str,
    result: &str,
) {
    if name == "update_user_memory" {
        persist_user_memory(user_id, agent_store, agent_id, args);
    }

    if name == "i_rs" {
        track_i_rs_usage(
            user_id,
            agent_store,
            i_rs_tool_index,
            agent_id,
            args,
            result,
        );
    } else if i_rs_tool_index.contains_key(name) || name.starts_with("skill_") {
        let Ok(mem) = agent_store.memory_for_mut(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return;
        };
        mem.record_tool_use(name);
    }
}

/// Async version of [`record_tool_memory`].
///
/// Avoids `sync_block_on` when persisting hot docs to a storage backend.
#[cfg(feature = "dashboard")]
pub async fn record_tool_memory_async(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    i_rs_tool_index: &HashMap<String, String>,
    agent_id: &str,
    name: &str,
    args: &str,
    result: &str,
) {
    if name == "update_user_memory" {
        persist_user_memory(user_id, agent_store, agent_id, args);
    }

    if name == "i_rs" {
        track_i_rs_usage_async(
            user_id,
            agent_store,
            i_rs_tool_index,
            agent_id,
            args,
            result,
        )
        .await;
    } else if i_rs_tool_index.contains_key(name) || name.starts_with("skill_") {
        let Ok(mem) = agent_store.memory_for_mut(user_id, agent_id) else {
            tracing::error!(agent_id, "agent runtime memory lookup failed");
            return;
        };
        mem.record_tool_use(name);
    }
}

fn persist_user_memory(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    agent_id: &str,
    args: &str,
) {
    let Ok(mem) = agent_store.memory_for_mut(user_id, agent_id) else {
        tracing::error!(agent_id, "agent runtime memory lookup failed");
        return;
    };
    if let Ok(parsed) = serde_json::from_str::<Value>(args) {
        if let Some(user_name) = parsed
            .get("user_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            mem.set_user_name(user_name);
        }
        if let Some(info) = parsed.get("user_info").and_then(|v| v.as_array()) {
            for item in info {
                if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                    mem.add_user_info(s);
                }
            }
        }
        if let Some(prefs) = parsed.get("preferences").and_then(|v| v.as_array()) {
            for item in prefs {
                if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                    mem.add_preference(s);
                }
            }
        }
        if let Some(nick) = parsed
            .get("assistant_nickname")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            mem.set_assistant_nickname(nick);
        }
    }
}

fn track_i_rs_usage(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    i_rs_tool_index: &HashMap<String, String>,
    agent_id: &str,
    args: &str,
    result: &str,
) {
    if let Ok(parsed) = serde_json::from_str::<Value>(args)
        && let Some(tool) = parsed.get("tool").and_then(|t| t.as_str())
    {
        if i_rs_tool_index.contains_key(tool) {
            let Ok(mem) = agent_store.memory_for_mut(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime memory lookup failed");
                return;
            };
            mem.record_tool_use(tool);
        }
        let cmd = parsed.get("command").and_then(|c| c.as_str());
        if cmd == Some("skill")
            && parsed
                .get("args")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().any(|v| v.as_str() == Some("teach")))
                .unwrap_or(false)
        {
            let Ok(cache) = agent_store.tool_cache_for_mut(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                return;
            };
            cache.hot_docs.insert(tool.to_string(), result.to_string());
            cache.save_hot_docs();
        }
    }
}

/// Async version of [`track_i_rs_usage`].
#[cfg(feature = "dashboard")]
async fn track_i_rs_usage_async(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    i_rs_tool_index: &HashMap<String, String>,
    agent_id: &str,
    args: &str,
    result: &str,
) {
    if let Ok(parsed) = serde_json::from_str::<Value>(args)
        && let Some(tool) = parsed.get("tool").and_then(|t| t.as_str())
    {
        if i_rs_tool_index.contains_key(tool) {
            let Ok(mem) = agent_store.memory_for_mut(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime memory lookup failed");
                return;
            };
            mem.record_tool_use(tool);
        }
        let cmd = parsed.get("command").and_then(|c| c.as_str());
        if cmd == Some("skill")
            && parsed
                .get("args")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().any(|v| v.as_str() == Some("teach")))
                .unwrap_or(false)
        {
            let Ok(cache) = agent_store.tool_cache_for_mut(user_id, agent_id) else {
                tracing::error!(agent_id, "agent runtime tool cache lookup failed");
                return;
            };
            cache.hot_docs.insert(tool.to_string(), result.to_string());
            cache.save_hot_docs_async().await;
        }
    }
}

pub fn record_layered_tool_memory(
    user_id: &str,
    agent_store: &mut AgentRuntimeStore,
    agent_id: &str,
    name: &str,
    result: &str,
) {
    let Ok(layered) = agent_store.layered_memory_for_mut(user_id, agent_id) else {
        tracing::error!(agent_id, "agent runtime layered memory lookup failed");
        return;
    };
    layered.record_tool_result(name, result);
}

/// Bridge sync → async for storage initialization.
#[cfg(any(
    feature = "sqlite",
    feature = "mysql",
    feature = "postgres",
    feature = "mongo",
    feature = "redis"
))]
fn block_on<F: std::future::Future + Send>(f: F) -> F::Output
where
    F::Output: Send,
{
    crate::utils::sync_block_on(f)
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
        let default_memory = store
            .memory_for("default", "default")
            .expect("test setup invariant");
        let default_skills = store
            .skill_store_for("default", "default")
            .expect("test setup invariant");
        let _ = default_memory;
        let _ = default_skills;
    }

    #[test]
    fn test_build_messages_for_default() {
        let (_config, core) = crate::test_helpers::test_core();
        let msgs = core.build_messages(&[], "hello", &None, None, "default");
        assert!(msgs.len() >= 2, "至少应有 system + user 消息");
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs.last().unwrap()["role"], "user");
        assert_eq!(msgs.last().unwrap()["content"], "hello");
    }

    #[test]
    fn test_build_messages_for_agent() {
        let (_config, core) = crate::test_helpers::test_core();
        let msgs = core.build_messages_for(&[], "test", &None, None, "default", "default");
        assert!(msgs.len() >= 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs.last().unwrap()["role"], "user");
    }

    #[test]
    fn test_spawn_chat_for() {
        // 在独立线程创建 tokio runtime 避免嵌套
        let handle = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (_config, core) = crate::test_helpers::test_core();
            let (tx, _rx) = mpsc::unbounded_channel();
            let messages = vec![json!({"role": "user", "content": "hi"})];
            // spawn_chat 不应 panic
            core.spawn_chat(&rt, tx, messages, "default");
            std::thread::sleep(std::time::Duration::from_millis(50));
        });

        match handle.join() {
            Ok(()) => {}
            Err(payload) => {
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "(unknown panic payload)".to_string()
                };
                panic!("spawn_chat_for test thread panicked: {}", msg);
            }
        }
    }
}
