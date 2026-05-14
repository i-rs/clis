use crate::models::PasswordEntry;
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

    if store.entries.contains_key(&name) {
        print_error(&format!("Entry '{}' already exists", name));
        anyhow::bail!("Entry '{}' already exists", name);
    }

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let entry = PasswordEntry {
        name: name.clone(),
        url,
        account,
        password: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}
