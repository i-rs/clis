use crate::utils::atomic_write;
use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::mcp::McpServerConfig;
use crate::providers::ProviderKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_provider")]
    pub provider: ProviderKind,
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub enabled_tools: HashSet<String>,
    /// i-rs CLI tools installed by the user (e.g. ["weight", "todo", "mood"]).
    /// Empty = no i-rs tools available.
    #[serde(default)]
    pub i_rs_tools: Vec<String>,
    /// Cached i-rs tool descriptions (name → description), populated at startup.
    #[serde(default, skip_serializing)]
    pub i_rs_tool_index: HashMap<String, String>,
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
    /// Automatically discover plugins from ~/.i-rs/claw/plugins/.
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
    /// Named provider configurations for LLM backends.
    /// Each entry is a (name, ProviderConfig) pair that can be
    /// referenced by agents via `provider_ref`.
    /// If empty at load time, legacy top-level fields (provider/
    /// api_key/base_url/model) are auto-migrated into a "default"
    /// provider entry.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    /// Name of the default provider to use when an agent does not
    /// specify `provider_ref`. Defaults to "default".
    #[serde(default = "default_providers_default_provider")]
    pub default_provider: String,
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
    /// Sub-agent delegation timeout in seconds.
    #[serde(default = "default_delegate_timeout_secs")]
    pub delegate_timeout_secs: u64,
    /// Whether sub-agents can recursively delegate (default: false).
    #[serde(default)]
    pub allow_recursive_delegation: bool,
    /// Internal flag: when true, strip delegate_task from tool schemas.
    #[serde(default, skip_serializing)]
    pub exclude_delegate_tool: bool,
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
    /// Storage backend configuration.
    #[serde(default)]
    pub storage: crate::storage::StorageConfig,
    /// Token usage statistics configuration.
    #[serde(default)]
    pub stats: crate::stats::StatsConfig,
    /// Quality judge configuration (LLM-as-Judge for response evaluation).
    #[serde(default)]
    pub quality_judge: QualityJudgeConfig,
    /// Timezone offset for date/time display (e.g., "+08:00", "UTC", "-05:00").
    /// If not set, uses the system's local timezone.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Image generation configuration (external service for chart/image creation).
    #[serde(default)]
    pub image_gen: ImageGenConfig,
    /// Behavior analyst sub-agent configuration (data analysis and chart generation).
    #[serde(default)]
    pub behavior_analyst: BehaviorAnalystConfig,
    /// HITL (Human-in-the-Loop) policy configuration.
    #[serde(default)]
    pub hitl: HitlConfig,
    /// Cached timezone offset computed at load time.
    #[serde(skip, default = "crate::utils::system_tz_offset")]
    pub tz_offset: FixedOffset,
}

fn default_max_react_rounds() -> u32 {
    20
}
fn default_max_tool_retries() -> u32 {
    2
}
fn default_cli_timeout_secs() -> u64 {
    30
}
fn default_delegate_timeout_secs() -> u64 {
    120
}
fn default_max_conversation_turns() -> usize {
    8
}

fn default_true() -> bool {
    true
}

fn default_providers_default_provider() -> String {
    "default".to_string()
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
    pub provider: Option<ProviderKind>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    /// Reference to a named provider config in the `[providers]` section.
    /// When set, the provider/api_key/base_url fields below are ignored
    /// for this agent; only `model` still acts as an override on top of
    /// the referenced provider config.
    #[serde(default)]
    pub provider_ref: Option<String>,
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
    /// Execution mode override for this agent (None = inherit from global).
    #[serde(default)]
    pub execution_mode: Option<ExecutionMode>,
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

/// Quality judge configuration for LLM-as-Judge response evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityJudgeConfig {
    /// Enable LLM-as-Judge evaluation (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Model to use for judging (defaults to the main model).
    #[serde(default)]
    pub model: Option<String>,
    /// Only run judge when heuristic evaluation detects issues (default: true).
    #[serde(default = "default_true")]
    pub on_issues_only: bool,
    /// Maximum judge evaluations per session (default: 3).
    #[serde(default = "default_judge_max_per_session")]
    pub max_per_session: u32,
}

fn default_judge_max_per_session() -> u32 {
    3
}

impl Default for QualityJudgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model: None,
            on_issues_only: true,
            max_per_session: 3,
        }
    }
}

/// A named provider configuration.
///
/// Defines the connection parameters for a single LLM provider.
/// Multiple provider configs can be defined under the `[providers]`
/// section of the config file and referenced by agents via
/// `provider_ref`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: ProviderKind,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

/// Resolved configuration for a specific agent, with all fields flattened.
/// Produced by `Config::agent_config()`.
#[derive(Debug, Clone)]
pub struct ResolvedAgentConfig {
    #[allow(dead_code)]
    pub agent_id: String,
    pub provider: ProviderKind,
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
    pub execution_mode: ExecutionMode,
}

impl Config {
    /// Resolve config for a given agent ID by merging agent overrides
    /// with the top-level defaults.
    pub fn agent_config(&self, id: &str) -> ResolvedAgentConfig {
        let agent = self.agents.get(id).or_else(|| self.sub_agents.get(id));

        let system_prompt = agent.and_then(|a| a.system_prompt.clone()).or_else(|| {
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

        let base = self.resolve_provider_config(agent);

        ResolvedAgentConfig {
            agent_id: id.to_string(),
            provider: agent.and_then(|a| a.provider).unwrap_or(base.provider),
            api_key: agent
                .and_then(|a| a.api_key.clone())
                .unwrap_or(base.api_key),
            base_url: agent
                .and_then(|a| a.base_url.clone())
                .unwrap_or(base.base_url),
            model: agent.and_then(|a| a.model.clone()).unwrap_or(base.model),
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
            capabilities: agent.map(|a| a.capabilities.clone()).unwrap_or_default(),
            execution_mode: agent
                .and_then(|a| a.execution_mode.clone())
                .unwrap_or(self.execution_mode.clone()),
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

    /// Resolve provider configuration with the following precedence:
    /// 1. Agent's `provider_ref` → look up in `providers` map
    /// 2. `default_provider` → look up in `providers` map
    /// 3. Legacy top-level `provider`/`api_key`/`base_url`/`model` fields
    ///
    /// This allows gradual migration: existing configs with top-level
    /// fields continue to work, while new configs can use the richer
    /// named-provider setup.
    pub fn resolve_provider_config(&self, agent: Option<&AgentConfig>) -> ProviderConfig {
        // 1. Try agent's provider_ref
        if let Some(ref_name) = agent.and_then(|a| a.provider_ref.as_ref())
            && let Some(pc) = self.providers.get(ref_name)
        {
            return pc.clone();
        }
        // 2. Try default_provider
        if let Some(pc) = self.providers.get(&self.default_provider) {
            return pc.clone();
        }
        // 3. Legacy fallback
        ProviderConfig {
            provider: self.provider,
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
        }
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

/// A multi-tenant user entry for token-based auth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUser {
    pub id: String,
    pub token: String,
}

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
    /// Multi-user token map (token → user_id).
    /// When empty, all requests use "default" user.
    #[serde(default)]
    pub users: Vec<DashboardUser>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_dashboard_host(),
            port: default_dashboard_port(),
            auth_token: None,
            users: Vec::new(),
        }
    }
}

fn default_dashboard_host() -> String {
    "127.0.0.1".to_string()
}

fn default_dashboard_port() -> u16 {
    3000
}

// ── Image Generation Configuration ──

/// Image generation provider configuration.
///
/// Supports multiple backends for generating charts and images:
/// - `svg_chart`: Built-in SVG chart generation (no external service needed, always available)
/// - `quick_chart`: QuickChart.io free chart API (supports all Chart.js chart types)
/// - `custom_http`: User-configured HTTP API endpoint for any image generation service
///
/// # Example
/// ```toml
/// [image_gen]
/// provider = "quick_chart"   # svg_chart | quick_chart | custom_http
/// api_key = ""               # API key for custom_http provider
/// base_url = ""              # Custom endpoint URL for custom_http provider
/// default_chart_type = "bar" # Default chart type: bar, line, pie, radar
/// default_width = 800
/// default_height = 500
/// images_dir = "images"      # Relative to ~/.i-rs/claw/
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenConfig {
    /// Image generation provider: "svg_chart" (built-in), "quick_chart" (free API),
    /// "custom_http" (user-configured API).
    #[serde(default = "default_image_gen_provider")]
    pub provider: String,
    /// API key for the provider (required for custom_http, optional for others).
    #[serde(default)]
    pub api_key: String,
    /// Base URL for the custom_http provider.
    #[serde(default)]
    pub base_url: String,
    /// Default chart type: "bar", "line", "pie", "radar".
    #[serde(default = "default_chart_type")]
    pub default_chart_type: String,
    /// Default image width in pixels.
    #[serde(default = "default_image_width")]
    pub default_width: u32,
    /// Default image height in pixels.
    #[serde(default = "default_image_height")]
    pub default_height: u32,
    /// Directory for generated images (relative to ~/.i-rs/claw/).
    #[serde(default = "default_images_dir")]
    pub images_dir: String,
}

fn default_image_gen_provider() -> String {
    "svg_chart".to_string()
}

fn default_chart_type() -> String {
    "bar".to_string()
}

fn default_image_width() -> u32 {
    800
}

fn default_image_height() -> u32 {
    500
}

fn default_images_dir() -> String {
    "images".to_string()
}

impl Default for ImageGenConfig {
    fn default() -> Self {
        Self {
            provider: default_image_gen_provider(),
            api_key: String::new(),
            base_url: String::new(),
            default_chart_type: default_chart_type(),
            default_width: default_image_width(),
            default_height: default_image_height(),
            images_dir: default_images_dir(),
        }
    }
}

// ── Behavior Analyst Configuration ──

/// Behavior analyst sub-agent configuration.
///
/// This sub-agent summarizes user behavior over different time ranges and
/// generates visual charts. It reads data from configured i-rs CLI tools,
/// analyzes trends, and produces daily/weekly/monthly reports.
///
/// # Example
/// ```toml
/// [behavior_analyst]
/// enabled = true
/// daily_summary = true
/// weekly_summary = true
/// monthly_summary = false
/// specific_behavior = true
/// track_behaviors = ["weight", "mood", "sleep", "exercise", "habit"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnalystConfig {
    /// Enable/disable the behavior analyst sub-agent.
    #[serde(default)]
    pub enabled: bool,
    /// Allow daily behavior summaries.
    #[serde(default = "default_true")]
    pub daily_summary: bool,
    /// Allow weekly behavior summaries.
    #[serde(default = "default_true")]
    pub weekly_summary: bool,
    /// Allow monthly behavior summaries.
    #[serde(default)]
    pub monthly_summary: bool,
    /// Allow per-behavior detailed analysis (e.g., "analyze my sleep patterns").
    #[serde(default = "default_true")]
    pub specific_behavior: bool,
    /// Behaviors to track. Empty or ["*"] means all installed i-rs tools.
    /// Specify individual tools like: ["weight", "mood", "sleep", "exercise", "habit"]
    #[serde(default)]
    pub track_behaviors: Vec<String>,
    /// Auto-open generated images in system viewer (macOS only).
    #[serde(default)]
    pub auto_open_images: bool,
}

impl Default for BehaviorAnalystConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_summary: true,
            weekly_summary: true,
            monthly_summary: false,
            specific_behavior: true,
            track_behaviors: vec![],
            auto_open_images: false,
        }
    }
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
/// on first run and persisted to ~/.i-rs/claw/claw/wechat_credentials.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeChatPlatformConfig {
    /// Whether the WeChat bot is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Optional agent profile to use for this platform.
    #[serde(default)]
    pub agent_id: Option<String>,
}

// ── HITL (Human-in-the-Loop) Configuration ──

/// Configuration for the HITL policy that gates high-risk tool calls.
///
/// # Example
/// ```toml
/// [hitl]
/// auto_approve_high_risk = false   # default; only set true in sandboxed envs
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlConfig {
    /// When true, the executor auto-approves High-risk tool calls.
    /// Default: false. Set to true ONLY in sandboxed/CI environments.
    #[serde(default)]
    pub auto_approve_high_risk: bool,
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            auto_approve_high_risk: false,
        }
    }
}

fn default_provider() -> ProviderKind {
    ProviderKind::OpenAI
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
            providers: HashMap::new(),
            default_provider: default_providers_default_provider(),
            enabled_tools: HashSet::new(),
            i_rs_tools: Vec::new(),
            i_rs_tool_index: HashMap::new(),
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
            delegate_timeout_secs: default_delegate_timeout_secs(),
            allow_recursive_delegation: false,
            exclude_delegate_tool: false,
            max_conversation_turns: default_max_conversation_turns(),
            execution_mode: ExecutionMode::default(),
            gateway: GatewayConfig::default(),
            dashboard: DashboardConfig::default(),
            theme: crate::theme::Theme::default(),
            storage: crate::storage::StorageConfig::default(),
            stats: crate::stats::StatsConfig::default(),
            quality_judge: QualityJudgeConfig::default(),
            image_gen: ImageGenConfig::default(),
            behavior_analyst: BehaviorAnalystConfig::default(),
            hitl: HitlConfig::default(),
            timezone: None,
            tz_offset: crate::utils::system_tz_offset(),
        }
    }

    fn config_path() -> anyhow::Result<std::path::PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let dir = home.join(".i-rs").join("claw");
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

        let config: Config =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("解析配置文件失败: {}", e))?;

        let mut config = config;

        // Load custom theme from ~/.i-rs/claw/theme.json
        if let Some(parent) = config_path.parent() {
            let theme_path = parent.join("theme.json");
            config.theme = crate::theme::Theme::load(&theme_path);
        }

        // Resolve timezone offset from config or system local
        config.tz_offset = crate::utils::parse_timezone(config.timezone.as_deref());

        // Override API key from environment variable if set
        if let Ok(env_key) = std::env::var("I_RS_CLAW_API_KEY")
            && !env_key.is_empty()
        {
            config.api_key = env_key;
        }

        // Auto-migrate legacy top-level provider fields into providers map.
        // This lets old configs (which store provider/api_key/base_url/model
        // at the top level) work seamlessly with the new named-provider system.
        if config.providers.is_empty() && !config.api_key.is_empty() {
            config.providers.insert(
                "default".to_string(),
                ProviderConfig {
                    provider: config.provider,
                    api_key: config.api_key.clone(),
                    base_url: config.base_url.clone(),
                    model: config.model.clone(),
                },
            );
        }

        // Validate default_provider exists in providers map, or add a
        // placeholder so resolution doesn't panic at runtime.
        if !config.providers.contains_key(&config.default_provider)
            && config.providers.contains_key("default")
        {
            config.default_provider = "default".to_string();
        }

        // Ensure "default" agent always exists (safety net against manual config edits)
        if config.agents.contains_key("default") {
            config.agents.remove("default");
            tracing::warn!(
                "配置文件中不应包含 [agents.default]，已自动移除（default 使用顶层配置）"
            );
        }

        // Validate config - check both providers map and legacy fields
        {
            let has_providers = !config.providers.is_empty();
            let has_valid_default = config
                .providers
                .get(&config.default_provider)
                .map(|pc| pc.provider == ProviderKind::Ollama || !pc.api_key.is_empty())
                .unwrap_or(false);
            let has_legacy = config.provider != ProviderKind::Ollama && !config.api_key.is_empty();

            if !has_valid_default && !has_legacy && has_providers {
                anyhow::bail!(
                    "配置文件中的 default provider '{}' 需要设置 api_key (Ollama 除外)",
                    config.default_provider
                );
            }
            if !has_providers
                && config.provider != ProviderKind::Ollama
                && config.api_key.is_empty()
            {
                anyhow::bail!("配置文件中 api_key 不能为空 (Ollama 除外)");
            }
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

        // Validate named providers in the providers map
        for (name, pc) in &self.providers {
            let known_providers = [
                ProviderKind::OpenAI,
                ProviderKind::Ollama,
                ProviderKind::Anthropic,
            ];
            if !known_providers.contains(&pc.provider) {
                tracing::info!("provider '{}' 不在已知列表中，将使用 OpenAI 兼容模式", name);
            }
            if pc.provider != ProviderKind::Ollama && pc.api_key.is_empty() {
                warnings.push(format!("provider '{}' 需要设置 api_key", name));
            }
            if pc.model.is_empty() {
                warnings.push(format!("provider '{}' 未设置 model", name));
            }
            if !pc.base_url.is_empty() && !pc.base_url.starts_with("http") {
                warnings.push(format!(
                    "provider '{}' 的 base_url 应该以 http:// 或 https:// 开头",
                    name
                ));
            }
        }

        // Also validate legacy top-level fields for backward compat
        // (only when they're the primary source, not auto-migrated)
        if self.providers.is_empty() || !self.providers.contains_key(&self.default_provider) {
            if self.provider != ProviderKind::Ollama && self.api_key.is_empty() {
                warnings.push(format!("{} provider 需要设置 api_key", self.provider));
            }
            if self.model.is_empty() {
                warnings.push("model 未设置".to_string());
            }
            if !self.base_url.is_empty() && !self.base_url.starts_with("http") {
                warnings.push("base_url 应该以 http:// 或 https:// 开头".to_string());
            }
        }

        if self.dashboard.enabled && self.dashboard.port > 0 && self.dashboard.port < 1024 {
            warnings.push("dashboard 使用了特权端口 (<1024)，可能需要 root 权限".to_string());
        }

        for (id, agent) in &self.agents {
            Self::validate_agent_config(id, agent, "agent", &mut warnings);
        }
        for (id, agent) in &self.sub_agents {
            Self::validate_agent_config(id, agent, "sub_agent", &mut warnings);
        }

        // Validate MCP servers (both top-level and in agent configs)
        for server in &self.mcp_servers {
            Self::validate_mcp_server(server, &mut warnings, "全局");
        }
        for (agent_id, agent) in &self.agents {
            if let Some(ref servers) = agent.mcp_servers {
                for server in servers {
                    Self::validate_mcp_server(
                        server,
                        &mut warnings,
                        &format!("agent '{}'", agent_id),
                    );
                }
            }
        }
        for (agent_id, agent) in &self.sub_agents {
            if let Some(ref servers) = agent.mcp_servers {
                for server in servers {
                    Self::validate_mcp_server(
                        server,
                        &mut warnings,
                        &format!("sub_agent '{}'", agent_id),
                    );
                }
            }
        }

        warnings
    }

    /// Validate a single MCP server configuration.
    fn validate_mcp_server(
        server: &crate::mcp::McpServerConfig,
        warnings: &mut Vec<String>,
        scope: &str,
    ) {
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

    /// Validate a single agent/sub_agent configuration.
    fn validate_agent_config(
        id: &str,
        agent: &AgentConfig,
        scope: &str,
        warnings: &mut Vec<String>,
    ) {
        if id.contains(' ') || id.contains('/') || id.contains('\\') {
            warnings.push(format!("{} ID '{}' 包含非法字符 (空格/斜杠)", scope, id));
        }
        if let Some(p) = agent.provider {
            match p {
                ProviderKind::OpenAI | ProviderKind::Ollama | ProviderKind::Anthropic => {}
                other => {
                    warnings.push(format!(
                        "{} '{}' 使用了未知 provider '{}'",
                        scope, id, other
                    ));
                }
            }
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
        self.agents
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))
    }

    /// Discover i-rs CLI tools and cache their descriptions.
    /// For each tool in `i_rs_tools`, runs `i-rs-{name} skill summary`
    /// to get the description, and caches the result.
    /// Uses a disk cache to avoid subprocess calls on every startup.
    pub fn discover_i_rs_tools(&mut self, claw_dir: &std::path::Path) {
        if self.i_rs_tools.is_empty() {
            self.i_rs_tool_index.clear();
            return;
        }

        let cache_path = claw_dir.join("i_rs_tool_index.json");

        // Try loading from cache first
        if let Ok(content) = std::fs::read_to_string(&cache_path)
            && let Ok(cached) = serde_json::from_str::<HashMap<String, String>>(&content)
        {
            // Only use cache if it covers all configured tools
            if self.i_rs_tools.iter().all(|t| cached.contains_key(t)) {
                self.i_rs_tool_index = cached;
                return;
            }
        }

        let max_parallel = 8usize;
        let tools: Vec<(String, String)> = self
            .i_rs_tools
            .iter()
            .map(|name| (name.clone(), format!("i-rs-{}", name)))
            .collect();

        for chunk in tools.chunks(max_parallel) {
            let mut chunk_handles = Vec::new();
            #[allow(clippy::unnecessary_to_owned)]
            for (name, binary) in chunk.to_vec() {
                let name = name.clone();
                chunk_handles.push(std::thread::spawn(move || {
                    let desc = match std::process::Command::new(&binary)
                        .arg("skill")
                        .arg("summary")
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::null())
                        .output()
                    {
                        Ok(output) if output.status.success() => {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        }
                        _ => {
                            tracing::warn!(
                                "i-rs 工具 '{}' 未安装或 skill summary 失败，已跳过",
                                name
                            );
                            return (name, None);
                        }
                    };
                    (name, Some(desc))
                }));
            }
            for handle in chunk_handles {
                if let Ok((name, Some(desc))) = handle.join() {
                    self.i_rs_tool_index.insert(name, desc);
                }
            }
        }

        // Write cache
        if let Ok(json) = serde_json::to_string(&self.i_rs_tool_index) {
            let _ = crate::utils::atomic_write(&cache_path, &json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: run a test closure with the env var temporarily set,
    /// restoring the original value (or clearing it) afterwards.
    fn with_env<F>(key: &str, value: Option<&str>, f: F)
    where
        F: FnOnce(),
    {
        let original = std::env::var(key).ok();
        // SAFETY: test-only, single-threaded via --test-threads=1
        unsafe {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        // SAFETY: restoring original, same thread
        unsafe {
            match original {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_new_config_defaults() {
        with_env("I_RS_CLAW_API_KEY", None, || {
            let config = Config::new();
            assert!(config.api_key.is_empty());
            assert_eq!(config.provider, ProviderKind::OpenAI);
            assert_eq!(config.base_url, "https://api.openai.com/v1");
            assert_eq!(config.model, "gpt-4o-mini");
            assert!(config.enabled_tools.is_empty());
        });
    }

    #[test]
    fn test_env_var_overrides_new() {
        with_env("I_RS_CLAW_API_KEY", Some("sk-test-key-from-env"), || {
            let config = Config::new();
            assert_eq!(config.api_key, "sk-test-key-from-env");
        });
    }

    #[test]
    fn test_env_var_empty_string() {
        with_env("I_RS_CLAW_API_KEY", Some(""), || {
            let config = Config::new();
            assert!(config.api_key.is_empty());
        });
    }
}
