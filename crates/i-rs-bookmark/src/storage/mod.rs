use crate::models::{Bookmark, BookmarkStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;
use std::path::PathBuf;

const SERVICE_NAME: &str = "i-rs-bookmark";

pub fn init_keyring() {
    let _ = use_native_store(false);
}

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("bookmarks.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("bookmarks.json")
    }
}

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {}", e))?;
    Ok(())
}

pub fn get_password(name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get password: {}", e))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

pub fn delete_password(name: &str) -> Result<()> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => {
            let _ = entry.delete_credential();
        }
        Err(_) => {}
    }
    Ok(())
}

pub fn load_store() -> Result<BookmarkStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(BookmarkStore::default())
    }
}

pub fn save_store(store: &BookmarkStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_bookmark(store: &mut BookmarkStore, bookmark: Bookmark) {
    store.bookmarks.insert(bookmark.name.clone(), bookmark);
}

pub fn remove_bookmark(store: &mut BookmarkStore, name: &str) -> Option<Bookmark> {
    store.bookmarks.remove(name)
}

pub fn get_bookmark<'a>(store: &'a BookmarkStore, name: &str) -> Option<&'a Bookmark> {
    store.bookmarks.get(name)
}

pub fn get_bookmark_mut<'a>(store: &'a mut BookmarkStore, name: &str) -> Option<&'a mut Bookmark> {
    store.bookmarks.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a BookmarkStore, tag: Option<&str>) -> Vec<&'a Bookmark> {
    if let Some(tag) = tag {
        store
            .bookmarks
            .values()
            .filter(|b| b.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.bookmarks.values().collect()
    }
}
