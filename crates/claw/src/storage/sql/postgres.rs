//! PostgreSQL storage backend.

use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct PgBackend {
    pool: sqlx::PgPool,
}

impl PgBackend {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::PgPool::connect(url).await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    pub fn into_storage(self) -> ClawStorage {
        let arc = Arc::new(self);
        ClawStorage {
            sessions: Box::new(PgSessionStore { db: arc.clone() }),
            messages: Box::new(PgMessageStore { db: arc.clone() }),
            message_log: std::sync::Arc::new(PgMessageLogStore { db: arc.clone() }),
            api_cache: Box::new(PgApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(PgPlanStepsStore { db: arc.clone() }),
            memory: Box::new(PgMemoryStore { db: arc.clone() }),
            stats: Box::new(PgStatsStore { db: arc.clone() }),
            skills: Box::new(PgSkillStore { db: arc.clone() }),
            tool_cache: Box::new(PgToolCacheStore { db: arc }),
        }
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id VARCHAR(36) PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                agent_id VARCHAR(64) NOT NULL DEFAULT 'default',
                state VARCHAR(32) NOT NULL DEFAULT 'Active',
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                message_count BIGINT NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id BIGSERIAL PRIMARY KEY,
                session_id VARCHAR(36) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                type VARCHAR(32) NOT NULL,
                text TEXT NOT NULL DEFAULT '',
                name VARCHAR(128),
                args TEXT,
                result TEXT,
                reasoning TEXT,
                extra TEXT,
                created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_cache (
                session_id VARCHAR(36) PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
                messages TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS plan_steps (
                session_id VARCHAR(36) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                step_order INT NOT NULL,
                description TEXT NOT NULL,
                done SMALLINT NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, step_order)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory (
                agent_id VARCHAR(64) PRIMARY KEY,
                data TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS token_records (
                id VARCHAR(36) PRIMARY KEY,
                timestamp BIGINT NOT NULL,
                agent_id VARCHAR(64) NOT NULL,
                model VARCHAR(128) NOT NULL,
                provider VARCHAR(32) NOT NULL,
                prompt_tokens BIGINT NOT NULL,
                completion_tokens BIGINT NOT NULL,
                total_tokens BIGINT NOT NULL,
                has_tool_calls SMALLINT NOT NULL,
                tool_call_count BIGINT NOT NULL,
                react_rounds BIGINT NOT NULL,
                success SMALLINT NOT NULL,
                latency_ms BIGINT NOT NULL,
                estimated_cost_usd DOUBLE PRECISION NOT NULL,
                trace_id TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_token_ts ON token_records(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS skills (
                agent_id VARCHAR(64) NOT NULL,
                name VARCHAR(128) NOT NULL,
                content TEXT NOT NULL,
                parameters TEXT,
                PRIMARY KEY (agent_id, name)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tool_cache (
                agent_id VARCHAR(64) NOT NULL,
                tool_name VARCHAR(128) NOT NULL,
                doc TEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS message_log (
                id          BIGSERIAL    PRIMARY KEY,
                session_id  TEXT         NOT NULL,
                seq         BIGINT       NOT NULL,
                ts          BIGINT       NOT NULL,
                schema_v    INT          NOT NULL DEFAULT 1,
                payload     JSONB        NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_message_log_session_seq
             ON message_log (session_id, seq)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Generate all 8 trait implementations
define_sql_stores!(
    sqlx::PgPool,
    PgBackend,
    PgSessionStore,
    PgMessageStore,
    PgMessageLogStore,
    PgApiCacheStore,
    PgPlanStepsStore,
    PgMemoryStore,
    PgStatsStore,
    PgSkillStore,
    PgToolCacheStore,
    "INSERT INTO sessions (id, title, agent_id, state, created_at, updated_at, message_count) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT (id) DO UPDATE SET title=excluded.title, agent_id=excluded.agent_id, state=excluded.state, created_at=excluded.created_at, updated_at=excluded.updated_at, message_count=excluded.message_count",
    "INSERT INTO api_cache (session_id, messages) VALUES (?, ?) ON CONFLICT (session_id) DO UPDATE SET messages = excluded.messages",
    "INSERT INTO memory (agent_id, data) VALUES (?, ?) ON CONFLICT (agent_id) DO UPDATE SET data = excluded.data",
    "INSERT INTO token_records (id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd, trace_id) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT (id) DO UPDATE SET timestamp=excluded.timestamp, agent_id=excluded.agent_id, model=excluded.model, provider=excluded.provider, prompt_tokens=excluded.prompt_tokens, completion_tokens=excluded.completion_tokens, total_tokens=excluded.total_tokens, has_tool_calls=excluded.has_tool_calls, tool_call_count=excluded.tool_call_count, react_rounds=excluded.react_rounds, success=excluded.success, latency_ms=excluded.latency_ms, estimated_cost_usd=excluded.estimated_cost_usd, trace_id=excluded.trace_id",
    "INSERT INTO skills (agent_id, name, content, parameters) VALUES (?, ?, ?, ?) ON CONFLICT (agent_id, name) DO UPDATE SET content = excluded.content, parameters = excluded.parameters",
);
