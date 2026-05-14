use crate::models::KeyEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    key_type: String,
    key_value: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        anyhow::bail!("Key '{}' already exists", name);
    }

    storage::store_key(&name, &key_value)?;

    let now = Utc::now();
    let entry = KeyEntry {
        name: name.clone(),
        key_type,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Key '{}' added and stored securely", name.green()));
    println!("  {}", "Value stored in OS keychain".dimmed());

    Ok(())
}
