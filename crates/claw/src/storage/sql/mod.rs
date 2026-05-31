//! SQLite storage backend via `sqlx`.
//!
//! Implements all repository traits using a single SQLite database file.
//! Schema is created automatically on first connection.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::*;

// ── SqlBackend ──

/// Single SQLite-backed storage.  One instance implements every repository trait.
///
/// Internally wraps an `sqlx::SqlitePool`.  Schema is auto-created on construction.
#[derive(Clone)]
pub struct SqlBackend {
    pool: sqlx::SqlitePool,
}

impl SqlBackend {
    /// Create a new SQLite backend, creating/opening the database at `path`
    /// and running all migrations.
    pub async fn new(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = sqlx::SqlitePool::connect(&url).await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    /// Create an in-memory SQLite backend (for tests).
    #[allow(dead_code)]
    pub async fn new_in_memory() -> anyhow::Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    /// Bundle into a `ClawStorage` for injection.
    pub fn into_storage(self) -> ClawStorage {
        let arc = Arc::new(self);
        ClawStorage {
            sessions: Box::new(SqlSessionStore { db: arc.clone() }),
            messages: Box::new(SqlMessageStore { db: arc.clone() }),
            api_cache: Box::new(SqlApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(SqlPlanStepsStore { db: arc.clone() }),
            memory: Box::new(SqlMemoryStore { db: arc.clone() }),
            stats: Box::new(SqlStatsStore { db: arc.clone() }),
            skills: Box::new(SqlSkillStore { db: arc.clone() }),
            tool_cache: Box::new(SqlToolCacheStore { db: arc }),
        }
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                agent_id    TEXT NOT NULL DEFAULT 'default',
                state       TEXT NOT NULL DEFAULT 'Active',
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL,
                message_count INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                type        TEXT NOT NULL,
                text        TEXT NOT NULL DEFAULT '',
                name        TEXT,
                args        TEXT,
                result      TEXT,
                reasoning   TEXT,
                extra       TEXT,
                created_at  INTEGER NOT NULL DEFAULT (unixepoch())
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_cache (
                session_id  TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
                messages    TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS plan_steps (
                session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                step_order  INTEGER NOT NULL,
                description TEXT NOT NULL,
                done        INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, step_order)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory (
                agent_id    TEXT PRIMARY KEY,
                data        TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS token_records (
                id              TEXT PRIMARY KEY,
                timestamp       INTEGER NOT NULL,
                agent_id        TEXT NOT NULL,
                model           TEXT NOT NULL,
                provider        TEXT NOT NULL,
                prompt_tokens   INTEGER NOT NULL,
                completion_tokens INTEGER NOT NULL,
                total_tokens    INTEGER NOT NULL,
                has_tool_calls  INTEGER NOT NULL,
                tool_call_count INTEGER NOT NULL,
                react_rounds    INTEGER NOT NULL,
                success         INTEGER NOT NULL,
                latency_ms      INTEGER NOT NULL,
                estimated_cost_usd REAL NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_token_ts ON token_records(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS skills (
                agent_id    TEXT NOT NULL,
                name        TEXT NOT NULL,
                content     TEXT NOT NULL,
                parameters  TEXT,
                PRIMARY KEY (agent_id, name)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tool_cache (
                agent_id    TEXT NOT NULL,
                tool_name   TEXT NOT NULL,
                doc         TEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            )",
        )
        .execute(&self.pool)
        .await?;

        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Trait implementations — each delegated to a thin wrapper struct
//  to avoid method-name collisions.
// ═══════════════════════════════════════════════════════════════════

// ── SessionRepo ──

#[derive(Clone)]
struct SqlSessionStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl SessionRepo for SqlSessionStore {
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, title, agent_id, state, created_at, updated_at, message_count FROM sessions ORDER BY updated_at DESC",
        )
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
        let mut tx = self.db.pool.begin().await?;
        sqlx::query("DELETE FROM sessions").execute(&mut *tx).await?;
        for s in sessions {
            sqlx::query(
                "INSERT INTO sessions (id, title, agent_id, state, created_at, updated_at, message_count) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&s.id)
            .bind(&s.title)
            .bind(&s.agent_id)
            .bind(serde_json::to_string(&s.state).unwrap_or_default())
            .bind(s.created_at)
            .bind(s.updated_at)
            .bind(s.message_count as i64)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    title: String,
    agent_id: String,
    state: String,
    created_at: i64,
    updated_at: i64,
    message_count: i64,
}

impl From<SessionRow> for crate::session::SessionMeta {
    fn from(r: SessionRow) -> Self {
        Self {
            id: r.id,
            title: r.title,
            agent_id: r.agent_id,
            state: serde_json::from_str(&r.state).unwrap_or_default(),
            created_at: r.created_at,
            updated_at: r.updated_at,
            message_count: r.message_count as usize,
        }
    }
}

// ── MessageRepo ──

#[derive(Clone)]
struct SqlMessageStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl MessageRepo for SqlMessageStore {
    async fn append(&self, session_id: &str, entry: &serde_json::Value) -> anyhow::Result<()> {
        let msg_type = entry["type"].as_str().unwrap_or("");
        let text = entry["text"].as_str().unwrap_or("");
        let name = entry["name"].as_str();
        let args = entry["args"].as_str();
        let result = entry["result"].as_str();
        let reasoning = entry["reasoning"].as_str();
        let extra = serde_json::to_string(entry)?;

        sqlx::query(
            "INSERT INTO messages (session_id, type, text, name, args, result, reasoning, extra) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(session_id)
        .bind(msg_type)
        .bind(text)
        .bind(name)
        .bind(args)
        .bind(result)
        .bind(reasoning)
        .bind(&extra)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str, limit: usize) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows: Vec<MessageRow> = sqlx::query_as(
            "SELECT type, text, name, args, result, reasoning, extra FROM messages WHERE session_id = ? ORDER BY id DESC LIMIT ?",
        )
        .bind(session_id)
        .bind(limit as i64)
        .fetch_all(&self.db.pool)
        .await?;

        // Results come in DESC order; reverse to chronological
        let values: Vec<serde_json::Value> = rows
            .into_iter()
            .rev()
            .filter_map(|r| serde_json::from_str(&r.extra).ok())
            .collect();
        Ok(values)
    }

    async fn save_all(&self, session_id: &str, records: &[serde_json::Value]) -> anyhow::Result<()> {
        let mut tx = self.db.pool.begin().await?;
        sqlx::query("DELETE FROM messages WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut *tx)
            .await?;
        for entry in records {
            let msg_type = entry["type"].as_str().unwrap_or("");
            let text = entry["text"].as_str().unwrap_or("");
            let name = entry["name"].as_str();
            let args = entry["args"].as_str();
            let result = entry["result"].as_str();
            let reasoning = entry["reasoning"].as_str();
            let extra = serde_json::to_string(entry)?;
            sqlx::query(
                "INSERT INTO messages (session_id, type, text, name, args, result, reasoning, extra) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(session_id)
            .bind(msg_type)
            .bind(text)
            .bind(name)
            .bind(args)
            .bind(result)
            .bind(reasoning)
            .bind(&extra)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        if query_lower.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Load session metadata first
        let sessions: Vec<crate::session::SessionMeta> = {
            let rows = sqlx::query_as::<_, SessionRow>(
                "SELECT id, title, agent_id, state, created_at, updated_at, message_count FROM sessions",
            )
            .fetch_all(&self.db.pool)
            .await?;
            rows.into_iter().map(|r| r.into()).collect()
        };

        let mut results: Vec<SearchResult> = Vec::new();

        for meta in &sessions {
            let rows: Vec<MessageRow> = sqlx::query_as(
                "SELECT type, text, name, args, result, reasoning, extra FROM messages WHERE session_id = ? ORDER BY id",
            )
            .bind(&meta.id)
            .fetch_all(&self.db.pool)
            .await?;

            for (i, row) in rows.iter().enumerate() {
                let msg_type = &row.r#type;
                let searchable = match msg_type.as_str() {
                    "user" | "assistant" | "error" => row.text.to_lowercase(),
                    "tool_call" => format!("[tool: {}]", row.name).to_lowercase(),
                    _ => continue,
                };
                if !searchable.contains(&query_lower) {
                    continue;
                }

                let excerpt = match msg_type.as_str() {
                    "tool_call" => format!("[工具调用: {}]", row.name),
                    _ => {
                        let t: String = row.text.chars().take(200).collect();
                        if row.text.len() > 200 {
                            format!("{}...", t)
                        } else {
                            t
                        }
                    }
                };

                let context_before: Vec<String> = rows[i.saturating_sub(2)..i]
                    .iter()
                    .filter_map(|m| {
                        let t: String = m.text.chars().take(100).collect();
                        Some(t)
                    })
                    .collect();

                let context_after: Vec<String> = if i + 1 < rows.len() {
                    let end = std::cmp::min(i + 1, rows.len() - 1);
                    rows[i + 1..=end]
                        .iter()
                        .filter_map(|m| {
                            let t: String = m.text.chars().take(100).collect();
                            Some(t)
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                results.push(SearchResult {
                    session_id: meta.id.clone(),
                    session_title: meta.title.clone(),
                    message_type: msg_type.clone(),
                    excerpt,
                    context_before,
                    context_after,
                    updated_at: meta.updated_at,
                });

                if results.len() >= max_results {
                    return Ok(results);
                }
            }
        }

        Ok(results)
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM messages WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        // Also clean up api_cache, plan_steps
        sqlx::query("DELETE FROM api_cache WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        sqlx::query("DELETE FROM plan_steps WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    #[allow(dead_code)]
    r#type: String,
    text: String,
    name: String,
    #[allow(dead_code)]
    args: String,
    #[allow(dead_code)]
    result: String,
    #[allow(dead_code)]
    reasoning: String,
    extra: String,
}

// ── ApiCacheRepo ──

#[derive(Clone)]
struct SqlApiCacheStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl ApiCacheRepo for SqlApiCacheStore {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()> {
        let json = serde_json::to_string(messages)?;
        sqlx::query(
            "INSERT OR REPLACE INTO api_cache (session_id, messages) VALUES (?, ?)",
        )
        .bind(session_id)
        .bind(&json)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT messages FROM api_cache WHERE session_id = ?")
                .bind(session_id)
                .fetch_optional(&self.db.pool)
                .await?;
        Ok(row.and_then(|(json,)| serde_json::from_str(&json).ok()))
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM api_cache WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }
}

// ── PlanStepsRepo ──

#[derive(Clone)]
struct SqlPlanStepsStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl PlanStepsRepo for SqlPlanStepsStore {
    async fn save(
        &self,
        session_id: &str,
        steps: &[crate::app::PlanStep],
    ) -> anyhow::Result<()> {
        let mut tx = self.db.pool.begin().await?;
        sqlx::query("DELETE FROM plan_steps WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut *tx)
            .await?;
        for (i, step) in steps.iter().enumerate() {
            sqlx::query(
                "INSERT INTO plan_steps (session_id, step_order, description, done) VALUES (?, ?, ?, ?)",
            )
            .bind(session_id)
            .bind(i as i64)
            .bind(&step.description)
            .bind(step.done as i64)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<crate::app::PlanStep>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT description, done FROM plan_steps WHERE session_id = ? ORDER BY step_order",
        )
        .bind(session_id)
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(desc, done)| crate::app::PlanStep {
                description: desc,
                done: done != 0,
            })
            .collect())
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM plan_steps WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }
}

// ── MemoryRepo ──

#[derive(Clone)]
struct SqlMemoryStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl MemoryRepo for SqlMemoryStore {
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<crate::memory::CrossSessionMemory> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT data FROM memory WHERE agent_id = ?")
                .bind(agent_id)
                .fetch_optional(&self.db.pool)
                .await?;
        match row {
            Some((json,)) => Ok(serde_json::from_str(&json)?),
            None => Ok(crate::memory::CrossSessionMemory::default_memory()),
        }
    }

    async fn save(
        &self,
        agent_id: &str,
        memory: &crate::memory::CrossSessionMemory,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string(memory)?;
        sqlx::query(
            "INSERT OR REPLACE INTO memory (agent_id, data) VALUES (?, ?)",
        )
        .bind(agent_id)
        .bind(&json)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}

// ── StatsRepo ──

#[derive(Clone)]
struct SqlStatsStore {
    db: Arc<SqlBackend>,
}

#[derive(sqlx::FromRow)]
struct TokenRecordRow {
    id: String,
    timestamp: i64,
    agent_id: String,
    model: String,
    provider: String,
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
    has_tool_calls: i64,
    tool_call_count: i64,
    react_rounds: i64,
    success: i64,
    latency_ms: i64,
    estimated_cost_usd: f64,
}

impl From<TokenRecordRow> for crate::stats::TokenRecord {
    fn from(r: TokenRecordRow) -> Self {
        Self {
            id: r.id,
            timestamp: r.timestamp,
            agent_id: r.agent_id,
            model: r.model,
            provider: r.provider,
            prompt_tokens: r.prompt_tokens as u32,
            completion_tokens: r.completion_tokens as u32,
            total_tokens: r.total_tokens as u32,
            has_tool_calls: r.has_tool_calls != 0,
            tool_call_count: r.tool_call_count as u32,
            react_rounds: r.react_rounds as u32,
            success: r.success != 0,
            latency_ms: r.latency_ms as u64,
            estimated_cost_usd: r.estimated_cost_usd,
        }
    }
}

#[async_trait]
impl StatsRepo for SqlStatsStore {
    async fn append_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
        for r in records {
            sqlx::query(
                "INSERT OR REPLACE INTO token_records (id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&r.id)
            .bind(r.timestamp)
            .bind(&r.agent_id)
            .bind(&r.model)
            .bind(&r.provider)
            .bind(r.prompt_tokens as i64)
            .bind(r.completion_tokens as i64)
            .bind(r.total_tokens as i64)
            .bind(r.has_tool_calls as i64)
            .bind(r.tool_call_count as i64)
            .bind(r.react_rounds as i64)
            .bind(r.success as i64)
            .bind(r.latency_ms as i64)
            .bind(r.estimated_cost_usd)
            .execute(&self.db.pool)
            .await?;
        }
        Ok(())
    }

    async fn read_range(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
        macro_rules! q {
            () => {
                "SELECT id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd FROM token_records"
            };
        }

        let rows: Vec<TokenRecordRow> = match (from, to) {
            (Some(f), Some(t)) => {
                sqlx::query_as::<_, TokenRecordRow>(
                    concat!(q!(), " WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp"),
                )
                .bind(f)
                .bind(t)
                .fetch_all(&self.db.pool)
                .await?
            }
            (Some(f), None) => {
                sqlx::query_as::<_, TokenRecordRow>(
                    concat!(q!(), " WHERE timestamp >= ? ORDER BY timestamp"),
                )
                .bind(f)
                .fetch_all(&self.db.pool)
                .await?
            }
            (None, Some(t)) => {
                sqlx::query_as::<_, TokenRecordRow>(
                    concat!(q!(), " WHERE timestamp <= ? ORDER BY timestamp"),
                )
                .bind(t)
                .fetch_all(&self.db.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, TokenRecordRow>(
                    concat!(q!(), " ORDER BY timestamp"),
                )
                .fetch_all(&self.db.pool)
                .await?
            }
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
        if keep_days == 0 {
            return Ok(0);
        }
        let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
        let result = sqlx::query("DELETE FROM token_records WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.db.pool)
            .await?;
        Ok(result.rows_affected() as usize)
    }
}

// ── SkillRepo ──

#[derive(Clone)]
struct SqlSkillStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl SkillRepo for SqlSkillStore {
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, content FROM skills WHERE agent_id = ? ORDER BY name",
        )
        .bind(agent_id)
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(name, content)| SkillEntry { name, content })
            .collect())
    }

    async fn get(
        &self,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT content, parameters FROM skills WHERE agent_id = ? AND name = ?",
        )
        .bind(agent_id)
        .bind(name)
        .fetch_optional(&self.db.pool)
        .await?;
        match row {
            Some((content, params_json)) => {
                let params = params_json.and_then(|p| serde_json::from_str(&p).ok());
                Ok(Some(crate::skill_store::SkillDefinition {
                    name: name.to_string(),
                    description: name.to_string(), // simplified
                    parameters: params,
                    content,
                }))
            }
            None => Ok(None),
        }
    }

    async fn install(&self, agent_id: &str, name: &str, content: &str) -> anyhow::Result<()> {
        // Parse frontmatter for parameters
        let (frontmatter, _body) =
            crate::skill_store::parse_frontmatter(content);
        let parameters = frontmatter
            .as_ref()
            .and_then(|t| t.get("parameters"))
            .and_then(|v| serde_json::to_string(v).ok());

        sqlx::query(
            "INSERT OR REPLACE INTO skills (agent_id, name, content, parameters) VALUES (?, ?, ?, ?)",
        )
        .bind(agent_id)
        .bind(name)
        .bind(content)
        .bind(&parameters)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM skills WHERE agent_id = ? AND name = ?")
            .bind(agent_id)
            .bind(name)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }

    async fn list_executable(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>> {
        let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT name, content, parameters FROM skills WHERE agent_id = ? AND parameters IS NOT NULL ORDER BY name",
        )
        .bind(agent_id)
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|(name, content, params_json)| {
                let params = params_json.and_then(|p| serde_json::from_str(&p).ok());
                Some(crate::skill_store::SkillDefinition {
                    name,
                    description: String::new(),
                    parameters: params,
                    content,
                })
            })
            .collect())
    }

    async fn format_skills(&self, agent_id: &str) -> anyhow::Result<String> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, content FROM skills WHERE agent_id = ? ORDER BY name",
        )
        .bind(agent_id)
        .fetch_all(&self.db.pool)
        .await?;
        if rows.is_empty() {
            return Ok(String::new());
        }
        let mut result = String::from("## 用户技能\n\n");
        result.push_str("以下是用户定义的自定义技能指令，请在对话中遵循这些指导：\n");
        for (name, content) in &rows {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (frontmatter, body) =
                crate::skill_store::parse_frontmatter(trimmed);
            let heading = frontmatter
                .as_ref()
                .and_then(|t| t.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or(name);
            let display = if body.is_empty() { trimmed } else { body };
            result.push_str(&format!("\n### {}\n{}\n", heading, display));
        }
        Ok(result)
    }
}

// ── ToolCacheRepo ──

#[derive(Clone)]
struct SqlToolCacheStore {
    db: Arc<SqlBackend>,
}

#[async_trait]
impl ToolCacheRepo for SqlToolCacheStore {
    async fn load(&self, agent_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT tool_name, doc FROM tool_cache WHERE agent_id = ?",
        )
        .bind(agent_id)
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows.into_iter().collect())
    }

    async fn save(&self, agent_id: &str, docs: &HashMap<String, String>) -> anyhow::Result<()> {
        let mut tx = self.db.pool.begin().await?;
        sqlx::query("DELETE FROM tool_cache WHERE agent_id = ?")
            .bind(agent_id)
            .execute(&mut *tx)
            .await?;
        for (tool_name, doc) in docs {
            sqlx::query(
                "INSERT INTO tool_cache (agent_id, tool_name, doc) VALUES (?, ?, ?)",
            )
            .bind(agent_id)
            .bind(tool_name)
            .bind(doc)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

// ── ClawStorage constructor ──

impl ClawStorage {
    /// Build with a SQLite backend (requires feature "sqlite").
    #[cfg(feature = "sqlite")]
    pub async fn sqlite(path: PathBuf) -> anyhow::Result<Self> {
        SqlBackend::new(path).await.map(|be| be.into_storage())
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_storage() -> ClawStorage {
        SqlBackend::new_in_memory()
            .await
            .expect("failed to create in-memory sqlite")
            .into_storage()
    }

    #[tokio::test]
    async fn test_sql_session_save_load() {
        let storage = test_storage().await;
        let sessions = vec![crate::session::SessionMeta {
            id: "s1".to_string(),
            title: "Hello".to_string(),
            agent_id: "default".to_string(),
            state: crate::session::SessionState::Active,
            created_at: 100,
            updated_at: 200,
            message_count: 0,
        }];
        storage.sessions.save_all(&sessions).await.unwrap();
        let loaded = storage.sessions.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Hello");
    }

    #[tokio::test]
    async fn test_sql_message_append_load() {
        let storage = test_storage().await;
        // Need a session first due to FK constraint
        storage.sessions.save_all(&[crate::session::SessionMeta {
            id: "s1".to_string(),
            title: "Test".to_string(),
            agent_id: "default".to_string(),
            state: crate::session::SessionState::Active,
            created_at: 100,
            updated_at: 200,
            message_count: 0,
        }]).await.unwrap();
        storage
            .messages
            .append("s1", &serde_json::json!({"type": "user", "text": "hi"}))
            .await
            .unwrap();
        storage
            .messages
            .append("s1", &serde_json::json!({"type": "assistant", "text": "hello"}))
            .await
            .unwrap();
        let loaded = storage.messages.load("s1", 10).await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0]["text"], "hi");
        assert_eq!(loaded[1]["text"], "hello");
    }

    #[tokio::test]
    async fn test_sql_memory_save_load() {
        let storage = test_storage().await;
        let mut mem = crate::memory::CrossSessionMemory::default_memory();
        mem.set_user_name("Alice");
        storage.memory.save("agent1", &mem).await.unwrap();

        let loaded = storage.memory.load("agent1").await.unwrap();
        assert!(loaded.has_user_profile());
    }

    #[tokio::test]
    async fn test_sql_stats_append_read() {
        let storage = test_storage().await;
        let record = crate::stats::TokenRecord {
            id: "r1".to_string(),
            timestamp: 1716220800,
            agent_id: "default".to_string(),
            model: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
            has_tool_calls: false,
            tool_call_count: 0,
            react_rounds: 1,
            success: true,
            latency_ms: 100,
            estimated_cost_usd: 0.001,
        };
        storage.stats.append_batch(&[record]).await.unwrap();
        let records = storage.stats.read_range(None, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "r1");
    }

    #[tokio::test]
    async fn test_sql_skill_crud() {
        let storage = test_storage().await;
        storage.skills.install("agent1", "my-skill", "test content").await.unwrap();
        let skills = storage.skills.list("agent1").await.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");

        storage.skills.remove("agent1", "my-skill").await.unwrap();
        assert!(storage.skills.list("agent1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_sql_tool_cache() {
        let storage = test_storage().await;
        let mut docs = HashMap::new();
        docs.insert("weight".to_string(), "weight doc".to_string());
        storage.tool_cache.save("agent1", &docs).await.unwrap();
        let loaded = storage.tool_cache.load("agent1").await.unwrap();
        assert_eq!(loaded.get("weight").unwrap(), "weight doc");
    }

    #[tokio::test]
    async fn test_sql_search() {
        let storage = test_storage().await;
        storage.sessions.save_all(&[crate::session::SessionMeta {
            id: "s1".to_string(),
            title: "Test".to_string(),
            agent_id: "default".to_string(),
            state: crate::session::SessionState::Active,
            created_at: 100,
            updated_at: 200,
            message_count: 0,
        }]).await.unwrap();
        storage.messages.append("s1", &serde_json::json!({"type": "user", "text": "search me"})).await.unwrap();
        let results = storage.messages.search("search", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "s1");
    }
}
