use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const DEFAULT_TOOLS: &[&str] = &[
    "kv", "weight", "water", "sleep", "meal", "pig", "mood", "sit", "spark", "todo",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Provider type: "openai", "anthropic", "ollama"
    #[serde(default = "default_provider")]
    pub provider: String,
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    /// Set of tool names to enable. Empty = all enabled.
    #[serde(default)]
    pub enabled_tools: HashSet<String>,
    /// Optional search API key for custom search engine.
    /// If not set, falls back to DuckDuckGo (free, no key needed).
    #[serde(default)]
    pub search_api_key: Option<String>,
    /// Optional custom search API endpoint.
    /// If not set, uses DuckDuckGo Instant Answer API.
    #[serde(default)]
    pub search_base_url: Option<String>,
    /// Allowed directories for file operations (read/write/list).
    /// Empty means file operations are disabled.
    #[serde(default)]
    pub allowed_dirs: Vec<String>,
    /// MCP server connections for external tool discovery.
    #[serde(default)]
    pub mcp_servers: Vec<crate::mcp::McpServerConfig>,
    /// Custom color theme (loaded from theme.json, not serialized)
    #[serde(skip)]
    pub theme: crate::theme::Theme,
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

impl Config {
    pub fn new() -> Self {
        let env_api_key = std::env::var("I_RS_CLAW_API_KEY").unwrap_or_default();
        Self {
            api_key: env_api_key,
            provider: default_provider(),
            base_url: default_base_url(),
            model: default_model(),
            enabled_tools: HashSet::new(),
            search_api_key: None,
            search_base_url: None,
            allowed_dirs: Vec::new(),
            mcp_servers: Vec::new(),
            theme: crate::theme::Theme::default(),
        }
    }

    fn config_path() -> anyhow::Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let dir = home.join(".i-rs-claw");
        Ok(dir.join("config.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            anyhow::bail!(
                "配置文件不存在: {}\n请运行 `i-rs-claw config` 交互式创建",
                config_path.display()
            );
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("读取配置文件失败 {}: {}", config_path.display(), e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("解析配置文件失败: {}", e))?;

        let mut config = config;

        // Load custom theme from ~/.i-rs-claw/theme.json
        if let Some(parent) = config_path.parent() {
            let theme_path = parent.join("theme.json");
            config.theme = crate::theme::Theme::load(&theme_path);
        }

        // Override API key from environment variable if set
        if let Ok(env_key) = std::env::var("I_RS_CLAW_API_KEY") {
            if !env_key.is_empty() {
                config.api_key = env_key;
            }
        }

        if config.api_key.is_empty() {
            anyhow::bail!("配置文件中 api_key 不能为空");
        }

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        println!("✓ 配置已保存: {}", config_path.display());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn all_tools() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::tools::TOOL_INDEX.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config_defaults() {
        // SAFETY: test runs single-threaded
        unsafe { std::env::remove_var("I_RS_CLAW_API_KEY") };
        let config = Config::new();
        assert!(config.api_key.is_empty());
        assert_eq!(config.provider, "openai");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.enabled_tools.is_empty());
    }

    #[test]
    fn test_env_var_overrides_new() {
        // SAFETY: test runs single-threaded
        unsafe { std::env::set_var("I_RS_CLAW_API_KEY", "sk-test-key-from-env") };
        let config = Config::new();
        assert_eq!(config.api_key, "sk-test-key-from-env");
        unsafe { std::env::remove_var("I_RS_CLAW_API_KEY") };
    }

    #[test]
    fn test_env_var_empty_string() {
        // SAFETY: test runs single-threaded
        unsafe { std::env::set_var("I_RS_CLAW_API_KEY", "") };
        let config = Config::new();
        assert!(config.api_key.is_empty());
        unsafe { std::env::remove_var("I_RS_CLAW_API_KEY") };
    }

    #[test]
    fn test_all_tools_contains_kv() {
        let tools = Config::all_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|(name, _, _)| *name == "kv"));
    }
}


