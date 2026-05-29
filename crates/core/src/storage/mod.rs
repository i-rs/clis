use fs2::FileExt;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::PathBuf;

pub struct Storage<T> {
    pub data: T,
    config_dir: PathBuf,
    filename: String,
}

fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".i-rs")
        .join("data")
}

impl<T: Default> Default for Storage<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            config_dir: default_data_dir(),
            filename: String::new(),
        }
    }
}

impl<T: Serialize + DeserializeOwned + Default> Storage<T> {
    #[must_use]
    pub fn new(filename: &str) -> Self {
        let config_dir = default_data_dir();

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

        // Open file and acquire shared lock for reading
        let file = fs::File::open(&path)?;
        file.lock_shared()?;
        let content = fs::read_to_string(&path)?;
        // Lock released when `file` is dropped
        drop(file);

        self.data = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;

        Ok(&mut self.data)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_data(&self.data)
    }

    /// Save arbitrary data to the storage file without modifying self.data.
    pub fn save_data(&self, data: &T) -> anyhow::Result<()> {
        let path = self.file_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize: {e}"))?;

        // Create (or truncate) file and acquire exclusive lock for writing
        let file = fs::File::create(&path)?;
        file.lock_exclusive()?;
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", path.display(), e))?;
        // Lock released when `file` is dropped
        drop(file);

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
        Some(t) => items
            .iter()
            .filter(|item| item.tags().contains(&t.to_string()))
            .collect(),
        None => items.iter().collect(),
    }
}
