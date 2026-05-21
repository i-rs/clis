use crate::utils::atomic_write;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::mcp::McpServerConfig;

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
    /// Plugins to disable (by name). These will be skipped during auto-discovery.
    /// Useful when you want a plugin installed but not loaded.
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
    /// Gateway configuration for social platform integration.
    #[serde(default)]
    pub gateway: GatewayConfig,
    /// Dashboard web server configuration.
    #[serde(default)]
    pub dashboard: DashboardConfig,
    /// Named agent profiles. Empty = default agent only.
    /// Each agent can override provider, model, tools, and system prompt.
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    /// Sub-agent profiles for delegation only (not shown in TUI).
    /// Accessible via delegate_task tool.
    #[serde(default)]
    pub sub_agents: HashMap<String, AgentConfig>,
    /// Maximum ReAct loop rounds before stopping.
    #[serde(default = "default_max_react_rounds")]
    pub max_react_rounds: u32,
    /// Maximum retries per tool call on error.
    #[serde(default = "default_max_tool_retries")]
    pub max_tool_retries: u32,
    /// CLI subprocess execution timeout in seconds.
    #[serde(default = "default_cli_timeout_secs")]
    pub cli_timeout_secs: u64,
    /// Number of recent conversation turns to preserve in context.
    #[serde(default = "default_max_conversation_turns")]
    pub max_conversation_turns: usize,
    /// Execution mode for multi-step tasks.
    /// Defaults to ReAct (no upfront planning).
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// Custom color theme (loaded from theme.json, not serialized)
    #[serde(skip)]
    pub theme: crate::theme::Theme,
    /// Token usage statistics configuration.
    #[serde(default)]
    pub stats: crate::stats::StatsConfig,
}

fn default_max_react_rounds() -> u32 { 20 }
fn default_max_tool_retries() -> u32 { 2 }
fn default_cli_timeout_secs() -> u64 { 30 }
fn default_max_conversation_turns() -> usize { 8 }

fn default_true() -> bool {
    true
}

// ── Agent Configuration ──

/// Configuration for a named agent profile.
///
/// All fields are optional — if not set, the agent inherits from
/// the top-level Config fields (provider, api_key, base_url, model).
///
/// If `system_prompt` is set, it overrides the default system prompt.
/// If `system_prompt_file` is set and `system_prompt` is not, the file
/// is loaded at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub enabled_tools: Option<HashSet<String>>,
    /// Inline system prompt override (takes precedence over file).
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// Path to a system prompt file (relative to config dir or absolute).
    #[serde(default)]
    pub system_prompt_file: Option<String>,
    /// MCP server connections for this agent.
    /// If None, inherits from global mcp_servers.
    #[serde(default)]
    pub mcp_servers: Option<Vec<McpServerConfig>>,
    /// Allowed directories for file operations (workspace).
    /// If None, inherits from global allowed_dirs.
    #[serde(default)]
    pub allowed_dirs: Option<Vec<String>>,
    /// Capability descriptions for task routing and delegation decisions.
    /// E.g., ["数据分析", "代码生成", "数据可视化"]
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Execution mode for multi-step tasks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ExecutionMode {
    /// ReAct loop: think → tool → observe → think → tool → ... → done.
    /// No upfront planning needed; the LLM decides each step based on previous results.
    #[default]
    React,
    /// Plan-then-Execute: LLM outputs a structured plan first, then executes step by step.
    /// Useful for complex workflows where steps need user confirmation.
    PlanThenExecute,
}

/// Resolved configuration for a specific agent, with all fields flattened.
/// Produced by `Config::agent_config()`.
#[derive(Debug, Clone)]
pub struct ResolvedAgentConfig {
    /// Agent profile ID
    #[allow(dead_code)]
    pub agent_id: String,
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub enabled_tools: HashSet<String>,
    pub system_prompt: Option<String>,
    pub mcp_servers: Vec<McpServerConfig>,
    #[allow(dead_code)]
    pub allowed_dirs: Vec<String>,
    #[allow(dead_code)]
    pub capabilities: Vec<String>,
}

impl Config {
    /// Resolve config for a given agent ID by merging agent overrides
    /// with the top-level defaults.
    pub fn agent_config(&self, id: &str) -> ResolvedAgentConfig {
        let agent = self.agents.get(id).or_else(|| self.sub_agents.get(id));

        let system_prompt = agent
            .and_then(|a| a.system_prompt.clone())
            .or_else(|| {
                agent
                    .and_then(|a| a.system_prompt_file.as_ref())
                    .and_then(|path| {
                        let p = if path.starts_with('/') {
                            std::path::PathBuf::from(path)
                        } else {
                            // Relative to config directory
                            Self::config_path()
                                .ok()
                                .and_then(|cp| cp.parent().map(|parent| parent.join(path)))
                                .unwrap_or_else(|| std::path::PathBuf::from(path))
                        };
                        std::fs::read_to_string(&p).ok()
                    })
            });

        ResolvedAgentConfig {
            agent_id: id.to_string(),
            provider: agent
                .and_then(|a| a.provider.clone())
                .unwrap_or_else(|| self.provider.clone()),
            api_key: agent
                .and_then(|a| a.api_key.clone())
                .unwrap_or_else(|| self.api_key.clone()),
            base_url: agent
                .and_then(|a| a.base_url.clone())
                .unwrap_or_else(|| self.base_url.clone()),
            model: agent
                .and_then(|a| a.model.clone())
                .unwrap_or_else(|| self.model.clone()),
            enabled_tools: agent
                .and_then(|a| a.enabled_tools.clone())
                .unwrap_or_else(|| self.enabled_tools.clone()),
            system_prompt,
            mcp_servers: agent
                .and_then(|a| a.mcp_servers.clone())
                .unwrap_or_else(|| self.mcp_servers.clone()),
            allowed_dirs: agent
                .and_then(|a| a.allowed_dirs.clone())
                .unwrap_or_else(|| self.allowed_dirs.clone()),
            capabilities: agent
                .map(|a| a.capabilities.clone())
                .unwrap_or_default(),
        }
    }

    /// Get the list of visible agent IDs for TUI (main agents + "default").
    pub fn agent_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.agents.keys().cloned().collect();
        ids.sort();
        if !ids.contains(&"default".to_string()) {
            ids.insert(0, "default".to_string());
        }
        ids
    }

    /// Get the list of ALL agent IDs (including sub_agents).
    /// Used for runtime initialization, NOT for TUI display.
    pub fn all_agent_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        ids.extend(self.agents.keys().cloned());
        ids.extend(self.sub_agents.keys().cloned());
        ids.sort();
        if !ids.contains(&"default".to_string()) {
            ids.insert(0, "default".to_string());
        }
        ids
    }
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
    /// Bearer token for API authentication.
    /// If not set, a random token is generated on startup and printed to console.
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_dashboard_host(),
            port: default_dashboard_port(),
            auth_token: None,
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
    /// Optional agent profile to use for this platform.
    #[serde(default)]
    pub agent_id: Option<String>,
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
    /// Optional agent profile to use for this platform.
    #[serde(default)]
    pub agent_id: Option<String>,
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
            disabled_plugins: Vec::new(),
            agents: HashMap::new(),
            sub_agents: HashMap::new(),
            max_react_rounds: default_max_react_rounds(),
            max_tool_retries: default_max_tool_retries(),
            cli_timeout_secs: default_cli_timeout_secs(),
            max_conversation_turns: default_max_conversation_turns(),
            execution_mode: ExecutionMode::default(),
            gateway: GatewayConfig::default(),
            dashboard: DashboardConfig::default(),
            theme: crate::theme::Theme::default(),
            stats: crate::stats::StatsConfig::default(),
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
        if let Ok(env_key) = std::env::var("I_RS_CLAW_API_KEY")
            && !env_key.is_empty() {
                config.api_key = env_key;
            }

        // Ensure "default" agent always exists (safety net against manual config edits)
        if config.agents.contains_key("default") {
            config.agents.remove("default");
            tracing::warn!("配置文件中不应包含 [agents.default]，已自动移除（default 使用顶层配置）");
        }

        // Validate config
        if config.provider != "ollama" && config.api_key.is_empty() {
            anyhow::bail!("配置文件中 api_key 不能为空 (Ollama 除外)");
        }

        // Print non-fatal validation warnings
        for warning in config.validate() {
            tracing::warn!("{}", warning);
        }

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        atomic_write(&config_path, &content)?;
        println!("✓ 配置已保存: {}", config_path.display());
        Ok(())
    }

    /// Validate configuration and return non-fatal warnings.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        match self.provider.as_str() {
            "openai" | "ollama" | "anthropic" => {}
            other => warnings.push(format!(
                "未知 provider: '{}' (支持: openai, ollama, anthropic)",
                other
            )),
        }

        if self.provider != "ollama" && self.api_key.is_empty() {
            warnings.push(format!("{} provider 需要设置 api_key", self.provider));
        }

        if self.model.is_empty() {
            warnings.push("model 未设置".to_string());
        }

        if !self.base_url.is_empty() && !self.base_url.starts_with("http") {
            warnings.push("base_url 应该以 http:// 或 https:// 开头".to_string());
        }

        if self.dashboard.enabled && self.dashboard.port > 0 && self.dashboard.port < 1024 {
            warnings
                .push("dashboard 使用了特权端口 (<1024)，可能需要 root 权限".to_string());
        }

        for (id, agent) in &self.agents {
            if id.contains(' ') || id.contains('/') || id.contains('\\') {
                warnings.push(format!("agent ID '{}' 包含非法字符 (空格/斜杠)", id));
            }
            if let Some(ref p) = agent.provider {
                match p.as_str() {
                    "openai" | "ollama" | "anthropic" => {}
                    other => {
                        warnings.push(format!(
                            "agent '{}' 使用了未知 provider '{}'",
                            id, other
                        ));
                    }
                }
            }
        }

        for (id, agent) in &self.sub_agents {
            if id.contains(' ') || id.contains('/') || id.contains('\\') {
                warnings.push(format!("sub_agent ID '{}' 包含非法字符 (空格/斜杠)", id));
            }
            if let Some(ref p) = agent.provider {
                match p.as_str() {
                    "openai" | "ollama" | "anthropic" => {}
                    other => {
                        warnings.push(format!(
                            "sub_agent '{}' 使用了未知 provider '{}'",
                            id, other
                        ));
                    }
                }
            }
        }

        // Validate MCP servers (both top-level and in agent configs)
        for server in &self.mcp_servers {
            Self::validate_mcp_server(server, &mut warnings, "全局");
        }
        for (agent_id, agent) in &self.agents {
            if let Some(ref servers) = agent.mcp_servers {
                for server in servers {
                    Self::validate_mcp_server(server, &mut warnings, &format!("agent '{}'", agent_id));
                }
            }
        }

        warnings
    }

    /// Validate a single MCP server configuration.
    fn validate_mcp_server(server: &crate::mcp::McpServerConfig, warnings: &mut Vec<String>, scope: &str) {
        match server.transport_type.as_str() {
            "stdio" => {
                if server.command.is_none() {
                    warnings.push(format!(
                        "MCP server '{}' ({}) 使用 stdio 但未设置 command",
                        server.name, scope
                    ));
                }
            }
            "sse" => {
                if server.url.is_none() {
                    warnings.push(format!(
                        "MCP server '{}' ({}) 使用 sse 但未设置 url",
                        server.name, scope
                    ));
                }
            }
            other => warnings.push(format!(
                "MCP server '{}' ({}) 使用了未知 transport '{}'",
                server.name, scope, other
            )),
        }
    }

    /// Add a named agent profile to config.
    /// Returns an error if the agent already exists.
    #[allow(dead_code)]
    pub fn add_agent(&mut self, id: &str, agent: AgentConfig) -> anyhow::Result<()> {
        if id.is_empty() {
            anyhow::bail!("Agent ID cannot be empty");
        }
        if self.agents.contains_key(id) {
            anyhow::bail!("Agent '{}' already exists", id);
        }
        self.agents.insert(id.to_string(), agent);
        Ok(())
    }

    /// Remove a named agent profile from config.
    /// "default" agent cannot be removed.
    #[allow(dead_code)]
    pub fn remove_agent(&mut self, id: &str) -> anyhow::Result<AgentConfig> {
        if id == "default" {
            anyhow::bail!("Cannot remove the default agent");
        }
        self.agents.remove(id).ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))
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


}



