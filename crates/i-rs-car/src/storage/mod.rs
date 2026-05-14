use crate::models::Store;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("i-rs");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    Ok(config_dir)
}

fn get_store_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("cars.json"))
}

pub fn load_store() -> Result<Store> {
    let path = get_store_path()?;

    if !path.exists() {
        return Ok(Store::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read store file")?;
    let store: Store = serde_json::from_str(&content)?;

    Ok(store)
}

pub fn save_store(store: &Store) -> Result<()> {
    let path = get_store_path()?;
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
    }
    let content = serde_json::to_string_pretty(store).context("Failed to serialize store")?;
    fs::write(&path, content).context("Failed to write store file")?;

    Ok(())
}
