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
    /// Automatically discover plugins from ~/.i-rs-claw/plugins/.
    /// Discovered plugins are merged into mcp_servers at startup.
    #[serde(default = "default_true")]
    pub plugins_auto_discover: bool,
    /// Gateway configuration for social platform integration.
    #[serde(default)]
    pub gateway: GatewayConfig,
    /// Dashboard web server configuration.
    #[serde(default)]
    pub dashboard: DashboardConfig,
    /// Custom color theme (loaded from theme.json, not serialized)
    #[serde(skip)]
    pub theme: crate::theme::Theme,
}

fn default_true() -> bool {
    true
}

// ── Gateway Configuration ──

/// Gateway configuration for social platform integration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// Master switch for the gateway server.
    #[serde(default)]
    pub enabled: bool,
    /// Telegram bot configuration.
    #[serde(default)]
    pub telegram: Option<PlatformConfig>,
    /// Discord bot configuration.
    #[serde(default)]
    pub discord: Option<PlatformConfig>,
    /// Slack bot configuration.
    #[serde(default)]
    pub slack: Option<PlatformConfig>,
    /// WeChat iLink Bot (personal WeChat) configuration.
    #[serde(default)]
    pub wechat: Option<WeChatPlatformConfig>,
}

// ── Dashboard Configuration ──

/// Dashboard web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Whether the dashboard is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Host address to bind to.
    #[serde(default = "default_dashboard_host")]
    pub host: String,
    /// Port to listen on.
    #[serde(default = "default_dashboard_port")]
    pub port: u16,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_dashboard_host(),
            port: default_dashboard_port(),
        }
    }
}

fn default_dashboard_host() -> String {
    "127.0.0.1".to_string()
}

fn default_dashboard_port() -> u16 {
    3000
}

/// Generic platform configuration used by Telegram/Discord/Slack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Whether this platform adapter is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// API token or key for the platform.
    #[serde(default)]
    pub token: Option<String>,
    /// Webhook URL (used by Slack).
    #[serde(default)]
    pub webhook_url: Option<String>,
    /// Additional configuration as key-value pairs.
    #[serde(default)]
    pub extra: Option<::std::collections::HashMap<String, String>>,
}

/// WeChat iLink Bot (personal WeChat) configuration.
///
/// No static token/URL needed -- credentials obtained via QR login
/// on first run and persisted to ~/.i-rs-claw/claw/wechat_credentials.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeChatPlatformConfig {
    /// Whether the WeChat bot is enabled.
    #[serde(default)]
    pub enabled: bool,
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
            plugins_auto_discover: true,
            gateway: GatewayConfig::default(),
            dashboard: DashboardConfig::default(),
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


