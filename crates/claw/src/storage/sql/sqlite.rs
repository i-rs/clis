//! SQLite storage backend.

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct SqliteBackend {
    pool: sqlx::SqlitePool,
}

impl SqliteBackend {
    pub async fn new(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // PRAGMA foreign_keys is per-connection; SqliteConnectOptions.foreign_keys(true)
        // ensures every connection in the pool has FK enforced, not just the first.
        let url = format!("sqlite:{}", path.display());
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = sqlx::SqlitePool::connect_with(options).await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    #[cfg(test)]
    pub async fn new_in_memory() -> anyhow::Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true);
        let pool = sqlx::SqlitePool::connect_with(options).await?;
        let backend = Self { pool };
        backend.migrate().await?;
        Ok(backend)
    }

    pub fn into_storage(self) -> ClawStorage {
        let arc = Arc::new(self);
        ClawStorage {
            sessions: Box::new(SqliteSessionStore { db: arc.clone() }),
            messages: Box::new(SqliteMessageStore { db: arc.clone() }),
            api_cache: Box::new(SqliteApiCacheStore { db: arc.clone() }),
            plan_steps: Box::new(SqlitePlanStepsStore { db: arc.clone() }),
            memory: Box::new(SqliteMemoryStore { db: arc.clone() }),
            stats: Box::new(SqliteStatsStore { db: arc.clone() }),
            skills: Box::new(SqliteSkillStore { db: arc.clone() }),
            tool_cache: Box::new(SqliteToolCacheStore { db: arc }),
        }
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query("CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, title TEXT NOT NULL, agent_id TEXT NOT NULL DEFAULT 'default', state TEXT NOT NULL DEFAULT 'Active', created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, message_count INTEGER NOT NULL DEFAULT 0)").execute(&self.pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE, type TEXT NOT NULL, text TEXT NOT NULL DEFAULT '', name TEXT, args TEXT, result TEXT, reasoning TEXT, extra TEXT, created_at INTEGER NOT NULL DEFAULT (unixepoch()))").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS api_cache (session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE, messages TEXT NOT NULL)").execute(&self.pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS plan_steps (session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE, step_order INTEGER NOT NULL, description TEXT NOT NULL, done INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (session_id, step_order))").execute(&self.pool).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS memory (agent_id TEXT PRIMARY KEY, data TEXT NOT NULL)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS token_records (id TEXT PRIMARY KEY, timestamp INTEGER NOT NULL, agent_id TEXT NOT NULL, model TEXT NOT NULL, provider TEXT NOT NULL, prompt_tokens INTEGER NOT NULL, completion_tokens INTEGER NOT NULL, total_tokens INTEGER NOT NULL, has_tool_calls INTEGER NOT NULL, tool_call_count INTEGER NOT NULL, react_rounds INTEGER NOT NULL, success INTEGER NOT NULL, latency_ms INTEGER NOT NULL, estimated_cost_usd REAL NOT NULL, trace_id TEXT NOT NULL DEFAULT '')").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_token_ts ON token_records(timestamp)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS skills (agent_id TEXT NOT NULL, name TEXT NOT NULL, content TEXT NOT NULL, parameters TEXT, PRIMARY KEY (agent_id, name))").execute(&self.pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS tool_cache (agent_id TEXT NOT NULL, tool_name TEXT NOT NULL, doc TEXT NOT NULL, PRIMARY KEY (agent_id, tool_name))").execute(&self.pool).await?;
        Ok(())
    }
}

// Generate all 8 trait implementations
define_sql_stores!(
    sqlx::SqlitePool,
    SqliteBackend,
    SqliteSessionStore,
    SqliteMessageStore,
    SqliteApiCacheStore,
    SqlitePlanStepsStore,
    SqliteMemoryStore,
    SqliteStatsStore,
    SqliteSkillStore,
    SqliteToolCacheStore,
    "INSERT INTO sessions (id, title, agent_id, state, created_at, updated_at, message_count) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET title=excluded.title, agent_id=excluded.agent_id, state=excluded.state, created_at=excluded.created_at, updated_at=excluded.updated_at, message_count=excluded.message_count",
    "INSERT OR REPLACE INTO api_cache (session_id, messages) VALUES (?, ?)",
    "INSERT OR REPLACE INTO memory (agent_id, data) VALUES (?, ?)",
    "INSERT OR REPLACE INTO token_records (id, timestamp, agent_id, model, provider, prompt_tokens, completion_tokens, total_tokens, has_tool_calls, tool_call_count, react_rounds, success, latency_ms, estimated_cost_usd, trace_id) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    "INSERT OR REPLACE INTO skills (agent_id, name, content, parameters) VALUES (?, ?, ?, ?)",
);

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_storage() -> ClawStorage {
        SqliteBackend::new_in_memory().await.unwrap().into_storage()
    }

    #[tokio::test]
    async fn test_session_save_load() {
        let s = test_storage().await;
        s.sessions
            .save_all(&[crate::session::SessionMeta {
                id: "s1".into(),
                title: "Hi".into(),
                agent_id: "d".into(),
                state: crate::session::SessionState::Active,
                created_at: 1,
                updated_at: 2,
                message_count: 0,
            }])
            .await
            .unwrap();
        assert_eq!(s.sessions.load_all().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_message_append_load() {
        let s = test_storage().await;
        s.sessions
            .save_all(&[crate::session::SessionMeta {
                id: "s1".into(),
                title: "T".into(),
                agent_id: "d".into(),
                state: crate::session::SessionState::Active,
                created_at: 1,
                updated_at: 2,
                message_count: 0,
            }])
            .await
            .unwrap();
        s.messages
            .append("s1", &serde_json::json!({"type":"user","text":"hi"}))
            .await
            .unwrap();
        assert_eq!(s.messages.load("s1", 10).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_memory_save_load() {
        let s = test_storage().await;
        let mut m = crate::memory::CrossSessionMemory::default_memory();
        m.set_user_name("A");
        s.memory.save("a", &m).await.unwrap();
        assert!(s.memory.load("a").await.unwrap().unwrap().has_user_profile());
    }
}
