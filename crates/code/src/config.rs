use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    let base = std::env::var("CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config")));
    base.join("i-rs-code").join("config.toml")
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
                std::fs::create_dir_all(parent).ok();
            }
            let config = Config::default();
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&path, &content).ok();
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
            dirs::home_dir().unwrap_or_default().join(".i-rs-code").join("tools")
        })
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.bin_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join(".i-rs-code").join("bin")
        })
    }

    pub fn manifest_path(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".i-rs-code").join("manifest.json")
    }
}
