//! SQL storage backends (SQLite / MySQL / PostgreSQL) via `sqlx`.
//!
//! Each dialect lives in its own sub-module, gated by a feature flag:
//! - `sql/sqlite.rs`  (feature = "sqlite")
//! - `sql/mysql.rs`   (feature = "mysql")
//! - `sql/postgres.rs` (feature = "postgres")
//!
//! A shared macro `define_sql_stores!` generates all repository trait
//! implementations for a given database pool type.
//!
//! **PostgreSQL note:** sqlx requires `&'static str` for all SQL queries.
//! The PG backend currently uses `?` placeholders which the PG protocol
//! expects as `$N`. Until a proper placeholder adaptation layer is built,
//! the PG backend is provided on a best-effort basis.
//!
//! ## Search semantics
//!
//! `MessageLog::search` performs case-insensitive substring matching across
//! sessions. It searches `payload` JSON for user/assistant/error `text` and
//! tool_call `name`. Each result includes a 200-char excerpt and up to 2
//! context messages before and 1 after the match. Results are grouped by
//! session and terminated at `max_results`.

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

use super::*;

/// Generates the store wrapper structs + their trait implementations
/// for a given database pool type.
macro_rules! define_sql_stores {
    (
        $pool:ty,
        $backend:ty,
        $sessions:ident, $messagelog:ident, $apicache:ident, $plansteps:ident,
        $memory:ident, $stats:ident, $skills:ident, $toolcache:ident,
        $upsert_session:expr,
        $upsert_apicache:expr, $upsert_memory:expr, $upsert_token:expr, $upsert_skill:expr,
    ) => {
        // ── SessionRepo ──

        #[derive(Clone)]
        struct $sessions {
            db: Arc<$backend>,
        }

        #[async_trait]
        impl SessionRepo for $sessions {
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

                let incoming_ids: HashSet<&str> =
                    sessions.iter().map(|s| s.id.as_str()).collect();

                let existing: Vec<(String,)> =
                    sqlx::query_as("SELECT id FROM sessions").fetch_all(&mut *tx).await?;

                for (id,) in &existing {
                    if !incoming_ids.contains(id.as_str()) {
                        sqlx::query("DELETE FROM sessions WHERE id = ?")
                            .bind(id)
                            .execute(&mut *tx)
                            .await?;
                    }
                }

                for s in sessions {
                    sqlx::query($upsert_session)
                        .bind(&s.id).bind(&s.title).bind(&s.agent_id)
                        .bind(serde_json::to_string(&s.state).unwrap_or_default())
                        .bind(s.created_at).bind(s.updated_at)
                        .bind(s.message_count as i64)
                        .execute(&mut *tx).await?;
                }
                tx.commit().await?;
                Ok(())
            }

            async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
                let row: Option<SessionRow> = sqlx::query_as(
                    "SELECT id, title, agent_id, state, created_at, updated_at, message_count                      FROM sessions WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(&self.db.pool)
                .await?;
                Ok(row.map(|r| r.into()))
            }

            async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
                sqlx::query($upsert_session)
                    .bind(&session.id).bind(&session.title).bind(&session.agent_id)
                    .bind(serde_json::to_string(&session.state).unwrap_or_default())
                    .bind(session.created_at).bind(session.updated_at)
                    .bind(session.message_count as i64)
                    .execute(&self.db.pool).await?;
                Ok(())
            }

            async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
                sqlx::query("DELETE FROM sessions WHERE id = ?")
                    .bind(id)
                    .execute(&self.db.pool).await?;
                Ok(())
            }

            async fn count(&self) -> anyhow::Result<usize> {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
                    .fetch_one(&self.db.pool).await?;
                Ok(count as usize)
            }
        }

        // ── MessageLog (append-only) ──

        #[derive(Clone)]
        struct $messagelog {
            db: Arc<$backend>,
        }

        #[async_trait]
        impl MessageLog for $messagelog {
            async fn append_batch(
                &self,
                session_id: &str,
                messages: &[crate::app::Message],
            ) -> anyhow::Result<()> {
                if messages.is_empty() {
                    return Ok(());
                }
                let mut tx = self.db.pool.begin().await?;
                let next_seq: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?",
                )
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await?;
                let mut seq = next_seq + 1;
                for msg in messages {
                    let rec = crate::message::StoredRecord::from_message(msg)?;
                    let payload = serde_json::to_string(&rec.payload)?;
                    sqlx::query(
                        "INSERT INTO message_log (session_id, seq, ts, schema_v, payload) \
                         VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(session_id)
                    .bind(seq)
                    .bind(rec.ts)
                    .bind(rec.schema_v as i64)
                    .bind(&payload)
                    .execute(&mut *tx)
                    .await?;
                    seq += 1;
                }
                tx.commit().await?;
                Ok(())
            }

            async fn load(
                &self,
                session_id: &str,
                limit: usize,
            ) -> anyhow::Result<Vec<crate::app::Message>> {
                // Use a safe limit value: clamp to i64 range to avoid
                // usize::MAX -> -1 integer overflow in the SQL subquery.
                let limit_i64 = if limit >= i64::MAX as usize {
                    i64::MAX
                } else {
                    limit as i64
                };
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT payload FROM ( \
                     SELECT payload FROM message_log \
                     WHERE session_id = ? \
                     ORDER BY seq DESC \
                     LIMIT ? \
                     ) sub ORDER BY seq ASC",
                )
                .bind(session_id)
                .bind(limit_i64)
                .fetch_all(&self.db.pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .filter_map(|(p,)| {
                        let value: serde_json::Value = serde_json::from_str(&p).ok()?;
                        let rec = crate::message::StoredRecord {
                            seq: 0,
                            ts: 0,
                            schema_v: 1,
                            payload: value,
                        };
                        rec.to_message()
                    })
                    .collect())
            }

            async fn search(
                &self,
                query: &str,
                max_results: usize,
            ) -> anyhow::Result<Vec<SearchResult>> {
                let q = query.trim().to_lowercase();
                if q.is_empty() {
                    return Ok(Vec::new());
                }
                let like = format!("%{}%", q);

                // Identify candidate sessions via payload substring. The
                // LIKE operates on the raw JSON text, which catches both
                // `"text":"…"`, `"name":"…"`, etc. PG would prefer
                // JSONB operators; this default is portable.
                let candidate_rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT DISTINCT session_id FROM message_log \
                     WHERE LOWER(payload) LIKE ?",
                )
                .bind(&like)
                .fetch_all(&self.db.pool)
                .await?;

                let mut results = Vec::new();
                for (sid,) in &candidate_rows {
                    let session_rows: Vec<SessionRow> = sqlx::query_as(
                        "SELECT id, title, agent_id, state, created_at, updated_at, message_count \
                         FROM sessions WHERE id = ?",
                    )
                    .bind(sid)
                    .fetch_all(&self.db.pool)
                    .await?;
                    if session_rows.is_empty() {
                        continue;
                    }
                    let meta: crate::session::SessionMeta = session_rows[0].clone().into();

                    let rows: Vec<(i64, String)> = sqlx::query_as(
                        "SELECT seq, payload FROM message_log WHERE session_id = ? ORDER BY seq",
                    )
                    .bind(sid)
                    .fetch_all(&self.db.pool)
                    .await?;
                    let records: Vec<(usize, serde_json::Value)> = rows
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, (_seq, p))| {
                            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
                            Some((idx, v))
                        })
                        .collect();

                    for (i, payload) in &records {
                        let mt = payload["type"].as_str().unwrap_or("");
                        let text = payload["text"].as_str().unwrap_or("");
                        let name = payload["name"].as_str().unwrap_or("");
                        let searchable = match mt {
                            "user" | "assistant" | "error" => text.to_lowercase(),
                            "tool_call" => name.to_lowercase(),
                            _ => continue,
                        };
                        if !searchable.contains(&q) {
                            continue;
                        }
                        let excerpt = match mt {
                            "tool_call" => format!("[工具调用: {}]", name),
                            _ => {
                                let t: String = text.chars().take(200).collect();
                                if text.len() > 200 { format!("{}...", t) } else { t }
                            }
                        };
                        let ctx_before: Vec<String> = records[i.saturating_sub(2)..*i]
                            .iter()
                            .filter_map(|(_, p)| {
                                let t = p["text"].as_str()?;
                                Some(t.chars().take(100).collect())
                            })
                            .collect();
                        let ctx_after: Vec<String> = records
                            .get(i + 1..)
                            .map(|slice| {
                                slice
                                    .iter()
                                    .take(1)
                                    .filter_map(|(_, p)| {
                                        let t = p["text"].as_str()?;
                                        Some(t.chars().take(100).collect())
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        results.push(SearchResult {
                            session_id: meta.id.clone(),
                            session_title: meta.title.clone(),
                            message_type: mt.to_string(),
                            excerpt,
                            context_before: ctx_before,
                            context_after: ctx_after,
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
                sqlx::query("DELETE FROM message_log WHERE session_id = ?")
                    .bind(session_id)
                    .execute(&self.db.pool)
                    .await?;
                Ok(())
            }

            async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
                let n: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM message_log WHERE session_id = ?",
                )
                .bind(session_id)
                .fetch_one(&self.db.pool)
                .await?;
                Ok(n as usize)
            }
        }

        // ── ApiCacheRepo ──

        #[derive(Clone)]
        struct $apicache { db: Arc<$backend> }

        #[async_trait]
        impl ApiCacheRepo for $apicache {
            async fn save(&self, sid: &str, msgs: &[serde_json::Value]) -> anyhow::Result<()> {
                let json = serde_json::to_string(msgs)?;
                sqlx::query($upsert_apicache).bind(sid).bind(&json).execute(&self.db.pool).await?;
                Ok(())
            }
            async fn load(&self, sid: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>> {
                let row: Option<(String,)> = sqlx::query_as("SELECT messages FROM api_cache WHERE session_id = ?").bind(sid).fetch_optional(&self.db.pool).await?;
                Ok(row.and_then(|(j,)| serde_json::from_str(&j).ok()))
            }
            async fn delete(&self, sid: &str) -> anyhow::Result<()> {
                sqlx::query("DELETE FROM api_cache WHERE session_id = ?").bind(sid).execute(&self.db.pool).await?;
                Ok(())
            }
        }

        // ── PlanStepsRepo ──

        #[derive(Clone)]
        struct $plansteps { db: Arc<$backend> }

        #[async_trait]
        impl PlanStepsRepo for $plansteps {
            async fn save(&self, sid: &str, steps: &[crate::app::PlanStep]) -> anyhow::Result<()> {
                let mut tx = self.db.pool.begin().await?;
                sqlx::query("DELETE FROM plan_steps WHERE session_id = ?").bind(sid).execute(&mut *tx).await?;
                for (i, s) in steps.iter().enumerate() {
                    sqlx::query("INSERT INTO plan_steps (session_id, step_order, description, done) VALUES (?, ?, ?, ?)")
                        .bind(sid).bind(i as i64).bind(&s.description).bind(s.done as i64).execute(&mut *tx).await?;
                }
                tx.commit().await?;
                Ok(())
            }
            async fn load(&self, sid: &str) -> anyhow::Result<Vec<crate::app::PlanStep>> {
                let rows: Vec<(String, i64)> = sqlx::query_as("SELECT description, done FROM plan_steps WHERE session_id = ? ORDER BY step_order")
                    .bind(sid).fetch_all(&self.db.pool).await?;
                Ok(rows.into_iter().map(|(d, done)| crate::app::PlanStep { description: d, done: done != 0 }).collect())
            }
            async fn delete(&self, sid: &str) -> anyhow::Result<()> {
                sqlx::query("DELETE FROM plan_steps WHERE session_id = ?").bind(sid).execute(&self.db.pool).await?;
                Ok(())
            }
        }

        // ── MemoryRepo ──

        #[derive(Clone)]
        struct $memory { db: Arc<$backend> }

        #[async_trait]
        impl MemoryRepo for $memory {
            async fn load(&self, aid: &str) -> anyhow::Result<Option<crate::memory::CrossSessionMemory>> {
                let row: Option<(String,)> = sqlx::query_as("SELECT data FROM memory WHERE agent_id = ?").bind(aid).fetch_optional(&self.db.pool).await?;
                Ok(row.map(|(j,)| serde_json::from_str(&j)).transpose()?)
            }
            async fn save(&self, aid: &str, mem: &crate::memory::CrossSessionMemory) -> anyhow::Result<()> {
                let json = serde_json::to_string(mem)?;
                sqlx::query($upsert_memory).bind(aid).bind(&json).execute(&self.db.pool).await?;
                Ok(())
            }
        }

        // ── StatsRepo ──

        #[derive(Clone)]
        struct $stats { db: Arc<$backend> }

        #[async_trait]
        impl StatsRepo for $stats {
            async fn upsert_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
                for r in records {
                    sqlx::query($upsert_token)
                        .bind(&r.id).bind(r.timestamp).bind(&r.agent_id).bind(&r.model).bind(&r.provider)
                        .bind(r.prompt_tokens as i64).bind(r.completion_tokens as i64).bind(r.total_tokens as i64)
                        .bind(r.has_tool_calls as i64).bind(r.tool_call_count as i64).bind(r.react_rounds as i64)
                        .bind(r.success as i64).bind(r.latency_ms as i64).bind(r.estimated_cost_usd)
                        .bind(&r.trace_id)
                        .execute(&self.db.pool).await?;
                }
                Ok(())
            }
            async fn read_range(&self, from: Option<i64>, to: Option<i64>) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
                macro_rules! cols { () => { "SELECT id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd, trace_id FROM token_records" }; }
                let rows: Vec<TokenRecordRow> = match (from, to) {
                    (Some(f), Some(t)) => sqlx::query_as::<_, TokenRecordRow>(concat!(cols!(), " WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp")).bind(f).bind(t).fetch_all(&self.db.pool).await?,
                    (Some(f), None)     => sqlx::query_as::<_, TokenRecordRow>(concat!(cols!(), " WHERE timestamp >= ? ORDER BY timestamp")).bind(f).fetch_all(&self.db.pool).await?,
                    (None, Some(t))     => sqlx::query_as::<_, TokenRecordRow>(concat!(cols!(), " WHERE timestamp <= ? ORDER BY timestamp")).bind(t).fetch_all(&self.db.pool).await?,
                    (None, None)        => sqlx::query_as::<_, TokenRecordRow>(concat!(cols!(), " ORDER BY timestamp")).fetch_all(&self.db.pool).await?,
                };
                Ok(rows.into_iter().map(|r| r.into()).collect())
            }
            async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
                if keep_days == 0 { return Ok(0); }
                let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
                Ok(sqlx::query("DELETE FROM token_records WHERE timestamp < ?").bind(cutoff).execute(&self.db.pool).await?.rows_affected() as usize)
            }
        }

        // ── SkillRepo ──

        #[derive(Clone)]
        struct $skills { db: Arc<$backend> }

        #[async_trait]
        impl SkillRepo for $skills {
            async fn list(&self, aid: &str) -> anyhow::Result<Vec<SkillEntry>> {
                Ok(sqlx::query_as::<_, (String,String)>("SELECT name, content FROM skills WHERE agent_id = ? ORDER BY name")
                    .bind(aid).fetch_all(&self.db.pool).await?.into_iter().map(|(n,c)| SkillEntry{name:n,content:c}).collect())
            }
            async fn get(&self, aid: &str, name: &str) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>> {
                let row: Option<(String,Option<String>)> = sqlx::query_as("SELECT content, parameters FROM skills WHERE agent_id = ? AND name = ?").bind(aid).bind(name).fetch_optional(&self.db.pool).await?;
                Ok(row.map(|(c,p)| crate::skill_store::SkillDefinition { name: name.into(), description: name.into(), parameters: p.and_then(|x| serde_json::from_str(&x).ok()), content: c }))
            }
            async fn install(&self, aid: &str, name: &str, content: &str) -> anyhow::Result<()> {
                let (fm, _) = crate::skill_store::parse_frontmatter(content);
                let params = fm.as_ref().and_then(|t| t.get("parameters")).and_then(|v| serde_json::to_string(v).ok());
                sqlx::query($upsert_skill)
                    .bind(aid).bind(name).bind(content).bind(&params).execute(&self.db.pool).await?;
                Ok(())
            }
            async fn remove(&self, aid: &str, name: &str) -> anyhow::Result<()> {
                sqlx::query("DELETE FROM skills WHERE agent_id = ? AND name = ?").bind(aid).bind(name).execute(&self.db.pool).await?;
                Ok(())
            }
            async fn list_executable(&self, aid: &str) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>> {
                Ok(sqlx::query_as::<_,(String,String,Option<String>)>("SELECT name, content, parameters FROM skills WHERE agent_id = ? AND parameters IS NOT NULL ORDER BY name")
                    .bind(aid).fetch_all(&self.db.pool).await?.into_iter()
                    .filter_map(|(n,c,p)| Some(crate::skill_store::SkillDefinition{name:n,description:String::new(),parameters:p.and_then(|x| serde_json::from_str(&x).ok()),content:c})).collect())
            }
        }

        // ── ToolCacheRepo ──

        #[derive(Clone)]
        struct $toolcache { db: Arc<$backend> }

        #[async_trait]
        impl ToolCacheRepo for $toolcache {
            async fn load(&self, aid: &str) -> anyhow::Result<HashMap<String,String>> {
                Ok(sqlx::query_as::<_,(String,String)>("SELECT tool_name, doc FROM tool_cache WHERE agent_id = ?").bind(aid).fetch_all(&self.db.pool).await?.into_iter().collect())
            }
            async fn save(&self, aid: &str, docs: &HashMap<String,String>) -> anyhow::Result<()> {
                let mut tx = self.db.pool.begin().await?;
                sqlx::query("DELETE FROM tool_cache WHERE agent_id = ?").bind(aid).execute(&mut *tx).await?;
                for (tn, doc) in docs {
                    sqlx::query("INSERT INTO tool_cache (agent_id, tool_name, doc) VALUES (?, ?, ?)")
                        .bind(aid).bind(tn).bind(doc).execute(&mut *tx).await?;
                }
                tx.commit().await?;
                Ok(())
            }
        }
    };
}

// ── Shared row types ──

#[derive(Debug, Clone, sqlx::FromRow)]
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
    trace_id: Option<String>,
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
            trace_id: r.trace_id.unwrap_or_default(),
        }
    }
}

// ── ClawStorage constructors ──

impl ClawStorage {
    #[cfg(feature = "sqlite")]
    pub async fn sqlite(path: std::path::PathBuf) -> anyhow::Result<Self> {
        sqlite::SqliteBackend::new(path)
            .await
            .map(|b| b.into_storage())
    }

    #[cfg(feature = "mysql")]
    pub async fn mysql(url: &str) -> anyhow::Result<Self> {
        mysql::MySqlBackend::new(url)
            .await
            .map(|b| b.into_storage())
    }

    #[cfg(feature = "postgres")]
    pub async fn postgres(url: &str) -> anyhow::Result<Self> {
        postgres::PgBackend::new(url)
            .await
            .map(|b| b.into_storage())
    }
}

pub(crate) use define_sql_stores;
