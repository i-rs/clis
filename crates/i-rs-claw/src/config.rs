use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const DEFAULT_TOOLS: &[&str] = &[
    "kv", "weight", "water", "sleep", "meal", "pig", "mood", "sit", "spark", "todo",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    /// Set of tool names to enable. Empty = all enabled.
    #[serde(default)]
    pub enabled_tools: HashSet<String>,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

impl Config {
    pub fn new() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_base_url(),
            model: default_model(),
            enabled_tools: HashSet::new(),
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
    pub fn all_tools() -> Vec<(&'static str, &'static str)> {
        crate::tools::TOOL_INDEX.to_vec()
    }
}


