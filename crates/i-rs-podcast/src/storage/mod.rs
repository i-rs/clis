use crate::models::PodcastStore;
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("podcasts.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("podcasts.json")
    }
}

pub fn load_store() -> Result<PodcastStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(PodcastStore::default())
    }
}

pub fn save_store(store: &PodcastStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn add_podcast(store: &mut PodcastStore, podcast: crate::models::Podcast) {
    store.add_podcast(podcast);
}

#[allow(dead_code)]
pub fn remove_podcast(store: &mut PodcastStore, name: &str) -> Option<crate::models::Podcast> {
    store.remove_podcast(name)
}
