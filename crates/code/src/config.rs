use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn i_rs_code_dir() -> PathBuf {
    std::env::var("I_RS_CODE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".i-rs-code"))
}

pub fn config_path() -> PathBuf {
    i_rs_code_dir().join("config.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub tools_dir: Option<String>,
    #[serde(default)]
    pub bin_dir: Option<String>,
    #[serde(default = "default_max_rounds")]
    pub max_rounds: u32,
    #[serde(default = "default_max_tool_retries")]
    pub max_tool_retries: u32,
    #[serde(default = "default_tool_timeout")]
    pub tool_timeout_secs: u64,
    #[serde(default)]
    pub agents: std::collections::HashMap<String, AgentConfig>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub search_provider: Option<String>,
    #[serde(default)]
    pub search_api_key: Option<String>,
}

fn default_max_rounds() -> u32 { 20 }
fn default_max_tool_retries() -> u32 { 2 }
fn default_tool_timeout() -> u64 { 120 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(default = "default_mcp_transport")]
    pub transport_type: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub env: Option<Vec<String>>,
}

fn default_mcp_transport() -> String { "stdio".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            api_key: None,
            base_url: None,
            model: None,
            workspace: None,
            tools_dir: None,
            bin_dir: None,
            max_rounds: default_max_rounds(),
            max_tool_retries: default_max_tool_retries(),
            tool_timeout_secs: default_tool_timeout(),
            agents: std::collections::HashMap::new(),
            mcp_servers: Vec::new(),
            search_provider: None,
            search_api_key: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let config = Config::default();
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&path, &content)?;
            Ok(config)
        }
    }

    pub fn effective_model(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-4o-mini")
    }

    pub fn effective_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://api.openai.com/v1")
    }

    pub fn tools_dir(&self) -> PathBuf {
        self.tools_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            i_rs_code_dir().join("tools")
        })
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.bin_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            i_rs_code_dir().join("bin")
        })
    }

    pub fn manifest_path(&self) -> PathBuf {
        i_rs_code_dir().join("manifest.json")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn agent_config(&self, agent_id: &str) -> Config {
        if let Some(agent) = self.agents.get(agent_id) {
            Config {
                provider: agent.provider.clone().unwrap_or_else(|| self.provider.clone()),
                api_key: agent.api_key.clone().or_else(|| self.api_key.clone()),
                base_url: agent.base_url.clone().or_else(|| self.base_url.clone()),
                model: agent.model.clone().or_else(|| self.model.clone()),
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let c = Config::default();
        assert_eq!(c.provider, "openai");
        assert!(c.api_key.is_none());
        assert_eq!(c.max_rounds, 20);
        assert_eq!(c.max_tool_retries, 2);
        assert_eq!(c.tool_timeout_secs, 120);
    }

    #[test]
    fn test_agent_config_override() {
        let mut c = Config::default();
        c.agents.insert("code".into(), AgentConfig {
            provider: Some("anthropic".into()),
            model: Some("claude-sonnet-4-20250514".into()),
            api_key: None,
            base_url: None,
            system_prompt: None,
        });
        let resolved = c.agent_config("code");
        assert_eq!(resolved.provider, "anthropic");
        assert_eq!(resolved.model.as_deref(), Some("claude-sonnet-4-20250514"));
        assert!(resolved.api_key.is_none());
    }

    #[test]
    fn test_agent_config_fallback() {
        let mut c = Config::default();
        c.agents.insert("missing_field".into(), AgentConfig {
            provider: Some("ollama".into()),
            model: None,
            api_key: Some("sk-test".into()),
            base_url: None,
            system_prompt: None,
        });
        let resolved = c.agent_config("missing_field");
        assert_eq!(resolved.provider, "ollama");
        assert_eq!(resolved.model.as_deref(), None);
        assert_eq!(resolved.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn test_agent_config_nonexistent() {
        let c = Config::default();
        let resolved = c.agent_config("no_such_agent");
        assert_eq!(resolved.provider, "openai");
    }
}
