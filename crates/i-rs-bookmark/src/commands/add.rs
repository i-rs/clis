use crate::models::Bookmark;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    url: String,
    account: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.bookmarks.contains_key(&name) {
        print_error(&format!("Bookmark '{}' already exists", name));
        anyhow::bail!("Bookmark '{}' already exists", name);
    }

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let bookmark = Bookmark {
        name: name.clone(),
        url,
        account,
        password: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_bookmark(&mut store, bookmark);
    storage::save_store(&store)?;

    print_success(&format!("✓ Bookmark '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}
