//! Redis storage backend.
//!
//! Each repository trait maps to a Redis key pattern. Data is stored as JSON
//! strings or structured Redis types (ZSET, HASH, LIST) depending on access
//! pattern.
//!
//! # Key Schema
//!
//! | Pattern | Type | Trait |
//! |---------|------|-------|
//! | `claw:sessions` | ZSET (score=updated_at) | SessionRepo |
//! | `claw:session:{id}` | HASH (`__json` field) | SessionRepo |
//! | `claw:msg:{sid}` | ZSET (score=seq) | MessageLog |
//! | `claw:msg:seq:{sid}` | COUNTER | MessageLog |
//! | `claw:apicache:{sid}` | STRING | ApiCacheRepo |
//! | `claw:plan:{sid}` | STRING (JSON array) | PlanStepsRepo |
//! | `claw:memory:{agent}` | STRING | MemoryRepo |
//! | `claw:stats:records` | HASH (field=id) | StatsRepo |
//! | `claw:stats:ts` | ZSET (score=timestamp) | StatsRepo |
//! | `claw:skill:{agent}:{name}` | HASH | SkillRepo |
//! | `claw:skills:{agent}` | SET | SkillRepo |
//! | `claw:toolcache:{agent}` | HASH | ToolCacheRepo |

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;

use crate::message::StoredRecord;
use crate::storage::{
    ApiCacheRepo, MemoryRepo, MessageLog, PlanStepsRepo, SearchResult, SessionRepo, SkillEntry,
    SkillRepo, StatsRepo, ToolCacheRepo,
};

// ── Backend ──

#[derive(Clone)]
pub struct RedisBackend {
    conn: MultiplexedConnection,
}

impl RedisBackend {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    pub fn into_storage(self) -> crate::storage::ClawStorage {
        let arc = Arc::new(self);
        crate::storage::ClawStorage {
            sessions: Box::new(RedisSessionStore {
                backend: arc.clone(),
            }),
            message_log: Arc::new(RedisMessageLog {
                backend: arc.clone(),
            }),
            api_cache: Box::new(RedisApiCacheStore {
                backend: arc.clone(),
            }),
            plan_steps: Box::new(RedisPlanStepsStore {
                backend: arc.clone(),
            }),
            memory: Box::new(RedisMemoryStore {
                backend: arc.clone(),
            }),
            stats: Box::new(RedisStatsStore {
                backend: arc.clone(),
            }),
            skills: Box::new(RedisSkillStore {
                backend: arc.clone(),
            }),
            tool_cache: Box::new(RedisToolCacheStore { backend: arc }),
        }
    }
}

// ── SessionRepo ──

#[derive(Clone)]
struct RedisSessionStore {
    backend: Arc<RedisBackend>,
}

fn session_key(id: &str) -> String {
    format!("claw:session:{id}")
}

const SESSIONS_ZSET: &str = "claw:sessions";

#[async_trait]
impl SessionRepo for RedisSessionStore {
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>> {
        let mut conn = self.backend.conn.clone();
        let ids: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(SESSIONS_ZSET)
            .arg(0)
            .arg(-1)
            .query_async(&mut conn)
            .await?;
        let mut sessions = Vec::with_capacity(ids.len());
        for id in &ids {
            let json: Option<String> = redis::cmd("HGET")
                .arg(session_key(id))
                .arg("__json")
                .query_async(&mut conn)
                .await?;
            if let Some(j) = json {
                if let Ok(meta) = serde_json::from_str::<crate::session::SessionMeta>(&j) {
                    sessions.push(meta);
                }
            }
        }
        Ok(sessions)
    }

    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let existing: Vec<String> = redis::cmd("ZRANGE")
            .arg(SESSIONS_ZSET)
            .arg(0)
            .arg(-1)
            .query_async(&mut conn)
            .await?;

        let incoming_ids: std::collections::HashSet<&str> =
            sessions.iter().map(|s| s.id.as_str()).collect();

        let mut pipe = redis::pipe();
        pipe.atomic();
        for id in &existing {
            if !incoming_ids.contains(id.as_str()) {
                pipe.cmd("ZREM").arg(SESSIONS_ZSET).arg(id).ignore();
                pipe.cmd("DEL").arg(session_key(id)).ignore();
                // Clean up orphaned message log
                pipe.cmd("DEL").arg(msg_zset_key(id)).ignore();
                pipe.cmd("DEL").arg(msg_seq_key(id)).ignore();
            }
        }
        for s in sessions {
            let json = serde_json::to_string(s)?;
            pipe.cmd("HSET")
                .arg(session_key(&s.id))
                .arg("__json")
                .arg(json)
                .ignore();
            pipe.cmd("ZADD")
                .arg(SESSIONS_ZSET)
                .arg(s.updated_at)
                .arg(&s.id)
                .ignore();
        }
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
        let mut conn = self.backend.conn.clone();
        let json: Option<String> = redis::cmd("HGET")
            .arg(session_key(id))
            .arg("__json")
            .query_async(&mut conn)
            .await?;
        match json {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let json = serde_json::to_string(session)?;
        redis::pipe()
            .atomic()
            .cmd("HSET")
            .arg(session_key(&session.id))
            .arg("__json")
            .arg(&json)
            .ignore()
            .cmd("ZADD")
            .arg(SESSIONS_ZSET)
            .arg(session.updated_at)
            .arg(&session.id)
            .ignore()
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        redis::pipe()
            .atomic()
            .cmd("DEL")
            .arg(session_key(id))
            .ignore()
            .cmd("ZREM")
            .arg(SESSIONS_ZSET)
            .arg(id)
            .ignore()
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let mut conn = self.backend.conn.clone();
        let count: i64 = redis::cmd("ZCARD")
            .arg(SESSIONS_ZSET)
            .query_async(&mut conn)
            .await?;
        Ok(count as usize)
    }
}

// ── MessageLog ──

#[derive(Clone)]
struct RedisMessageLog {
    backend: Arc<RedisBackend>,
}

fn msg_zset_key(sid: &str) -> String {
    format!("claw:msg:{sid}")
}

fn msg_seq_key(sid: &str) -> String {
    format!("claw:msg:seq:{sid}")
}

// Lua script for atomic seq allocation + ZADD
const APPEND_BATCH_SCRIPT: &str = r#"
    local count = tonumber(ARGV[1])
    local start_seq = redis.call('INCRBY', KEYS[1], count) - count + 1
    for i = 1, count do
        local seq = start_seq + i - 1
        redis.call('ZADD', KEYS[2], seq, ARGV[i + 1])
    end
    return start_seq
"#;

#[async_trait]
impl MessageLog for RedisMessageLog {
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let mut conn = self.backend.conn.clone();

        let script = redis::Script::new(APPEND_BATCH_SCRIPT);
        let seq_key = msg_seq_key(session_id);
        let zset_key = msg_zset_key(session_id);
        let mut invocation = script.key(seq_key);
        invocation.key(zset_key);
        invocation.arg(messages.len());
        for msg in messages {
            let mut rec = StoredRecord::from_message(msg)?;
            rec.seq = 0;
            let json = serde_json::to_string(&rec)?;
            invocation.arg(json);
        }
        let _: i64 = invocation.invoke_async(&mut conn).await?;
        Ok(())
    }

    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>> {
        let mut conn = self.backend.conn.clone();
        let key = msg_zset_key(session_id);
        let limit = if limit == usize::MAX {
            -1
        } else {
            limit as isize - 1
        };
        let raw: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&key)
            .arg(0)
            .arg(limit)
            .query_async(&mut conn)
            .await?;
        let mut records: Vec<StoredRecord> = raw
            .into_iter()
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();
        records.reverse();
        records
            .into_iter()
            .map(|r| {
                r.to_message()
                    .ok_or_else(|| anyhow::anyhow!("message decode error"))
            })
            .collect()
    }

    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.backend.conn.clone();
        let session_ids: Vec<String> = redis::cmd("ZRANGE")
            .arg(SESSIONS_ZSET)
            .arg(0)
            .arg(-1)
            .query_async(&mut conn)
            .await?;
        let mut results = Vec::new();

        for sid in &session_ids {
            let meta_json: Option<String> = redis::cmd("HGET")
                .arg(session_key(sid))
                .arg("__json")
                .query_async(&mut conn)
                .await?;
            let Some(meta_json) = meta_json else { continue };
            let meta: crate::session::SessionMeta = match serde_json::from_str(&meta_json) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let key = msg_zset_key(sid);
            let raw: Vec<String> = redis::cmd("ZRANGE")
                .arg(&key)
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await?;
            let records: Vec<serde_json::Value> = raw
                .into_iter()
                .filter_map(|s| {
                    let rec: StoredRecord = serde_json::from_str(&s).ok()?;
                    Some(rec.payload)
                })
                .collect();

            if super::scan_records_for_query(&records, &q, &meta, max_results, &mut results) {
                return Ok(results);
            }
        }
        Ok(results)
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("DEL").arg(msg_zset_key(session_id)).ignore();
        pipe.cmd("DEL").arg(msg_seq_key(session_id)).ignore();
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
        let mut conn = self.backend.conn.clone();
        let n: usize = redis::cmd("ZCARD")
            .arg(msg_zset_key(session_id))
            .query_async(&mut conn)
            .await?;
        Ok(n)
    }
}

// ── ApiCacheRepo ──

#[derive(Clone)]
struct RedisApiCacheStore {
    backend: Arc<RedisBackend>,
}

fn apicache_key(sid: &str) -> String {
    format!("claw:apicache:{sid}")
}

#[async_trait]
impl ApiCacheRepo for RedisApiCacheStore {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let json = serde_json::to_string(messages)?;
        let _: () = redis::cmd("SET")
            .arg(apicache_key(session_id))
            .arg(json)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>> {
        let mut conn = self.backend.conn.clone();
        let json: Option<String> = redis::cmd("GET")
            .arg(apicache_key(session_id))
            .query_async(&mut conn)
            .await?;
        match json {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let _: () = redis::cmd("DEL")
            .arg(apicache_key(session_id))
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

// ── PlanStepsRepo ──

#[derive(Clone)]
struct RedisPlanStepsStore {
    backend: Arc<RedisBackend>,
}

fn plan_key(sid: &str) -> String {
    format!("claw:plan:{sid}")
}

#[async_trait]
impl PlanStepsRepo for RedisPlanStepsStore {
    async fn save(&self, session_id: &str, steps: &[crate::app::PlanStep]) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let json = serde_json::to_string(steps)?;
        let _: () = redis::cmd("SET")
            .arg(plan_key(session_id))
            .arg(json)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<crate::app::PlanStep>> {
        let mut conn = self.backend.conn.clone();
        let json: Option<String> = redis::cmd("GET")
            .arg(plan_key(session_id))
            .query_async(&mut conn)
            .await?;
        match json {
            Some(j) => Ok(serde_json::from_str(&j)?),
            None => Ok(Vec::new()),
        }
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let _: () = redis::cmd("DEL")
            .arg(plan_key(session_id))
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

// ── MemoryRepo ──

#[derive(Clone)]
struct RedisMemoryStore {
    backend: Arc<RedisBackend>,
}

fn memory_key(agent: &str) -> String {
    format!("claw:memory:{agent}")
}

#[async_trait]
impl MemoryRepo for RedisMemoryStore {
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Option<crate::memory::CrossSessionMemory>> {
        let mut conn = self.backend.conn.clone();
        let json: Option<String> = redis::cmd("GET")
            .arg(memory_key(agent_id))
            .query_async(&mut conn)
            .await?;
        match json {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    async fn save(
        &self,
        agent_id: &str,
        memory: &crate::memory::CrossSessionMemory,
    ) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let json = serde_json::to_string(memory)?;
        let _: () = redis::cmd("SET")
            .arg(memory_key(agent_id))
            .arg(json)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

// ── StatsRepo ──

#[derive(Clone)]
struct RedisStatsStore {
    backend: Arc<RedisBackend>,
}

const STATS_HASH: &str = "claw:stats:records";
const STATS_TS_ZSET: &str = "claw:stats:ts";

#[async_trait]
impl StatsRepo for RedisStatsStore {
    async fn upsert_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
        if records.is_empty() {
            return Ok(());
        }
        let mut conn = self.backend.conn.clone();
        let mut pipe = redis::pipe();
        pipe.atomic();
        for r in records {
            let json = serde_json::to_string(r)?;
            pipe.cmd("HSET")
                .arg(STATS_HASH)
                .arg(&r.id)
                .arg(json)
                .ignore();
            pipe.cmd("ZADD")
                .arg(STATS_TS_ZSET)
                .arg(r.timestamp)
                .arg(&r.id)
                .ignore();
        }
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn read_range(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
        let mut conn = self.backend.conn.clone();
        let (min, max) = match (from, to) {
            (Some(f), Some(t)) => (f, t),
            (Some(f), None) => (f, i64::MAX),
            (None, Some(t)) => (i64::MIN, t),
            (None, None) => (i64::MIN, i64::MAX),
        };
        let ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(STATS_TS_ZSET)
            .arg(min)
            .arg(max)
            .query_async(&mut conn)
            .await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        // HMGET: collect values per ID
        let mut records = Vec::with_capacity(ids.len());
        for id in &ids {
            let val: Option<String> = redis::cmd("HGET")
                .arg(STATS_HASH)
                .arg(id)
                .query_async(&mut conn)
                .await?;
            if let Some(s) = val {
                if let Ok(r) = serde_json::from_str::<crate::stats::TokenRecord>(&s) {
                    records.push(r);
                }
            }
        }
        records.sort_by_key(|r| r.timestamp);
        Ok(records)
    }

    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
        let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
        let mut conn = self.backend.conn.clone();
        let ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(STATS_TS_ZSET)
            .arg(i64::MIN)
            .arg(cutoff)
            .query_async(&mut conn)
            .await?;
        if ids.is_empty() {
            return Ok(0);
        }
        let mut pipe = redis::pipe();
        pipe.atomic();
        for id in &ids {
            pipe.cmd("HDEL").arg(STATS_HASH).arg(id).ignore();
            pipe.cmd("ZREM").arg(STATS_TS_ZSET).arg(id).ignore();
        }
        pipe.query_async::<()>(&mut conn).await?;
        Ok(ids.len())
    }
}

// ── SkillRepo ──

#[derive(Clone)]
struct RedisSkillStore {
    backend: Arc<RedisBackend>,
}

fn skills_set_key(agent: &str) -> String {
    format!("claw:skills:{agent}")
}

fn skill_hash_key(agent: &str, name: &str) -> String {
    format!("claw:skill:{agent}:{name}")
}

#[async_trait]
impl SkillRepo for RedisSkillStore {
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>> {
        let mut conn = self.backend.conn.clone();
        let names: Vec<String> = redis::cmd("SMEMBERS")
            .arg(skills_set_key(agent_id))
            .query_async(&mut conn)
            .await?;
        let mut entries = Vec::with_capacity(names.len());
        for name in &names {
            let content: Option<String> = redis::cmd("HGET")
                .arg(skill_hash_key(agent_id, name))
                .arg("content")
                .query_async(&mut conn)
                .await?;
            entries.push(SkillEntry {
                name: name.clone(),
                content: content.unwrap_or_default(),
            });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    async fn get(
        &self,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>> {
        let mut conn = self.backend.conn.clone();
        let key = skill_hash_key(agent_id, name);
        let content: Option<String> = redis::cmd("HGET")
            .arg(&key)
            .arg("content")
            .query_async(&mut conn)
            .await?;
        match content {
            Some(content) => {
                let parameters: Option<String> = redis::cmd("HGET")
                    .arg(&key)
                    .arg("parameters")
                    .query_async(&mut conn)
                    .await?;
                let params_json = parameters.and_then(|s| serde_json::from_str(&s).ok());
                Ok(Some(crate::skill_store::SkillDefinition {
                    name: name.to_string(),
                    description: name.to_string(),
                    parameters: params_json,
                    content,
                }))
            }
            None => Ok(None),
        }
    }

    async fn install(&self, agent_id: &str, name: &str, content: &str) -> anyhow::Result<()> {
        let (frontmatter, _) = crate::skill_store::parse_frontmatter(content);
        let parameters = frontmatter
            .as_ref()
            .and_then(|f| f.get("parameters"))
            .and_then(|p| serde_json::to_string(p).ok());

        let mut conn = self.backend.conn.clone();
        let key = skill_hash_key(agent_id, name);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("HSET")
            .arg(&key)
            .arg("content")
            .arg(content)
            .ignore();
        if let Some(ref p) = parameters {
            pipe.cmd("HSET").arg(&key).arg("parameters").arg(p).ignore();
        } else {
            pipe.cmd("HDEL").arg(&key).arg("parameters").ignore();
        }
        pipe.cmd("SADD")
            .arg(skills_set_key(agent_id))
            .arg(name)
            .ignore();
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("DEL").arg(skill_hash_key(agent_id, name)).ignore();
        pipe.cmd("SREM")
            .arg(skills_set_key(agent_id))
            .arg(name)
            .ignore();
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn list_executable(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>> {
        let mut conn = self.backend.conn.clone();
        let names: Vec<String> = redis::cmd("SMEMBERS")
            .arg(skills_set_key(agent_id))
            .query_async(&mut conn)
            .await?;
        let mut defs = Vec::new();
        for name in &names {
            let key = skill_hash_key(agent_id, name);
            let parameters: Option<String> = redis::cmd("HGET")
                .arg(&key)
                .arg("parameters")
                .query_async(&mut conn)
                .await?;
            if let Some(p) = parameters {
                let content: Option<String> = redis::cmd("HGET")
                    .arg(&key)
                    .arg("content")
                    .query_async(&mut conn)
                    .await?;
                let content = content.unwrap_or_default();
                let params_json = serde_json::from_str(&p).ok();
                defs.push(crate::skill_store::SkillDefinition {
                    name: name.clone(),
                    description: name.clone(),
                    parameters: params_json,
                    content,
                });
            }
        }
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(defs)
    }
}

// ── ToolCacheRepo ──

#[derive(Clone)]
struct RedisToolCacheStore {
    backend: Arc<RedisBackend>,
}

fn toolcache_key(agent: &str) -> String {
    format!("claw:toolcache:{agent}")
}

#[async_trait]
impl ToolCacheRepo for RedisToolCacheStore {
    async fn load(&self, agent_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let mut conn = self.backend.conn.clone();
        let map: HashMap<String, String> = redis::cmd("HGETALL")
            .arg(toolcache_key(agent_id))
            .query_async(&mut conn)
            .await?;
        Ok(map)
    }

    async fn save(&self, agent_id: &str, docs: &HashMap<String, String>) -> anyhow::Result<()> {
        let mut conn = self.backend.conn.clone();
        let key = toolcache_key(agent_id);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("DEL").arg(&key).ignore();
        for (k, v) in docs {
            pipe.cmd("HSET").arg(&key).arg(k).arg(v).ignore();
        }
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }
}

// ── ClawStorage constructor ──

impl crate::storage::ClawStorage {
    #[cfg(feature = "redis")]
    pub async fn redis(url: &str) -> anyhow::Result<Self> {
        Ok(RedisBackend::new(url).await?.into_storage())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn redis_url() -> Option<String> {
        std::env::var("CLAW_TEST_REDIS_URL").ok()
    }

    async fn test_storage() -> Option<crate::storage::ClawStorage> {
        let url = redis_url()?;
        RedisBackend::new(&url).await.ok().map(|b| b.into_storage())
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_session_save_load() {
        let Some(s) = test_storage().await else {
            return;
        };
        let meta = crate::session::SessionMeta {
            id: "s1".into(),
            title: "Test".into(),
            agent_id: "default".into(),
            user_id: "default".to_string(),
            state: crate::session::SessionState::Active,
            created_at: 1,
            updated_at: 2,
            message_count: 0,
        };
        s.sessions.save_all(&[meta]).await.unwrap();
        let loaded = s.sessions.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "s1");
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_message_log_append_load() {
        let Some(s) = test_storage().await else {
            return;
        };
        let msgs = vec![
            crate::app::Message::User {
                text: "hello".into(),
            },
            crate::app::Message::Assistant {
                text: "hi there".into(),
                reasoning: String::new(),
                token_usage: None,
            },
        ];
        s.message_log.append_batch("s1", &msgs).await.unwrap();
        let loaded = s.message_log.load("s1", 10).await.unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_message_log_search() {
        let Some(s) = test_storage().await else {
            return;
        };
        s.sessions
            .save_all(&[crate::session::SessionMeta {
                id: "s1".into(),
                title: "Search Test".into(),
                agent_id: "default".into(),
                user_id: "default".to_string(),
                state: crate::session::SessionState::Active,
                created_at: 1,
                updated_at: 2,
                message_count: 0,
            }])
            .await
            .unwrap();
        s.message_log
            .append_batch(
                "s1",
                &[crate::app::Message::User {
                    text: "Hello world".into(),
                }],
            )
            .await
            .unwrap();
        let results = s.message_log.search("world", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "s1");
    }
}
