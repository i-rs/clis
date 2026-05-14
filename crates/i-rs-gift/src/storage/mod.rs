use crate::models::{Gift, GiftStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("gifts.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("gifts.json")
    }
}

pub fn load_store() -> Result<GiftStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(GiftStore::default())
    }
}

pub fn save_store(store: &GiftStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_gift(store: &mut GiftStore, gift: Gift) {
    store.gifts.insert(gift.name.clone(), gift);
}

pub fn remove_gift(store: &mut GiftStore, name: &str) -> Option<Gift> {
    store.gifts.remove(name)
}

pub fn get_gift<'a>(store: &'a GiftStore, name: &str) -> Option<&'a Gift> {
    store.gifts.get(name)
}

#[allow(dead_code)]
pub fn get_gift_mut<'a>(store: &'a mut GiftStore, name: &str) -> Option<&'a mut Gift> {
    store.gifts.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a GiftStore, tag: Option<&str>) -> Vec<&'a Gift> {
    if let Some(tag) = tag {
        store
            .gifts
            .values()
            .filter(|g| g.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.gifts.values().collect()
    }
}

pub fn filter_by_type<'a>(store: &'a GiftStore, gift_type: Option<&str>) -> Vec<&'a Gift> {
    if let Some(gt) = gift_type {
        store
            .gifts
            .values()
            .filter(|g| g.gift_type.to_string() == gt)
            .collect()
    } else {
        store.gifts.values().collect()
    }
}
