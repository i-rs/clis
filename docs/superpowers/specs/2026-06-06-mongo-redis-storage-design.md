# MongoDB + Redis Storage Backend Design

**Date:** 2026-06-06
**Status:** Draft
**Depends on:** Append-only MessageLog refactor (complete)

## Overview

Add two new storage backends to the `ClawStorage` abstraction:

| Backend | Crate | Feature gate | Module |
|---------|-------|-------------|--------|
| MongoDB | `mongodb` (official driver) | `mongo` | `storage/mongo.rs` |
| Redis | `redis` (with `tokio-comp`) | `redis` | `storage/redis.rs` |

Both backends implement all 8 traits: `SessionRepo`, `MessageLog`, `ApiCacheRepo`,
`PlanStepsRepo`, `MemoryRepo`, `StatsRepo`, `SkillRepo`, `ToolCacheRepo`.

---

## 1. Shared Infrastructure

### 1.1 Feature gates (`Cargo.toml`)

```toml
[dependencies]
mongodb = { version = "3", optional = true }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"], optional = true }

[features]
mongo = ["dep:mongodb"]
redis = ["dep:redis"]
```

### 1.2 Module structure

```
storage/
├── mod.rs          # ClawStorage, traits, StorageBackend enum
├── file.rs         # File backend (always available)
├── sql/
│   ├── mod.rs      # define_sql_stores! macro
│   ├── sqlite.rs
│   ├── mysql.rs
│   └── postgres.rs
├── mongo.rs        # NEW: MongoDB backend (single file)
└── redis.rs        # NEW: Redis backend (single file)
```

Each new module is gated independently:
```rust
// storage/mod.rs
#[cfg(feature = "mongo")]  pub mod mongo;
#[cfg(feature = "redis")]  pub mod redis;
```

### 1.3 StorageBackend + StorageConfig changes

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum StorageBackend {
    #[default]
    File,
    Sqlite,
    Mysql,
    Postgres,
    Mongo,
    Redis,                    // ← NEW
}

pub struct StorageConfig {
    pub backend: StorageBackend,
    pub file_dir: Option<PathBuf>,
    pub sqlite_path: Option<PathBuf>,
    pub sql_url: Option<String>,
    pub mongo_url: Option<String>,         // already exists
    pub mongo_database: Option<String>,    // already exists
    pub redis_url: Option<String>,         // ← NEW: e.g. "redis://127.0.0.1:6379/0"
}
```

### 1.4 ClawStorage constructors

```rust
impl ClawStorage {
    #[cfg(feature = "mongo")]
    pub async fn mongo(url: &str, db: &str) -> anyhow::Result<Self> { ... }

    #[cfg(feature = "redis")]
    pub async fn redis(url: &str) -> anyhow::Result<Self> { ... }
}
```

---

## 2. MongoDB Backend

### 2.1 Design rationale

MongoDB is a natural fit for this schema: every trait stores JSON-like documents,
queries are composable, and indexes handle the access patterns efficiently. No
macro is needed — the driver API is consistent and doesn't have placeholder
dialect issues like SQL.

### 2.2 Connection

```rust
#[derive(Clone)]
pub struct MongoBackend {
    db: mongodb::Database,
}
```

- Connect: `mongodb::Client::with_uri_str(url).await?.database(db_name)`.
- Each wrapper struct holds `Arc<MongoBackend>` (matching SQL pattern).
- Indexes created on first connect (idempotent `create_index` with partial filter
  expressions where appropriate).

### 2.3 Collection mapping

| Trait | Collection | Document shape |
|-------|-----------|---------------|
| SessionRepo | `sessions` | `{_id: id, title, agent_id, state, created_at, updated_at, message_count}` |
| MessageLog | `message_log` | `{_id: ObjectId, session_id, seq, ts, schema_v, payload: {...Message}}` |
| ApiCacheRepo | `api_cache` | `{_id: session_id, messages: [...]}` |
| PlanStepsRepo | `plan_steps` | `{_id: session_id, steps: [{description, done}, ...]}` |
| MemoryRepo | `memory` | `{_id: agent_id, data: {...CrossSessionMemory}}` |
| StatsRepo | `token_records` | `{_id: id, timestamp, agent_id, model, ...}` |
| SkillRepo | `skills` | `{_id: {agent_id, name}, agent_id, name, content, parameters}` |
| ToolCacheRepo | `tool_cache` | `{_id: agent_id, docs: {tool_name: doc, ...}}` |

### 2.4 Indexes

```javascript
// message_log: primary access pattern is "last N messages in session"
db.message_log.createIndex({ session_id: 1, seq: 1 }, { unique: true })

// token_records: range scans by timestamp
db.token_records.createIndex({ timestamp: 1 })

// skills: list by agent
db.skills.createIndex({ agent_id: 1, name: 1 })

// sessions: ordered list by updated_at
db.sessions.createIndex({ updated_at: -1 })
```

### 2.5 Trait implementations

#### SessionRepo

```rust
async fn load_all(&self) -> Result<Vec<SessionMeta>> {
    // Sort by updated_at DESC, project away _id (use id field = _id)
    let cursor = self.db.collection("sessions")
        .find(doc! {})
        .sort(doc! { "updated_at": -1 })
        .await?;
    cursor.map(|doc| doc_to_session_meta(&doc?)).try_collect().await
}

async fn save_all(&self, sessions: &[SessionMeta]) -> Result<()> {
    // Full-replace: collect incoming IDs, delete missing, upsert all
    let incoming_ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
    self.db.collection("sessions")
        .delete_many(doc! { "_id": { "$nin": &incoming_ids } }).await?;

    let docs: Vec<Document> = sessions.iter().map(session_meta_to_doc).collect();
    let models: Vec<UpdateOneModel> = docs.iter().map(|d| {
        UpdateOneModel::new()
            .filter(doc! { "_id": d.get("_id").unwrap().clone() })
            .update(doc! { "$set": d.clone() })
            .upsert(true)
    }).collect();
    self.db.collection("sessions").bulk_write(models).await?;
    Ok(())
}
```

#### MessageLog (critical path)

```rust
async fn append_batch(&self, session_id: &str, messages: &[Message]) -> Result<()> {
    if messages.is_empty() { return Ok(()); }

    // Atomically get next seq via findOneAndUpdate on a counter document
    let counter = self.db.collection::<Document>("seq_counters");
    let result = counter.find_one_and_update(
        doc! { "session_id": session_id },
        doc! { "$inc": { "next_seq": messages.len() as i64 } },
        FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::BEFORE)
            .build(),
    ).await?;

    let start_seq = result.and_then(|d| d.get_i64("next_seq").ok()).unwrap_or(0) + 1;

    let docs: Vec<Document> = messages.iter().enumerate().map(|(i, msg)| {
        let rec = StoredRecord::from_message(msg).unwrap();
        doc! {
            "session_id": session_id,
            "seq": start_seq + i as i64,
            "ts": rec.ts,
            "schema_v": rec.schema_v as i32,
            "payload": mongodb::bson::to_bson(&rec.payload).unwrap(),
        }
    }).collect();

    self.db.collection("message_log").insert_many(docs).await?;
    Ok(())
}

async fn load(&self, session_id: &str, limit: usize) -> Result<Vec<Message>> {
    let limit = limit.min(i64::MAX as usize) as i64;
    let cursor = self.db.collection("message_log")
        .find(doc! { "session_id": session_id })
        .sort(doc! { "seq": -1 })
        .limit(limit)
        .await?;
    let mut records: Vec<Document> = cursor.try_collect().await?;
    records.reverse(); // back to ascending order
    records.iter().map(|d| {
        let payload = d.get_document("payload")?;
        Ok(serde_json::from_value(mongodb::bson::serde::bson_to_json(payload))?)
    }).collect()
}

async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
    let q = query.to_lowercase();
    // Use $regex for case-insensitive substring match on payload fields
    let filter = doc! {
        "$or": [
            { "payload.type": { "$in": ["user", "assistant", "error"] },
              "payload.text": { "$regex": &q, "$options": "i" } },
            { "payload.type": "tool_call",
              "payload.name": { "$regex": &q, "$options": "i" } },
        ]
    };
    // ... iterate sessions, build excerpts + context (same algorithm as SQL/file)
}
```

**Key decision**: `seq_counters` collection uses `findOneAndUpdate` with `$inc` for
atomic sequence generation — avoids the transaction overhead that SQL needs for
`SELECT MAX(seq) + INSERT`.

#### StatsRepo

```rust
async fn upsert_batch(&self, records: &[TokenRecord]) -> Result<()> {
    let models: Vec<UpdateOneModel> = records.iter().map(|r| {
        let doc = mongodb::bson::to_document(r).unwrap();
        UpdateOneModel::new()
            .filter(doc! { "_id": &r.id })
            .update(doc! { "$set": doc })
            .upsert(true)
    }).collect();
    self.db.collection("token_records").bulk_write(models).await?;
    Ok(())
}

async fn read_range(&self, from: Option<i64>, to: Option<i64>) -> Result<Vec<TokenRecord>> {
    let mut filter = doc! {};
    if let Some(f) = from { filter.insert("timestamp", doc! { "$gte": f }); }
    if let Some(t) = to   { filter.insert("timestamp", doc! { "$lte": t }); }
    // merge if both present...
    let cursor = self.db.collection::<Document>("token_records")
        .find(filter).sort(doc! { "timestamp": 1 }).await?;
    cursor.map(|d| Ok(bson::from_document(d)?)).try_collect().await
}
```

#### Remaining traits

`ApiCacheRepo`, `PlanStepsRepo`, `MemoryRepo`, `SkillRepo`, `ToolCacheRepo` —
straightforward `findOne` / `replaceOne` / `deleteOne` on the `_id` field.
No special patterns needed.

---

## 3. Redis Backend

### 3.1 Design rationale

Redis is **not** a document store. Each data type needs a different Redis structure.
The design uses:
- **Hashes** for single-document entities (memory, tool_cache, api_cache)
- **Sorted Sets** for ordered entities (message_log, plan_steps)
- **Lists** for append-only logs (token_records)
- **Sets** for membership queries (skills index)
- **RediSearch** (optional, via module) for full-text search

### 3.2 Connection

```rust
#[derive(Clone)]
pub struct RedisBackend {
    pool: redis::aio::ConnectionManager,     // multiplexed async connection
}
```

- Connect: `redis::Client::open(url)?` → `ConnectionManager::new(client).await?`.
- `ConnectionManager` auto-reconnects on failure (unlike `MultiplexedConnection`).
- Each wrapper struct holds `Arc<RedisBackend>`.

### 3.3 Key schema

```
# SessionRepo
claw:sessions                    → ZSET (score=updated_at, member=session_id)
claw:session:{id}                → HASH (title, agent_id, state, created_at, updated_at, message_count)

# MessageLog
claw:msg:{session_id}            → ZSET (score=seq, value=JSON(StoredRecord))
claw:msg:seq:{session_id}        → COUNTER (next seq number, initialized to 0)

# ApiCacheRepo
claw:apicache:{session_id}       → STRING (JSON array of API messages)

# PlanStepsRepo
claw:plan:{session_id}           → LIST of JSON(PlanStep)

# MemoryRepo
claw:memory:{agent_id}           → STRING (JSON(CrossSessionMemory))

# StatsRepo
claw:stats:records               → LIST of JSON(TokenRecord)
claw:stats:ts                    → SORTED SET (score=timestamp, member=id) for range queries

# SkillRepo
claw:skills:{agent_id}           → SET of skill names
claw:skill:{agent_id}:{name}     → HASH (content, parameters)

# ToolCacheRepo
claw:toolcache:{agent_id}        → HASH (field=tool_name, value=doc)

# Search index
claw:search:index                → SORTED SET or RediSearch index
```

### 3.4 Trait implementations

#### SessionRepo

```rust
async fn load_all(&self) -> Result<Vec<SessionMeta>> {
    let mut conn = self.pool.clone();
    // ZRANGEBYSCORE gets all session IDs ordered by updated_at DESC
    let ids: Vec<String> = conn.zrevrange("claw:sessions", 0, -1).await?;
    let mut sessions = Vec::with_capacity(ids.len());
    for id in &ids {
        let data: Option<String> = conn.hget(format!("claw:session:{id}"), "__json").await?;
        if let Some(json) = data {
            sessions.push(serde_json::from_str(&json)?);
        }
    }
    Ok(sessions)
}

async fn save_all(&self, sessions: &[SessionMeta]) -> Result<()> {
    let mut conn = self.pool.clone();
    let existing: Vec<String> = conn.zrange("claw:sessions", 0, -1).await?;
    let incoming_ids: HashSet<&str> = sessions.iter().map(|s| s.id.as_str()).collect();

    // Delete removed sessions
    let to_delete: Vec<&str> = existing.iter()
        .map(|s| s.as_str())
        .filter(|id| !incoming_ids.contains(*id))
        .collect();
    // Use pipeline for batch
    let mut pipe = redis::pipe();
    pipe.atomic();
    for id in &to_delete {
        pipe.zrem("claw:sessions", *id).ignore();
        pipe.del(format!("claw:session:{id}")).ignore();
    }
    // Upsert all sessions
    for s in sessions {
        let json = serde_json::to_string(s)?;
        pipe.hset(format!("claw:session:{}", s.id), "__json", json).ignore();
        pipe.zadd("claw:sessions", &s.id, s.updated_at).ignore();
    }
    pipe.query_async(&mut conn).await?;
    Ok(())
}
```

#### MessageLog

```rust
async fn append_batch(&self, session_id: &str, messages: &[Message]) -> Result<()> {
    if messages.is_empty() { return Ok(()); }
    let mut conn = self.pool.clone();
    let seq_key = format!("claw:msg:seq:{session_id}");
    let zset_key = format!("claw:msg:{session_id}");

    // Lua script: atomically INCRBY seq counter + ZADD all records
    let script = redis::Script::new(r#"
        local start_seq = redis.call('INCRBY', KEYS[1], #ARGV / 2)
        start_seq = start_seq - #ARGV / 2 + 1
        for i = 1, #ARGV, 2 do
            local seq = start_seq + (i - 1) / 2
            redis.call('ZADD', KEYS[2], seq, ARGV[i+1])
        end
        return start_seq
    "#);

    let mut args: Vec<String> = Vec::new();
    for msg in messages {
        let rec = StoredRecord::from_message(msg)?;
        // Overwrite seq with placeholder (Lua assigns real seq)
        let mut rec = rec;
        rec.seq = 0; // will be replaced
        args.push(serde_json::to_string(&rec)?);
    }

    script.key(seq_key).key(zset_key);
    for a in &args { script.arg(a); }
    script.invoke_async(&mut conn).await?;
    Ok(())
}

async fn load(&self, session_id: &str, limit: usize) -> Result<Vec<Message>> {
    let mut conn = self.pool.clone();
    let key = format!("claw:msg:{session_id}");
    // ZREVRANGE 0 limit-1 → reverse → oldest first
    let raw: Vec<String> = conn.zrevrange(&key, 0, (limit.saturating_sub(1)) as isize).await?;
    let mut records: Vec<StoredRecord> = raw.iter()
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();
    records.reverse();
    records.iter().map(|r| r.to_message().ok_or(anyhow::anyhow!("decode error"))).collect()
}

async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
    // Without RediSearch: scan all session ZSETs, filter client-side
    // With RediSearch: FT.SEARCH on index
    //
    // Phase 1 (MVP): client-side scan. Slow but correct.
    // Phase 2 (optimization): require RediSearch module.
    let q = query.to_lowercase();
    let mut conn = self.pool.clone();
    let session_ids: Vec<String> = conn.zrange("claw:sessions", 0, -1).await?;
    let mut results = Vec::new();
    for sid in &session_ids {
        let key = format!("claw:msg:{sid}");
        let raw: Vec<String> = conn.zrange(&key, 0, -1).await?;
        // Parse each, check payload.type + payload.text/name
        // ... (same algorithm as file/SQL backends)
        if results.len() >= max_results { break; }
    }
    Ok(results)
}

async fn delete_session(&self, session_id: &str) -> Result<()> {
    let mut conn = self.pool.clone();
    let mut pipe = redis::pipe();
    pipe.atomic();
    pipe.del(format!("claw:msg:{session_id}")).ignore();
    pipe.del(format!("claw:msg:seq:{session_id}")).ignore();
    pipe.query_async(&mut conn).await?;
    Ok(())
}
```

#### StatsRepo

```rust
async fn upsert_batch(&self, records: &[TokenRecord]) -> Result<()> {
    // HSET is naturally idempotent — store each record as a hash field
    let mut conn = self.pool.clone();
    let mut pipe = redis::pipe();
    pipe.atomic();
    for r in records {
        let json = serde_json::to_string(r)?;
        pipe.hset("claw:stats:records", &r.id, json).ignore();
        pipe.zadd("claw:stats:ts", &r.id, r.timestamp).ignore();
    }
    pipe.query_async(&mut conn).await?;
    Ok(())
}

async fn read_range(&self, from: Option<i64>, to: Option<i64>) -> Result<Vec<TokenRecord>> {
    let mut conn = self.pool.clone();
    let (min, max) = match (from, to) {
        (Some(f), Some(t)) => (f, t),
        (Some(f), None) => (f, i64::MAX),
        (None, Some(t)) => (0, t),
        (None, None) => (0, i64::MAX),
    };
    let ids: Vec<String> = conn.zrangebyscore("claw:stats:ts", min, max).await?;
    if ids.is_empty() { return Ok(Vec::new()); }
    let vals: Vec<Option<String>> = conn.hget("claw:stats:records", &ids).await?;
    vals.into_iter()
        .filter_map(|v| v.and_then(|s| serde_json::from_str(&s).ok()))
        .collect::<Vec<_>>()
        .pipe(Ok)  // TODO: sort by timestamp
}

async fn prune(&self, keep_days: u32) -> Result<usize> {
    let cutoff = chrono::Local::now().timestamp() - (keep_days as i64 * 86400);
    let mut conn = self.pool.clone();
    // Get IDs older than cutoff
    let ids: Vec<String> = conn.zrangebyscore("claw:stats:ts", 0, cutoff).await?;
    if ids.is_empty() { return Ok(0); }
    let mut pipe = redis::pipe();
    pipe.atomic();
    for id in &ids {
        pipe.hdel("claw:stats:records", id).ignore();
        pipe.zrem("claw:stats:ts", id).ignore();
    }
    pipe.query_async(&mut conn).await?;
    Ok(ids.len())
}
```

#### Remaining traits

- **ApiCacheRepo** — `SET`/`GET`/`DEL` on `claw:apicache:{session_id}`.
- **PlanStepsRepo** — `DEL` + `RPUSH` for save, `LRANGE` for load.
- **MemoryRepo** — `SET`/`GET` on `claw:memory:{agent_id}`.
- **SkillRepo** — `SADD`/`SMEMBERS` for index, `HSET`/`HGET` for skill data.
- **ToolCacheRepo** — `HSET`/`HGETALL` on `claw:toolcache:{agent_id}`.

---

## 4. Implementation Plan

### Phase 1: MongoDB backend

1. Add `mongodb` dependency to `Cargo.toml`
2. Create `storage/mongo.rs` with `MongoBackend` struct
3. Implement all 8 trait impls (single file, ~600 LOC estimated)
4. Add `ClawStorage::mongo()` constructor
5. Add indexes in `MongoBackend::new()`
6. Tests: `test_mongo_*` (require running MongoDB — skip in CI via `#[ignore]`)
7. Wire into `StorageConfig` dispatch

### Phase 2: Redis backend

1. Add `redis` dependency to `Cargo.toml`
2. Create `storage/redis.rs` with `RedisBackend` struct
3. Implement all 8 trait impls (single file, ~700 LOC estimated — Lua scripts add length)
4. Add `ClawStorage::redis()` constructor
5. Tests: `test_redis_*` (require running Redis — skip in CI via `#[ignore]`)
6. Wire into `StorageConfig` dispatch

### Phase 3: Config + docs

1. Update `StorageConfig` / `StorageBackend` enum
2. Update config.toml parsing to support `redis_url`
3. Update `ClawStorage::from_config()` to dispatch new backends
4. Update VitePress docs if needed

---

## 5. Risk Assessment

| Risk | Mitigation |
|------|------------|
| MongoDB `seq_counters` race condition | `findOneAndUpdate` with `$inc` is atomic by design |
| Redis `search` is slow without RediSearch | Document limitation; provide RediSearch path as opt-in via feature |
| Redis key explosion | Document TTL strategy; use `SCAN` (not `KEYS`) for maintenance |
| BSON ↔ JSON conversion overhead | Use `mongodb::bson::serde` for direct (de)serialization where possible |
| Redis Lua script portability | Test against Redis 6+ and Valkey 7+ |
| Test infrastructure | Tests require live DB instances; use `#[ignore]` + `CLAW_TEST_MONGO_URL` env var pattern |

---

## 6. Testing Strategy

```rust
#[cfg(feature = "mongo")]
#[cfg(test)]
mod mongo_tests {
    use super::*;

    fn mongo_url() -> Option<String> = std::env::var("CLAW_TEST_MONGO_URL").ok();

    #[tokio::test]
    #[ignore]  // requires live MongoDB
    async fn test_mongo_session_save_load() { ... }

    #[tokio::test]
    #[ignore]
    async fn test_mongo_message_log_append_load() { ... }
    // ... etc
}
```

Run locally: `CLAW_TEST_MONGO_URL=mongodb://localhost:27017 cargo test --features mongo -- --ignored`
CI: only run non-ignored tests (matching existing SQL backend pattern).

---

## 7. LOC Estimates

| Component | LOC |
|-----------|-----|
| `storage/mongo.rs` | ~600 |
| `storage/redis.rs` | ~700 |
| `Cargo.toml` changes | ~10 |
| `storage/mod.rs` (StorageConfig, dispatch) | ~40 |
| Tests | ~400 |
| **Total** | **~1750** |
