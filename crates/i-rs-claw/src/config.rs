use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .join(".i-rs-claw");

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            anyhow::bail!(
                "配置文件不存在: {}\n请创建该文件，示例：\n\
                 [config]\n\
                 api_key = \"sk-...\"\n\
                 base_url = \"https://openrouter.ai/api/v1\"\n\
                 model = \"deepseek/deepseek-chat\"",
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
}
