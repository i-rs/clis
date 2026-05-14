use crate::models::{AssetType, Investment, InvestmentStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("invest.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("invest.json")
    }
}

pub fn load_store() -> Result<InvestmentStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(InvestmentStore::default())
    }
}

pub fn save_store(store: &InvestmentStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_investment(store: &mut InvestmentStore, investment: Investment) {
    store.investments.insert(investment.name.clone(), investment);
}

pub fn remove_investment(store: &mut InvestmentStore, name: &str) -> Option<Investment> {
    store.investments.remove(name)
}

#[allow(dead_code)]
pub fn get_investment<'a>(store: &'a InvestmentStore, name: &str) -> Option<&'a Investment> {
    store.investments.get(name)
}

pub fn get_investment_mut<'a>(store: &'a mut InvestmentStore, name: &str) -> Option<&'a mut Investment> {
    store.investments.get_mut(name)
}

pub fn filter_by_type<'a>(store: &'a InvestmentStore, asset_type: Option<&AssetType>) -> Vec<&'a Investment> {
    if let Some(asset_type) = asset_type {
        store
            .investments
            .values()
            .filter(|i| &i.asset_type == asset_type)
            .collect()
    } else {
        store.investments.values().collect()
    }
}

pub fn filter_by_tag<'a>(store: &'a InvestmentStore, tag: Option<&str>) -> Vec<&'a Investment> {
    if let Some(tag) = tag {
        store
            .investments
            .values()
            .filter(|i| i.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.investments.values().collect()
    }
}
