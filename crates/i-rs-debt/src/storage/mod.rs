use crate::models::{Debt, Store};
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
    Ok(get_config_dir()?.join("debt.json"))
}

pub fn load_store() -> Result<Store> {
    let path = get_store_path()?;

    if !path.exists() {
        return Ok(Store::default());
    }

    let content = fs::read_to_string(&path).context("Failed to read store file")?;
    let store: Store = serde_json::from_str(&content).unwrap_or_default();

    Ok(store)
}

pub fn save_store(store: &Store) -> Result<()> {
    let path = get_store_path()?;
    let content = serde_json::to_string_pretty(store).context("Failed to serialize store")?;
    fs::write(&path, content).context("Failed to write store file")?;

    Ok(())
}

pub fn add_debt(store: &mut Store, debt: Debt) {
    store.debts.insert(debt.name.clone(), debt);
}

pub fn get_debt<'a>(store: &'a Store, name: &str) -> Option<&'a Debt> {
    store.debts.get(name)
}

pub fn get_debt_mut<'a>(store: &'a mut Store, name: &str) -> Option<&'a mut Debt> {
    store.debts.get_mut(name)
}

pub fn delete_debt(store: &mut Store, name: &str) -> bool {
    store.debts.remove(name).is_some()
}

pub fn list_debts(store: &Store) -> Vec<&Debt> {
    store.debts.values().collect()
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a Store, tag: &str) -> Vec<&'a Debt> {
    store
        .debts
        .values()
        .filter(|d| d.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()))
        .collect()
}
