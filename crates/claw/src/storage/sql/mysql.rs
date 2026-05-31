//! MySQL storage backend.

use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct MySqlBackend {
    pool: sqlx::MySqlPool,
}

impl MySqlBackend {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::MySqlPool::connect(url).await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    pub fn into_storage(self) -> ClawStorage {
        let arc = Arc::new(self);
        ClawStorage {
            sessions: Box::new(MySqlSessionStore { db: arc.clone() }),
            messages: Box::new(MySqlMessageStore { db: arc.clone() }),
            api_cache: Box::new(MySqlApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(MySqlPlanStepsStore { db: arc.clone() }),
            memory: Box::new(MySqlMemoryStore { db: arc.clone() }),
            stats: Box::new(MySqlStatsStore { db: arc.clone() }),
            skills: Box::new(MySqlSkillStore { db: arc.clone() }),
            tool_cache: Box::new(MySqlToolCacheStore { db: arc }),
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
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                session_id VARCHAR(36) NOT NULL,
                type VARCHAR(32) NOT NULL,
                text TEXT NOT NULL,
                name VARCHAR(128),
                args TEXT,
                result TEXT,
                reasoning TEXT,
                extra TEXT,
                created_at BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;
        sqlx::query("CREATE INDEX idx_messages_session ON messages(session_id)").execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_cache (
                session_id VARCHAR(36) PRIMARY KEY,
                messages LONGTEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS plan_steps (
                session_id VARCHAR(36) NOT NULL,
                step_order INT NOT NULL,
                description TEXT NOT NULL,
                done TINYINT NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, step_order),
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory (
                agent_id VARCHAR(64) PRIMARY KEY,
                data LONGTEXT NOT NULL
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

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
                has_tool_calls TINYINT NOT NULL,
                tool_call_count BIGINT NOT NULL,
                react_rounds BIGINT NOT NULL,
                success TINYINT NOT NULL,
                latency_ms BIGINT NOT NULL,
                estimated_cost_usd DOUBLE NOT NULL
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;
        sqlx::query("CREATE INDEX idx_token_ts ON token_records(timestamp)").execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS skills (
                agent_id VARCHAR(64) NOT NULL,
                name VARCHAR(128) NOT NULL,
                content LONGTEXT NOT NULL,
                parameters TEXT,
                PRIMARY KEY (agent_id, name)
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tool_cache (
                agent_id VARCHAR(64) NOT NULL,
                tool_name VARCHAR(128) NOT NULL,
                doc LONGTEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            ) ENGINE=InnoDB",
        ).execute(&self.pool).await?;

        Ok(())
    }
}

// Generate all 8 trait implementations
define_sql_stores!(
    sqlx::MySqlPool, MySqlBackend,
    MySqlSessionStore, MySqlMessageStore, MySqlApiCacheStore, MySqlPlanStepsStore,
    MySqlMemoryStore, MySqlStatsStore, MySqlSkillStore, MySqlToolCacheStore,
    "REPLACE INTO api_cache (session_id, messages) VALUES (?, ?)",
    "REPLACE INTO memory (agent_id, data) VALUES (?, ?)",
    "REPLACE INTO token_records (id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    "REPLACE INTO skills (agent_id, name, content, parameters) VALUES (?, ?, ?, ?)",
);
