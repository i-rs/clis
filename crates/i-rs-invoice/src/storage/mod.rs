use crate::models::{Invoice, InvoiceStore};
use anyhow::Result;
use std::path::PathBuf;

const STORE_FILE: &str = "invoice.json";

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = if let Some(dir) = std::env::var_os("CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    let app_dir = config_dir.join("i-rs");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    Ok(app_dir)
}

pub fn get_store_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(STORE_FILE))
}

pub fn load_store() -> Result<InvoiceStore> {
    let path = get_store_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let store: InvoiceStore = serde_json::from_str(&content)?;
        Ok(store)
    } else {
        Ok(InvoiceStore::default())
    }
}

pub fn save_store(store: &InvoiceStore) -> Result<()> {
    let path = get_store_path()?;
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut InvoiceStore, entry: Invoice) {
    store.entries.insert(entry.id.clone(), entry);
}

pub fn remove_entry(store: &mut InvoiceStore, id: &str) -> Option<Invoice> {
    store.entries.remove(id)
}

pub fn get_entry<'a>(store: &'a InvoiceStore, id: &str) -> Option<&'a Invoice> {
    store.entries.get(id)
}

pub fn get_entry_mut<'a>(store: &'a mut InvoiceStore, id: &str) -> Option<&'a mut Invoice> {
    store.entries.get_mut(id)
}

pub fn get_all_entries(store: &InvoiceStore) -> Vec<&Invoice> {
    store.entries.values().collect()
}

pub fn filter_by_reimbursed(store: &InvoiceStore, reimbursed: bool) -> Vec<&Invoice> {
    store
        .entries
        .values()
        .filter(|inv| inv.reimbursed == reimbursed)
        .collect()
}
