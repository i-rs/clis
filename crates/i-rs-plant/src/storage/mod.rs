use crate::models::{Plant, PlantStore};
use anyhow::Result;
use std::path::PathBuf;

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("i-rs")
}

fn get_store_path() -> PathBuf {
    get_config_dir().join("plant.json")
}

pub fn load_store() -> Result<PlantStore> {
    let path = get_store_path();
    if !path.exists() {
        return Ok(PlantStore::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let store: PlantStore = serde_json::from_str(&content)?;
    Ok(store)
}

pub fn save_store(store: &PlantStore) -> Result<()> {
    let path = get_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_plant(store: &mut PlantStore, plant: Plant) {
    store.plants.push(plant);
}

pub fn find_plant<'a>(store: &'a PlantStore, name: &str) -> Option<&'a Plant> {
    store.plants.iter().find(|p| p.name == name)
}

pub fn find_plant_mut<'a>(store: &'a mut PlantStore, name: &str) -> Option<&'a mut Plant> {
    store.plants.iter_mut().find(|p| p.name == name)
}

pub fn remove_plant(store: &mut PlantStore, name: &str) -> bool {
    let len_before = store.plants.len();
    store.plants.retain(|p| p.name != name);
    store.plants.len() < len_before
}
