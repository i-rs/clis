//! Pluggable storage backend abstraction.

pub mod file;
#[cfg(feature = "mongo")]
pub mod mongo;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(any(feature = "sqlite", feature = "mysql", feature = "postgres"))]
pub mod sql;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── shared data types (lightweight, used in trait signatures) ──

/// A single search result from conversation history.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub session_id: String,
    pub session_title: String,
    pub message_type: String,
    pub excerpt: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    #[allow(dead_code)]
    pub updated_at: i64,
}

/// A skill entry with name and content.
#[derive(Debug, Clone, Serialize)]
pub struct SkillEntry {
    pub name: String,
    pub content: String,
}

// ── Storage configuration ──

/// Which storage backend to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    #[default]
    File,
    Sqlite,
    Mysql,
    Postgres,
    #[serde(rename = "mongodb")]
    Mongo,
    Redis,
}

/// Per-backend connection parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// Which backend to activate.
    #[serde(default)]
    pub backend: StorageBackend,
    /// Base directory for file backend (defaults to ~/.i-rs/claw).
    #[serde(default)]
    pub file_dir: Option<PathBuf>,
    /// SQLite file path.
    #[serde(default)]
    pub sqlite_path: Option<PathBuf>,
    /// MySQL / PostgreSQL connection URL.
    #[serde(default)]
    pub sql_url: Option<String>,
    /// MongoDB connection URL.
    #[serde(default)]
    pub mongo_url: Option<String>,
    /// MongoDB database name.
    #[serde(default)]
    pub mongo_database: Option<String>,
    /// Redis connection URL (e.g. `redis://127.0.0.1:6379/0`).
    #[serde(default)]
    pub redis_url: Option<String>,
}

// ── Repository traits ──

/// CRUD for session metadata (index.json).
///
/// Implementations SHOULD provide efficient single-row operations
/// (get_one, upsert, delete_one, count). The file backend falls back
/// to load_all/save_all internally.
#[async_trait]
#[allow(dead_code)]
pub trait SessionRepo: Send + Sync {
    /// Load all session metadata.
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>>;
    /// Atomically replace all session metadata.
    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()>;

    /// Get a single session by ID.
    async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>>;
    /// Upsert a single session metadata.
    async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()>;
    /// Delete a single session by ID.
    async fn delete_one(&self, id: &str) -> anyhow::Result<()>;
    /// Count sessions.
    async fn count(&self) -> anyhow::Result<usize>;
}

/// Append-only message log for per-session persistence.
///
/// Callers always work in terms of the domain `Message` enum; the
/// storage layer wraps each one in a `StoredRecord` envelope (`seq`,
/// `ts`, `schema_v`) internally.
#[async_trait]
pub trait MessageLog: Send + Sync {
    /// Append a batch of messages to the end of the session's log.
    /// Implementations must be atomic (all-or-nothing) and must assign
    /// monotonic `seq` values within the session.
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()>;

    /// Convenience: append a single message.
    #[allow(dead_code)]
    async fn append_one(
        &self,
        session_id: &str,
        message: &crate::app::Message,
    ) -> anyhow::Result<()> {
        self.append_batch(session_id, std::slice::from_ref(message))
            .await
    }

    /// Load the last `limit` messages from a session (oldest-first within
    /// the returned window). Pass `usize::MAX` for "all".
    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>>;

    /// Case-insensitive substring search across all sessions.
    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>>;

    /// Delete all messages for a session.
    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()>;

    /// Count messages for a session.
    #[allow(dead_code)]
    async fn count(&self, session_id: &str) -> anyhow::Result<usize>;
}

/// API-format message cache (one JSON blob per session).
#[async_trait]
pub trait ApiCacheRepo: Send + Sync {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>>;
    async fn delete(&self, session_id: &str) -> anyhow::Result<()>;
}

/// Plan steps per session.
#[async_trait]
pub trait PlanStepsRepo: Send + Sync {
    async fn save(&self, session_id: &str, steps: &[crate::app::PlanStep]) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<crate::app::PlanStep>>;
    async fn delete(&self, session_id: &str) -> anyhow::Result<()>;
}

/// Cross-session user memory (one JSON doc per agent).
#[async_trait]
pub trait MemoryRepo: Send + Sync {
    /// Load the full CrossSessionMemory for an agent.
    /// Returns `None` if no memory exists for this agent yet.
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Option<crate::memory::CrossSessionMemory>>;
    /// Persist the full CrossSessionMemory for an agent.
    async fn save(
        &self,
        agent_id: &str,
        memory: &crate::memory::CrossSessionMemory,
    ) -> anyhow::Result<()>;
}

/// Token usage statistics (JSONL append).
#[async_trait]
pub trait StatsRepo: Send + Sync {
    /// Idempotently upsert token records (dedup by id).
    async fn upsert_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()>;
    async fn read_range(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> anyhow::Result<Vec<crate::stats::TokenRecord>>;
    /// Remove records older than `keep_days`. Returns count removed.
    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize>;
}

/// User-defined skill files (per agent).
#[async_trait]
pub trait SkillRepo: Send + Sync {
    /// List all skill names and content for an agent.
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>>;
    /// Get a single skill definition by name.
    async fn get(
        &self,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>>;
    /// Install (create or update) a skill.
    async fn install(&self, agent_id: &str, name: &str, content: &str) -> anyhow::Result<()>;
    /// Remove a skill.
    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()>;
    /// List skills that have `parameters` (callable as tools).
    async fn list_executable(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>>;
}

/// Tool documentation cache (per agent).
#[async_trait]
pub trait ToolCacheRepo: Send + Sync {
    async fn load(&self, agent_id: &str) -> anyhow::Result<HashMap<String, String>>;
    async fn save(&self, agent_id: &str, docs: &HashMap<String, String>) -> anyhow::Result<()>;
}

/// Convenience container that bundles all repository trait objects.
///
/// Construct via `ClawStorage::file(claw_dir)` for Phase 1 (file backend),
/// or `ClawStorage::sqlite(path)` etc. in future phases.
pub struct ClawStorage {
    pub sessions: Box<dyn SessionRepo>,
    pub message_log: std::sync::Arc<dyn MessageLog>,
    pub api_cache: Box<dyn ApiCacheRepo>,
    pub plan_steps: Box<dyn PlanStepsRepo>,
    pub memory: Box<dyn MemoryRepo>,
    pub stats: Box<dyn StatsRepo>,
    pub skills: Box<dyn SkillRepo>,
    pub tool_cache: Box<dyn ToolCacheRepo>,
}

// ClawStorage::file() is implemented in file.rs alongside the concrete stores.

impl std::fmt::Debug for ClawStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClawStorage").finish_non_exhaustive()
    }
}
