use crate::models::{KvEntry, ListItem};
use crate::presentation::{output_item, print_success, OutputFormat};
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
    format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&key) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&key) {
        anyhow::bail!("Key '{key}' already exists");
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

    store.add_entry(entry);
    storage::save_store(&store)?;

    if format.is_json() {
        let saved = store.get_entry(&key);
        if let Some(entry) = saved {
            let output = ListItem::from(entry);
            println!("{}", output_item(&output, format));
        }
        return Ok(());
    }

    print_success(&format!("✓ Key '{}' added successfully", key.green()));

    Ok(())
}
