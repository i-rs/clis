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
            message_log: std::sync::Arc::new(PgMessageLogStore { db: arc.clone() }),
            api_cache: Box::new(PgApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(PgPlanStepsStore { db: arc.clone() }),
            memory: Box::new(PgMemoryStore { db: arc.clone() }),
            stats: Box::new(PgStatsStore { db: arc.clone() }),
            skills: Box::new(PgSkillStore { db: arc.clone() }),
            tool_cache: Box::new(PgToolCacheStore { db: arc }),
        }
    }

    pub fn into_config_store(self) -> super::config_store::ConfigStore {
        let arc = Arc::new(self);
        super::config_store::ConfigStore {
            agent_configs: Box::new(PgAgentConfigStore { db: arc.clone() }),
            provider_configs: Box::new(PgProviderConfigStore { db: arc.clone() }),
            dashboard_users: Box::new(PgDashboardUserStore { db: arc.clone() }),
            mcp_servers: Box::new(PgMcpServerConfigStore { db: arc.clone() }),
            app_settings: Box::new(PgAppSettingsStore { db: arc }),
        }
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id VARCHAR(36) PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                agent_id VARCHAR(64) NOT NULL DEFAULT 'default',
                user_id VARCHAR(64) NOT NULL DEFAULT 'default',
                state VARCHAR(32) NOT NULL DEFAULT 'Active',
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                message_count BIGINT NOT NULL DEFAULT 0
            )",
        )
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
                user_id VARCHAR(64) NOT NULL DEFAULT 'default',
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
                payload     TEXT         NOT NULL,
                UNIQUE (session_id, seq)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_message_log_session_seq
             ON message_log (session_id, seq)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_configs (
                user_id VARCHAR(255) NOT NULL,
                agent_id VARCHAR(255) NOT NULL,
                provider_ref VARCHAR(255),
                provider VARCHAR(255) NOT NULL DEFAULT 'openai',
                api_key TEXT NOT NULL DEFAULT '',
                base_url VARCHAR(1024) NOT NULL DEFAULT '',
                model VARCHAR(255) NOT NULL DEFAULT '',
                enabled_tools_json JSONB NOT NULL DEFAULT '[]',
                system_prompt TEXT NOT NULL DEFAULT '',
                system_prompt_file VARCHAR(512),
                capabilities_json JSONB NOT NULL DEFAULT '[]',
                execution_mode VARCHAR(64) NOT NULL DEFAULT 'React',
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                PRIMARY KEY (user_id, agent_id)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS provider_configs (
                name VARCHAR(255) PRIMARY KEY,
                provider VARCHAR(255) NOT NULL,
                api_key TEXT NOT NULL DEFAULT '',
                base_url VARCHAR(1024) NOT NULL DEFAULT '',
                model VARCHAR(255) NOT NULL DEFAULT '',
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS dashboard_users (
                user_id VARCHAR(255) PRIMARY KEY,
                token_hash VARCHAR(255) NOT NULL,
                display_name VARCHAR(255) NOT NULL DEFAULT '',
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_dashboard_token_hash ON dashboard_users(token_hash)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS mcp_server_configs (
                user_id VARCHAR(255) NOT NULL,
                agent_id VARCHAR(255),
                name VARCHAR(255) NOT NULL,
                transport_type VARCHAR(64) NOT NULL DEFAULT 'stdio',
                command TEXT,
                args_json JSONB,
                url VARCHAR(1024),
                env_json JSONB,
                enabled BOOLEAN NOT NULL DEFAULT true,
                PRIMARY KEY (user_id, agent_id, name)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key VARCHAR(255) PRIMARY KEY,
                value_json JSONB NOT NULL DEFAULT 'null',
                updated_at BIGINT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Generate all trait implementations
define_sql_stores!(
    sqlx::PgPool,
    PgBackend,
    PgSessionStore,
    PgMessageLogStore,
    PgApiCacheStore,
    PgPlanStepsStore,
    PgMemoryStore,
    PgStatsStore,
    PgSkillStore,
    PgToolCacheStore,
    "INSERT INTO sessions (id, title, agent_id, user_id, state, created_at, updated_at, message_count) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET title=excluded.title, agent_id=excluded.agent_id, user_id=excluded.user_id, state=excluded.state, created_at=excluded.created_at, updated_at=excluded.updated_at, message_count=excluded.message_count",
    "INSERT INTO api_cache (session_id, messages) VALUES ($1, $2) ON CONFLICT (session_id) DO UPDATE SET messages = excluded.messages",
    "INSERT INTO memory (agent_id, data) VALUES ($1, $2) ON CONFLICT (agent_id) DO UPDATE SET data = excluded.data",
    "INSERT INTO token_records (id, timestamp, user_id, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd, trace_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) ON CONFLICT (id) DO UPDATE SET timestamp=excluded.timestamp, user_id=excluded.user_id, agent_id=excluded.agent_id, model=excluded.model, provider=excluded.provider, prompt_tokens=excluded.prompt_tokens, completion_tokens=excluded.completion_tokens, total_tokens=excluded.total_tokens, has_tool_calls=excluded.has_tool_calls, tool_call_count=excluded.tool_call_count, react_rounds=excluded.react_rounds, success=excluded.success, latency_ms=excluded.latency_ms, estimated_cost_usd=excluded.estimated_cost_usd, trace_id=excluded.trace_id",
    "INSERT INTO skills (agent_id, name, content, parameters) VALUES ($1, $2, $3, $4) ON CONFLICT (agent_id, name) DO UPDATE SET content = excluded.content, parameters = excluded.parameters",
    "SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = $1 FOR UPDATE",
    "$1", "$2", "$3", "$4", "$5",
);

define_config_sql_stores!(
    sqlx::PgPool,
    PgBackend,
    PgAgentConfigStore,
    PgProviderConfigStore,
    PgDashboardUserStore,
    PgMcpServerConfigStore,
    PgAppSettingsStore,
    "INSERT INTO agent_configs (user_id, agent_id, provider_ref, provider, api_key, base_url, model, enabled_tools_json, system_prompt, system_prompt_file, capabilities_json, execution_mode, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) ON CONFLICT (user_id, agent_id) DO UPDATE SET provider_ref=EXCLUDED.provider_ref, provider=EXCLUDED.provider, api_key=EXCLUDED.api_key, base_url=EXCLUDED.base_url, model=EXCLUDED.model, enabled_tools_json=EXCLUDED.enabled_tools_json, system_prompt=EXCLUDED.system_prompt, system_prompt_file=EXCLUDED.system_prompt_file, capabilities_json=EXCLUDED.capabilities_json, execution_mode=EXCLUDED.execution_mode, updated_at=EXCLUDED.updated_at",
    "INSERT INTO provider_configs (name, provider, api_key, base_url, model, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (name) DO UPDATE SET provider=EXCLUDED.provider, api_key=EXCLUDED.api_key, base_url=EXCLUDED.base_url, model=EXCLUDED.model, updated_at=EXCLUDED.updated_at",
    "INSERT INTO dashboard_users (user_id, token_hash, display_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (user_id) DO UPDATE SET token_hash=EXCLUDED.token_hash, display_name=EXCLUDED.display_name, updated_at=EXCLUDED.updated_at",
    "INSERT INTO mcp_server_configs (user_id, agent_id, name, transport_type, command, args_json, url, env_json, enabled) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (user_id, agent_id, name) DO UPDATE SET transport_type=EXCLUDED.transport_type, command=EXCLUDED.command, args_json=EXCLUDED.args_json, url=EXCLUDED.url, env_json=EXCLUDED.env_json, enabled=EXCLUDED.enabled",
    "INSERT INTO app_settings (key, value_json, updated_at) VALUES ($1, $2, $3) ON CONFLICT (key) DO UPDATE SET value_json=EXCLUDED.value_json, updated_at=EXCLUDED.updated_at",
    "$1", "$2", "$3",
);
