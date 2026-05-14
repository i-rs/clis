use crate::models::KvEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    key: String,
    value: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&key) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&key) {
        anyhow::bail!("Key '{}' already exists", key);
    }

    let now = Utc::now();
    let entry = KvEntry {
        key: key.clone(),
        value,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Key '{}' added successfully", key.green()));

    Ok(())
}
