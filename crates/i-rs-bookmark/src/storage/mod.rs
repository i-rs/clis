use crate::models::{Bookmark, BookmarkStore};
use anyhow::Result;
use keyring::use_native_store;
use keyring_core::Entry;

i_rs_core::create_store!(BookmarkStore, "bookmark");

const SERVICE_NAME: &str = "i-rs-bookmark";

pub fn init_keyring() {
    let _ = use_native_store(false);
}

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {e}"))?;
    entry
        .set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {e}"))?;
    Ok(())
}

pub fn get_password(name: &str) -> Result<Option<String>> {
    match Entry::new(SERVICE_NAME, name) {
        Ok(entry) => match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(e) => {
                let err_str = format!("{e}");
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get password: {e}"))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

pub fn delete_password(name: &str) -> Result<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, name) {
        let _ = entry.delete_credential();
    }
    Ok(())
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
