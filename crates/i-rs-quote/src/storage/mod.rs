use crate::models::{Quote, QuoteStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("quotes.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("quotes.json")
    }
}

pub fn load_store() -> Result<QuoteStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(QuoteStore::default())
    }
}

pub fn save_store(store: &QuoteStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_quote(store: &mut QuoteStore, quote: Quote) {
    store.quotes.insert(quote.id.clone(), quote);
}

pub fn remove_quote(store: &mut QuoteStore, id: &str) -> Option<Quote> {
    store.quotes.remove(id)
}

pub fn get_quote<'a>(store: &'a QuoteStore, id: &str) -> Option<&'a Quote> {
    store.quotes.get(id)
}

pub fn get_all_quotes<'a>(store: &'a QuoteStore) -> Vec<&'a Quote> {
    store.quotes.values().collect()
}

pub fn filter_by_tag<'a>(store: &'a QuoteStore, tag: Option<&str>) -> Vec<&'a Quote> {
    if let Some(tag) = tag {
        store
            .quotes
            .values()
            .filter(|q| q.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.quotes.values().collect()
    }
}

pub fn filter_by_author<'a>(store: &'a QuoteStore, author: Option<&str>) -> Vec<&'a Quote> {
    if let Some(author) = author {
        let author_lower = author.to_lowercase();
        store
            .quotes
            .values()
            .filter(|q| {
                q.author
                    .as_ref()
                    .map(|a| a.to_lowercase().contains(&author_lower))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        store.quotes.values().collect()
    }
}
