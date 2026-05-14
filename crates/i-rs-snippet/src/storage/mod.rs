use crate::models::{Snippet, SnippetStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("snippets.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("snippets.json")
    }
}

pub fn load_store() -> Result<SnippetStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(SnippetStore::default())
    }
}

pub fn save_store(store: &SnippetStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_snippet(store: &mut SnippetStore, snippet: Snippet) {
    store.snippets.insert(snippet.name.clone(), snippet);
}

pub fn remove_snippet(store: &mut SnippetStore, name: &str) -> Option<Snippet> {
    store.snippets.remove(name)
}

pub fn get_snippet<'a>(store: &'a SnippetStore, name: &str) -> Option<&'a Snippet> {
    store.snippets.get(name)
}

pub fn get_snippet_mut<'a>(store: &'a mut SnippetStore, name: &str) -> Option<&'a mut Snippet> {
    store.snippets.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a SnippetStore, tag: Option<&str>) -> Vec<&'a Snippet> {
    if let Some(tag) = tag {
        store
            .snippets
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.snippets.values().collect()
    }
}

pub fn search_snippets<'a>(store: &'a SnippetStore, query: &str) -> Vec<&'a Snippet> {
    let query_lower = query.to_lowercase();
    store
        .snippets
        .values()
        .filter(|s| {
            s.name.to_lowercase().contains(&query_lower)
                || s.language.to_lowercase().contains(&query_lower)
                || s.description.iter().any(|d| d.to_lowercase().contains(&query_lower))
                || s.code.iter().any(|c| c.to_lowercase().contains(&query_lower))
                || s.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect()
}
