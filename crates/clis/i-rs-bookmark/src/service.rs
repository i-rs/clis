use crate::models::{Bookmark, BookmarkStore};
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;

/// List bookmarks, optionally filtered by tag.
pub fn list_bookmarks(store: &BookmarkStore, tag: Option<String>) -> Result<Vec<Bookmark>> {
    let bookmarks: Vec<Bookmark> = if let Some(ref tag_filter) = tag {
        store
            .bookmarks
            .values()
            .filter(|b| b.tags.contains(tag_filter))
            .cloned()
            .collect()
    } else {
        store.bookmarks.values().cloned().collect()
    };
    Ok(bookmarks)
}

/// Get a single bookmark by name.
pub fn get_bookmark(store: &BookmarkStore, name: &str) -> Result<Bookmark> {
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Bookmark '{name}' not found"))
}

/// Add a new bookmark. Optionally stores password in OS keychain.
pub fn add_bookmark(
    store: &mut BookmarkStore,
    name: String,
    url: String,
    account: Option<String>,
    password: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<Bookmark> {
    if store.bookmarks.contains_key(&name) {
        anyhow::bail!("Bookmark '{name}' already exists");
    }

    // Store password in keychain if provided
    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let bookmark = Bookmark {
        name: name.clone(),
        url,
        account,
        password: None, // Never stored in JSON
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(bookmark.clone());
    Ok(bookmark)
}

/// Update a bookmark.
pub fn update_bookmark(
    store: &mut BookmarkStore,
    name: String,
    url: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Bookmark> {
    let bookmark = store
        .get_entry_mut(&name)
        .with_context(|| format!("Bookmark '{name}' not found"))?;

    if let Some(u) = url {
        bookmark.url = u;
    }
    if let Some(a) = account {
        bookmark.account = Some(a);
    }
    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }
    if let Some(t) = tags {
        bookmark.tags = t;
    }
    if let Some(r) = remark {
        bookmark.remark = r;
    }
    bookmark.updated_at = Utc::now();

    let updated = bookmark.clone();
    Ok(updated)
}

/// Delete a bookmark and its keychain password.
pub fn delete_bookmark(store: &mut BookmarkStore, name: &str) -> Result<()> {
    if store.remove_entry(name).is_none() {
        anyhow::bail!("Bookmark '{name}' not found");
    }

    storage::delete_password(name)?;
    Ok(())
}
