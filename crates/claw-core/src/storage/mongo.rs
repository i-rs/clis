//! MongoDB storage backend.
//!
//! Each repository trait maps to a MongoDB collection. Documents use the
//! struct's `Serialize`/`Deserialize` via `bson::to_document` / `bson::from_document`
//! for simple types, and JSON strings for complex nested types (`Message` payloads,
//! API cache arrays, etc.) to avoid BSON↔JSON edge cases.
//!
//! # Collections
//!
//! | Collection | Primary key |
//! |-----------|-------------|
//! | `sessions` | `_id = session_id` |
//! | `message_log` | auto ObjectId + compound index `(session_id, seq)` |
//! | `seq_counters` | `_id = session_id` (atomic seq generation) |
//! | `api_cache` | `_id = session_id` |
//! | `plan_steps` | `_id = session_id` |
//! | `memory` | `_id = agent_id` |
//! | `token_records` | `_id = record_id` |
//! | `skills` | `_id = {agent_id, name}` |
//! | `tool_cache` | `_id = agent_id` |
//! | `agent_configs` | `(user_id, agent_id)` |
//! | `provider_configs` | `name` |
//! | `dashboard_users` | `user_id` |
//! | `mcp_server_configs` | `(user_id, agent_id, name)` |
//! | `app_settings` | `_id = key` |

#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::IndexModel;
use mongodb::bson::Document;
use mongodb::bson::doc;
use mongodb::options::{IndexOptions, ReturnDocument};

use crate::message::StoredRecord;
use crate::storage::config_store::{
    AgentConfigRepo, AgentConfigRow, AppSettingRow, AppSettingsRepo, DashboardUserRepo,
    DashboardUserRow, McpServerConfigRepo, McpServerConfigRow, ProviderConfigRepo,
    ProviderConfigRow,
};
use crate::storage::{
    ApiCacheRepo, MemoryRepo, MessageLog, PlanStepsRepo, SearchResult, SessionRepo, SkillEntry,
    SkillRepo, StatsRepo, ToolCacheRepo,
};

// ── Backend ──

#[derive(Clone)]
pub struct MongoBackend {
    client: mongodb::Client,
    db: mongodb::Database,
}

impl MongoBackend {
    pub async fn new(url: &str, db_name: &str) -> anyhow::Result<Self> {
        let client = mongodb::Client::with_uri_str(url).await?;
        let db = client.database(db_name);
        let backend = Self { client, db };
        backend.ensure_indexes().await?;
        Ok(backend)
    }

    async fn ensure_indexes(&self) -> anyhow::Result<()> {
        // message_log: compound index for "last N messages in session"
        self.db
            .collection::<Document>("message_log")
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "session_id": 1, "seq": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // token_records: range scans by timestamp
        self.db
            .collection::<Document>("token_records")
            .create_index(IndexModel::builder().keys(doc! { "timestamp": 1 }).build())
            .await?;

        // sessions: list ordered by updated_at
        self.db
            .collection::<Document>("sessions")
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "updated_at": -1 })
                    .build(),
            )
            .await?;

        // dashboard_users: unique index on token_hash for auth lookups
        self.db
            .collection::<Document>("dashboard_users")
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "token_hash": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // mcp_server_configs: compound index for user+agent lookups
        self.db
            .collection::<Document>("mcp_server_configs")
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "agent_id": 1 })
                    .build(),
            )
            .await?;

        Ok(())
    }

    pub fn into_storage(self) -> crate::storage::ClawStorage {
        let arc = Arc::new(self);
        crate::storage::ClawStorage {
            sessions: Box::new(MongoSessionStore { db: arc.clone() }),
            message_log: Arc::new(MongoMessageLog { db: arc.clone() }),
            api_cache: Box::new(MongoApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(MongoPlanStepsStore { db: arc.clone() }),
            memory: Box::new(MongoMemoryStore { db: arc.clone() }),
            stats: Box::new(MongoStatsStore { db: arc.clone() }),
            skills: Box::new(MongoSkillStore { db: arc.clone() }),
            tool_cache: Box::new(MongoToolCacheStore { db: arc }),
        }
    }

    pub fn into_config_store(self) -> crate::storage::config_store::ConfigStore {
        let arc = Arc::new(self);
        crate::storage::config_store::ConfigStore {
            agent_configs: Box::new(MongoAgentConfigStore { db: arc.clone() }),
            provider_configs: Box::new(MongoProviderConfigStore { db: arc.clone() }),
            dashboard_users: Box::new(MongoDashboardUserStore { db: arc.clone() }),
            mcp_servers: Box::new(MongoMcpServerConfigStore { db: arc.clone() }),
            app_settings: Box::new(MongoAppSettingsStore { db: arc }),
        }
    }
}

// ── SessionRepo ──

#[derive(Clone)]
struct MongoSessionStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl SessionRepo for MongoSessionStore {
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("sessions")
            .find(doc! {})
            .sort(doc! { "updated_at": -1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter().map(|d| doc_to_session_meta(&d)).collect()
    }

    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
        if sessions.is_empty() {
            tracing::warn!("save_all(empty): 清空所有 session + message_log + seq_counters");
            self.db
                .db
                .collection::<Document>("sessions")
                .delete_many(doc! {})
                .await?;
            self.db
                .db
                .collection::<Document>("message_log")
                .delete_many(doc! {})
                .await?;
            self.db
                .db
                .collection::<Document>("seq_counters")
                .delete_many(doc! {})
                .await?;
            return Ok(());
        }

        let incoming_ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();

        // Delete sessions not in the incoming set
        self.db
            .db
            .collection::<Document>("sessions")
            .delete_many(doc! { "_id": { "$nin": incoming_ids.clone() } })
            .await?;

        // Upsert all incoming sessions
        for s in sessions {
            let mut doc = session_meta_to_doc(s)?;
            doc.insert("_id", s.id.clone());
            self.db
                .db
                .collection::<Document>("sessions")
                .update_one(doc! { "_id": &s.id }, doc! { "$set": doc })
                .upsert(true)
                .await?;
        }
        Ok(())
    }

    async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
        let doc = self
            .db
            .db
            .collection::<Document>("sessions")
            .find_one(doc! { "_id": id })
            .await?;
        doc.as_ref().map(doc_to_session_meta).transpose()
    }

    async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
        let mut doc = session_meta_to_doc(session)?;
        doc.insert("_id", session.id.clone());
        self.db
            .db
            .collection::<Document>("sessions")
            .update_one(doc! { "_id": &session.id }, doc! { "$set": doc })
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("sessions")
            .delete_one(doc! { "_id": id })
            .await?;
        Ok(())
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let count = self
            .db
            .db
            .collection::<Document>("sessions")
            .estimated_document_count()
            .await?;
        Ok(count as usize)
    }
}

fn session_meta_to_doc(s: &crate::session::SessionMeta) -> anyhow::Result<Document> {
    Ok(doc! {
        "title": s.title.clone(),
        "agent_id": s.agent_id.clone(),
        "user_id": s.user_id.clone(),
        "state": serde_json::to_string(&s.state)?,
        "created_at": s.created_at,
        "updated_at": s.updated_at,
        "message_count": s.message_count as i64,
    })
}

fn doc_to_session_meta(d: &Document) -> anyhow::Result<crate::session::SessionMeta> {
    let id = d.get_str("_id")?.to_string();
    let state_str = d.get_str("state").unwrap_or("\"Active\"");
    let state: crate::session::SessionState = serde_json::from_str(state_str).unwrap_or_default();
    Ok(crate::session::SessionMeta {
        id,
        title: d.get_str("title").unwrap_or("").to_string(),
        agent_id: d.get_str("agent_id").unwrap_or("default").to_string(),
        user_id: d.get_str("user_id").unwrap_or("default").to_string(),
        state,
        created_at: d.get_i64("created_at").unwrap_or(0),
        updated_at: d.get_i64("updated_at").unwrap_or(0),
        message_count: d.get_i64("message_count").unwrap_or(0).max(0) as usize,
    })
}

// ── MessageLog ──

#[derive(Clone)]
struct MongoMessageLog {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl MessageLog for MongoMessageLog {
    async fn append_batch(
        &self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        // Use a transaction so seq counter reservation + message inserts
        // are all-or-nothing, satisfying the trait contract.
        let mut session = self.db.client.start_session().await?;
        session.start_transaction().await?;

        let result: anyhow::Result<()> = async {
            let counter_coll = self.db.db.collection::<Document>("seq_counters");
            let result = counter_coll
                .find_one_and_update(
                    doc! { "_id": session_id },
                    doc! { "$inc": { "next_seq": messages.len() as i64 } },
                )
                .upsert(true)
                .return_document(ReturnDocument::Before)
                .session(&mut session)
                .await?;

            let prev_seq = result.and_then(|d| d.get_i64("next_seq").ok()).unwrap_or(0);
            let start_seq = prev_seq + 1;

            let docs: Vec<Document> = messages
                .iter()
                .enumerate()
                .map(|(i, msg)| {
                    let rec = StoredRecord::from_message(msg)?;
                    Ok(doc! {
                        "session_id": session_id,
                        "seq": start_seq + i as i64,
                        "ts": rec.ts,
                        "schema_v": rec.schema_v as i32,
                        "payload": serde_json::to_string(&rec.payload)?,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            self.db
                .db
                .collection::<Document>("message_log")
                .insert_many(&docs)
                .session(&mut session)
                .await?;

            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                session.commit_transaction().await?;
                Ok(())
            }
            Err(e) => {
                let _ = session.abort_transaction().await;
                Err(e)
            }
        }
    }

    async fn load(
        &self,
        session_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::app::Message>> {
        let limit_i64 = limit.min(i64::MAX as usize) as i64;

        let cursor = self
            .db
            .db
            .collection::<Document>("message_log")
            .find(doc! { "session_id": session_id })
            .sort(doc! { "seq": -1 })
            .limit(limit_i64)
            .await?;

        let mut docs: Vec<Document> = cursor.try_collect().await?;
        docs.reverse(); // back to ascending order

        docs.into_iter()
            .map(|d| {
                let payload_str = d.get_str("payload")?;
                let payload: serde_json::Value = serde_json::from_str(payload_str)?;
                let rec = StoredRecord {
                    seq: d.get_i64("seq").unwrap_or(0) as u64,
                    ts: d.get_i64("ts").unwrap_or(0),
                    schema_v: d.get_i32("schema_v").unwrap_or(1) as u16,
                    payload,
                };
                rec.to_message()
                    .ok_or_else(|| anyhow::anyhow!("decode error"))
            })
            .collect()
    }

    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        // Case-insensitive substring match via $regex
        let escaped = q
            .replace('\\', "\\\\")
            .replace('.', "\\.")
            .replace('*', "\\*")
            .replace('+', "\\+")
            .replace('?', "\\?")
            .replace('^', "\\^")
            .replace('$', "\\$")
            .replace('|', "\\|")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('[', "\\[")
            .replace('{', "\\{");
        let candidate_docs: Vec<Document> = self
            .db
            .db
            .collection::<Document>("message_log")
            .find(doc! { "payload": { "$regex": &escaped, "$options": "i" } })
            .projection(doc! { "session_id": 1 })
            .await?
            .try_collect()
            .await?;

        let candidate_ids: Vec<String> = candidate_docs
            .into_iter()
            .filter_map(|d| d.get_str("session_id").ok().map(String::from))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut results = Vec::new();
        for sid in &candidate_ids {
            // Load session meta
            let session_doc = self
                .db
                .db
                .collection::<Document>("sessions")
                .find_one(doc! { "_id": sid })
                .await?;
            let Some(session_doc) = session_doc else {
                continue;
            };
            let meta = doc_to_session_meta(&session_doc)?;

            // Load all messages for this session
            let cursor = self
                .db
                .db
                .collection::<Document>("message_log")
                .find(doc! { "session_id": sid })
                .sort(doc! { "seq": 1 })
                .await?;
            let docs: Vec<Document> = cursor.try_collect().await?;

            let records: Vec<serde_json::Value> = docs
                .into_iter()
                .filter_map(|d| {
                    let p = d.get_str("payload").ok()?;
                    serde_json::from_str(p).ok()
                })
                .collect();

            if super::scan_records_for_query(&records, &q, &meta, max_results, &mut results) {
                return Ok(results);
            }
        }
        Ok(results)
    }

    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("message_log")
            .delete_many(doc! { "session_id": session_id })
            .await?;
        self.db
            .db
            .collection::<Document>("seq_counters")
            .delete_one(doc! { "_id": session_id })
            .await?;
        Ok(())
    }

    async fn count(&self, session_id: &str) -> anyhow::Result<usize> {
        let n = self
            .db
            .db
            .collection::<Document>("message_log")
            .count_documents(doc! { "session_id": session_id })
            .await?;
        Ok(n as usize)
    }
}

// ── ApiCacheRepo ──

#[derive(Clone)]
struct MongoApiCacheStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl ApiCacheRepo for MongoApiCacheStore {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()> {
        let json = serde_json::to_string(messages)?;
        self.db
            .db
            .collection::<Document>("api_cache")
            .update_one(
                doc! { "_id": session_id },
                doc! { "$set": { "messages": json } },
            )
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>> {
        let doc = self
            .db
            .db
            .collection::<Document>("api_cache")
            .find_one(doc! { "_id": session_id })
            .await?;
        match doc {
            Some(d) => {
                let json = d.get_str("messages").unwrap_or("[]");
                let msgs: Vec<serde_json::Value> = serde_json::from_str(json)?;
                Ok(Some(msgs))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("api_cache")
            .delete_one(doc! { "_id": session_id })
            .await?;
        Ok(())
    }
}

// ── PlanStepsRepo ──

#[derive(Clone)]
struct MongoPlanStepsStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl PlanStepsRepo for MongoPlanStepsStore {
    async fn save(&self, session_id: &str, steps: &[crate::app::PlanStep]) -> anyhow::Result<()> {
        let steps_doc: Vec<Document> = steps
            .iter()
            .map(|s| {
                doc! {
                    "description": s.description.clone(),
                    "done": s.done,
                }
            })
            .collect();
        self.db
            .db
            .collection::<Document>("plan_steps")
            .update_one(
                doc! { "_id": session_id },
                doc! { "$set": { "steps": steps_doc } },
            )
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<crate::app::PlanStep>> {
        let doc = self
            .db
            .db
            .collection::<Document>("plan_steps")
            .find_one(doc! { "_id": session_id })
            .await?;
        match doc {
            Some(d) => {
                let arr = d
                    .get_array("steps")
                    .map_err(|_| anyhow::anyhow!("invalid steps field"))?;
                arr.iter()
                    .map(|v| {
                        let d = v
                            .as_document()
                            .ok_or_else(|| anyhow::anyhow!("not a doc"))?;
                        Ok(crate::app::PlanStep {
                            description: d.get_str("description").unwrap_or("").to_string(),
                            done: d.get_bool("done").unwrap_or(false),
                        })
                    })
                    .collect()
            }
            None => Ok(Vec::new()),
        }
    }

    async fn delete(&self, session_id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("plan_steps")
            .delete_one(doc! { "_id": session_id })
            .await?;
        Ok(())
    }
}

// ── MemoryRepo ──

#[derive(Clone)]
struct MongoMemoryStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl MemoryRepo for MongoMemoryStore {
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Option<crate::memory::CrossSessionMemory>> {
        let doc = self
            .db
            .db
            .collection::<Document>("memory")
            .find_one(doc! { "_id": agent_id })
            .await?;
        match doc {
            Some(d) => {
                let json = d.get_str("data").unwrap_or("{}");
                let mem: crate::memory::CrossSessionMemory = serde_json::from_str(json)?;
                Ok(Some(mem))
            }
            None => Ok(None),
        }
    }

    async fn save(
        &self,
        agent_id: &str,
        memory: &crate::memory::CrossSessionMemory,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string(memory)?;
        self.db
            .db
            .collection::<Document>("memory")
            .update_one(doc! { "_id": agent_id }, doc! { "$set": { "data": json } })
            .upsert(true)
            .await?;
        Ok(())
    }
}

// ── StatsRepo ──

#[derive(Clone)]
struct MongoStatsStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl StatsRepo for MongoStatsStore {
    async fn upsert_batch(&self, records: &[crate::stats::TokenRecord]) -> anyhow::Result<()> {
        for r in records {
            let mut doc = mongodb::bson::to_document(r)?;
            doc.insert("_id", r.id.clone());
            self.db
                .db
                .collection::<Document>("token_records")
                .update_one(doc! { "_id": &r.id }, doc! { "$set": doc })
                .upsert(true)
                .await?;
        }
        Ok(())
    }

    async fn read_range(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> anyhow::Result<Vec<crate::stats::TokenRecord>> {
        let mut filter = doc! {};
        if let Some(f) = from {
            filter.insert("timestamp", doc! { "$gte": f });
        }
        if let Some(t) = to {
            if filter.contains_key("timestamp") {
                filter.get_document_mut("timestamp")?.insert("$lte", t);
            } else {
                filter.insert("timestamp", doc! { "$lte": t });
            }
        }
        let cursor = self
            .db
            .db
            .collection::<Document>("token_records")
            .find(filter)
            .sort(doc! { "timestamp": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|mut d| {
                // Remove _id to avoid conflict with the struct's id field
                d.remove("_id");
                Ok(mongodb::bson::from_document::<crate::stats::TokenRecord>(
                    d,
                )?)
            })
            .collect()
    }

    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
        let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
        let result = self
            .db
            .db
            .collection::<Document>("token_records")
            .delete_many(doc! { "timestamp": { "$lt": cutoff } })
            .await?;
        Ok(result.deleted_count as usize)
    }
}

// ── SkillRepo ──

#[derive(Clone)]
struct MongoSkillStore {
    db: Arc<MongoBackend>,
}

fn skill_doc_id(agent_id: &str, name: &str) -> Document {
    doc! { "agent_id": agent_id, "name": name }
}

#[async_trait]
impl SkillRepo for MongoSkillStore {
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("skills")
            .find(doc! { "agent_id": agent_id })
            .sort(doc! { "name": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        Ok(docs
            .into_iter()
            .map(|d| SkillEntry {
                name: d.get_str("name").unwrap_or("").to_string(),
                content: d.get_str("content").unwrap_or("").to_string(),
            })
            .collect())
    }

    async fn get(
        &self,
        agent_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<crate::skill_store::SkillDefinition>> {
        let doc = self
            .db
            .db
            .collection::<Document>("skills")
            .find_one(skill_doc_id(agent_id, name))
            .await?;
        match doc {
            Some(d) => {
                let content = d.get_str("content").unwrap_or("").to_string();
                let parameters = d.get_str("parameters").ok().map(String::from);
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
        let (frontmatter, _body) = crate::skill_store::parse_frontmatter(content);
        let parameters = frontmatter
            .as_ref()
            .and_then(|f| f.get("parameters"))
            .and_then(|p| serde_json::to_string(p).ok());

        let mut set_doc = doc! {
            "agent_id": agent_id,
            "name": name,
            "content": content,
        };
        if let Some(ref p) = parameters {
            set_doc.insert("parameters", p.clone());
        }

        self.db
            .db
            .collection::<Document>("skills")
            .update_one(skill_doc_id(agent_id, name), doc! { "$set": set_doc })
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("skills")
            .delete_one(skill_doc_id(agent_id, name))
            .await?;
        Ok(())
    }

    async fn list_executable(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<Vec<crate::skill_store::SkillDefinition>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("skills")
            .find(doc! { "agent_id": agent_id, "parameters": { "$exists": true, "$ne": null } })
            .sort(doc! { "name": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|d| {
                let name = d.get_str("name").unwrap_or("").to_string();
                let content = d.get_str("content").unwrap_or("").to_string();
                let parameters_str = d.get_str("parameters").unwrap_or("");
                let parameters = serde_json::from_str(parameters_str).ok();
                Ok(crate::skill_store::SkillDefinition {
                    name: name.clone(),
                    description: name,
                    parameters,
                    content,
                })
            })
            .collect()
    }
}

// ── ToolCacheRepo ──

#[derive(Clone)]
struct MongoToolCacheStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl ToolCacheRepo for MongoToolCacheStore {
    async fn load(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        let doc = self
            .db
            .db
            .collection::<Document>("tool_cache")
            .find_one(doc! { "_id": agent_id })
            .await?;
        match doc {
            Some(d) => {
                let empty_doc = doc! {};
                let docs = d.get_document("docs").unwrap_or(&empty_doc);
                let mut map = std::collections::HashMap::new();
                for (k, v) in docs.iter() {
                    if let Some(s) = v.as_str() {
                        map.insert(k.to_string(), s.to_string());
                    }
                }
                Ok(map)
            }
            None => Ok(std::collections::HashMap::new()),
        }
    }

    async fn save(
        &self,
        agent_id: &str,
        docs: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let docs_bson: Document = docs
            .iter()
            .map(|(k, v)| (k.clone(), mongodb::bson::Bson::String(v.clone())))
            .collect();
        self.db
            .db
            .collection::<Document>("tool_cache")
            .update_one(
                doc! { "_id": agent_id },
                doc! { "$set": { "docs": docs_bson } },
            )
            .upsert(true)
            .await?;
        Ok(())
    }
}

// ── AgentConfigRepo ──

#[derive(Clone)]
struct MongoAgentConfigStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl AgentConfigRepo for MongoAgentConfigStore {
    async fn load_all(&self, user_id: &str) -> anyhow::Result<Vec<AgentConfigRow>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("agent_configs")
            .find(doc! { "user_id": user_id })
            .sort(doc! { "agent_id": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|mut d| {
                d.remove("_id");
                Ok(mongodb::bson::from_document(d)?)
            })
            .collect()
    }

    async fn upsert(&self, row: &AgentConfigRow) -> anyhow::Result<()> {
        let doc = mongodb::bson::to_document(row)?;
        self.db
            .db
            .collection::<Document>("agent_configs")
            .update_one(
                doc! { "user_id": &row.user_id, "agent_id": &row.agent_id },
                doc! { "$set": doc },
            )
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn delete(&self, user_id: &str, agent_id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("agent_configs")
            .delete_one(doc! { "user_id": user_id, "agent_id": agent_id })
            .await?;
        Ok(())
    }
}

// ── ProviderConfigRepo ──

#[derive(Clone)]
struct MongoProviderConfigStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl ProviderConfigRepo for MongoProviderConfigStore {
    async fn load_all(&self) -> anyhow::Result<Vec<ProviderConfigRow>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("provider_configs")
            .find(doc! {})
            .sort(doc! { "name": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|mut d| {
                d.remove("_id");
                Ok(mongodb::bson::from_document(d)?)
            })
            .collect()
    }

    async fn upsert(&self, row: &ProviderConfigRow) -> anyhow::Result<()> {
        let doc = mongodb::bson::to_document(row)?;
        self.db
            .db
            .collection::<Document>("provider_configs")
            .update_one(doc! { "name": &row.name }, doc! { "$set": doc })
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn delete(&self, name: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("provider_configs")
            .delete_one(doc! { "name": name })
            .await?;
        Ok(())
    }
}

// ── DashboardUserRepo ──

#[derive(Clone)]
struct MongoDashboardUserStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl DashboardUserRepo for MongoDashboardUserStore {
    async fn load_all(&self) -> anyhow::Result<Vec<DashboardUserRow>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("dashboard_users")
            .find(doc! {})
            .sort(doc! { "user_id": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|mut d| {
                d.remove("_id");
                Ok(mongodb::bson::from_document(d)?)
            })
            .collect()
    }

    async fn upsert(&self, row: &DashboardUserRow) -> anyhow::Result<()> {
        let doc = mongodb::bson::to_document(row)?;
        self.db
            .db
            .collection::<Document>("dashboard_users")
            .update_one(doc! { "user_id": &row.user_id }, doc! { "$set": doc })
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn find_by_token_hash(&self, hash: &str) -> anyhow::Result<Option<DashboardUserRow>> {
        let doc = self
            .db
            .db
            .collection::<Document>("dashboard_users")
            .find_one(doc! { "token_hash": hash })
            .await?;
        match doc {
            Some(mut d) => {
                d.remove("_id");
                Ok(Some(mongodb::bson::from_document(d)?))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, user_id: &str) -> anyhow::Result<()> {
        self.db
            .db
            .collection::<Document>("dashboard_users")
            .delete_one(doc! { "user_id": user_id })
            .await?;
        Ok(())
    }
}

// ── McpServerConfigRepo ──

#[derive(Clone)]
struct MongoMcpServerConfigStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl McpServerConfigRepo for MongoMcpServerConfigStore {
    async fn load_for(
        &self,
        user_id: &str,
        agent_id: Option<&str>,
    ) -> anyhow::Result<Vec<McpServerConfigRow>> {
        let mut filter = doc! { "user_id": user_id };
        match agent_id {
            Some(aid) => {
                filter.insert("agent_id", aid);
            }
            None => {
                filter.insert("agent_id", doc! { "$exists": false });
            }
        }
        let cursor = self
            .db
            .db
            .collection::<Document>("mcp_server_configs")
            .find(filter)
            .sort(doc! { "name": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|mut d| {
                d.remove("_id");
                Ok(mongodb::bson::from_document(d)?)
            })
            .collect()
    }

    async fn upsert(&self, row: &McpServerConfigRow) -> anyhow::Result<()> {
        let doc = mongodb::bson::to_document(row)?;
        let mut filter = doc! { "user_id": &row.user_id, "name": &row.name };
        match &row.agent_id {
            Some(aid) => {
                filter.insert("agent_id", aid.as_str());
            }
            None => {
                filter.insert("agent_id", doc! { "$exists": false });
            }
        }
        self.db
            .db
            .collection::<Document>("mcp_server_configs")
            .update_one(filter, doc! { "$set": doc })
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn delete(
        &self,
        user_id: &str,
        agent_id: Option<&str>,
        name: &str,
    ) -> anyhow::Result<()> {
        let mut filter = doc! { "user_id": user_id, "name": name };
        match agent_id {
            Some(aid) => {
                filter.insert("agent_id", aid);
            }
            None => {
                filter.insert("agent_id", doc! { "$exists": false });
            }
        }
        self.db
            .db
            .collection::<Document>("mcp_server_configs")
            .delete_one(filter)
            .await?;
        Ok(())
    }
}

// ── AppSettingsRepo ──

#[derive(Clone)]
struct MongoAppSettingsStore {
    db: Arc<MongoBackend>,
}

#[async_trait]
impl AppSettingsRepo for MongoAppSettingsStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<serde_json::Value>> {
        let doc = self
            .db
            .db
            .collection::<Document>("app_settings")
            .find_one(doc! { "_id": key })
            .await?;
        match doc {
            Some(d) => {
                let json_str = d.get_str("value").unwrap_or("null");
                Ok(Some(serde_json::from_str(json_str)?))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: &serde_json::Value) -> anyhow::Result<()> {
        let value_str = serde_json::to_string(value)?;
        let now = chrono::Utc::now().timestamp();
        self.db
            .db
            .collection::<Document>("app_settings")
            .update_one(
                doc! { "_id": key },
                doc! { "$set": { "value": &value_str, "updated_at": now } },
            )
            .upsert(true)
            .await?;
        Ok(())
    }

    async fn load_all(&self) -> anyhow::Result<Vec<AppSettingRow>> {
        let cursor = self
            .db
            .db
            .collection::<Document>("app_settings")
            .find(doc! {})
            .sort(doc! { "_id": 1 })
            .await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        docs.into_iter()
            .map(|d| {
                Ok(AppSettingRow {
                    key: d.get_str("_id").unwrap_or("").to_string(),
                    value: serde_json::from_str(d.get_str("value").unwrap_or("null"))
                        .unwrap_or(serde_json::Value::Null),
                    updated_at: d.get_i64("updated_at").unwrap_or(0),
                })
            })
            .collect()
    }
}

// ── ClawStorage constructor ──

impl crate::storage::ClawStorage {
    #[cfg(feature = "mongo")]
    pub async fn mongo(url: &str, db: &str) -> anyhow::Result<Self> {
        Ok(MongoBackend::new(url, db).await?.into_storage())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    // All MongoDB tests require a live instance. Run with:
    //   CLAW_TEST_MONGO_URL=mongodb://localhost:27017 \
    //   cargo test --features mongo -- --ignored --test-threads=1

    use super::*;

    fn mongo_url() -> Option<String> {
        std::env::var("CLAW_TEST_MONGO_URL").ok()
    }

    async fn test_storage() -> Option<crate::storage::ClawStorage> {
        let url = mongo_url()?;
        let db_name = format!("claw-test-{}", uuid::Uuid::new_v4());
        MongoBackend::new(&url, &db_name)
            .await
            .ok()
            .map(|b| b.into_storage())
    }

    #[tokio::test]
    #[ignore]
    async fn test_mongo_session_save_load() {
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
    async fn test_mongo_message_log_append_load() {
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
    async fn test_mongo_message_log_search() {
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
