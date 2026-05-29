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
    #[serde(default)]
    pub max_cost_per_session: Option<f64>,
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProjectInfo {
    pub project_type: String,
    pub has_cargo: bool,
    pub has_package_json: bool,
    pub has_pyproject: bool,
    pub has_makefile: bool,
    pub has_agents_md: bool,
    pub has_cursor_rules: bool,
    pub agents_md_content: Option<String>,
    pub cursor_rules_content: Option<String>,
}

impl ProjectInfo {
    pub fn detect() -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        let has_cargo = cwd.join("Cargo.toml").exists();
        let has_package_json = cwd.join("package.json").exists();
        let has_pyproject = cwd.join("pyproject.toml").exists();
        let has_makefile = cwd.join("Makefile").exists();
        let agents_md = cwd.join("AGENTS.md");
        let has_agents_md = agents_md.exists();
        let agents_md_content = if has_agents_md {
            std::fs::read_to_string(&agents_md).ok()
        } else {
            cwd.join(".cursor/rules").exists().then(|| {
                std::fs::read_dir(cwd.join(".cursor/rules"))
                    .ok()
                    .map(|entries| {
                        entries.filter_map(|e| e.ok())
                            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
                    .unwrap_or_default()
            })
        };
        let cursor_rules = cwd.join(".cursor/rules");
        let has_cursor_rules = cursor_rules.exists();

        let project_type = if has_cargo { "Rust (Cargo)" }
            else if has_package_json { "Node.js" }
            else if has_pyproject { "Python" }
            else if has_makefile { "Make/C" }
            else { "Unknown" };

        Self {
            project_type: project_type.into(),
            has_cargo,
            has_package_json,
            has_pyproject,
            has_makefile,
            has_agents_md,
            has_cursor_rules,
            agents_md_content,
            cursor_rules_content: if has_cursor_rules {
                std::fs::read_dir(&cursor_rules).ok()
                    .map(|entries| {
                        entries.filter_map(|e| e.ok())
                            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
            } else {
                None
            },
        }
    }
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
            max_cost_per_session: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path();
        let mut config = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let c: Config = toml::from_str(&content)?;
            c
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let config = Config::default();
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&path, &content)?;
            config
        };

        if !matches!(config.provider.as_str(), "openai" | "anthropic" | "ollama") {
            anyhow::bail!("Unknown provider: '{}'. Supported: openai, anthropic, ollama", config.provider);
        }

        if let Some(max_cost) = config.max_cost_per_session {
            if max_cost <= 0.0 || max_cost > 1000.0 {
                config.max_cost_per_session = None;
            }
        }

        if config.max_rounds == 0 || config.max_rounds > 100 {
            config.max_rounds = 20;
        }

        if config.tool_timeout_secs == 0 {
            config.tool_timeout_secs = 120;
        }

        if let Ok(key) = std::env::var("I_RS_CODE_API_KEY") {
            if !key.is_empty() {
                config.api_key = Some(key);
            }
        }

        Ok(config)
    }

    pub fn effective_model(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-4o-mini")
    }

    pub fn effective_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://api.openai.com/v1")
    }

    #[allow(dead_code)]
    pub fn tools_dir(&self) -> PathBuf {
        self.tools_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            i_rs_code_dir().join("tools")
        })
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
