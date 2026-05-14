use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct Storage<T> {
    pub data: T,
    config_dir: PathBuf,
    filename: String,
}

impl<T: Default> Default for Storage<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("i-rs"),
            filename: String::new(),
        }
    }
}

impl<T: Serialize + DeserializeOwned + Default> Storage<T> {
    pub fn new(filename: &str) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("i-rs");

        if let Ok(env_dir) = std::env::var("CONFIG_DIR") {
            let env_path = PathBuf::from(env_dir);
            return Self {
                data: T::default(),
                config_dir: env_path,
                filename: filename.to_string(),
            };
        }

        Self {
            data: T::default(),
            config_dir,
            filename: filename.to_string(),
        }
    }

    pub fn load(&mut self) -> anyhow::Result<&mut T> {
        let path = self.file_path();

        if !path.exists() {
            fs::create_dir_all(&self.config_dir)?;
            self.data = T::default();
            return Ok(&mut self.data);
        }

        let content = fs::read_to_string(&path)?;
        self.data = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;

        Ok(&mut self.data)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = self.file_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))?;

        Ok(())
    }

    fn file_path(&self) -> PathBuf {
        self.config_dir.join(format!("{}.json", self.filename))
    }
}

pub trait HasTags {
    fn tags(&self) -> &[String];
}

pub fn filter_by_tag<'a, T: HasTags>(items: &'a [T], tag: Option<&str>) -> Vec<&'a T> {
    match tag {
        Some(t) => items.iter().filter(|item| item.tags().contains(&t.to_string())).collect(),
        None => items.iter().collect(),
    }
}