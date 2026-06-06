//! Runtime config store — agent/provider/dashboard/mcp/settings.
//!
//! Replaces config.toml sections ([agents], [providers], etc.) when
//! using a DB backend. File backend delegates to config.toml (unchanged).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Row types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigRow {
    pub user_id: String,
    pub agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_ref: Option<String>,
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub enabled_tools: Vec<String>,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_file: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub execution_mode: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for AgentConfigRow {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            user_id: "default".into(), agent_id: "default".into(),
            provider_ref: None, provider: "openai".into(),
            api_key: String::new(), base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(), enabled_tools: vec![],
            system_prompt: String::new(), system_prompt_file: None,
            capabilities: vec![], execution_mode: "React".into(),
            created_at: now, updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigRow {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUserRow {
    pub user_id: String,
    pub token_hash: String,
    pub display_name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfigRow {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub name: String,
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_json: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingRow {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: i64,
}

// ── Repository traits ──

#[async_trait]
pub trait AgentConfigRepo: Send + Sync {
    async fn load_all(&self, user_id: &str) -> anyhow::Result<Vec<AgentConfigRow>>;
    async fn upsert(&self, row: &AgentConfigRow) -> anyhow::Result<()>;
    async fn delete(&self, user_id: &str, agent_id: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ProviderConfigRepo: Send + Sync {
    async fn load_all(&self) -> anyhow::Result<Vec<ProviderConfigRow>>;
    async fn upsert(&self, row: &ProviderConfigRow) -> anyhow::Result<()>;
    async fn delete(&self, name: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait DashboardUserRepo: Send + Sync {
    async fn load_all(&self) -> anyhow::Result<Vec<DashboardUserRow>>;
    async fn upsert(&self, row: &DashboardUserRow) -> anyhow::Result<()>;
    async fn find_by_token_hash(&self, hash: &str) -> anyhow::Result<Option<DashboardUserRow>>;
    async fn delete(&self, user_id: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait McpServerConfigRepo: Send + Sync {
    async fn load_for(&self, user_id: &str, agent_id: Option<&str>) -> anyhow::Result<Vec<McpServerConfigRow>>;
    async fn upsert(&self, row: &McpServerConfigRow) -> anyhow::Result<()>;
    async fn delete(&self, user_id: &str, agent_id: Option<&str>, name: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AppSettingsRepo: Send + Sync {
    async fn get(&self, key: &str) -> anyhow::Result<Option<serde_json::Value>>;
    async fn set(&self, key: &str, value: &serde_json::Value) -> anyhow::Result<()>;
    async fn load_all(&self) -> anyhow::Result<Vec<AppSettingRow>>;
}

// ── ConfigStore — bundles all 5 repos ──

pub struct ConfigStore {
    pub agent_configs: Box<dyn AgentConfigRepo>,
    pub provider_configs: Box<dyn ProviderConfigRepo>,
    pub dashboard_users: Box<dyn DashboardUserRepo>,
    pub mcp_servers: Box<dyn McpServerConfigRepo>,
    pub app_settings: Box<dyn AppSettingsRepo>,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self {
            agent_configs: Box::new(EmptyAgentConfigStore),
            provider_configs: Box::new(EmptyProviderConfigStore),
            dashboard_users: Box::new(EmptyDashboardUserStore),
            mcp_servers: Box::new(EmptyMcpServerConfigStore),
            app_settings: Box::new(EmptyAppSettingsStore),
        }
    }
}

impl ConfigStore {
    /// Create a file-backed ConfigStore.
    pub fn file(claw_dir: PathBuf) -> Self {
        Self {
            agent_configs: Box::new(FileAgentConfigStore::new(claw_dir.clone())),
            provider_configs: Box::new(FileProviderConfigStore::new(claw_dir.clone())),
            dashboard_users: Box::new(FileDashboardUserStore::new(claw_dir.clone())),
            mcp_servers: Box::new(FileMcpServerConfigStore::new(claw_dir.clone())),
            app_settings: Box::new(FileAppSettingsStore::new(claw_dir)),
        }
    }
}

// ── File backend implementations ──

pub struct FileAgentConfigStore { claw_dir: PathBuf }
impl FileAgentConfigStore { pub fn new(claw_dir: PathBuf) -> Self { Self { claw_dir } } }
#[async_trait]
impl AgentConfigRepo for FileAgentConfigStore {
    async fn load_all(&self, user_id: &str) -> anyhow::Result<Vec<AgentConfigRow>> {
        let path = self.claw_dir.join("agents").join(user_id).join("agent_configs.json");
        if !path.exists() { return Ok(vec![]); }
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default())
    }
    async fn upsert(&self, row: &AgentConfigRow) -> anyhow::Result<()> {
        let path = self.claw_dir.join("agents").join(&row.user_id).join("agent_configs.json");
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let mut rows = self.load_all(&row.user_id).await?;
        rows.retain(|r| r.agent_id != row.agent_id);
        rows.push(row.clone());
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
    async fn delete(&self, user_id: &str, agent_id: &str) -> anyhow::Result<()> {
        let path = self.claw_dir.join("agents").join(user_id).join("agent_configs.json");
        let mut rows = self.load_all(user_id).await?;
        rows.retain(|r| r.agent_id != agent_id);
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
}

pub struct FileProviderConfigStore { claw_dir: PathBuf }
impl FileProviderConfigStore { pub fn new(claw_dir: PathBuf) -> Self { Self { claw_dir } } }
#[async_trait]
impl ProviderConfigRepo for FileProviderConfigStore {
    async fn load_all(&self) -> anyhow::Result<Vec<ProviderConfigRow>> {
        let path = self.claw_dir.join("providers.json");
        if !path.exists() { return Ok(vec![]); }
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default())
    }
    async fn upsert(&self, row: &ProviderConfigRow) -> anyhow::Result<()> {
        let path = self.claw_dir.join("providers.json");
        let mut rows = self.load_all().await?;
        rows.retain(|r| r.name != row.name);
        rows.push(row.clone());
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
    async fn delete(&self, name: &str) -> anyhow::Result<()> {
        let path = self.claw_dir.join("providers.json");
        let mut rows = self.load_all().await?;
        rows.retain(|r| r.name != name);
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
}

pub struct FileDashboardUserStore { claw_dir: PathBuf }
impl FileDashboardUserStore { pub fn new(claw_dir: PathBuf) -> Self { Self { claw_dir } } }
#[async_trait]
impl DashboardUserRepo for FileDashboardUserStore {
    async fn load_all(&self) -> anyhow::Result<Vec<DashboardUserRow>> {
        let path = self.claw_dir.join("dashboard").join("users.json");
        if !path.exists() { return Ok(vec![]); }
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default())
    }
    async fn upsert(&self, row: &DashboardUserRow) -> anyhow::Result<()> {
        let path = self.claw_dir.join("dashboard").join("users.json");
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let mut rows = self.load_all().await?;
        rows.retain(|r| r.user_id != row.user_id);
        rows.push(row.clone());
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
    async fn find_by_token_hash(&self, hash: &str) -> anyhow::Result<Option<DashboardUserRow>> {
        Ok(self.load_all().await?.into_iter().find(|r| r.token_hash == hash))
    }
    async fn delete(&self, user_id: &str) -> anyhow::Result<()> {
        let path = self.claw_dir.join("dashboard").join("users.json");
        let mut rows = self.load_all().await?;
        rows.retain(|r| r.user_id != user_id);
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
}

pub struct FileMcpServerConfigStore { claw_dir: PathBuf }
impl FileMcpServerConfigStore { pub fn new(claw_dir: PathBuf) -> Self { Self { claw_dir } } }
#[async_trait]
impl McpServerConfigRepo for FileMcpServerConfigStore {
    async fn load_for(&self, user_id: &str, agent_id: Option<&str>) -> anyhow::Result<Vec<McpServerConfigRow>> {
        let path = self.claw_dir.join("mcp_servers.json");
        if !path.exists() { return Ok(vec![]); }
        let all: Vec<McpServerConfigRow> = serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default();
        Ok(all.into_iter()
            .filter(|r| r.user_id == user_id)
            .filter(|r| r.agent_id.as_deref() == agent_id || (r.agent_id.is_none() && agent_id.is_none()))
            .collect())
    }
    async fn upsert(&self, row: &McpServerConfigRow) -> anyhow::Result<()> {
        let path = self.claw_dir.join("mcp_servers.json");
        let mut rows: Vec<McpServerConfigRow> = if path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default()
        } else { vec![] };
        rows.retain(|r| !(r.user_id == row.user_id
            && r.agent_id.as_deref() == row.agent_id.as_deref() && r.name == row.name));
        rows.push(row.clone());
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
    async fn delete(&self, user_id: &str, agent_id: Option<&str>, name: &str) -> anyhow::Result<()> {
        let path = self.claw_dir.join("mcp_servers.json");
        let mut rows: Vec<McpServerConfigRow> = if path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default()
        } else { return Ok(()) };
        rows.retain(|r| !(r.user_id == user_id
            && r.agent_id.as_deref() == agent_id && r.name == name));
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&rows)?)?;
        Ok(())
    }
}

pub struct FileAppSettingsStore { claw_dir: PathBuf }
impl FileAppSettingsStore { pub fn new(claw_dir: PathBuf) -> Self { Self { claw_dir } } }
#[async_trait]
impl AppSettingsRepo for FileAppSettingsStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<serde_json::Value>> {
        let path = self.claw_dir.join("settings.json");
        if !path.exists() { return Ok(None); }
        let map: HashMap<String, serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default();
        Ok(map.get(key).cloned())
    }
    async fn set(&self, key: &str, value: &serde_json::Value) -> anyhow::Result<()> {
        let path = self.claw_dir.join("settings.json");
        let mut map: HashMap<String, serde_json::Value> = if path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default()
        } else { HashMap::new() };
        map.insert(key.to_string(), value.clone());
        crate::utils::atomic_write(&path, &serde_json::to_string_pretty(&map)?)?;
        Ok(())
    }
    async fn load_all(&self) -> anyhow::Result<Vec<AppSettingRow>> {
        let path = self.claw_dir.join("settings.json");
        if !path.exists() { return Ok(vec![]); }
        let map: HashMap<String, serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default();
        let now = chrono::Utc::now().timestamp();
        Ok(map.into_iter().map(|(key, value)| AppSettingRow { key, value, updated_at: now }).collect())
    }
}

// ── Empty stub implementations (for DB backends, to be implemented later) ──

struct EmptyAgentConfigStore;
#[async_trait]
impl AgentConfigRepo for EmptyAgentConfigStore {
    async fn load_all(&self, _user_id: &str) -> anyhow::Result<Vec<AgentConfigRow>> { Ok(vec![]) }
    async fn upsert(&self, _row: &AgentConfigRow) -> anyhow::Result<()> { Ok(()) }
    async fn delete(&self, _user_id: &str, _agent_id: &str) -> anyhow::Result<()> { Ok(()) }
}

struct EmptyProviderConfigStore;
#[async_trait]
impl ProviderConfigRepo for EmptyProviderConfigStore {
    async fn load_all(&self) -> anyhow::Result<Vec<ProviderConfigRow>> { Ok(vec![]) }
    async fn upsert(&self, _row: &ProviderConfigRow) -> anyhow::Result<()> { Ok(()) }
    async fn delete(&self, _name: &str) -> anyhow::Result<()> { Ok(()) }
}

struct EmptyDashboardUserStore;
#[async_trait]
impl DashboardUserRepo for EmptyDashboardUserStore {
    async fn load_all(&self) -> anyhow::Result<Vec<DashboardUserRow>> { Ok(vec![]) }
    async fn upsert(&self, _row: &DashboardUserRow) -> anyhow::Result<()> { Ok(()) }
    async fn find_by_token_hash(&self, _hash: &str) -> anyhow::Result<Option<DashboardUserRow>> { Ok(None) }
    async fn delete(&self, _user_id: &str) -> anyhow::Result<()> { Ok(()) }
}

struct EmptyMcpServerConfigStore;
#[async_trait]
impl McpServerConfigRepo for EmptyMcpServerConfigStore {
    async fn load_for(&self, _user_id: &str, _agent_id: Option<&str>) -> anyhow::Result<Vec<McpServerConfigRow>> { Ok(vec![]) }
    async fn upsert(&self, _row: &McpServerConfigRow) -> anyhow::Result<()> { Ok(()) }
    async fn delete(&self, _user_id: &str, _agent_id: Option<&str>, _name: &str) -> anyhow::Result<()> { Ok(()) }
}

struct EmptyAppSettingsStore;
#[async_trait]
impl AppSettingsRepo for EmptyAppSettingsStore {
    async fn get(&self, _key: &str) -> anyhow::Result<Option<serde_json::Value>> { Ok(None) }
    async fn set(&self, _key: &str, _value: &serde_json::Value) -> anyhow::Result<()> { Ok(()) }
    async fn load_all(&self) -> anyhow::Result<Vec<AppSettingRow>> { Ok(vec![]) }
}
