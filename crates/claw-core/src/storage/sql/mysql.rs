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
            message_log: std::sync::Arc::new(MySqlMessageLogStore { db: arc.clone() }),
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
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_cache (
                session_id VARCHAR(36) PRIMARY KEY,
                messages LONGTEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            ) ENGINE=InnoDB",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS plan_steps (
                session_id VARCHAR(36) NOT NULL,
                step_order INT NOT NULL,
                description TEXT NOT NULL,
                done TINYINT NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, step_order),
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            ) ENGINE=InnoDB",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory (
                agent_id VARCHAR(64) PRIMARY KEY,
                data LONGTEXT NOT NULL
            ) ENGINE=InnoDB",
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
                has_tool_calls TINYINT NOT NULL,
                tool_call_count BIGINT NOT NULL,
                react_rounds BIGINT NOT NULL,
                success TINYINT NOT NULL,
                latency_ms BIGINT NOT NULL,
                estimated_cost_usd DOUBLE NOT NULL,
                trace_id VARCHAR(64) NOT NULL DEFAULT ''
            ) ENGINE=InnoDB",
        )
        .execute(&self.pool)
        .await?;
        // MySQL doesn't support CREATE INDEX IF NOT EXISTS; wrap in a procedural
        // guard so migrate() stays idempotent.
        sqlx::query(
            "SELECT 1 FROM information_schema.statistics
             WHERE table_schema = DATABASE()
               AND table_name = 'token_records'
               AND index_name = 'idx_token_ts'
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        // CREATE INDEX IF NOT EXISTS is not supported by MySQL; try creation
        // and ignore only "duplicate key" errors from idempotent re-runs.
        if let Err(e) = sqlx::query("CREATE INDEX idx_token_ts ON token_records(timestamp)")
            .execute(&self.pool)
            .await
        {
            let msg = e.to_string();
            if !msg.contains("Duplicate") && !msg.contains("already exists") {
                return Err(anyhow::anyhow!("创建 token_records 索引失败: {}", e));
            }
        }

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS skills (
                agent_id VARCHAR(64) NOT NULL,
                name VARCHAR(128) NOT NULL,
                content LONGTEXT NOT NULL,
                parameters TEXT,
                PRIMARY KEY (agent_id, name)
            ) ENGINE=InnoDB",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tool_cache (
                agent_id VARCHAR(64) NOT NULL,
                tool_name VARCHAR(128) NOT NULL,
                doc LONGTEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            ) ENGINE=InnoDB",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS message_log (
                id          BIGINT       AUTO_INCREMENT PRIMARY KEY,
                session_id  VARCHAR(64)  NOT NULL,
                seq         BIGINT       NOT NULL,
                ts          BIGINT       NOT NULL,
                schema_v    INT          NOT NULL DEFAULT 1,
                payload     TEXT         NOT NULL,
                UNIQUE KEY idx_message_log_session_seq (session_id, seq)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Generate all trait implementations
define_sql_stores!(
    sqlx::MySqlPool,
    MySqlBackend,
    MySqlSessionStore,
    MySqlMessageLogStore,
    MySqlApiCacheStore,
    MySqlPlanStepsStore,
    MySqlMemoryStore,
    MySqlStatsStore,
    MySqlSkillStore,
    MySqlToolCacheStore,
    "INSERT INTO sessions (id, title, agent_id, state, created_at, updated_at, message_count) VALUES (?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE title=VALUES(title), agent_id=VALUES(agent_id), state=VALUES(state), created_at=VALUES(created_at), updated_at=VALUES(updated_at), message_count=VALUES(message_count)",
    "INSERT INTO api_cache (session_id, messages) VALUES (?, ?) ON DUPLICATE KEY UPDATE messages=VALUES(messages)",
    "INSERT INTO memory (agent_id, data) VALUES (?, ?) ON DUPLICATE KEY UPDATE data=VALUES(data)",
    "INSERT INTO token_records (id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd, trace_id) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE timestamp=VALUES(timestamp), agent_id=VALUES(agent_id), model=VALUES(model), provider=VALUES(provider), prompt_tokens=VALUES(prompt_tokens), completion_tokens=VALUES(completion_tokens), total_tokens=VALUES(total_tokens), has_tool_calls=VALUES(has_tool_calls), tool_call_count=VALUES(tool_call_count), react_rounds=VALUES(react_rounds), success=VALUES(success), latency_ms=VALUES(latency_ms), estimated_cost_usd=VALUES(estimated_cost_usd), trace_id=VALUES(trace_id)",
    "INSERT INTO skills (agent_id, name, content, parameters) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE content=VALUES(content), parameters=VALUES(parameters)",
    "SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ? FOR UPDATE",
    "?", "?", "?", "?", "?",
);
